use std::time::Duration;

use ureq::tls::{RootCerts, TlsConfig};
use ureq::{Agent, Proxy};
use url::{Host, Url};

use super::AcquireError;

const MAX_REDIRECTS: u32 = 10;
const RESOLVE_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
/// Also bounds the wait for the response headers.
const SEND_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
/// An attempt fails when no body bytes arrive for this long. A transfer that keeps
/// progressing has no total time limit.
const BODY_STALL_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub(crate) struct HttpClient {
    agent: Agent,
    direct_agent: Agent,
}

pub(crate) struct HttpResponse {
    pub response: ureq::http::Response<ureq::Body>,
    pub final_url: String,
    /// Redirect hosts outside the link's own host and its reviewed redirect hosts, in the order
    /// the request reached them. Only [`HttpClient::get_following_https_redirects`] fills it.
    pub unreviewed_hosts: Vec<String>,
}

/// Which cross-host redirects a request follows.
#[derive(Clone, Copy)]
enum RedirectPolicy<'a> {
    /// Any host.
    AnyHost,
    /// Only the link's own host and the reviewed hosts; any other host fails the request.
    ReviewedOnly(&'a [String]),
    /// Any HTTPS host, noting each one that is neither the link's host nor reviewed.
    NoteUnreviewed(&'a [String]),
}

/// Connection setup, header wait and body stall limits of one request.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HttpTimeouts {
    pub resolve: Duration,
    pub connect: Duration,
    pub send_request: Duration,
    pub body_stall: Duration,
}

impl HttpTimeouts {
    const PRODUCTION: Self = Self {
        resolve: RESOLVE_TIMEOUT,
        connect: CONNECT_TIMEOUT,
        send_request: SEND_REQUEST_TIMEOUT,
        body_stall: BODY_STALL_TIMEOUT,
    };
}

impl HttpClient {
    pub fn new() -> Self {
        Self::with_timeouts(HttpTimeouts::PRODUCTION)
    }

    /// A client with other limits, so that tests can exercise them in milliseconds.
    pub(crate) fn with_timeouts(timeouts: HttpTimeouts) -> Self {
        Self {
            agent: build_agent(Proxy::try_from_env(), timeouts),
            direct_agent: build_agent(None, timeouts),
        }
    }

    pub fn get(
        &self,
        original_url: &str,
        range: Option<(u64, &str)>,
    ) -> Result<HttpResponse, AcquireError> {
        self.get_with_redirect_policy(original_url, range, RedirectPolicy::AnyHost)
    }

    pub fn get_with_reviewed_redirects(
        &self,
        original_url: &str,
        range: Option<(u64, &str)>,
        redirect_hosts: &[String],
    ) -> Result<HttpResponse, AcquireError> {
        self.get_with_redirect_policy(
            original_url,
            range,
            RedirectPolicy::ReviewedOnly(redirect_hosts),
        )
    }

    /// Follows redirects to any host while the transfer stays on HTTPS, and reports the hosts
    /// that neither the link nor its reviewed redirect hosts name. An HTTPS-to-HTTP downgrade is
    /// still refused before the HTTP target is contacted.
    pub fn get_following_https_redirects(
        &self,
        original_url: &str,
        range: Option<(u64, &str)>,
        redirect_hosts: &[String],
    ) -> Result<HttpResponse, AcquireError> {
        self.get_with_redirect_policy(
            original_url,
            range,
            RedirectPolicy::NoteUnreviewed(redirect_hosts),
        )
    }

    fn get_with_redirect_policy(
        &self,
        original_url: &str,
        range: Option<(u64, &str)>,
        policy: RedirectPolicy<'_>,
    ) -> Result<HttpResponse, AcquireError> {
        let mut current = parse_download_url(original_url)?;
        let original_host = current.host_str().map(str::to_owned);
        let mut unreviewed_hosts = Vec::<String>::new();
        for redirect_count in 0..=MAX_REDIRECTS {
            let agent = if is_loopback_http(&current) {
                &self.direct_agent
            } else {
                &self.agent
            };
            let mut request = agent.get(current.as_str());
            if let Some((start, validator)) = range {
                request = request
                    .header("Range", format!("bytes={start}-"))
                    .header("If-Range", validator);
            }
            let response = request.call().map_err(|source| AcquireError::Http {
                url: current.to_string(),
                message: source.to_string(),
            })?;
            let status = response.status().as_u16();
            if !(300..400).contains(&status) {
                return Ok(HttpResponse {
                    response,
                    final_url: current.to_string(),
                    unreviewed_hosts,
                });
            }
            if redirect_count == MAX_REDIRECTS {
                return Err(AcquireError::TooManyRedirects {
                    url: original_url.to_owned(),
                });
            }
            let location = response
                .headers()
                .get("location")
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| AcquireError::MissingRedirectLocation {
                    url: current.to_string(),
                })?;
            let target = resolve_redirect(&current, location)?;
            let reviewed = |redirect_hosts: &[String]| {
                target.host_str().is_some_and(|host| {
                    original_host
                        .as_deref()
                        .is_some_and(|original| original.eq_ignore_ascii_case(host))
                        || redirect_hosts
                            .iter()
                            .any(|reviewed| reviewed.eq_ignore_ascii_case(host))
                })
            };
            match policy {
                RedirectPolicy::AnyHost => {}
                RedirectPolicy::ReviewedOnly(redirect_hosts) => {
                    if !reviewed(redirect_hosts) {
                        return Err(AcquireError::UnreviewedRedirect {
                            from: current.to_string(),
                            to: target.to_string(),
                        });
                    }
                }
                RedirectPolicy::NoteUnreviewed(redirect_hosts) => {
                    if !reviewed(redirect_hosts) {
                        let host = target.host_str().unwrap_or_default().to_ascii_lowercase();
                        if !unreviewed_hosts.contains(&host) {
                            unreviewed_hosts.push(host);
                        }
                    }
                }
            }
            current = target;
        }
        Err(AcquireError::TooManyRedirects {
            url: original_url.to_owned(),
        })
    }
}

/// Longest served file name kept for records; longer names are cut at a character boundary.
const MAX_SERVED_FILENAME_CHARS: usize = 200;

/// The file name a server gave its download: the `Content-Disposition` file name, else the last
/// segment of the final URL. Only for records and messages; it never names a file on disk.
pub(crate) fn served_filename(response: &HttpResponse) -> Option<String> {
    let disposition = response
        .response
        .headers()
        .get("content-disposition")
        .and_then(|value| value.to_str().ok());
    filename_from(disposition, &response.final_url)
}

fn filename_from(content_disposition: Option<&str>, final_url: &str) -> Option<String> {
    content_disposition
        .and_then(content_disposition_filename)
        .or_else(|| {
            Url::parse(final_url)
                .ok()?
                .path_segments()?
                .rev()
                .find(|segment| !segment.is_empty())
                .and_then(|segment| clean_filename(&percent_decode(segment)?))
        })
}

/// RFC 6266 `filename*` (RFC 8187 encoding) or plain `filename` parameter.
fn content_disposition_filename(header: &str) -> Option<String> {
    let mut plain = None;
    let mut extended = None;
    for parameter in split_parameters(header).into_iter().skip(1) {
        let Some((name, value)) = parameter.split_once('=') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim();
        if name == "filename*" {
            // charset'language'percent-encoded-value; UTF-8 and ASCII are what servers send.
            let mut parts = value.splitn(3, '\'');
            let (Some(charset), Some(_language), Some(encoded)) =
                (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            if charset.eq_ignore_ascii_case("utf-8") || charset.eq_ignore_ascii_case("us-ascii") {
                extended = percent_decode(encoded).and_then(|name| clean_filename(&name));
            }
        } else if name == "filename" {
            // Browsers also decode percent escapes here, which some download managers send.
            let value = unquote(value);
            plain = clean_filename(&percent_decode(&value).unwrap_or(value));
        }
    }
    extended.or(plain)
}

/// Splits a header at semicolons outside quoted strings.
fn split_parameters(header: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for character in header.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        match character {
            '\\' if quoted => {
                current.push(character);
                escaped = true;
            }
            '"' => {
                quoted = !quoted;
                current.push(character);
            }
            ';' if !quoted => parts.push(std::mem::take(&mut current)),
            _ => current.push(character),
        }
    }
    parts.push(current);
    parts
}

fn unquote(value: &str) -> String {
    let Some(inner) = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    else {
        return value.to_owned();
    };
    let mut unquoted = String::with_capacity(inner.len());
    let mut characters = inner.chars();
    while let Some(character) = characters.next() {
        if character == '\\' {
            if let Some(next) = characters.next() {
                unquoted.push(next);
            }
        } else {
            unquoted.push(character);
        }
    }
    unquoted
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let pair = value.get(index + 1..index + 3)?;
            decoded.push(u8::from_str_radix(pair, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

/// The last path component, without control characters, or `None` when nothing usable remains.
fn clean_filename(value: &str) -> Option<String> {
    let last = value.rsplit(['/', '\\']).next().unwrap_or(value);
    let cleaned: String = last
        .chars()
        .filter(|character| !character.is_control())
        .take(MAX_SERVED_FILENAME_CHARS)
        .collect();
    let cleaned = cleaned.trim();
    (!cleaned.is_empty() && cleaned != "." && cleaned != "..").then(|| cleaned.to_owned())
}

// ureq also applies the limit of the phase before the current one, counted from that phase's
// end: send_request therefore bounds the wait for the response headers too. recv_response is
// left unset because, for the same reason, it would cap the whole body after the headers;
// recv_body is checked anew before every read and so only detects a stall. A global or
// per-call limit would cap the whole transfer, so neither is set.
fn build_agent(proxy: Option<Proxy>, timeouts: HttpTimeouts) -> Agent {
    Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .timeout_resolve(Some(timeouts.resolve))
        .timeout_connect(Some(timeouts.connect))
        .timeout_send_request(Some(timeouts.send_request))
        .timeout_recv_body(Some(timeouts.body_stall))
        .proxy(proxy)
        .tls_config(
            TlsConfig::builder()
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .new_agent()
}

/// Resolve and validate a redirect target without issuing a request.
///
/// The policy rejects HTTPS-to-HTTP downgrades before the HTTP target can be contacted.
/// Relative locations are resolved against `from`.
pub fn validate_redirect_target(from: &str, target: &str) -> Result<String, AcquireError> {
    let from_url = parse_download_url(from)?;
    resolve_redirect(&from_url, target).map(|url| url.to_string())
}

fn resolve_redirect(from: &Url, target: &str) -> Result<Url, AcquireError> {
    let target_url = from
        .join(target)
        .map_err(|source| AcquireError::InvalidUrl {
            url: target.to_owned(),
            message: source.to_string(),
        })?;
    if from.scheme() == "https" && target_url.scheme() != "https" {
        return Err(AcquireError::RedirectDowngrade {
            from: from.to_string(),
            to: target_url.to_string(),
        });
    }
    validate_parsed_url(&target_url)?;
    Ok(target_url)
}

fn parse_download_url(value: &str) -> Result<Url, AcquireError> {
    let url = Url::parse(value).map_err(|source| AcquireError::InvalidUrl {
        url: value.to_owned(),
        message: source.to_string(),
    })?;
    validate_parsed_url(&url)?;
    Ok(url)
}

pub(crate) fn validate_download_url(value: &str) -> Result<(), AcquireError> {
    parse_download_url(value).map(|_| ())
}

fn validate_parsed_url(url: &Url) -> Result<(), AcquireError> {
    match url.scheme() {
        "https" => Ok(()),
        "http" if is_loopback_http(url) => Ok(()),
        "http" => Err(AcquireError::InsecureUrl {
            url: url.to_string(),
        }),
        scheme => Err(AcquireError::InvalidUrl {
            url: url.to_string(),
            message: format!("unsupported scheme `{scheme}`"),
        }),
    }
}

fn is_loopback_http(url: &Url) -> bool {
    if url.scheme() != "http" {
        return false;
    }
    match url.host() {
        Some(Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_agent_uses_platform_verifier_and_manual_redirects() {
        let client = HttpClient::new();
        assert!(matches!(
            client.agent.config().tls_config().root_certs(),
            RootCerts::PlatformVerifier
        ));
        assert_eq!(client.agent.config().max_redirects(), 0);
    }

    #[test]
    fn production_agents_bound_setup_and_stalls_but_not_the_whole_transfer() {
        assert_eq!(RESOLVE_TIMEOUT, Duration::from_secs(30));
        assert_eq!(CONNECT_TIMEOUT, Duration::from_secs(30));
        assert_eq!(SEND_REQUEST_TIMEOUT, Duration::from_secs(60));
        assert_eq!(BODY_STALL_TIMEOUT, Duration::from_secs(60));
        let client = HttpClient::new();
        for agent in [&client.agent, &client.direct_agent] {
            let timeouts = agent.config().timeouts();
            assert_eq!(timeouts.global, None);
            assert_eq!(timeouts.per_call, None);
            assert_eq!(timeouts.resolve, Some(RESOLVE_TIMEOUT));
            assert_eq!(timeouts.connect, Some(CONNECT_TIMEOUT));
            assert_eq!(timeouts.send_request, Some(SEND_REQUEST_TIMEOUT));
            assert_eq!(timeouts.recv_response, None);
            assert_eq!(timeouts.recv_body, Some(BODY_STALL_TIMEOUT));
        }
    }

    const LINK: &str = "https://weasel.example.invalid/download/?wpdmdl=557";

    fn named(disposition: &str) -> Option<String> {
        filename_from(Some(disposition), LINK)
    }

    #[test]
    fn served_filename_reads_plain_and_quoted_content_disposition_names() {
        assert_eq!(
            named("attachment; filename=Snowytoes-v1.7.zip").as_deref(),
            Some("Snowytoes-v1.7.zip")
        );
        assert_eq!(
            named(r#"attachment;filename="Snowy \"toes\"; v1.7.zip""#).as_deref(),
            Some(r#"Snowy "toes"; v1.7.zip"#)
        );
        assert_eq!(
            named(r#"ATTACHMENT; FileName = "spaced name.zip" ; size=12"#).as_deref(),
            Some("spaced name.zip")
        );
        // Weasel Mods percent-encodes plain names; a lone percent sign stays as it is.
        assert_eq!(
            named(r#"attachment;filename="Lanna%20v1.8.zip""#).as_deref(),
            Some("Lanna v1.8.zip")
        );
        assert_eq!(
            named("attachment; filename=100%.zip").as_deref(),
            Some("100%.zip")
        );
    }

    #[test]
    fn served_filename_prefers_the_encoded_name_in_either_order() {
        for header in [
            r#"attachment; filename="fallback.zip"; filename*=UTF-8''Sn%C3%B6wy%20toes.zip"#,
            r#"attachment; filename*=utf-8'en'Sn%C3%B6wy%20toes.zip; filename="fallback.zip""#,
        ] {
            assert_eq!(named(header).as_deref(), Some("Snöwy toes.zip"));
        }
        // An unsupported charset or a broken encoding falls back to the plain name.
        assert_eq!(
            named(r#"attachment; filename*=ISO-8859-1''caf%E9.zip; filename="cafe.zip""#)
                .as_deref(),
            Some("cafe.zip")
        );
        assert_eq!(
            named(r#"attachment; filename*=UTF-8''bad%ZZ.zip; filename="good.zip""#).as_deref(),
            Some("good.zip")
        );
    }

    #[test]
    fn served_filename_keeps_only_a_final_name_without_control_characters() {
        assert_eq!(
            named(r#"attachment; filename="../../Windows\\System32\\evil.zip""#).as_deref(),
            Some("evil.zip")
        );
        assert_eq!(
            named("attachment; filename*=UTF-8''..%2F..%2Fmods%2Fa%07b.zip").as_deref(),
            Some("ab.zip")
        );
        let long = "x".repeat(300);
        assert_eq!(
            named(&format!("attachment; filename={long}.zip"))
                .unwrap()
                .chars()
                .count(),
            MAX_SERVED_FILENAME_CHARS
        );
        // Nothing usable in the header: the final URL names the file instead.
        for header in [
            "inline",
            "attachment",
            r#"attachment; filename="..""#,
            r#"attachment; filename="  ""#,
        ] {
            assert_eq!(
                filename_from(
                    Some(header),
                    "https://mirror.example.invalid/files/My%20Mod.zip"
                )
                .as_deref(),
                Some("My Mod.zip"),
                "{header}"
            );
        }
    }

    #[test]
    fn served_filename_falls_back_to_the_final_url_and_may_find_none() {
        assert_eq!(
            filename_from(
                None,
                "https://mirror.example.invalid/a/Snowytoes-v1.7.zip?x=1#y"
            )
            .as_deref(),
            Some("Snowytoes-v1.7.zip")
        );
        assert_eq!(
            filename_from(None, "https://mirror.example.invalid/releases/v1.7/").as_deref(),
            Some("v1.7")
        );
        assert_eq!(filename_from(None, LINK).as_deref(), Some("download"));
        // A link that is only a query, such as `/?wpdmdl=557`, names no file.
        assert_eq!(
            filename_from(None, "https://weasel.example.invalid/?wpdmdl=557"),
            None
        );
        assert_eq!(filename_from(None, "not a url"), None);
    }

    #[test]
    fn redirects_may_change_host_but_never_downgrade_to_plain_http() {
        let from = Url::parse("https://weasel.example.invalid/?wpdmdl=557").unwrap();
        let target =
            resolve_redirect(&from, "https://cdn.example.invalid/Snowytoes-v1.7.zip").unwrap();
        assert_eq!(target.host_str(), Some("cdn.example.invalid"));
        assert!(matches!(
            resolve_redirect(&from, "http://cdn.example.invalid/Snowytoes-v1.7.zip"),
            Err(AcquireError::RedirectDowngrade { .. })
        ));
        assert!(matches!(
            resolve_redirect(&from, "ftp://cdn.example.invalid/Snowytoes-v1.7.zip"),
            Err(AcquireError::RedirectDowngrade { .. })
        ));
    }
}

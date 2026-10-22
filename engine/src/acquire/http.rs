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
        self.get_with_redirect_hosts(original_url, range, None)
    }

    pub fn get_with_reviewed_redirects(
        &self,
        original_url: &str,
        range: Option<(u64, &str)>,
        redirect_hosts: &[String],
    ) -> Result<HttpResponse, AcquireError> {
        self.get_with_redirect_hosts(original_url, range, Some(redirect_hosts))
    }

    fn get_with_redirect_hosts(
        &self,
        original_url: &str,
        range: Option<(u64, &str)>,
        redirect_hosts: Option<&[String]>,
    ) -> Result<HttpResponse, AcquireError> {
        let mut current = parse_download_url(original_url)?;
        let original_host = current.host_str().map(str::to_owned);
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
            if let Some(redirect_hosts) = redirect_hosts {
                let target_host = target.host_str();
                let reviewed = target_host.is_some_and(|host| {
                    original_host
                        .as_deref()
                        .is_some_and(|original| original.eq_ignore_ascii_case(host))
                        || redirect_hosts
                            .iter()
                            .any(|reviewed| reviewed.eq_ignore_ascii_case(host))
                });
                if !reviewed {
                    return Err(AcquireError::UnreviewedRedirect {
                        from: current.to_string(),
                        to: target.to_string(),
                    });
                }
            }
            current = target;
        }
        Err(AcquireError::TooManyRedirects {
            url: original_url.to_owned(),
        })
    }
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
}

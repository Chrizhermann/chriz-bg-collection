//! Publication routes that add response checks to the general acquisition contract.
//!
//! The general contract (HTTPS, reviewed redirects, pinned length and SHA-256, archive limits)
//! applies to every route. A route only adds checks for a host whose responses have a known
//! shape, so that anything else that host might answer fails before bytes are kept.

use ureq::http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use url::{Host, Url};

use super::http::{HttpResponse, RedirectPolicy};
use super::AcquireError;
use crate::manifest::{Source, SourceKind};

/// The only host from which Weasel Mods packages are fetched.
pub const WEASELMODS_DOWNLOAD_HOST: &str = "downloads.weaselmods.net";

const WEASELMODS_PACKAGE_PARAMETER: &str = "wpdmdl";
const ZIP_MEDIA_TYPE: &str = "application/zip";
const MAX_FILENAME_CHARS: usize = 255;

/// How one download request was published, and the response checks that follow from it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum DownloadRoute {
    /// Only the general contract.
    #[default]
    Generic,
    /// A Weasel Mods download-manager package link (`SourceKind::WeaselmodsDownload`).
    ///
    /// The request URL must have the canonical package-link shape. Redirects are followed only
    /// to the request's reviewed hosts; with none, every redirect fails. A response is kept only
    /// when it declares one `application/zip` body of an exact length and one attachment
    /// filename that, percent-decoded, equals `expected_filename`.
    WeaselmodsDownload {
        /// Filename a browser saves for the reviewed archive.
        expected_filename: String,
    },
}

impl DownloadRoute {
    /// The route an artifact's source kind requires.
    pub fn for_source(source: &Source) -> Result<Self, AcquireError> {
        match source.kind {
            SourceKind::WeaselmodsDownload => {
                let expected_filename = source.expected_filename.clone().ok_or_else(|| {
                    AcquireError::InvalidRequest(
                        "a weaselmods-download source needs an expected filename".to_owned(),
                    )
                })?;
                Ok(Self::WeaselmodsDownload { expected_filename })
            }
            SourceKind::GithubRelease
            | SourceKind::GithubTagArchive
            | SourceKind::GithubCommitZip
            | SourceKind::Manual => Ok(Self::Generic),
        }
    }

    /// Rejects a request URL or expected filename this route can never fetch, before any cache
    /// lookup or connection. Loopback HTTP stands in for the host, as everywhere in acquisition,
    /// so that hermetic tests can serve the same link shape.
    pub(crate) fn check_request(&self, url: &str) -> Result<(), AcquireError> {
        match self {
            Self::Generic => Ok(()),
            Self::WeaselmodsDownload { expected_filename } => {
                check_weaselmods_download_url(url, true).map_err(|message| {
                    AcquireError::InvalidUrl {
                        url: url.to_owned(),
                        message,
                    }
                })?;
                check_saved_filename(expected_filename).map_err(|message| {
                    AcquireError::InvalidRequest(format!(
                        "expected filename {expected_filename:?} is unusable: {message}"
                    ))
                })
            }
        }
    }

    pub(crate) fn redirect_policy<'a>(&self, redirect_hosts: &'a [String]) -> RedirectPolicy<'a> {
        match self {
            Self::Generic => RedirectPolicy::OriginalOrReviewed(redirect_hosts),
            Self::WeaselmodsDownload { .. } => RedirectPolicy::ReviewedOnly(redirect_hosts),
        }
    }

    /// Checks the headers of a response whose body is about to be kept.
    pub(crate) fn check_response(&self, response: &HttpResponse) -> Result<(), AcquireError> {
        match self {
            Self::Generic => Ok(()),
            Self::WeaselmodsDownload { expected_filename } => {
                check_weaselmods_response(response, expected_filename).map_err(|message| {
                    AcquireError::UnexpectedResponse {
                        url: response.final_url.clone(),
                        message,
                    }
                })
            }
        }
    }
}

/// Checks that `value` is exactly a canonical Weasel Mods package link:
/// `https://downloads.weaselmods.net/download/<slug>/?wpdmdl=<package id>`, with a lowercase
/// ASCII slug, a decimal package id, and no port, credentials, fragment or other parameter.
///
/// With `allow_loopback_http`, an `http://` loopback origin may stand in for the host; the path
/// and query rules stay the same.
pub fn check_weaselmods_download_url(value: &str, allow_loopback_http: bool) -> Result<(), String> {
    let url = Url::parse(value).map_err(|error| error.to_string())?;
    if url.as_str() != value {
        return Err(format!(
            "the URL is not in canonical form ({})",
            url.as_str()
        ));
    }
    let production = url.scheme() == "https"
        && url.host_str() == Some(WEASELMODS_DOWNLOAD_HOST)
        && url.port().is_none();
    let loopback = allow_loopback_http
        && url.scheme() == "http"
        && match url.host() {
            Some(Host::Domain(host)) => host == "localhost",
            Some(Host::Ipv4(address)) => address.is_loopback(),
            Some(Host::Ipv6(address)) => address.is_loopback(),
            None => false,
        };
    if !production && !loopback {
        return Err(format!(
            "weaselmods-download sources must use https://{WEASELMODS_DOWNLOAD_HOST}/ without a port"
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("the URL must not carry credentials".to_owned());
    }
    if url.fragment().is_some() {
        return Err("the URL must not have a fragment".to_owned());
    }
    let slug = url
        .path()
        .strip_prefix("/download/")
        .and_then(|rest| rest.strip_suffix('/'))
        .ok_or_else(|| "the path must be /download/<slug>/".to_owned())?;
    if slug.is_empty()
        || !slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(format!(
            "the package slug {slug:?} must be lowercase ASCII letters, digits and hyphens"
        ));
    }
    let package = url
        .query()
        .and_then(|query| query.strip_prefix(WEASELMODS_PACKAGE_PARAMETER))
        .and_then(|rest| rest.strip_prefix('='))
        .ok_or_else(|| {
            format!("the query must be exactly {WEASELMODS_PACKAGE_PARAMETER}=<package id>")
        })?;
    if package.is_empty()
        || package.len() > 10
        || package.starts_with('0')
        || !package.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(format!(
            "the query must be exactly {WEASELMODS_PACKAGE_PARAMETER}=<package id>, got {:?}",
            url.query().unwrap_or_default()
        ));
    }
    Ok(())
}

/// Checks that `name` is one plain Windows filename: no path, no reserved or control
/// characters, no trailing dot or space, and at most 255 characters.
pub fn check_saved_filename(name: &str) -> Result<(), String> {
    if name.is_empty() || name.chars().count() > MAX_FILENAME_CHARS {
        return Err("a filename must have 1 to 255 characters".to_owned());
    }
    if name == "." || name == ".." {
        return Err("a filename must not be a relative path component".to_owned());
    }
    if let Some(character) = name
        .chars()
        .find(|character| character.is_control() || r#"/\:*?"<>|"#.contains(*character))
    {
        return Err(format!(
            "a filename must not contain {character:?} (path separators, reserved and control characters are refused)"
        ));
    }
    if name.ends_with(['.', ' ']) || name.starts_with(' ') {
        return Err("a filename must not start with a space or end with a dot or space".to_owned());
    }
    Ok(())
}

fn check_weaselmods_response(
    response: &HttpResponse,
    expected_filename: &str,
) -> Result<(), String> {
    let content_type = single_header(response, CONTENT_TYPE.as_str())?
        .ok_or_else(|| "the response has no Content-Type; a ZIP archive was expected".to_owned())?;
    let media_type = content_type.split(';').next().unwrap_or_default().trim();
    if !media_type.eq_ignore_ascii_case(ZIP_MEDIA_TYPE) {
        return Err(format!(
            "the response is {content_type:?}, not a ZIP archive ({ZIP_MEDIA_TYPE}); the package link may be gone or need a visit to its page"
        ));
    }
    if single_header(response, CONTENT_LENGTH.as_str())?.is_none() {
        return Err("the response has no Content-Length".to_owned());
    }
    let disposition = single_header(response, CONTENT_DISPOSITION.as_str())?
        .ok_or_else(|| "the response has no Content-Disposition filename".to_owned())?;
    let filename = attachment_filename(&disposition)?;
    check_saved_filename(&filename)
        .map_err(|message| format!("the served filename {filename:?} is unusable: {message}"))?;
    if filename != expected_filename {
        return Err(format!(
            "the served filename {filename:?} is not the reviewed {expected_filename:?}"
        ));
    }
    Ok(())
}

/// The value of a header that may appear at most once, as UTF-8 text.
fn single_header(response: &HttpResponse, name: &str) -> Result<Option<String>, String> {
    let mut values = response.response.headers().get_all(name).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(format!("the response repeats the {name} header"));
    }
    value
        .to_str()
        .map(|value| Some(value.to_owned()))
        .map_err(|_| format!("the {name} header is not visible ASCII"))
}

/// The filename a browser saves for a `Content-Disposition: attachment` value.
///
/// An RFC 6266 `filename*` (UTF-8, percent-encoded) wins over `filename`, as in browsers.
/// A plain `filename` is percent-decoded too, as browsers do for servers that encode it there.
/// Malformed syntax, repeated parameters, a malformed escape or non-UTF-8 bytes fail.
pub(crate) fn attachment_filename(value: &str) -> Result<String, String> {
    let mut rest = value.trim_start();
    let kind_end = rest.find(';').unwrap_or(rest.len());
    let kind = rest[..kind_end].trim();
    if !kind.eq_ignore_ascii_case("attachment") {
        return Err(format!(
            "the Content-Disposition {value:?} is not an attachment"
        ));
    }
    rest = &rest[kind_end..];
    let mut plain = None;
    let mut extended = None;
    loop {
        rest = rest.trim_start();
        let Some(after) = rest.strip_prefix(';') else {
            if rest.is_empty() {
                break;
            }
            return Err(format!("the Content-Disposition {value:?} is malformed"));
        };
        rest = after.trim_start();
        if rest.is_empty() {
            break;
        }
        let name_end = rest.find('=').ok_or_else(|| {
            format!("the Content-Disposition {value:?} has a parameter without a value")
        })?;
        let name = rest[..name_end].trim().to_ascii_lowercase();
        if name.is_empty() || !name.bytes().all(is_token_byte) {
            return Err(format!("the Content-Disposition {value:?} is malformed"));
        }
        rest = rest[name_end + 1..].trim_start();
        let parameter;
        if let Some(quoted) = rest.strip_prefix('"') {
            let (text, remaining) = quoted_string(quoted).ok_or_else(|| {
                format!("the Content-Disposition {value:?} has an unterminated quote")
            })?;
            parameter = text;
            rest = remaining;
        } else {
            let end = rest.find(';').unwrap_or(rest.len());
            let token = rest[..end].trim_end();
            if token.is_empty()
                || token
                    .bytes()
                    .any(|byte| byte == b'"' || byte.is_ascii_whitespace())
            {
                return Err(format!("the Content-Disposition {value:?} is malformed"));
            }
            parameter = token.to_owned();
            rest = &rest[end..];
        }
        let slot = match name.as_str() {
            "filename" => &mut plain,
            "filename*" => &mut extended,
            _ => continue,
        };
        if slot.replace(parameter).is_some() {
            return Err(format!(
                "the Content-Disposition {value:?} repeats the {name} parameter"
            ));
        }
    }
    if let Some(extended) = extended {
        let (charset, rest) = extended
            .split_once('\'')
            .ok_or_else(|| format!("the filename* value {extended:?} is malformed"))?;
        let (_language, encoded) = rest
            .split_once('\'')
            .ok_or_else(|| format!("the filename* value {extended:?} is malformed"))?;
        if !charset.eq_ignore_ascii_case("utf-8") {
            return Err(format!("the filename* value {extended:?} is not UTF-8"));
        }
        return percent_decode(encoded);
    }
    let plain =
        plain.ok_or_else(|| format!("the Content-Disposition {value:?} names no filename"))?;
    percent_decode(&plain)
}

/// Parses the rest of a quoted string after its opening quote, returning its text and what
/// follows the closing quote.
fn quoted_string(input: &str) -> Option<(String, &str)> {
    let mut text = String::new();
    let mut characters = input.char_indices();
    while let Some((index, character)) = characters.next() {
        match character {
            '"' => return Some((text, &input[index + 1..])),
            '\\' => text.push(characters.next()?.1),
            other => text.push(other),
        }
    }
    None
}

fn is_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"!#$&+-.^_`|~*".contains(&byte)
}

/// Decodes `%XX` escapes once; anything else that follows `%` fails, and so do bytes that are
/// not UTF-8.
fn percent_decode(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let digit = |offset: usize| {
                bytes
                    .get(index + offset)
                    .and_then(|byte| char::from(*byte).to_digit(16))
            };
            let (Some(high), Some(low)) = (digit(1), digit(2)) else {
                return Err(format!(
                    "the filename {value:?} has a malformed percent escape"
                ));
            };
            // Both digits are below 16, so the byte cannot overflow.
            decoded.push((high * 16 + low) as u8);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded)
        .map_err(|_| format!("the filename {value:?} does not decode to UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINK: &str = "https://downloads.weaselmods.net/download/bethany/?wpdmdl=751";

    #[test]
    fn accepts_only_the_canonical_package_link_on_the_host() {
        assert_eq!(check_weaselmods_download_url(LINK, false), Ok(()));
        assert_eq!(
            check_weaselmods_download_url(
                "https://downloads.weaselmods.net/download/the-old-tale-2/?wpdmdl=1001",
                false
            ),
            Ok(())
        );
        for rejected in [
            "http://downloads.weaselmods.net/download/bethany/?wpdmdl=751",
            "https://weaselmods.net/download/bethany/?wpdmdl=751",
            "https://downloads.weaselmods.net.example.invalid/download/bethany/?wpdmdl=751",
            "https://example.invalid/download/bethany/?wpdmdl=751",
            "https://downloads.weaselmods.net:8443/download/bethany/?wpdmdl=751",
            "https://user@downloads.weaselmods.net/download/bethany/?wpdmdl=751",
            "https://downloads.weaselmods.net/download/bethany/?wpdmdl=751#top",
            "https://downloads.weaselmods.net/download/bethany/?wpdmdl=751&refresh=6adad8d9a8013",
            "https://downloads.weaselmods.net/download/bethany/?refresh=6adad8d9a8013&wpdmdl=751",
            "https://downloads.weaselmods.net/download/bethany/?wpdmdl=",
            "https://downloads.weaselmods.net/download/bethany/?wpdmdl=0751",
            "https://downloads.weaselmods.net/download/bethany/?wpdmdl=75a",
            "https://downloads.weaselmods.net/download/bethany/",
            "https://downloads.weaselmods.net/download/bethany?wpdmdl=751",
            "https://downloads.weaselmods.net/download/Bethany/?wpdmdl=751",
            "https://downloads.weaselmods.net/download/beth%61ny/?wpdmdl=751",
            "https://downloads.weaselmods.net/download/a/b/?wpdmdl=751",
            "https://downloads.weaselmods.net/wp-content/uploads/bethany.zip",
            "HTTPS://Downloads.WeaselMods.net/download/bethany/?wpdmdl=751",
            "https://downloads.weaselmods.net/download/x/../bethany/?wpdmdl=751",
            "http://127.0.0.1:8080/download/bethany/?wpdmdl=751",
        ] {
            assert!(
                check_weaselmods_download_url(rejected, false).is_err(),
                "{rejected} was accepted"
            );
        }
    }

    #[test]
    fn loopback_http_stands_in_for_the_host_only_when_allowed() {
        let local = "http://127.0.0.1:8080/download/bethany/?wpdmdl=751";
        assert_eq!(check_weaselmods_download_url(local, true), Ok(()));
        assert!(check_weaselmods_download_url(
            "http://127.0.0.1:8080/download/bethany/?wpdmdl=751&x=1",
            true
        )
        .is_err());
        assert!(check_weaselmods_download_url(
            "http://downloads.weaselmods.net/download/bethany/?wpdmdl=751",
            true
        )
        .is_err());
        assert!(check_weaselmods_download_url(
            "https://127.0.0.1:8080/download/bethany/?wpdmdl=751",
            true
        )
        .is_err());
    }

    #[test]
    fn reads_the_filename_a_browser_saves() {
        for (header, expected) in [
            (
                r#"attachment;filename="Bethany-v1.03.zip""#,
                "Bethany-v1.03.zip",
            ),
            (
                r#"attachment;filename="Lanna%20v1.8.zip""#,
                "Lanna v1.8.zip",
            ),
            (
                r#"Attachment; filename = "Lanna v1.8.zip" "#,
                "Lanna v1.8.zip",
            ),
            (
                "attachment; filename=Bethany-v1.03.zip",
                "Bethany-v1.03.zip",
            ),
            (
                r#"attachment; filename="fallback.zip"; filename*=UTF-8''Lanna%20v1.8.zip"#,
                "Lanna v1.8.zip",
            ),
            ("attachment; filename*=utf-8'en'Tviy%C3%A9.zip", "Tviyé.zip"),
            (r#"attachment; size=12; filename="a\"b.zip""#, "a\"b.zip"),
        ] {
            assert_eq!(
                attachment_filename(header).as_deref(),
                Ok(expected),
                "{header}"
            );
        }
    }

    #[test]
    fn refuses_dispositions_a_browser_could_read_differently() {
        for header in [
            r#"inline; filename="Bethany-v1.03.zip""#,
            "attachment",
            r#"attachment; filename="a.zip"; filename="b.zip""#,
            r#"attachment; filename="unterminated.zip"#,
            r#"attachment; filename="Lanna%2v1.8.zip""#,
            r#"attachment; filename="Lanna%ZZv1.8.zip""#,
            r#"attachment; filename="bad%FF.zip""#,
            "attachment; filename*=ISO-8859-1''Tviy%E9.zip",
            "attachment; filename*=Lanna.zip",
            "attachment; filename=two words.zip",
            r#"attachment; filename="a.zip" junk"#,
            "attachment; =a.zip",
        ] {
            assert!(
                attachment_filename(header).is_err(),
                "{header} was accepted"
            );
        }
    }

    #[test]
    fn saved_filenames_are_single_plain_windows_names() {
        for accepted in ["Lanna v1.8.zip", "Bethany-v1.03.zip", "Tviyé.zip"] {
            assert_eq!(check_saved_filename(accepted), Ok(()), "{accepted}");
        }
        for rejected in [
            "",
            ".",
            "..",
            "../Lanna.zip",
            r"..\Lanna.zip",
            "C:Lanna.zip",
            "Lanna.zip.",
            "Lanna.zip ",
            " Lanna.zip",
            "Lan\u{0}na.zip",
            "Lan\nna.zip",
            "Lanna?.zip",
        ] {
            assert!(
                check_saved_filename(rejected).is_err(),
                "{rejected:?} was accepted"
            );
        }
        assert!(check_saved_filename(&"a".repeat(256)).is_err());
    }
}

//! Direct player report submission to the `bg.chrizfader.org` relay.
//!
//! No webhook or secret ever ships in the client. Instead, the client fetches a short-lived
//! signed challenge from the Cloudflare Worker, solves a small SHA-256 Proof-of-Work puzzle, and
//! uploads the redacted report (plus an optional redacted diagnostics ZIP) over HTTPS.

use std::collections::hash_map::RandomState;
use std::hash::BuildHasher as _;
use std::io::Read as _;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use thiserror::Error;
use ureq::tls::{RootCerts, TlsConfig};
use ureq::{Agent, Proxy};
use url::{Host, Url};

/// Production Cloudflare Worker endpoint that forwards verified installer reports.
pub const REPORT_API_BASE: &str = "https://bg.chrizfader.org/api/installer-report";

/// Largest diagnostics ZIP accepted by the relay (7.5 MB, safely below Discord's 10 MB limit).
pub const MAX_DIAGNOSTICS_ATTACHMENT_BYTES: usize = 7_500_000;

/// Highest Proof-of-Work difficulty the client will solve, bounding worst-case CPU work.
pub const MAX_POW_DIFFICULTY: u32 = 22;

const POW_PREFIX: &str = "chriz-report-v1:";
const CLIENT_TAG_FILE: &str = "report-sender-tag.txt";
const MAX_RESPONSE_BODY_BYTES: u64 = 64 * 1024;
const RESOLVE_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const SEND_REQUEST_TIMEOUT: Duration = Duration::from_secs(45);
const RECV_BODY_TIMEOUT: Duration = Duration::from_secs(30);

/// Optional diagnostics ZIP attached to a direct report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportAttachment {
    pub filename: String,
    pub bytes: Vec<u8>,
}

/// Complete report payload handed to [`submit_report_to`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportSubmissionRequest {
    pub client_tag: Option<String>,
    pub flavor: String,
    pub app_version: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub contact: String,
    pub diagnostics: Option<ReportAttachment>,
}

/// Acknowledgement returned by the relay after forwarding (or silently dropping) a report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportSubmissionResult {
    pub report_id: String,
}

/// Errors produced while requesting a challenge, solving PoW, or uploading a report.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ReportSubmitError {
    #[error("direct report service is unavailable: {0}")]
    Unavailable(String),
    #[error("report rate limit reached")]
    RateLimited { retry_after_seconds: Option<u64> },
    #[error("report exceeds the maximum upload size")]
    TooLarge,
    #[error("report rejected: {code}")]
    Rejected { code: String },
    #[error("network error while sending report: {0}")]
    Network(String),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChallengeResponse {
    challenge: String,
    difficulty: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubmitMetadata<'a> {
    challenge: &'a str,
    nonce: &'a str,
    client_tag: Option<&'a str>,
    flavor: &'a str,
    app_version: &'a str,
    kind: &'a str,
    title: &'a str,
    body: &'a str,
    contact: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubmitResponse {
    report_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    code: Option<String>,
    retry_after_seconds: Option<u64>,
}

/// Whether `digest` starts with at least `difficulty` zero bits (`0..=32`).
pub fn has_leading_zero_bits(digest: &[u8], difficulty: u32) -> bool {
    if difficulty > 32 {
        return false;
    }
    let mut remaining = difficulty;
    for &byte in digest {
        if remaining == 0 {
            return true;
        }
        if remaining >= 8 {
            if byte != 0 {
                return false;
            }
            remaining -= 8;
        } else {
            let mask = 0xff_u8 << (8 - remaining);
            return (byte & mask) == 0;
        }
    }
    remaining == 0
}

/// Solves `SHA-256("chriz-report-v1:<challenge>:<nonce>")` up to [`MAX_POW_DIFFICULTY`].
pub fn solve_pow_challenge(challenge: &str, difficulty: u32) -> Result<String, ReportSubmitError> {
    if difficulty > MAX_POW_DIFFICULTY {
        return Err(ReportSubmitError::Rejected {
            code: "excessive_challenge_difficulty".to_owned(),
        });
    }
    let prefix = format!("{POW_PREFIX}{challenge}:");
    for nonce in 0_u64.. {
        let candidate = nonce.to_string();
        let mut hasher = Sha256::new();
        hasher.update(prefix.as_bytes());
        hasher.update(candidate.as_bytes());
        let digest = hasher.finalize();
        if has_leading_zero_bits(&digest, difficulty) {
            return Ok(candidate);
        }
    }
    Err(ReportSubmitError::Rejected {
        code: "pow_exhausted".to_owned(),
    })
}

/// Loads or creates a random 32-hex-character tag in `cache_dir` used only for anti-spam hashing.
pub fn load_or_create_client_tag(cache_dir: &Path) -> Option<String> {
    let path = cache_dir.join(CLIENT_TAG_FILE);
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim().to_ascii_lowercase();
        if is_valid_client_tag(&trimmed) {
            return Some(trimmed);
        }
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let mut hasher = Sha256::new();
    hasher.update(b"chriz-report-client-tag-v1");
    hasher.update(nanos.to_le_bytes());
    hasher.update(std::process::id().to_le_bytes());
    for round in 0_u64..4 {
        let entropy = RandomState::new().hash_one((round, nanos));
        hasher.update(entropy.to_le_bytes());
    }
    let generated = hex::encode(hasher.finalize())[..32].to_owned();
    if std::fs::create_dir_all(cache_dir).is_ok() {
        let _ = std::fs::write(&path, &generated);
    }
    Some(generated)
}

fn is_valid_client_tag(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

/// Submits `request` to `base_url` (requesting `<base_url>/challenge` first, then `<base_url>`).
pub fn submit_report_to(
    base_url: &str,
    request: &ReportSubmissionRequest,
) -> Result<ReportSubmissionResult, ReportSubmitError> {
    if let Some(attachment) = &request.diagnostics {
        if attachment.bytes.len() > MAX_DIAGNOSTICS_ATTACHMENT_BYTES {
            return Err(ReportSubmitError::TooLarge);
        }
        if !is_safe_attachment_filename(&attachment.filename) {
            return Err(ReportSubmitError::Rejected {
                code: "invalid_diagnostics_zip".to_owned(),
            });
        }
    }

    let trimmed_base = base_url.trim_end_matches('/');
    let parsed_base = validate_endpoint_url(trimmed_base)?;
    let agent = build_report_agent(&parsed_base);

    let challenge_url = format!("{trimmed_base}/challenge");
    let challenge_response = agent
        .post(&challenge_url)
        .send_empty()
        .map_err(|error| ReportSubmitError::Network(error.to_string()))?;
    let challenge_status = challenge_response.status().as_u16();
    let challenge_retry_after = retry_after_header(&challenge_response);
    let challenge_bytes = read_bounded_body(challenge_response)?;

    if challenge_status != 200 {
        return Err(classify_error_status(
            challenge_status,
            challenge_retry_after,
            &challenge_bytes,
        ));
    }

    let challenge: ChallengeResponse = serde_json::from_slice(&challenge_bytes).map_err(|_| {
        ReportSubmitError::Unavailable("invalid challenge response from report service".to_owned())
    })?;
    if challenge.challenge.trim().is_empty() || challenge.challenge.len() > 512 {
        return Err(ReportSubmitError::Unavailable(
            "malformed challenge token from report service".to_owned(),
        ));
    }

    let nonce = solve_pow_challenge(&challenge.challenge, challenge.difficulty)?;
    let metadata = SubmitMetadata {
        challenge: &challenge.challenge,
        nonce: &nonce,
        client_tag: request.client_tag.as_deref(),
        flavor: &request.flavor,
        app_version: &request.app_version,
        kind: &request.kind,
        title: &request.title,
        body: &request.body,
        contact: &request.contact,
    };
    let metadata_json =
        serde_json::to_vec(&metadata).map_err(|error| ReportSubmitError::Rejected {
            code: format!("metadata_encode_failed:{error}"),
        })?;

    let (boundary, multipart_body) =
        build_multipart_body(&metadata_json, request.diagnostics.as_ref());
    let content_type = format!("multipart/form-data; boundary={boundary}");

    let submit_response = agent
        .post(trimmed_base)
        .header("Content-Type", &content_type)
        .send(&multipart_body[..])
        .map_err(|error| ReportSubmitError::Network(error.to_string()))?;
    let submit_status = submit_response.status().as_u16();
    let submit_retry_after = retry_after_header(&submit_response);
    let submit_bytes = read_bounded_body(submit_response)?;

    if submit_status != 200 {
        return Err(classify_error_status(
            submit_status,
            submit_retry_after,
            &submit_bytes,
        ));
    }

    let parsed: SubmitResponse = serde_json::from_slice(&submit_bytes).map_err(|_| {
        ReportSubmitError::Unavailable("invalid confirmation from report service".to_owned())
    })?;
    if parsed.report_id.trim().is_empty() {
        return Err(ReportSubmitError::Unavailable(
            "missing report id in confirmation".to_owned(),
        ));
    }
    Ok(ReportSubmissionResult {
        report_id: parsed.report_id,
    })
}

fn validate_endpoint_url(raw: &str) -> Result<Url, ReportSubmitError> {
    let url = Url::parse(raw).map_err(|_| ReportSubmitError::Rejected {
        code: "invalid_endpoint".to_owned(),
    })?;
    if !url.username().is_empty() || url.password().is_some() || url.host().is_none() {
        return Err(ReportSubmitError::Rejected {
            code: "invalid_endpoint".to_owned(),
        });
    }
    match url.scheme() {
        "https" => Ok(url),
        "http" if is_loopback_host(&url) => Ok(url),
        _ => Err(ReportSubmitError::Rejected {
            code: "invalid_endpoint".to_owned(),
        }),
    }
}

fn is_loopback_host(url: &Url) -> bool {
    match url.host() {
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        Some(Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        None => false,
    }
}

fn build_report_agent(endpoint: &Url) -> Agent {
    let proxy = if endpoint.scheme() == "http" {
        None
    } else {
        Proxy::try_from_env()
    };
    Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .timeout_resolve(Some(RESOLVE_TIMEOUT))
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_send_request(Some(SEND_REQUEST_TIMEOUT))
        .timeout_recv_body(Some(RECV_BODY_TIMEOUT))
        .proxy(proxy)
        .tls_config(
            TlsConfig::builder()
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .new_agent()
}

fn is_safe_attachment_filename(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 120
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn build_multipart_body(
    metadata_json: &[u8],
    diagnostics: Option<&ReportAttachment>,
) -> (String, Vec<u8>) {
    let boundary = choose_unique_boundary(metadata_json, diagnostics);
    let extra = diagnostics.map_or(0, |file| file.bytes.len() + 256);
    let mut body = Vec::with_capacity(metadata_json.len() + 256 + extra);

    body.extend_from_slice(b"--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(
        b"\r\nContent-Disposition: form-data; name=\"metadata\"\r\nContent-Type: application/json; charset=utf-8\r\n\r\n",
    );
    body.extend_from_slice(metadata_json);
    body.extend_from_slice(b"\r\n");

    if let Some(attachment) = diagnostics {
        body.extend_from_slice(b"--");
        body.extend_from_slice(boundary.as_bytes());
        body.extend_from_slice(
            b"\r\nContent-Disposition: form-data; name=\"diagnostics\"; filename=\"",
        );
        body.extend_from_slice(attachment.filename.as_bytes());
        body.extend_from_slice(b"\"\r\nContent-Type: application/zip\r\n\r\n");
        body.extend_from_slice(&attachment.bytes);
        body.extend_from_slice(b"\r\n");
    }

    body.extend_from_slice(b"--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(b"--\r\n");
    (boundary, body)
}

fn choose_unique_boundary(metadata_json: &[u8], diagnostics: Option<&ReportAttachment>) -> String {
    for counter in 0_u64.. {
        let mut hasher = Sha256::new();
        hasher.update(b"chriz-report-multipart-v1:");
        hasher.update(counter.to_le_bytes());
        hasher.update((metadata_json.len() as u64).to_le_bytes());
        hasher.update(metadata_json);
        if let Some(attachment) = diagnostics {
            hasher.update(attachment.filename.as_bytes());
            hasher.update((attachment.bytes.len() as u64).to_le_bytes());
        }
        let candidate = format!("----ChrizReport{}", &hex::encode(hasher.finalize())[..32]);
        let needle = candidate.as_bytes();
        let in_metadata = contains_subslice(metadata_json, needle);
        let in_zip = diagnostics.is_some_and(|file| contains_subslice(&file.bytes, needle));
        if !in_metadata && !in_zip {
            return candidate;
        }
    }
    unreachable!("256-bit boundary space cannot be exhausted by finite slices")
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn retry_after_header(response: &ureq::http::Response<ureq::Body>) -> Option<u64> {
    response
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .and_then(|raw| raw.trim().parse::<u64>().ok())
}

fn read_bounded_body(
    response: ureq::http::Response<ureq::Body>,
) -> Result<Vec<u8>, ReportSubmitError> {
    let mut buffer = Vec::new();
    response
        .into_body()
        .into_reader()
        .take(MAX_RESPONSE_BODY_BYTES)
        .read_to_end(&mut buffer)
        .map_err(|error| ReportSubmitError::Network(error.to_string()))?;
    Ok(buffer)
}

fn classify_error_status(
    status: u16,
    header_retry_after: Option<u64>,
    body: &[u8],
) -> ReportSubmitError {
    let parsed = serde_json::from_slice::<ErrorResponse>(body).ok();
    let code = parsed
        .as_ref()
        .and_then(|entry| entry.code.clone())
        .filter(|value| !value.trim().is_empty());
    let body_retry_after = parsed.and_then(|entry| entry.retry_after_seconds);

    match status {
        413 => ReportSubmitError::TooLarge,
        429 => ReportSubmitError::RateLimited {
            retry_after_seconds: header_retry_after.or(body_retry_after),
        },
        502..=504 => {
            ReportSubmitError::Unavailable(code.unwrap_or_else(|| format!("http_status_{status}")))
        }
        _ => {
            if code.as_deref() == Some("report_too_large") {
                ReportSubmitError::TooLarge
            } else if status >= 500 {
                ReportSubmitError::Unavailable(
                    code.unwrap_or_else(|| format!("http_status_{status}")),
                )
            } else {
                ReportSubmitError::Rejected {
                    code: code.unwrap_or_else(|| format!("http_status_{status}")),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leading_zero_bits_matches_bit_boundaries() {
        assert!(has_leading_zero_bits(&[0x00, 0x00, 0x3f], 18));
        assert!(!has_leading_zero_bits(&[0x00, 0x00, 0x40], 18));
        assert!(has_leading_zero_bits(&[0x00, 0x80], 8));
        assert!(!has_leading_zero_bits(&[0x00, 0x80], 9));
        assert!(has_leading_zero_bits(&[0xff], 0));
        assert!(!has_leading_zero_bits(&[0x00; 4], 33));
    }

    #[test]
    fn client_tag_persists_across_reads() {
        let dir = tempfile::tempdir().unwrap();
        let first = load_or_create_client_tag(dir.path()).expect("generated client tag");
        assert!(is_valid_client_tag(&first));
        let second = load_or_create_client_tag(dir.path()).expect("reloaded client tag");
        assert_eq!(first, second);
    }

    #[test]
    fn non_https_non_loopback_endpoint_is_rejected_before_network_io() {
        let request = ReportSubmissionRequest {
            client_tag: None,
            flavor: "bg".to_owned(),
            app_version: "0.1.3".to_owned(),
            kind: "feedback".to_owned(),
            title: "[Feedback] Test".to_owned(),
            body: "Test".to_owned(),
            contact: String::new(),
            diagnostics: None,
        };
        let err =
            submit_report_to("http://example.com/api/installer-report", &request).unwrap_err();
        assert_eq!(
            err,
            ReportSubmitError::Rejected {
                code: "invalid_endpoint".to_owned()
            }
        );
    }
}

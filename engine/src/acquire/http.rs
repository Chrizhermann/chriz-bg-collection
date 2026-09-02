use std::time::Duration;

use ureq::tls::{RootCerts, TlsConfig};
use ureq::{Agent, Proxy};
use url::{Host, Url};

use super::AcquireError;

const MAX_REDIRECTS: u32 = 10;

#[derive(Clone)]
pub(crate) struct HttpClient {
    agent: Agent,
    direct_agent: Agent,
}

pub(crate) struct HttpResponse {
    pub response: ureq::http::Response<ureq::Body>,
    pub final_url: String,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            agent: build_agent(Proxy::try_from_env()),
            direct_agent: build_agent(None),
        }
    }

    pub fn get(
        &self,
        original_url: &str,
        range: Option<(u64, &str)>,
    ) -> Result<HttpResponse, AcquireError> {
        let mut current = parse_download_url(original_url)?;
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
            current = resolve_redirect(&current, location)?;
        }
        Err(AcquireError::TooManyRedirects {
            url: original_url.to_owned(),
        })
    }
}

fn build_agent(proxy: Option<Proxy>) -> Agent {
    Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .timeout_global(Some(Duration::from_secs(30)))
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
}

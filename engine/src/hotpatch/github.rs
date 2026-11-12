//! The release GitHub marks as Latest for one repository: its tag, its one ZIP asset, and the
//! length and SHA-256 GitHub publishes for that asset. The download itself goes through the
//! artifact cache, which checks both.
use std::io::Read;

use serde::Deserialize;

/// Hosts GitHub redirects release downloads to.
pub(super) const DOWNLOAD_HOSTS: &[&str] = &[
    "release-assets.githubusercontent.com",
    "objects.githubusercontent.com",
];
const API: &str = "https://api.github.com";
const MAX_JSON_BYTES: u64 = 1024 * 1024;
const MAX_ASSET_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LatestRelease {
    pub tag: String,
    pub title: Option<String>,
    pub page: Option<String>,
    pub asset_name: String,
    pub asset_url: String,
    pub asset_length: u64,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    html_url: Option<String>,
    draft: bool,
    prerelease: bool,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
    digest: Option<String>,
}

/// Ask GitHub for `repository`'s Latest release. `asset` names its ZIP, with `{tag}` standing for
/// the release's tag.
pub(super) fn latest_release(repository: &str, asset: &str) -> Result<LatestRelease, String> {
    let url = format!("{API}/repos/{repository}/releases/latest");
    let response = crate::radar::release_agent(false)
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "chriz-easy-bg")
        .call()
        .map_err(|error| format!("GitHub could not be reached: {error}"))?;
    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        return Err(format!("GitHub answered with status {status}."));
    }
    let mut body = Vec::new();
    response
        .into_body()
        .into_reader()
        .take(MAX_JSON_BYTES + 1)
        .read_to_end(&mut body)
        .map_err(|error| format!("GitHub's answer could not be read: {error}"))?;
    if body.len() as u64 > MAX_JSON_BYTES {
        return Err("GitHub's answer is too large.".into());
    }
    parse(repository, asset, &body)
}

/// Check GitHub's answer `body` for `repository`'s Latest release.
pub(super) fn parse(repository: &str, asset: &str, body: &[u8]) -> Result<LatestRelease, String> {
    let body = body.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(body);
    let release: GitHubRelease = serde_json::from_slice(body)
        .map_err(|error| format!("GitHub's answer is not a release: {error}"))?;
    if release.draft || release.prerelease {
        return Err("The Latest release is a draft or a pre-release.".into());
    }
    let tag = release.tag_name;
    if tag.is_empty()
        || tag.len() > 64
        || !tag
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
    {
        return Err(format!("The Latest release has an unusable tag {tag:?}."));
    }
    let name = asset.replace("{tag}", &tag);
    let mut matching = release.assets.into_iter().filter(|item| item.name == name);
    let found = matching
        .next()
        .ok_or_else(|| format!("The Latest release {tag} has no file {name}."))?;
    if matching.next().is_some() {
        return Err(format!(
            "The Latest release {tag} has two files named {name}."
        ));
    }
    if found.size == 0 || found.size > MAX_ASSET_BYTES {
        return Err(format!(
            "{name} has an unusable size ({} bytes).",
            found.size
        ));
    }
    let sha256 = found
        .digest
        .as_deref()
        .and_then(|digest| digest.strip_prefix("sha256:"))
        .filter(|hex| hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| format!("GitHub publishes no SHA-256 for {name}, so it cannot be checked."))?
        .to_ascii_lowercase();
    let expected = format!("https://github.com/{repository}/releases/download/{tag}/{name}");
    if found.browser_download_url != expected {
        return Err(format!(
            "{name} is not served from {repository}'s releases."
        ));
    }
    Ok(LatestRelease {
        title: release.name.filter(|title| !title.trim().is_empty()),
        page: release
            .html_url
            .filter(|url| url.starts_with(&format!("https://github.com/{repository}/releases/"))),
        tag,
        asset_name: name,
        asset_url: found.browser_download_url,
        asset_length: found.size,
        sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPO: &str = "Chrizhermann/bg-eeex-buffbot";

    fn answer(tag: &str, assets: &str) -> Vec<u8> {
        format!(
            r#"{{"tag_name":"{tag}","name":"{tag} — a summon can be buffed","html_url":"https://github.com/{REPO}/releases/tag/{tag}","draft":false,"prerelease":false,"assets":[{assets}]}}"#
        )
        .into_bytes()
    }

    fn asset(tag: &str, name: &str, digest: &str) -> String {
        format!(
            r#"{{"name":"{name}","size":5797041,"browser_download_url":"https://github.com/{REPO}/releases/download/{tag}/{name}","digest":"{digest}"}}"#
        )
    }

    const DIGEST: &str = "sha256:CE38D81102F717ACF709CC8A7827586F8AB6FF5FF450AACA42F0B5D513DA0082";

    #[test]
    fn the_latest_release_names_its_zip_length_and_digest() {
        let body = answer(
            "v1.16.0",
            &[
                asset("v1.16.0", "buffbot-v1.16.0.zip", DIGEST),
                asset("v1.16.0", "notes.txt", DIGEST),
            ]
            .join(","),
        );
        let release = parse(REPO, "buffbot-{tag}.zip", &body).unwrap();
        assert_eq!(release.tag, "v1.16.0");
        assert_eq!(release.asset_name, "buffbot-v1.16.0.zip");
        assert_eq!(release.asset_length, 5797041);
        assert_eq!(
            release.sha256,
            "ce38d81102f717acf709cc8a7827586f8ab6ff5ff450aaca42f0b5d513da0082"
        );
        assert_eq!(
            release.page.as_deref(),
            Some("https://github.com/Chrizhermann/bg-eeex-buffbot/releases/tag/v1.16.0")
        );
    }

    #[test]
    fn a_release_without_a_checkable_zip_is_refused() {
        for (body, says) in [
            (answer("v1.16.0", ""), "has no file"),
            (
                answer("v1.16.0", &asset("v1.16.0", "buffbot-v1.16.0.zip", "")),
                "no SHA-256",
            ),
            (
                answer(
                    "v1.16.0",
                    &asset("v1.16.0", "buffbot-v1.16.0.zip", "sha1:0000"),
                ),
                "no SHA-256",
            ),
            (
                answer("v1.16.0", &asset("v1.15.0", "buffbot-v1.16.0.zip", DIGEST)),
                "not served",
            ),
            (
                answer("../x", &asset("../x", "buffbot-../x.zip", DIGEST)),
                "unusable tag",
            ),
        ] {
            let error = parse(REPO, "buffbot-{tag}.zip", &body).unwrap_err();
            assert!(error.contains(says), "{error}");
        }
        let draft = String::from_utf8(answer(
            "v1.16.0",
            &asset("v1.16.0", "buffbot-v1.16.0.zip", DIGEST),
        ))
        .unwrap()
        .replace(r#""prerelease":false"#, r#""prerelease":true"#);
        assert!(parse(REPO, "buffbot-{tag}.zip", draft.as_bytes())
            .unwrap_err()
            .contains("pre-release"));
    }
}

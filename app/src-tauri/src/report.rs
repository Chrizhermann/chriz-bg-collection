//! Player reports: direct submission helpers and fixed browser fallback addresses.
//!
//! When a player chooses **Send report**, the native bridge posts to the fixed relay endpoint in
//! `bg_engine::report_submit`; when they choose GitHub or Discord instead, the addresses below are
//! fixed here and in the flavor, and title and text enter the GitHub address only as
//! percent-encoded query values, so they can never change where the browser goes.

use std::fmt::Write as _;

/// Invitation to ChrizFader's Discord server, where reports are read.
pub const CHAT_INVITATION: &str = "https://discord.gg/RtXjRzy68k";

/// Longest address handed to the operating system. Windows cuts a longer one off without an
/// error when it passes it to the browser.
pub const MAX_ADDRESS_LENGTH: usize = 2000;

/// Maximum report body characters accepted by the direct relay.
pub const MAX_BODY_CHARACTERS: usize = 8000;

/// Maximum optional contact characters accepted by the direct relay.
pub const MAX_CONTACT_CHARACTERS: usize = 120;

const MAX_TITLE_CHARACTERS: usize = 120;
const SHORTENED_NOTE: &str = "\n\n[Shortened to fit the link. Paste the whole report here.]";

/// The address of a new issue with a report filled in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IssueAddress {
    pub url: String,
    /// The text did not fit: the address ends with a note that asks for the whole report.
    pub shortened: bool,
}

/// The new-issue page of `repository` ("owner/name") on GitHub with `title` and `body` filled
/// in. The player reads and submits it there.
pub fn issue_address(repository: &str, title: &str, body: &str) -> IssueAddress {
    let title = clean_title(title);
    let body = plain_text(body);
    let mut url = format!(
        "https://github.com/{repository}/issues/new?title={}&body=",
        encode(&title)
    );
    let budget = MAX_ADDRESS_LENGTH.saturating_sub(url.len());
    let encoded = encode(&body);
    if encoded.len() <= budget {
        url.push_str(&encoded);
        return IssueAddress {
            url,
            shortened: false,
        };
    }
    let note = encode(SHORTENED_NOTE);
    let budget = budget.saturating_sub(note.len());
    let mut kept = String::new();
    let mut length = 0_usize;
    for character in body.chars() {
        let size = encoded_length(character);
        if length + size > budget {
            break;
        }
        length += size;
        kept.push(character);
    }
    url.push_str(&encode(kept.trim_end()));
    url.push_str(&note);
    IssueAddress {
        url,
        shortened: true,
    }
}

/// Name suggested for an exported diagnostics ZIP: the product and the UTC time of the export.
pub fn diagnostics_file_name(product_name: &str, now: time::OffsetDateTime) -> String {
    let product = product_name
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_ascii_lowercase();
    format!(
        "{product}-diagnostics-{:04}{:02}{:02}-{:02}{:02}.zip",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute()
    )
}

/// Trims a report title to one clean line of at most 120 characters.
pub fn clean_title(text: &str) -> String {
    single_line_limited(text, MAX_TITLE_CHARACTERS)
}

/// Trims optional contact details to one clean line of at most 120 characters without redacting
/// handles or email addresses the player deliberately typed.
pub fn clean_contact(text: &str) -> String {
    single_line_limited(text, MAX_CONTACT_CHARACTERS)
}

/// Normalizes newlines, strips control characters, and caps a report body at 8,000 characters.
pub fn clean_body(text: &str) -> String {
    plain_text(text).chars().take(MAX_BODY_CHARACTERS).collect()
}

fn single_line_limited(text: &str, max_characters: usize) -> String {
    let line = text
        .split(|character: char| character.is_control())
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ");
    line.chars().take(max_characters).collect()
}

fn plain_text(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .chars()
        .filter(|character| matches!(character, '\n' | '\t') || !character.is_control())
        .collect::<String>()
        .trim()
        .to_owned()
}

fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

fn encoded_length(character: char) -> usize {
    let mut buffer = [0_u8; 4];
    character
        .encode_utf8(&mut buffer)
        .bytes()
        .map(|byte| if is_unreserved(byte) { 1 } else { 3 })
        .sum()
}

fn encode(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        if is_unreserved(byte) {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

/// Exclusive temporary directory for staging an inline diagnostics ZIP during direct report
/// submission; deleted automatically when dropped.
pub(crate) struct ScopedReportTempDir {
    path: std::path::PathBuf,
}

impl ScopedReportTempDir {
    pub(crate) fn new() -> std::io::Result<Self> {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let root = std::env::temp_dir();
        let pid = std::process::id();
        let nanos = time::OffsetDateTime::now_utc().unix_timestamp_nanos();
        let mut last_error = None;
        for _ in 0..32 {
            let seq = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let candidate = root.join(format!("cebg-report-{pid}-{nanos}-{seq}"));
            match std::fs::create_dir(&candidate) {
                Ok(()) => return Ok(Self { path: candidate }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    last_error = Some(error);
                }
                Err(error) => return Err(error),
            }
        }
        Err(last_error.unwrap_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "exhausted temporary directory name attempts",
            )
        }))
    }

    pub(crate) fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for ScopedReportTempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPOSITORY: &str = "Chrizhermann/chriz-easy-bg";

    fn decode(text: &str) -> String {
        let bytes = text.as_bytes();
        let mut decoded = Vec::new();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'%' {
                let value = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap();
                decoded.push(u8::from_str_radix(value, 16).unwrap());
                index += 3;
            } else {
                decoded.push(bytes[index]);
                index += 1;
            }
        }
        String::from_utf8(decoded).unwrap()
    }

    #[test]
    fn a_report_enters_the_fixed_issue_page_as_encoded_query_values() {
        let address = issue_address(
            REPOSITORY,
            "[Problem] Installation stopped at \"Install mods\"",
            "It broke.\r\n\r\nApp: Chriz Easy BG 0.1.3 & more #1?x=y",
        );
        assert!(!address.shortened);
        assert_eq!(
            address.url,
            "https://github.com/Chrizhermann/chriz-easy-bg/issues/new?title=%5BProblem%5D%20Installation%20stopped%20at%20%22Install%20mods%22&body=It%20broke.%0A%0AApp%3A%20Chriz%20Easy%20BG%200.1.3%20%26%20more%20%231%3Fx%3Dy"
        );
    }

    /// Whatever the interface sends stays inside the two query values.
    #[test]
    fn report_text_cannot_change_where_the_browser_goes() {
        let address = issue_address(
            REPOSITORY,
            "x&body=evil#fragment\nhttps://example.test/",
            "a&title=b\u{0}\u{7}/../../other?labels=x",
        );
        let (page, query) = address.url.split_once('?').unwrap();
        assert_eq!(
            page,
            "https://github.com/Chrizhermann/chriz-easy-bg/issues/new"
        );
        let parameters = query.split('&').collect::<Vec<_>>();
        assert_eq!(parameters.len(), 2);
        assert_eq!(
            decode(parameters[0]),
            "title=x&body=evil#fragment https://example.test/"
        );
        assert_eq!(decode(parameters[1]), "body=a&title=b/../../other?labels=x");
        assert!(query
            .bytes()
            .all(|byte| is_unreserved(byte) || matches!(byte, b'%' | b'&' | b'=')));
    }

    #[test]
    fn a_long_report_is_cut_at_a_character_and_says_so() {
        let body = "Die Installation hält an – ärgerlich. ".repeat(200);
        let address = issue_address(REPOSITORY, "[Problem] Zu lang", &body);
        assert!(address.shortened);
        assert!(address.url.len() <= MAX_ADDRESS_LENGTH);
        // Nearly the whole budget is used, and the cut never splits a character.
        assert!(address.url.len() > MAX_ADDRESS_LENGTH - 12);
        let text = decode(address.url.split_once("&body=").unwrap().1);
        let kept = text.strip_suffix(SHORTENED_NOTE).unwrap();
        assert!(body.starts_with(kept));
        assert!(!kept.ends_with(char::is_whitespace));
    }

    #[test]
    fn a_report_that_just_fits_is_not_cut() {
        let prefix = issue_address(REPOSITORY, "t", "").url.len();
        let fits = "a".repeat(MAX_ADDRESS_LENGTH - prefix);
        let address = issue_address(REPOSITORY, "t", &fits);
        assert!(!address.shortened);
        assert_eq!(address.url.len(), MAX_ADDRESS_LENGTH);
        assert!(issue_address(REPOSITORY, "t", &format!("{fits}a")).shortened);
    }

    #[test]
    fn a_title_is_one_short_line() {
        let address = issue_address(REPOSITORY, &"wörter \t\n".repeat(60), "b");
        let title = decode(
            address
                .url
                .split_once("?title=")
                .unwrap()
                .1
                .split_once('&')
                .unwrap()
                .0,
        );
        assert_eq!(title.chars().count(), MAX_TITLE_CHARACTERS);
        assert!(title.starts_with("wörter wörter wörter"));
        assert!(address.url.len() <= MAX_ADDRESS_LENGTH);
    }

    #[test]
    fn the_chat_invitation_is_a_fixed_discord_address() {
        assert!(CHAT_INVITATION.starts_with("https://discord.gg/"));
        assert!(CHAT_INVITATION["https://discord.gg/".len()..]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric()));
    }

    #[test]
    fn a_diagnostics_file_is_named_after_the_product_and_the_time() {
        let now = time::OffsetDateTime::from_unix_timestamp(1_794_300_000).unwrap();
        assert_eq!(
            diagnostics_file_name("Chriz Easy IWD", now),
            format!(
                "chriz-easy-iwd-diagnostics-{:04}{:02}{:02}-{:02}{:02}.zip",
                now.year(),
                u8::from(now.month()),
                now.day(),
                now.hour(),
                now.minute()
            )
        );
        assert_eq!(
            diagnostics_file_name(
                "Chriz Easy BG",
                time::OffsetDateTime::from_unix_timestamp(0).unwrap()
            ),
            "chriz-easy-bg-diagnostics-19700101-0000.zip"
        );
    }

    #[test]
    fn contact_and_body_helpers_strip_controls_and_enforce_caps() {
        assert_eq!(
            clean_contact("  @chriz_player\r\n<player@example.com>\u{7}  "),
            "@chriz_player <player@example.com>"
        );
        assert_eq!(
            clean_contact(&"ä".repeat(200)).chars().count(),
            MAX_CONTACT_CHARACTERS
        );
        assert_eq!(
            clean_body(&"x".repeat(MAX_BODY_CHARACTERS + 250))
                .chars()
                .count(),
            MAX_BODY_CHARACTERS
        );
    }

    #[test]
    fn scoped_temp_dir_creates_exclusive_directory_and_removes_it_on_drop() {
        let captured_path = {
            let temp_dir = ScopedReportTempDir::new().expect("create report temp dir");
            let file_path = temp_dir.path().join("sample.zip");
            std::fs::write(&file_path, b"PK\x05\x06").expect("write staged file");
            assert!(file_path.is_file());
            temp_dir.path().to_path_buf()
        };
        assert!(!captured_path.exists());
    }
}

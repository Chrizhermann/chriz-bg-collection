//! Pure parsing of active WeiDU.log entries and terminal debug markers.

use thiserror::Error;

/// One active component entry with its original source evidence retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// TP2 path exactly as written between tildes.
    pub tp2: String,
    /// Case-insensitive, separator-normalized TP2 identity retaining the full path.
    pub tp2_key: String,
    /// WeiDU language number recorded for the component.
    pub language: u32,
    /// Installed component number.
    pub component: u32,
    /// Optional human-readable text following `//`.
    pub annotation: Option<String>,
    /// One-based source line number.
    pub line_number: usize,
    /// Complete original source line.
    pub source_line: String,
}

/// A malformed line that looked like an active WeiDU.log entry.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("malformed active WeiDU.log entry on line {line_number}: {reason}; source {source_line:?}")]
pub struct LogParseError {
    line_number: usize,
    source_line: String,
    reason: &'static str,
}

impl LogParseError {
    /// Returns the one-based source line containing the malformed entry.
    pub const fn line_number(&self) -> usize {
        self.line_number
    }

    /// Returns the complete source line containing the malformed entry.
    pub fn source_line(&self) -> &str {
        &self.source_line
    }
}

/// One terminal component status found in a per-attempt WeiDU debug log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugStatus {
    /// Component committed without reported warnings.
    SuccessfullyInstalled,
    /// Component committed while reporting warnings.
    InstalledWithWarnings,
    /// Component did not commit because installation failed.
    NotInstalledDueToErrors,
    /// Component was skipped and did not commit.
    Skipped,
}

/// Parses active component rows while ignoring headers, blanks, and comment-only evidence.
///
/// A non-entry line is ignored, but a line beginning with `~` is treated as intended active
/// evidence and must parse completely. This prevents a damaged component row from disappearing.
pub fn parse_active_entries(input: &str) -> Result<Vec<LogEntry>, LogParseError> {
    let mut entries = Vec::new();
    for (index, source_line) in input.lines().enumerate() {
        let line_number = index + 1;
        let line = source_line.trim();
        if line.is_empty() || !line.starts_with('~') {
            continue;
        }

        let closing_tilde = line[1..]
            .find('~')
            .map(|offset| offset + 1)
            .ok_or_else(|| {
                parse_error(
                    line_number,
                    source_line,
                    "missing closing tilde after TP2 path",
                )
            })?;
        let tp2 = &line[1..closing_tilde];
        if tp2.is_empty() {
            return Err(parse_error(line_number, source_line, "TP2 path is empty"));
        }

        let remainder = line[closing_tilde + 1..].trim();
        let (fields, annotation) = match remainder.split_once("//") {
            Some((fields, annotation)) => {
                let annotation = annotation.trim();
                (
                    fields,
                    (!annotation.is_empty()).then(|| annotation.to_owned()),
                )
            }
            None => (remainder, None),
        };
        let mut fields = fields.split_whitespace();
        let language =
            parse_number_field(fields.next(), line_number, source_line, "language field")?;
        let component =
            parse_number_field(fields.next(), line_number, source_line, "component field")?;
        if fields.next().is_some() {
            return Err(parse_error(
                line_number,
                source_line,
                "unexpected fields after component",
            ));
        }

        entries.push(LogEntry {
            tp2: tp2.to_owned(),
            tp2_key: normalize_tp2_key(tp2),
            language,
            component,
            annotation,
            line_number,
            source_line: source_line.to_owned(),
        });
    }
    Ok(entries)
}

/// Returns the stable case-insensitive identity of a full TP2 path.
pub fn normalize_tp2_key(tp2: &str) -> String {
    tp2.replace('\\', "/").to_ascii_lowercase()
}

/// Extracts only component-terminal markers from a per-attempt debug transcript.
///
/// Other text, including parse noise emitted while scanning unrelated installers, is ignored.
pub fn parse_terminal_statuses(debug: &str) -> Vec<DebugStatus> {
    debug
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            if line.starts_with("SUCCESSFULLY INSTALLED") {
                Some(DebugStatus::SuccessfullyInstalled)
            } else if line.starts_with("INSTALLED WITH WARNINGS") {
                Some(DebugStatus::InstalledWithWarnings)
            } else if line.starts_with("NOT INSTALLED DUE TO ERRORS") {
                Some(DebugStatus::NotInstalledDueToErrors)
            } else if line.starts_with("SKIPPING") {
                Some(DebugStatus::Skipped)
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn has_new_uninstall_comment(before: &str, after: &str) -> bool {
    use std::collections::BTreeMap;

    let mut existing = BTreeMap::<String, usize>::new();
    for comment in uninstall_comments(before) {
        *existing.entry(comment).or_default() += 1;
    }
    for comment in uninstall_comments(after) {
        let count = existing.entry(comment).or_default();
        if *count == 0 {
            return true;
        }
        *count -= 1;
    }
    false
}

fn uninstall_comments(input: &str) -> impl Iterator<Item = String> + '_ {
    input.lines().filter_map(|line| {
        let comment = line.trim().strip_prefix("//")?.trim_start();
        const PREFIX: &str = "Recently Uninstalled:";
        let head = comment.get(..PREFIX.len())?;
        head.eq_ignore_ascii_case(PREFIX)
            .then(|| comment.to_ascii_lowercase())
    })
}

fn parse_number_field(
    field: Option<&str>,
    line_number: usize,
    source_line: &str,
    name: &'static str,
) -> Result<u32, LogParseError> {
    let Some(value) = field.and_then(|field| field.strip_prefix('#')) else {
        return Err(parse_error(line_number, source_line, name));
    };
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(parse_error(line_number, source_line, name));
    }
    value
        .parse()
        .map_err(|_| parse_error(line_number, source_line, name))
}

fn parse_error(line_number: usize, source_line: &str, reason: &'static str) -> LogParseError {
    LogParseError {
        line_number,
        source_line: source_line.to_owned(),
        reason,
    }
}

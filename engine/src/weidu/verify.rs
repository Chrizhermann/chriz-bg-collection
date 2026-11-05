//! Fail-closed reconciliation of before/after WeiDU.log and attempt-debug evidence.

use std::collections::BTreeSet;

use super::log::{
    has_new_uninstall_comment, normalize_tp2_key, parse_active_entries, parse_terminal_statuses,
    DebugStatus, LogEntry,
};

/// Expected identity and observed process exit for one pure install attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedRun {
    /// Full recipe TP2 path; matching is case-insensitive but retains path identity.
    pub tp2: String,
    /// Authored installer language number.
    pub language: u32,
    /// Exact ordered component sequence requested from WeiDU.
    pub components: Vec<u32>,
    /// Observed child-process exit code.
    pub exit_code: i32,
}

/// Result of reconciling immutable before, after, and debug snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reconciliation {
    /// The exact requested tail and every terminal success marker are present.
    ProvenDone,
    /// A strict leading component prefix committed; only this suffix remains.
    PartialPrefix {
        /// Exact remaining components in authored order.
        remaining: Vec<u32>,
    },
    /// The active stack is unchanged and debug evidence proves no component committed.
    UnchangedRetryable,
    /// WeiDU went on after a failed component, as it does: the components it reports as
    /// installed are exactly the new log rows, in order, and at least one came after a failure.
    FailedAmong {
        /// Requested components not installed, in authored order.
        failed: Vec<u32>,
        /// Components installed after the first failure, in authored order.
        installed_after: Vec<u32>,
    },
    /// Existing entries changed, an unexpected entry appeared, or an uninstall was introduced.
    StackDisturbed,
    /// Evidence is malformed, incomplete, or internally contradictory.
    Ambiguous,
}

/// Reconciles one pure install attempt without trusting its exit code as success evidence.
pub fn reconcile(before: &str, after: &str, debug: &str, expected: &ExpectedRun) -> Reconciliation {
    if expected.components.is_empty()
        || expected.tp2.is_empty()
        || expected.components.iter().collect::<BTreeSet<_>>().len() != expected.components.len()
    {
        return Reconciliation::Ambiguous;
    }

    let Ok(before_entries) = parse_active_entries(before) else {
        return Reconciliation::Ambiguous;
    };
    let Ok(after_entries) = parse_active_entries(after) else {
        return Reconciliation::Ambiguous;
    };

    if has_new_uninstall_comment(before, after)
        || after_entries.len() < before_entries.len()
        || !before_entries
            .iter()
            .zip(&after_entries)
            .all(|(before, after)| same_active_identity(before, after))
    {
        return Reconciliation::StackDisturbed;
    }

    let appended = &after_entries[before_entries.len()..];
    if appended.len() > expected.components.len() {
        return Reconciliation::StackDisturbed;
    }
    let tp2_key = normalize_tp2_key(&expected.tp2);
    if appended
        .iter()
        .any(|entry| entry.tp2_key != tp2_key || entry.language != expected.language)
    {
        return Reconciliation::StackDisturbed;
    }

    let statuses = parse_terminal_statuses(debug);
    if let Some(verdict) = failed_among(appended, &statuses, expected) {
        return verdict;
    }
    if appended
        .iter()
        .enumerate()
        .any(|(index, entry)| entry.component != expected.components[index])
    {
        return Reconciliation::StackDisturbed;
    }

    if statuses.len() != expected.components.len() {
        return Reconciliation::Ambiguous;
    }

    let completed = appended.len();
    if completed == expected.components.len() {
        if !statuses.iter().all(|status| status.is_success())
            || !full_exit_matches(&statuses, expected.exit_code)
        {
            return Reconciliation::Ambiguous;
        }
        return Reconciliation::ProvenDone;
    }

    if !statuses[..completed]
        .iter()
        .all(|status| status.is_success())
        || !statuses[completed..]
            .iter()
            .all(|status| status.is_failure())
    {
        return Reconciliation::Ambiguous;
    }

    if completed == 0 {
        Reconciliation::UnchangedRetryable
    } else {
        Reconciliation::PartialPrefix {
            remaining: expected.components[completed..].to_vec(),
        }
    }
}

/// The verdict when the evidence shows WeiDU going on after a failed component: one status per
/// requested component, a failing exit code, and new log rows that are exactly the components
/// reported as installed, in order, with at least one after a failure.
fn failed_among(
    appended: &[LogEntry],
    statuses: &[DebugStatus],
    expected: &ExpectedRun,
) -> Option<Reconciliation> {
    if statuses.len() != expected.components.len() || expected.exit_code == 0 {
        return None;
    }
    let reported = expected.components.iter().zip(statuses);
    let installed = reported
        .clone()
        .filter(|(_, status)| status.is_success())
        .map(|(component, _)| *component)
        .collect::<Vec<_>>();
    let failed = reported
        .clone()
        .filter(|(_, status)| status.is_failure())
        .map(|(component, _)| *component)
        .collect::<Vec<_>>();
    let first_failed = statuses.iter().position(|status| status.is_failure())?;
    let installed_after = reported
        .skip(first_failed)
        .filter(|(_, status)| status.is_success())
        .map(|(component, _)| *component)
        .collect::<Vec<_>>();
    let recorded = appended
        .iter()
        .map(|entry| entry.component)
        .collect::<Vec<_>>();
    (!installed_after.is_empty() && recorded == installed).then_some(Reconciliation::FailedAmong {
        failed,
        installed_after,
    })
}

impl DebugStatus {
    fn is_success(self) -> bool {
        matches!(
            self,
            Self::SuccessfullyInstalled | Self::InstalledWithWarnings
        )
    }

    fn is_failure(self) -> bool {
        matches!(self, Self::NotInstalledDueToErrors | Self::Skipped)
    }
}

fn full_exit_matches(statuses: &[DebugStatus], exit_code: i32) -> bool {
    if statuses.contains(&DebugStatus::InstalledWithWarnings) {
        exit_code == 3
    } else {
        exit_code == 0
    }
}

fn same_active_identity(left: &LogEntry, right: &LogEntry) -> bool {
    left.tp2_key == right.tp2_key
        && left.language == right.language
        && left.component == right.component
}

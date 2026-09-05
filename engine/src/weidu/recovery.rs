//! Fail-closed planning for repairing an interrupted pure-install tail.

use std::collections::BTreeSet;

use thiserror::Error;

use super::log::{
    has_new_uninstall_comment, normalize_tp2_key, parse_active_entries, parse_terminal_statuses,
    DebugStatus, LogEntry,
};

/// Exact mod invocation whose partial result is being assessed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedInstall {
    /// Full TP2 path expected in every newly active row.
    pub tp2: String,
    /// WeiDU language number expected in every newly active row.
    pub language: u32,
    /// Unique component numbers in authored installation order.
    pub components: Vec<u32>,
}

/// Stable active-row identity retained for rollback verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveComponent {
    /// Case-folded, separator-normalized TP2 path.
    pub tp2_key: String,
    /// WeiDU language number.
    pub language: u32,
    /// WeiDU component number.
    pub component: u32,
    /// Exact optional annotation, which commonly carries version/name evidence.
    pub annotation: Option<String>,
}

/// Safe operation selected from the partial-install evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Existing verifier semantics apply: keep the strict successful prefix and install the rest.
    ContinueSuffix { remaining: Vec<u32> },
    /// Undo the whole installed top tail, then reinstall the complete authored component list.
    RollbackAndReinstall {
        /// Components ordered from current stack top downward for uninstall.
        installed_to_uninstall: Vec<u32>,
        /// Complete authored component sequence to install after rollback.
        reinstall: Vec<u32>,
    },
    /// Every requested component is already installed successfully.
    AlreadyComplete,
}

/// Recovery action together with the exact historical active stack it must restore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialTailRecoveryPlan {
    pub action: RecoveryAction,
    pub expected_rollback_prefix: Vec<ActiveComponent>,
}

/// Evidence that cannot safely produce or validate a partial-tail recovery.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RecoveryPlanError {
    #[error("expected install identity is empty or has duplicate components")]
    InvalidExpectedInstall,
    #[error("WeiDU.log evidence is malformed")]
    MalformedLog,
    #[error("the historical active stack changed before the candidate tail")]
    HistoricalPrefixChanged,
    #[error("the candidate tail contains a foreign, duplicate, or out-of-order component")]
    InvalidTail,
    #[error("terminal status evidence does not exactly match the requested components")]
    StatusMismatch,
    #[error("rollback verification requires a rollback-and-reinstall plan")]
    RollbackNotPlanned,
    #[error("rollback did not restore the exact historical active stack")]
    RollbackMismatch,
}

/// Plans recovery only when `after` is the unchanged `before` active stack followed by an
/// ordered subset of the expected mod's requested components.
pub fn plan_partial_tail_recovery(
    before: &str,
    after: &str,
    debug: &str,
    expected: &ExpectedInstall,
) -> Result<PartialTailRecoveryPlan, RecoveryPlanError> {
    if expected.tp2.is_empty()
        || expected.components.is_empty()
        || expected.components.iter().collect::<BTreeSet<_>>().len() != expected.components.len()
    {
        return Err(RecoveryPlanError::InvalidExpectedInstall);
    }

    let before_entries =
        parse_active_entries(before).map_err(|_| RecoveryPlanError::MalformedLog)?;
    let after_entries = parse_active_entries(after).map_err(|_| RecoveryPlanError::MalformedLog)?;
    if has_duplicate_active_rows(&before_entries)
        || has_duplicate_active_rows(&after_entries)
        || has_new_uninstall_comment(before, after)
    {
        return Err(RecoveryPlanError::InvalidTail);
    }
    let expected_prefix = before_entries
        .iter()
        .map(active_component)
        .collect::<Vec<_>>();
    if after_entries.len() < before_entries.len()
        || after_entries[..before_entries.len()]
            .iter()
            .map(active_component)
            .ne(expected_prefix.iter().cloned())
    {
        return Err(RecoveryPlanError::HistoricalPrefixChanged);
    }

    let expected_tp2 = normalize_tp2_key(&expected.tp2);
    let mut installed = Vec::new();
    let mut cursor = 0_usize;
    for entry in &after_entries[before_entries.len()..] {
        if entry.tp2_key != expected_tp2 || entry.language != expected.language {
            return Err(RecoveryPlanError::InvalidTail);
        }
        let Some(offset) = expected.components[cursor..]
            .iter()
            .position(|component| *component == entry.component)
        else {
            return Err(RecoveryPlanError::InvalidTail);
        };
        cursor += offset + 1;
        installed.push(entry.component);
    }

    let statuses = parse_terminal_statuses(debug);
    if statuses.len() != expected.components.len()
        || expected
            .components
            .iter()
            .zip(statuses)
            .any(|(component, status)| installed.contains(component) != status_is_success(status))
    {
        return Err(RecoveryPlanError::StatusMismatch);
    }

    let action = if installed.len() == expected.components.len() {
        RecoveryAction::AlreadyComplete
    } else if installed == expected.components[..installed.len()] {
        RecoveryAction::ContinueSuffix {
            remaining: expected.components[installed.len()..].to_vec(),
        }
    } else {
        RecoveryAction::RollbackAndReinstall {
            installed_to_uninstall: installed.iter().rev().copied().collect(),
            reinstall: expected.components.clone(),
        }
    };
    Ok(PartialTailRecoveryPlan {
        action,
        expected_rollback_prefix: expected_prefix,
    })
}

/// Proves that a planned rollback restored exactly the historical active stack.
/// Comment-only lines, including WeiDU's uninstall notes, are intentionally ignored.
pub fn verify_rollback(
    plan: &PartialTailRecoveryPlan,
    rollback_log: &str,
) -> Result<(), RecoveryPlanError> {
    if !matches!(plan.action, RecoveryAction::RollbackAndReinstall { .. }) {
        return Err(RecoveryPlanError::RollbackNotPlanned);
    }
    let actual = parse_active_entries(rollback_log)
        .map_err(|_| RecoveryPlanError::MalformedLog)?
        .iter()
        .map(active_component)
        .collect::<Vec<_>>();
    if actual != plan.expected_rollback_prefix {
        return Err(RecoveryPlanError::RollbackMismatch);
    }
    Ok(())
}

fn active_component(entry: &LogEntry) -> ActiveComponent {
    ActiveComponent {
        tp2_key: entry.tp2_key.clone(),
        language: entry.language,
        component: entry.component,
        annotation: entry.annotation.clone(),
    }
}

fn has_duplicate_active_rows(entries: &[LogEntry]) -> bool {
    let mut seen = BTreeSet::new();
    entries
        .iter()
        .any(|entry| !seen.insert((&entry.tp2_key, entry.language, entry.component)))
}

fn status_is_success(status: DebugStatus) -> bool {
    matches!(
        status,
        DebugStatus::SuccessfullyInstalled | DebugStatus::InstalledWithWarnings
    )
}

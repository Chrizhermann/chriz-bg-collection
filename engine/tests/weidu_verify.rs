use bg_engine::weidu::log::{parse_active_entries, DebugStatus};
use bg_engine::weidu::verify::{reconcile, ExpectedRun, Reconciliation};

const BEFORE: &str = include_str!("fixtures/weidu/before.log");
const COMPLETE: &str = include_str!("fixtures/weidu/complete.log");
const PARTIAL: &str = include_str!("fixtures/weidu/partial.log");
const UNEXPECTED_ADDITION: &str = include_str!("fixtures/weidu/unexpected-addition.log");
const REMOVAL: &str = include_str!("fixtures/weidu/removal.log");
const NEW_UNINSTALL_COMMENT: &str = include_str!("fixtures/weidu/new-uninstall-comment.log");
const DEBUG_OK: &str = include_str!("fixtures/weidu/debug-ok.log");
const DEBUG_WARNING: &str = include_str!("fixtures/weidu/debug-warning.log");
const DEBUG_PARTIAL: &str = include_str!("fixtures/weidu/debug-partial.log");
const DEBUG_RETRYABLE: &str = include_str!("fixtures/weidu/debug-retryable.log");
const DEBUG_INCOMPLETE: &str = include_str!("fixtures/weidu/debug-incomplete.log");

fn run(exit_code: i32) -> ExpectedRun {
    ExpectedRun {
        tp2: "testmod/setup-testmod.tp2".to_owned(),
        language: 0,
        components: vec![0, 10],
        exit_code,
    }
}

#[test]
fn parser_preserves_annotations_paths_and_line_evidence() {
    let entries = parse_active_entries(COMPLETE).unwrap();
    let last = entries.last().unwrap();

    assert_eq!(entries.len(), 4);
    assert_eq!(last.tp2, "TESTMOD/SETUP-TESTMOD.TP2");
    assert_eq!(last.tp2_key, "testmod/setup-testmod.tp2");
    assert_eq!(last.language, 0);
    assert_eq!(last.component, 10);
    assert_eq!(last.annotation.as_deref(), Some("Expected component name"));
    assert!(last.line_number > 1);
    assert!(last.source_line.contains("#10"));
}

#[test]
fn parser_skips_blank_headers_and_old_recently_uninstalled_comments() {
    let entries = parse_active_entries(BEFORE).unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].component, 1);
    assert_eq!(entries[1].component, 0);
}

#[test]
fn malformed_active_entry_is_not_silently_ignored() {
    let error = parse_active_entries("~BROKEN/SETUP-BROKEN.TP2~ #0\n").unwrap_err();

    assert_eq!(error.line_number(), 1);
    assert!(error.to_string().contains("component"), "{error}");
}

#[test]
fn exact_tail_and_complete_success_markers_prove_done() {
    assert_eq!(
        reconcile(BEFORE, COMPLETE, DEBUG_OK, &run(0)),
        Reconciliation::ProvenDone
    );
}

#[test]
fn warning_success_requires_the_warning_marker_and_exact_tail() {
    assert_eq!(
        reconcile(BEFORE, COMPLETE, DEBUG_WARNING, &run(3)),
        Reconciliation::ProvenDone
    );
    assert_eq!(
        reconcile(BEFORE, COMPLETE, DEBUG_OK, &run(3)),
        Reconciliation::Ambiguous
    );
    assert_eq!(
        reconcile(BEFORE, COMPLETE, DEBUG_INCOMPLETE, &run(3)),
        Reconciliation::Ambiguous
    );
}

#[test]
fn strict_successful_prefix_returns_only_the_remaining_suffix() {
    assert_eq!(
        reconcile(BEFORE, PARTIAL, DEBUG_PARTIAL, &run(2)),
        Reconciliation::PartialPrefix {
            remaining: vec![10]
        }
    );
}

#[test]
fn unchanged_log_is_retryable_only_with_complete_failure_evidence() {
    assert_eq!(
        reconcile(BEFORE, BEFORE, DEBUG_RETRYABLE, &run(2)),
        Reconciliation::UnchangedRetryable
    );
    assert_eq!(
        reconcile(BEFORE, BEFORE, "", &run(0)),
        Reconciliation::Ambiguous
    );
}

#[test]
fn exit_zero_cannot_promote_missing_log_evidence_to_success() {
    assert_eq!(
        reconcile(BEFORE, BEFORE, DEBUG_OK, &run(0)),
        Reconciliation::Ambiguous
    );
}

#[test]
fn additions_removals_and_new_uninstall_comments_disturb_the_stack() {
    for after in [UNEXPECTED_ADDITION, REMOVAL, NEW_UNINSTALL_COMMENT] {
        assert_eq!(
            reconcile(BEFORE, after, DEBUG_OK, &run(0)),
            Reconciliation::StackDisturbed
        );
    }
}

#[test]
fn terminal_markers_ignore_unrelated_debug_noise() {
    let debug = "PARSE ERROR in unrelated METADATA\nSUCCESSFULLY INSTALLED Core\n";
    let statuses = bg_engine::weidu::log::parse_terminal_statuses(debug);

    assert_eq!(statuses, vec![DebugStatus::SuccessfullyInstalled]);
}

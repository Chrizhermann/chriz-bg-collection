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
/// The transcript of WeiDU 249 itself, with `--force-install-list 0 1 2 --safe-exit`, whose
/// component 1 fails: it goes on with component 2 and exits with 2.
const DEBUG_WENT_PAST: &str = include_str!("fixtures/weidu/debug-went-past.log");

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
fn setup_name_requires_the_exact_nested_tp2_path_logged_by_weidu() {
    let after = format!(
        "{BEFORE}~CHRIZ-SOD-REMIX/SETUP-CHRIZ-SOD-REMIX.TP2~ #0 #100 // first\n\
         ~CHRIZ-SOD-REMIX/SETUP-CHRIZ-SOD-REMIX.TP2~ #0 #900 // last\n"
    );
    let debug = "SUCCESSFULLY INSTALLED first\nSUCCESSFULLY INSTALLED last\n";
    let nested = ExpectedRun {
        tp2: "chriz-sod-remix/setup-chriz-sod-remix.tp2".to_owned(),
        language: 0,
        components: vec![100, 900],
        exit_code: 0,
    };

    assert_eq!(
        reconcile(BEFORE, &after, debug, &nested),
        Reconciliation::ProvenDone
    );

    let root_alias = ExpectedRun {
        tp2: "setup-chriz-sod-remix.tp2".to_owned(),
        ..nested
    };
    assert_eq!(
        reconcile(BEFORE, &after, debug, &root_alias),
        Reconciliation::StackDisturbed
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

/// WeiDU goes on after a failed component, verified with WeiDU 249 and `--force-install-list`:
/// the later components install, each requested component has a terminal line, and it exits 2.
#[test]
fn weidu_going_on_after_a_failed_component_is_no_disturbed_stack() {
    let log = |components: &[u32]| {
        let mut log = BEFORE.to_owned();
        for component in components {
            log.push_str(&format!(
                "~TestMod/Setup-TestMod.tp2~ #0 #{component} // Component {component}\n"
            ));
        }
        log
    };
    let expected = |exit_code: i32| ExpectedRun {
        tp2: "testmod/setup-testmod.tp2".to_owned(),
        language: 0,
        components: vec![0, 5, 10, 15],
        exit_code,
    };
    let debug = "\
Automatically Skipping [Five] because of error.
SUCCESSFULLY INSTALLED      Zero
NOT INSTALLED DUE TO ERRORS Five
SUCCESSFULLY INSTALLED      Ten
NOT INSTALLED DUE TO ERRORS Fifteen
";

    assert_eq!(
        reconcile(BEFORE, &log(&[0, 10]), debug, &expected(2)),
        Reconciliation::FailedAmong {
            failed: vec![5, 15],
            installed_after: vec![10],
        }
    );
    // The log must say what the terminal lines say, in order, and the exit code that it failed.
    for after in [log(&[10, 0]), log(&[0]), log(&[0, 5, 10]), log(&[0, 15])] {
        assert_ne!(
            reconcile(BEFORE, &after, debug, &expected(2)),
            Reconciliation::FailedAmong {
                failed: vec![5, 15],
                installed_after: vec![10],
            },
            "{after}"
        );
    }
    assert_eq!(
        reconcile(BEFORE, &log(&[0, 10]), debug, &expected(0)),
        Reconciliation::StackDisturbed
    );
    // A failure with nothing installed after it stays a strict prefix.
    let at_the_end = "\
SUCCESSFULLY INSTALLED      Zero
SUCCESSFULLY INSTALLED      Five
NOT INSTALLED DUE TO ERRORS Ten
NOT INSTALLED DUE TO ERRORS Fifteen
";
    assert_eq!(
        reconcile(BEFORE, &log(&[0, 5]), at_the_end, &expected(2)),
        Reconciliation::PartialPrefix {
            remaining: vec![10, 15],
        }
    );
}

#[test]
fn the_real_transcript_of_weidu_going_on_after_a_failure_is_recognized() {
    let after = format!(
        "{BEFORE}~TESTMOD/SETUP-TESTMOD.TP2~ #0 #0 // First\n~TESTMOD/SETUP-TESTMOD.TP2~ #0 #2 // Third\n"
    );
    let expected = ExpectedRun {
        tp2: "testmod/setup-testmod.tp2".to_owned(),
        language: 0,
        components: vec![0, 1, 2],
        exit_code: 2,
    };

    assert_eq!(
        reconcile(BEFORE, &after, DEBUG_WENT_PAST, &expected),
        Reconciliation::FailedAmong {
            failed: vec![1],
            installed_after: vec![2],
        }
    );
    assert_eq!(
        bg_engine::weidu::log::failure_message(DEBUG_WENT_PAST).as_deref(),
        Some("Failure(\"synthetic failure of the second component\")")
    );
}

/// The player sees the installer's own last error before the failed component, not the noise
/// of an earlier one, of what follows it, or WeiDU's rollback line.
#[test]
fn the_failure_message_is_the_last_error_before_the_failed_component() {
    let debug = "\
ERROR: an earlier error the installer recovered from
SUCCESSFULLY INSTALLED Core
Copying 3 files ...
ERROR: cannot convert variable to an integer
Stopping installation because of error.
ERROR Installing [Balance], rolling back to previous state
ERROR: Failure(\"cannot convert variable to an integer\")
PLEASE email the file SETUP.DEBUG to the author
NOT INSTALLED DUE TO ERRORS Balance
SKIPPING: Later component
ERROR: noise after the component
";
    assert_eq!(
        bg_engine::weidu::log::failure_message(debug).as_deref(),
        Some("Failure(\"cannot convert variable to an integer\")")
    );

    assert_eq!(
        bg_engine::weidu::log::failure_message("NOT INSTALLED DUE TO ERRORS Balance\n"),
        None
    );
    assert_eq!(
        bg_engine::weidu::log::failure_message("ERROR: no failed component here\n"),
        None
    );

    let long = format!(
        "ERROR: {}\nNOT INSTALLED DUE TO ERRORS Balance\n",
        "x".repeat(400)
    );
    let message = bg_engine::weidu::log::failure_message(&long).unwrap();
    assert_eq!(message.chars().count(), 301);
    assert!(message.ends_with('…'));
}

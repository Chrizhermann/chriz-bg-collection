use std::path::PathBuf;

use bg_engine::error::EngineError;
use bg_engine::manifest::{
    AcquisitionPolicy, Component, ComponentRef, Conflict, Decision, Feature, FeatureInputRef,
    InputOption, InputSpec, InvocationMode, Phase, Postcondition, PromptAnswer, PromptStep,
    Readiness, Run, SourceKind,
};
use bg_engine::validate::{check, validate, Finding, Severity};
use bg_engine::Manifest;

fn good() -> Manifest {
    Manifest::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest"))
        .unwrap()
}

fn run(run_id: &str, phase: Phase, components: &[u32]) -> Run {
    Run {
        run_id: run_id.to_owned(),
        mod_id: "eefixpack".to_owned(),
        phase,
        components: components.to_vec(),
        args: Vec::new(),
        postconditions: Vec::new(),
    }
}

fn feature(id: &str, decision: Decision) -> Feature {
    Feature {
        id: id.to_owned(),
        title: id.to_owned(),
        description: format!("Description for {id}"),
        category: "test".to_owned(),
        source_label: None,
        group_label: None,
        choice_group: None,
        decision,
        readiness: Readiness::Ready,
        unavailable_reason: None,
        parent: None,
        components: Vec::new(),
        requires: Vec::new(),
        conflicts: Vec::new(),
        inputs: Vec::new(),
    }
}

fn error_rules(findings: &[Finding]) -> Vec<&str> {
    findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .map(|finding| finding.rule)
        .collect()
}

fn text_file_markers(
    path: &str,
    required: &[&str],
    forbidden: &[&str],
    max_bytes: u64,
) -> Postcondition {
    Postcondition::TextFileMarkers {
        path: path.to_owned(),
        required: required.iter().map(|marker| (*marker).to_owned()).collect(),
        forbidden: forbidden
            .iter()
            .map(|marker| (*marker).to_owned())
            .collect(),
        max_bytes,
    }
}

#[track_caller]
fn assert_has_error(findings: &[Finding], rule: &str) {
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule == rule && finding.severity == Severity::Error),
        "expected error rule {rule:?}, got {findings:#?}"
    );
}

#[test]
fn fixture_is_valid_and_repeats_one_component_across_game_roots() {
    let manifest = good();

    assert_eq!(manifest.collection.runs[0].components, vec![0, 2]);
    assert_eq!(manifest.collection.runs[1].components, vec![0]);
    assert_eq!(error_rules(&validate(&manifest)), Vec::<&str>::new());
}

#[test]
fn collection_must_contain_at_least_one_explicit_run() {
    let mut manifest = good();
    manifest.collection.runs.clear();

    let findings = validate(&manifest);

    assert_has_error(&findings, "nonempty");
}

#[test]
fn installer_must_declare_at_least_one_component() {
    let mut manifest = good();
    manifest
        .mods
        .get_mut("eefixpack")
        .unwrap()
        .components
        .clear();

    let findings = validate(&manifest);

    assert_has_error(&findings, "nonempty");
}

#[test]
fn setup_name_aliases_are_rejected_during_recipe_validation() {
    let mut manifest = good();
    let original = manifest.mods.get_mut("eefixpack").unwrap();
    original.invocation_mode = InvocationMode::SetupName;
    let mut alias = original.clone();
    alias.id = "eefixpack-alias".to_owned();
    alias.tp2 = "other/setup-EE_Fixpack.tp2".to_owned();
    manifest.mods.insert(alias.id.clone(), alias);

    let findings = validate(&manifest);

    assert_has_error(&findings, "setup-name-ambiguity");
}

#[test]
fn run_ids_are_globally_unique() {
    let mut manifest = good();
    manifest.collection.runs[1].run_id = manifest.collection.runs[0].run_id.clone();

    let findings = validate(&manifest);

    assert_has_error(&findings, "run-ids");
}

#[test]
fn every_run_has_an_explicit_nonempty_component_list() {
    let mut manifest = good();
    manifest.collection.runs[0].components.clear();

    let findings = validate(&manifest);

    assert_has_error(&findings, "run-components");
}

#[test]
fn text_marker_postcondition_accepts_a_bounded_normalized_target_relative_path() {
    let mut manifest = good();
    manifest.collection.runs[0]
        .postconditions
        .push(text_file_markers(
            "override/ui.menu",
            &["required marker"],
            &["forbidden marker"],
            4096,
        ));

    assert!(!validate(&manifest)
        .iter()
        .any(|finding| finding.rule == "run-postconditions"));
}

#[test]
fn text_marker_postcondition_rejects_unsafe_or_non_normalized_paths() {
    for path in [
        "",
        "/weidu.conf",
        "C:/weidu.conf",
        "../weidu.conf",
        "override/../weidu.conf",
        "override\\weidu.conf",
        "override//weidu.conf",
        "./weidu.conf",
        "override/./weidu.conf",
        "override/weidu.conf:stream",
        "override/trailing.",
        "override/trailing ",
        "override/NUL.txt",
        "override/control\u{0001}.txt",
        "override/wild*.txt",
        "override/question?.txt",
        "override/pipe|.txt",
        "override/quote\".txt",
        "override/less<than.txt",
        "override/greater>than.txt",
    ] {
        let mut manifest = good();
        manifest.collection.runs[0]
            .postconditions
            .push(text_file_markers(path, &["marker"], &[], 4096));

        assert_has_error(&validate(&manifest), "run-postconditions");
    }
}

#[test]
fn text_marker_postcondition_rejects_empty_markers_and_unbounded_reads() {
    for postcondition in [
        text_file_markers("weidu.conf", &[], &[], 4096),
        text_file_markers("weidu.conf", &[""], &[], 4096),
        text_file_markers("weidu.conf", &["marker"], &[""], 4096),
        text_file_markers("weidu.conf", &["marker"], &[], 0),
        text_file_markers("weidu.conf", &["marker"], &[], 64 * 1024 * 1024 + 1),
        text_file_markers("weidu.conf", &["marker"], &["marker"], 4096),
        text_file_markers("weidu.conf", &["fives"], &[], 4),
    ] {
        let mut manifest = good();
        manifest.collection.runs[0]
            .postconditions
            .push(postcondition);

        assert_has_error(&validate(&manifest), "run-postconditions");
    }
}

#[test]
fn run_must_reference_an_existing_installer() {
    let mut manifest = good();
    manifest.collection.runs[0].mod_id = "missing-installer".to_owned();

    let findings = validate(&manifest);

    assert_has_error(&findings, "references");
}

#[test]
fn installer_must_reference_an_existing_payload_artifact() {
    let mut manifest = good();
    manifest.mods.get_mut("eefixpack").unwrap().artifact_id = "missing-payload".to_owned();

    let findings = validate(&manifest);

    assert_has_error(&findings, "references");
}

#[test]
fn installer_must_reference_an_existing_weidu_artifact() {
    let mut manifest = good();
    manifest
        .mods
        .get_mut("eefixpack")
        .unwrap()
        .weidu_artifact_id = "missing-weidu".to_owned();

    let findings = validate(&manifest);

    assert_has_error(&findings, "references");
}

#[test]
fn run_cannot_use_a_blocked_payload_artifact() {
    let mut manifest = good();
    manifest.artifacts.get_mut("eefixpack").unwrap().acquisition = AcquisitionPolicy::Blocked;

    let findings = validate(&manifest);

    assert_has_error(&findings, "blocked-artifacts");
}

#[test]
fn run_cannot_use_a_blocked_weidu_artifact() {
    let mut manifest = good();
    manifest.artifacts.get_mut("weidu").unwrap().acquisition = AcquisitionPolicy::Blocked;

    let findings = validate(&manifest);

    assert_has_error(&findings, "blocked-artifacts");
}

#[test]
fn unused_blocked_artifact_declaration_is_legal() {
    let mut manifest = good();
    let mut unused = manifest.artifacts["eefixpack"].clone();
    unused.id = "unused-blocked".to_owned();
    unused.acquisition = AcquisitionPolicy::Blocked;
    manifest.artifacts.insert(unused.id.clone(), unused);

    assert_eq!(error_rules(&validate(&manifest)), Vec::<&str>::new());
}

#[test]
fn run_components_must_be_declared_by_the_installer() {
    let mut manifest = good();
    manifest.collection.runs[0].components.push(999);

    let findings = validate(&manifest);

    assert_has_error(&findings, "references");
}

#[test]
fn tp2_path_must_be_relative_and_traversal_free() {
    for path in ["C:\\games\\setup.tp2", "../outside/setup.tp2"] {
        let mut manifest = good();
        manifest.mods.get_mut("eefixpack").unwrap().tp2 = path.to_owned();

        let findings = validate(&manifest);

        assert_has_error(&findings, "paths");
    }
}

#[test]
fn archive_path_must_be_relative_and_traversal_free() {
    for path in ["C:\\downloads\\payload", "../payload"] {
        let mut manifest = good();
        manifest
            .artifacts
            .get_mut("eefixpack")
            .unwrap()
            .archive
            .publish_roots = vec![path.to_owned()];

        let findings = validate(&manifest);

        assert_has_error(&findings, "paths");
    }
}

#[test]
fn tp2_path_rejects_nested_windows_ads_components() {
    for path in ["mods/setup.tp2:stream", "mods\\setup.tp2::$DATA"] {
        let mut manifest = good();
        manifest.mods.get_mut("eefixpack").unwrap().tp2 = path.to_owned();

        let findings = validate(&manifest);

        assert_has_error(&findings, "paths");
    }
}

#[test]
fn archive_path_rejects_nested_windows_ads_components() {
    for path in ["payload/file:stream", "payload\\file::$DATA"] {
        let mut manifest = good();
        manifest
            .artifacts
            .get_mut("eefixpack")
            .unwrap()
            .archive
            .publish_roots = vec![path.to_owned()];

        let findings = validate(&manifest);

        assert_has_error(&findings, "paths");
    }
}

#[test]
fn one_component_cannot_repeat_on_one_game_root() {
    let mut manifest = good();
    manifest
        .collection
        .runs
        .push(run("eefixpack-bg1-again", Phase::Bg1Preparation, &[2]));

    let findings = validate(&manifest);

    assert_has_error(&findings, "component-placement");
}

#[test]
fn one_run_cannot_repeat_a_component_number() {
    let mut manifest = good();
    manifest.collection.runs[0].components.push(2);

    let findings = validate(&manifest);

    assert_has_error(&findings, "component-placement");
}

#[test]
fn phase_order_must_be_monotonic() {
    let mut manifest = good();
    manifest.collection.runs[0].phase = Phase::Bg2Preparation;
    manifest.collection.runs[1].phase = Phase::Bg1Preparation;

    let findings = validate(&manifest);

    assert_has_error(&findings, "phase-order");
}

#[test]
fn eet_initialization_is_required_first_after_preparation() {
    let mut manifest = good();
    manifest.collection.runs = vec![run("ordinary-main", Phase::Main, &[0])];

    let findings = validate(&manifest);

    assert_has_error(&findings, "eet-anchors");
}

#[test]
fn eet_finalization_is_required_last_before_tail_runs() {
    let mut manifest = good();
    manifest.collection.runs = vec![
        run("eet-init", Phase::EetInitialization, &[0]),
        run("ordinary-main", Phase::Main, &[2]),
        run("tail", Phase::PostEetEnd, &[0]),
    ];

    let findings = validate(&manifest);

    assert_has_error(&findings, "eet-anchors");
}

#[test]
fn finalization_then_explicit_tail_is_valid() {
    let mut manifest = good();
    manifest
        .mods
        .get_mut("eefixpack")
        .unwrap()
        .components
        .extend([3, 4].map(|id| Component {
            id,
            name: format!("Synthetic component {id}"),
            stdin: None,
            prompts: Vec::new(),
        }));
    manifest.collection.runs = vec![
        run("eet-init", Phase::EetInitialization, &[0]),
        run("ordinary-main", Phase::Main, &[2]),
        run("eet-final", Phase::EetFinalization, &[3]),
        run("tail", Phase::PostEetEnd, &[4]),
    ];

    let findings = validate(&manifest);

    assert_eq!(error_rules(&findings), Vec::<&str>::new(), "{findings:#?}");
}

#[test]
fn duplicate_installer_component_ids_remain_invalid() {
    let mut manifest = good();
    manifest
        .mods
        .get_mut("eefixpack")
        .unwrap()
        .components
        .push(Component {
            id: 0,
            name: "Core again".to_owned(),
            stdin: None,
            prompts: Vec::new(),
        });

    let findings = validate(&manifest);

    assert_has_error(&findings, "component-ids-unique");
}

#[test]
fn fetched_artifact_url_must_use_https() {
    let mut manifest = good();
    manifest.artifacts.get_mut("eefixpack").unwrap().source.url =
        "http://example.invalid/archive.zip".to_owned();

    let findings = validate(&manifest);

    assert_has_error(&findings, "sources");
}

#[test]
fn fetch_only_manual_source_is_inconsistent_and_cannot_bypass_integrity_checks() {
    let mut manifest = good();
    let artifact = manifest.artifacts.get_mut("eefixpack").unwrap();
    artifact.acquisition = AcquisitionPolicy::FetchOnly;
    artifact.source.kind = SourceKind::Manual;
    artifact.source.url = "http://example.invalid/download-page".to_owned();
    artifact.source.sha256 = "pending".to_owned();

    let findings = validate(&manifest);

    assert_has_error(&findings, "acquisition-policy");
    assert_has_error(&findings, "sources");
}

#[test]
fn manual_user_supplied_policy_controls_source_relaxation() {
    let mut manifest = good();
    let artifact = manifest.artifacts.get_mut("eefixpack").unwrap();
    artifact.acquisition = AcquisitionPolicy::ManualUserSupplied;
    artifact.source.url = "http://example.invalid/download-page".to_owned();
    artifact.source.sha256 = "pending-user-supplied-file".to_owned();

    assert_eq!(error_rules(&validate(&manifest)), Vec::<&str>::new());
}

#[test]
fn bundle_permitted_artifact_still_requires_https_and_sha256() {
    let mut manifest = good();
    let artifact = manifest.artifacts.get_mut("eefixpack").unwrap();
    artifact.acquisition = AcquisitionPolicy::BundlePermitted;
    artifact.source.url = "http://example.invalid/archive.zip".to_owned();
    artifact.source.sha256 = "pending".to_owned();

    let findings = validate(&manifest);

    assert_has_error(&findings, "sources");
}

#[test]
fn fetched_artifact_sha256_must_be_exactly_hexadecimal() {
    for sha256 in ["a".repeat(63), "g".repeat(64), "a".repeat(65)] {
        let mut manifest = good();
        manifest
            .artifacts
            .get_mut("eefixpack")
            .unwrap()
            .source
            .sha256 = sha256;

        let findings = validate(&manifest);

        assert_has_error(&findings, "sources");
    }
}

#[test]
fn uppercase_sha256_is_accepted() {
    let mut manifest = good();
    manifest
        .artifacts
        .get_mut("eefixpack")
        .unwrap()
        .source
        .sha256 = "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855".to_owned();

    assert_eq!(error_rules(&validate(&manifest)), Vec::<&str>::new());
}

#[test]
fn manual_user_supplied_artifact_may_use_an_authoring_placeholder() {
    let mut manifest = good();
    let artifact = manifest.artifacts.get_mut("eefixpack").unwrap();
    artifact.acquisition = AcquisitionPolicy::ManualUserSupplied;
    let source = &mut artifact.source;
    source.kind = SourceKind::Manual;
    source.url = "http://example.invalid/download-page".to_owned();
    source.sha256 = "pending-user-supplied-file".to_owned();

    assert_eq!(error_rules(&validate(&manifest)), Vec::<&str>::new());
}

#[test]
fn all_zero_fetched_digest_remains_an_authoring_warning() {
    let mut manifest = good();
    manifest
        .artifacts
        .get_mut("eefixpack")
        .unwrap()
        .source
        .sha256 = "0".repeat(64);

    let findings = validate(&manifest);

    assert_eq!(error_rules(&findings), Vec::<&str>::new());
    assert!(findings.iter().any(|finding| {
        finding.rule == "unpinned-source" && finding.severity == Severity::Warning
    }));
}

#[test]
fn check_accepts_a_recipe_with_only_warnings() {
    let mut manifest = good();
    manifest
        .artifacts
        .get_mut("eefixpack")
        .unwrap()
        .source
        .sha256 = "0".repeat(64);

    assert!(check(&manifest).is_ok());
}

#[test]
fn legacy_stdin_without_newline_remains_a_warning() {
    let mut manifest = good();
    manifest.mods.get_mut("eefixpack").unwrap().components[1].stdin = Some("1".to_owned());

    let findings = validate(&manifest);

    assert!(findings.iter().any(|finding| {
        finding.rule == "stdin-newline" && finding.severity == Severity::Warning
    }));
    assert_eq!(error_rules(&findings), Vec::<&str>::new());
}

#[test]
fn check_returns_all_independent_findings() {
    let mut manifest = good();
    manifest.collection.runs[0].run_id = manifest.collection.runs[1].run_id.clone();
    manifest.mods.get_mut("eefixpack").unwrap().tp2 = "../escape.tp2".to_owned();

    let error = check(&manifest).unwrap_err();
    let EngineError::Validation(findings) = error else {
        panic!("expected validation error");
    };

    assert_has_error(&findings, "run-ids");
    assert_has_error(&findings, "paths");
}

#[test]
fn validation_error_preserves_warnings_alongside_errors() {
    let mut manifest = good();
    manifest.collection.runs[0].run_id = manifest.collection.runs[1].run_id.clone();
    manifest
        .artifacts
        .get_mut("eefixpack")
        .unwrap()
        .source
        .sha256 = "0".repeat(64);

    let error = check(&manifest).unwrap_err();
    let EngineError::Validation(findings) = &error else {
        panic!("expected validation error, got {error:?}");
    };

    assert_has_error(findings, "run-ids");
    assert!(findings.iter().any(|finding| {
        finding.rule == "unpinned-source" && finding.severity == Severity::Warning
    }));
    assert_eq!(error.to_string().lines().count(), findings.len());
}

#[test]
fn findings_have_exact_stable_rule_order() {
    let mut manifest = good();
    manifest.collection.runs[0].run_id = manifest.collection.runs[1].run_id.clone();
    manifest.collection.runs[0].components.clear();
    manifest.mods.get_mut("eefixpack").unwrap().tp2 = "../escape.tp2".to_owned();
    manifest
        .artifacts
        .get_mut("eefixpack")
        .unwrap()
        .source
        .sha256 = "0".repeat(64);
    manifest.mods.get_mut("eefixpack").unwrap().components[1].stdin = Some("1".to_owned());

    let findings = validate(&manifest);

    assert_eq!(findings, validate(&manifest));
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.rule)
            .collect::<Vec<_>>(),
        vec![
            "run-ids",
            "run-components",
            "paths",
            "unpinned-source",
            "stdin-newline",
        ]
    );
}

#[test]
fn feature_parents_and_requirements_must_exist() {
    let mut manifest = good();
    let mut child = feature("child", Decision::Optional);
    child.parent = Some("missing-parent".to_owned());
    child.requires.push("missing-requirement".to_owned());
    manifest.collection.features.push(child);

    let findings = validate(&manifest);

    assert_has_error(&findings, "feature-references");
}

#[test]
fn root_mandatory_feature_is_valid() {
    let mut manifest = good();
    manifest
        .collection
        .features
        .push(feature("required-collection-core", Decision::Mandatory));

    let findings = validate(&manifest);

    assert_eq!(error_rules(&findings), Vec::<&str>::new(), "{findings:#?}");
}

#[test]
fn feature_parent_and_requirement_cycles_are_rejected() {
    let mut manifest = good();
    let mut first = feature("first", Decision::Default);
    first.parent = Some("second".to_owned());
    let mut second = feature("second", Decision::Mandatory);
    second.requires.push("first".to_owned());
    manifest.collection.features = vec![first, second];

    let findings = validate(&manifest);

    assert_has_error(&findings, "feature-cycles");
}

#[test]
fn directly_conflicting_default_features_are_rejected() {
    let mut manifest = good();
    let mut first = feature("first", Decision::Default);
    first.conflicts.push(Conflict {
        feature_id: "second".to_owned(),
        reason: "Choose one default.".to_owned(),
    });
    manifest.collection.features = vec![first, feature("second", Decision::Default)];

    let findings = validate(&manifest);

    assert_has_error(&findings, "feature-default-conflicts");
}

#[test]
fn one_run_component_has_exactly_one_feature_owner() {
    let mut manifest = good();
    let owned = ComponentRef {
        run_id: "eefixpack-bg1".to_owned(),
        component: 0,
    };
    let mut first = feature("first", Decision::Default);
    first.components.push(owned.clone());
    let mut second = feature("second", Decision::Optional);
    second.components.push(owned);
    manifest.collection.features = vec![first, second];

    let findings = validate(&manifest);

    assert_has_error(&findings, "feature-component-ownership");
}

#[test]
fn feature_components_must_reference_declared_run_components() {
    let mut manifest = good();
    let mut selected = feature("selected", Decision::Default);
    selected.components.push(ComponentRef {
        run_id: "missing-run".to_owned(),
        component: 999,
    });
    manifest.collection.features.push(selected);

    let findings = validate(&manifest);

    assert_has_error(&findings, "feature-references");
}

#[test]
fn input_defaults_and_bounds_are_validated() {
    let mut manifest = good();
    let mut configured = feature("configured", Decision::Default);
    configured.inputs = vec![
        InputSpec::Choice {
            id: "choice".to_owned(),
            default: "missing".to_owned(),
            options: vec![InputOption {
                id: "none".to_owned(),
                title: "None".to_owned(),
                answer: String::new(),
            }],
        },
        InputSpec::Integer {
            id: "count".to_owned(),
            default: 11,
            min: 1,
            max: 10,
        },
    ];
    manifest.collection.features.push(configured);

    let findings = validate(&manifest);

    assert_has_error(&findings, "feature-inputs");
}

#[test]
fn prompt_input_references_must_name_a_declared_feature_input() {
    let mut manifest = good();
    manifest.mods.get_mut("eefixpack").unwrap().components[0]
        .prompts
        .push(PromptStep {
            expected_output: "Choose".to_owned(),
            answer: PromptAnswer::Input(FeatureInputRef {
                feature_id: "missing-feature".to_owned(),
                input_id: "missing-input".to_owned(),
            }),
            when_any_features: Vec::new(),
        });

    let findings = validate(&manifest);

    assert_has_error(&findings, "prompt-input-references");
}

#[test]
fn prompt_conditions_must_name_declared_features_exactly() {
    let mut manifest = good();
    manifest.mods.get_mut("eefixpack").unwrap().components[0]
        .prompts
        .push(PromptStep {
            expected_output: "Choose".to_owned(),
            answer: PromptAnswer::Literal(bg_engine::manifest::InputValue::Boolean(true)),
            when_any_features: vec!["feature-with-a-typo".to_owned()],
        });

    let findings = validate(&manifest);

    assert_has_error(&findings, "feature-references");
    assert!(findings.iter().any(|finding| {
        finding.rule == "feature-references"
            && finding.message.contains("feature-with-a-typo")
            && finding.message.contains("prompt")
    }));
}

#[test]
fn blocked_visible_features_require_an_authored_reason() {
    let mut manifest = good();
    let mut blocked = feature("blocked", Decision::Default);
    blocked.readiness = Readiness::Blocked;
    manifest.collection.features.push(blocked);

    let findings = validate(&manifest);

    assert_has_error(&findings, "feature-readiness");
}

#[test]
fn finding_display_includes_severity_rule_and_message() {
    let finding = Finding {
        severity: Severity::Error,
        rule: "references",
        message: "boom".to_owned(),
    };

    assert_eq!(finding.to_string(), "error[references]: boom");
}

#[test]
fn warning_display_includes_severity_rule_and_message() {
    let finding = Finding {
        severity: Severity::Warning,
        rule: "unpinned-source",
        message: "digest is pending".to_owned(),
    };

    assert_eq!(
        finding.to_string(),
        "warning[unpinned-source]: digest is pending"
    );
}

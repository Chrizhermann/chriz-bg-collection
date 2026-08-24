use std::path::PathBuf;

use bg_engine::error::EngineError;
use bg_engine::manifest::{ChoiceGroup, ChoiceOption, Component, ComponentRef, OrderEntry, Phase};
use bg_engine::validate::{check, validate, Finding, Severity};
use bg_engine::Manifest;

/// sha256 of the empty input - any real-looking pin works here.
const REAL_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

fn good() -> Manifest {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest");
    Manifest::load(&root).unwrap()
}

/// Pins every source so `unpinned-source` warnings do not mask the assertion
/// under test.
fn pinned() -> Manifest {
    let mut manifest = good();
    for mod_file in manifest.mods.values_mut() {
        mod_file.source.sha256 = REAL_SHA256.to_owned();
    }
    manifest
}

fn rules_of(findings: &[Finding], severity: Severity) -> Vec<&str> {
    findings
        .iter()
        .filter(|finding| finding.severity == severity)
        .map(|finding| finding.rule)
        .collect()
}

#[track_caller]
fn assert_has(findings: &[Finding], rule: &str, severity: Severity) {
    assert!(
        findings
            .iter()
            .any(|finding| finding.rule == rule && finding.severity == severity),
        "expected a {severity:?} finding for rule {rule:?}, got {findings:#?}"
    );
}

#[track_caller]
fn assert_error_rules(findings: &[Finding], expected: &[&str]) {
    assert_eq!(
        rules_of(findings, Severity::Error),
        expected,
        "unexpected error findings: {findings:#?}"
    );
}

fn component_ref(mod_id: &str, component: u32) -> ComponentRef {
    ComponentRef {
        mod_id: mod_id.to_owned(),
        component,
    }
}

fn order_entry(id: &str, components: Option<Vec<u32>>) -> OrderEntry {
    OrderEntry {
        id: id.to_owned(),
        components,
    }
}

// ---------------------------------------------------------------- good fixture

#[test]
fn good_fixture_has_no_errors() {
    let findings = validate(&good());

    assert_error_rules(&findings, &[]);
}

#[test]
fn good_fixture_warnings_are_exactly_the_unpinned_sources() {
    let findings = validate(&good());

    // Both fixture mods carry an all-zero sha256, and the only `stdin` in the
    // fixture ("1\n") is newline-terminated.
    assert_eq!(
        rules_of(&findings, Severity::Warning),
        vec!["unpinned-source", "unpinned-source"]
    );
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings[0].message.contains("\"eet\""), "{findings:#?}");
    assert!(findings[1].message.contains("\"testmod\""), "{findings:#?}");
}

#[test]
fn check_accepts_a_manifest_with_only_warnings() {
    assert!(check(&good()).is_ok());
}

// ------------------------------------------------------------ 1. order-refs

#[test]
fn order_entry_naming_an_unknown_mod_is_an_error() {
    let mut manifest = pinned();
    manifest.collection.order.push(order_entry("nope", None));

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["order-refs"]);
    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("nope")),
        "{findings:#?}"
    );
}

#[test]
fn mod_missing_from_the_order_is_an_error() {
    let mut manifest = pinned();
    manifest
        .collection
        .order
        .retain(|entry| entry.id != "testmod");

    let findings = validate(&manifest);

    assert_has(&findings, "order-refs", Severity::Error);
}

#[test]
fn split_mod_entry_without_components_is_an_error() {
    let mut manifest = pinned();
    manifest.collection.order[1].components = None;

    let findings = validate(&manifest);

    // An "all components" entry also puts component 10 in two slots at once,
    // so `option-slot` legitimately fires alongside `order-refs` here.
    assert_has(&findings, "order-refs", Severity::Error);
}

#[test]
fn split_mod_entries_must_be_disjoint() {
    let mut manifest = pinned();
    manifest.collection.order[1].components = Some(vec![0, 10]);

    let findings = validate(&manifest);

    assert_has(&findings, "order-refs", Severity::Error);
}

// -------------------------------------------------------- 2. component-refs

#[test]
fn order_components_must_be_declared() {
    let mut manifest = pinned();
    manifest.collection.order[1].components = Some(vec![7]);

    let findings = validate(&manifest);

    assert_has(&findings, "component-refs", Severity::Error);
}

#[test]
fn toggle_component_refs_must_be_declared() {
    let mut manifest = pinned();
    manifest.collection.toggles[0].removes_components = vec![component_ref("testmod", 99)];

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["component-refs"]);
}

#[test]
fn toggle_removes_mods_must_exist() {
    let mut manifest = pinned();
    manifest.collection.toggles[0].removes_mods = vec!["ghost".to_owned()];

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["component-refs"]);
}

#[test]
fn option_component_refs_must_name_an_existing_mod() {
    let mut manifest = pinned();
    manifest.collection.choice_groups[0].options[1].removes_components =
        vec![component_ref("ghost", 0)];

    let findings = validate(&manifest);

    assert_has(&findings, "component-refs", Severity::Error);
}

// -------------------------------------------------- 3. component-ids-unique

#[test]
fn duplicate_component_ids_within_a_mod_are_an_error() {
    let mut manifest = pinned();
    manifest
        .mods
        .get_mut("testmod")
        .unwrap()
        .components
        .push(Component {
            id: 0,
            name: "Core again".to_owned(),
            stdin: None,
        });

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["component-ids-unique"]);
}

// ------------------------------------------------------------ 4. selector-ids

#[test]
fn a_choice_group_may_not_reuse_a_toggle_id() {
    let mut manifest = pinned();
    manifest.collection.choice_groups[0].id = "testmod-optional".to_owned();

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["selector-ids"]);
}

#[test]
fn duplicate_choice_group_ids_are_an_error() {
    let mut manifest = pinned();
    manifest.collection.choice_groups.push(ChoiceGroup {
        id: "flavor".to_owned(),
        name: "Flavor again".to_owned(),
        default: "plain".to_owned(),
        options: vec![ChoiceOption {
            id: "plain".to_owned(),
            name: "Plain".to_owned(),
            adds_components: Vec::new(),
            removes_components: Vec::new(),
        }],
    });

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["selector-ids"]);
}

#[test]
fn option_ids_must_be_unique_within_a_group() {
    let mut manifest = pinned();
    manifest.collection.choice_groups[0].options[1].id = "plain".to_owned();

    let findings = validate(&manifest);

    assert_has(&findings, "selector-ids", Severity::Error);
}

#[test]
fn choice_default_must_name_an_option() {
    let mut manifest = pinned();
    manifest.collection.choice_groups[0].default = "mild".to_owned();

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["selector-ids"]);
}

// ---------------------------------------------------------------- 5. sources

#[test]
fn unknown_platform_is_an_error() {
    let mut manifest = pinned();
    manifest.mods.get_mut("testmod").unwrap().platforms = vec!["haiku".to_owned()];

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["sources"]);
}

#[test]
fn duplicate_platform_is_an_error() {
    let mut manifest = pinned();
    manifest.mods.get_mut("testmod").unwrap().platforms =
        vec!["windows".to_owned(), "windows".to_owned()];

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["sources"]);
}

#[test]
fn non_https_url_is_an_error() {
    let mut manifest = pinned();
    manifest.mods.get_mut("testmod").unwrap().source.url =
        "http://example.invalid/testmod-1.0.zip".to_owned();

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["sources"]);
}

#[test]
fn malformed_sha256_is_an_error() {
    let mut manifest = pinned();
    manifest.mods.get_mut("testmod").unwrap().source.sha256 = REAL_SHA256.to_uppercase();

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["sources"]);
}

#[test]
fn all_zero_sha256_is_a_warning_not_an_error() {
    let findings = validate(&good());

    assert_error_rules(&findings, &[]);
    assert_has(&findings, "unpinned-source", Severity::Warning);
}

// ------------------------------------------------------------- 6. phase-order

#[test]
fn phases_must_not_go_backwards_along_the_order() {
    let mut manifest = pinned();
    manifest.mods.get_mut("testmod").unwrap().phase = Phase::Bg1PreMerge;

    let findings = validate(&manifest);

    assert_has(&findings, "phase-order", Severity::Error);
}

#[test]
fn eet_must_be_the_first_main_phase_entry() {
    let mut manifest = pinned();
    let eet = manifest.collection.order.remove(0);
    manifest.collection.order.push(eet);

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["phase-order"]);
}

#[test]
fn eet_end_must_be_the_last_main_phase_entry() {
    let mut manifest = pinned();
    let mut eet_end = manifest.mods.get("eet").unwrap().clone();
    eet_end.id = "eet_end".to_owned();
    manifest.mods.insert("eet_end".to_owned(), eet_end);
    manifest
        .collection
        .order
        .insert(1, order_entry("eet_end", None));

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["phase-order"]);
}

// ---------------------------------------------------------------- 7. nonempty

#[test]
fn empty_order_is_an_error() {
    let mut manifest = pinned();
    manifest.collection.order.clear();

    let findings = validate(&manifest);

    assert_has(&findings, "nonempty", Severity::Error);
}

#[test]
fn empty_explicit_order_components_is_an_error() {
    let mut manifest = pinned();
    manifest.collection.order[1].components = Some(Vec::new());

    let findings = validate(&manifest);

    assert_has(&findings, "nonempty", Severity::Error);
}

#[test]
fn empty_mod_components_is_an_error() {
    let mut manifest = pinned();
    manifest.mods.get_mut("eet").unwrap().components.clear();

    let findings = validate(&manifest);

    assert_has(&findings, "nonempty", Severity::Error);
}

#[test]
fn empty_platforms_is_an_error() {
    let mut manifest = pinned();
    manifest.mods.get_mut("testmod").unwrap().platforms.clear();

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["nonempty"]);
}

// ------------------------------------------------------------- 8. option-slot

#[test]
fn added_component_must_sit_in_exactly_one_slot_of_a_split_mod() {
    let mut manifest = pinned();
    // testmod is split across two order entries; point the second slot at a
    // different component so the added component 10 has no slot at all.
    manifest
        .mods
        .get_mut("testmod")
        .unwrap()
        .components
        .push(Component {
            id: 20,
            name: "Extra".to_owned(),
            stdin: None,
        });
    manifest.collection.order[2].components = Some(vec![20]);

    let findings = validate(&manifest);

    assert_error_rules(&findings, &["option-slot"]);
}

#[test]
fn added_component_of_an_unsplit_mod_is_fine() {
    let mut manifest = pinned();
    manifest.collection.order.remove(2);
    manifest.collection.order[1].components = Some(vec![0, 10]);

    let findings = validate(&manifest);

    assert_error_rules(&findings, &[]);
}

// ----------------------------------------------------------- 9. stdin-newline

#[test]
fn stdin_without_trailing_newline_is_a_warning() {
    let mut manifest = pinned();
    manifest.mods.get_mut("testmod").unwrap().components[1].stdin = Some("1".to_owned());

    let findings = validate(&manifest);

    assert_error_rules(&findings, &[]);
    assert_has(&findings, "stdin-newline", Severity::Warning);
}

// -------------------------------------------------------------- check / order

#[test]
fn check_reports_every_finding_including_warnings() {
    let mut manifest = good();
    manifest.collection.toggles[0].removes_mods = vec!["ghost".to_owned()];

    let error = check(&manifest).unwrap_err();

    let EngineError::Validation(findings) = &error else {
        panic!("expected a validation error, got {error:?}");
    };
    assert_has(findings, "component-refs", Severity::Error);
    assert_has(findings, "unpinned-source", Severity::Warning);

    let rendered = error.to_string();
    assert!(rendered.contains("error[component-refs]:"), "{rendered}");
    assert!(rendered.contains("warning[unpinned-source]:"), "{rendered}");
    assert_eq!(rendered.lines().count(), findings.len(), "{rendered}");
}

#[test]
fn findings_are_reported_in_rule_order_and_are_stable() {
    let mut manifest = good();
    manifest.collection.order.push(order_entry("nope", None));
    manifest.mods.get_mut("testmod").unwrap().components[1].stdin = Some("1".to_owned());

    let findings = validate(&manifest);
    assert_eq!(findings, validate(&manifest));

    let rules = findings
        .iter()
        .map(|finding| finding.rule)
        .collect::<Vec<_>>();
    assert_eq!(
        rules,
        vec![
            "order-refs",
            "unpinned-source",
            "unpinned-source",
            "stdin-newline"
        ],
        "{findings:#?}"
    );
}

#[test]
fn finding_display_shows_severity_rule_and_message() {
    let finding = Finding {
        severity: Severity::Error,
        rule: "order-refs",
        message: "boom".to_owned(),
    };

    assert_eq!(finding.to_string(), "error[order-refs]: boom");
    assert_eq!(
        Finding {
            severity: Severity::Warning,
            ..finding
        }
        .to_string(),
        "warning[order-refs]: boom"
    );
}

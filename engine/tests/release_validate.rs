use std::fs;
use std::path::{Path, PathBuf};

use bg_engine::release_validate::{
    validate_public_alpha_at, RULE_RELEASE_FAIL_STUB, RULE_RELEASE_OMISSION,
    RULE_RELEASE_OMISSION_APPROVAL, RULE_RELEASE_STATIC_EVIDENCE, RULE_RELEASE_TAIL,
};
use bg_engine::Manifest;
use tempfile::TempDir;

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn release_root() -> PathBuf {
    recipe_root().join("releases/v0.1.0-alpha.1")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

fn scratch_release() -> TempDir {
    let temp = tempfile::tempdir().expect("create scratch release directory");
    for name in ["known-limitations.toml", "acceptance.toml"] {
        fs::copy(release_root().join(name), temp.path().join(name))
            .unwrap_or_else(|error| panic!("copy {name}: {error}"));
    }
    temp
}

fn edit_toml(path: &Path, edit: impl FnOnce(&mut toml::Value)) {
    let mut value: toml::Value = toml::from_str(
        &fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()));
    edit(&mut value);
    fs::write(
        path,
        toml::to_string_pretty(&value)
            .unwrap_or_else(|error| panic!("serialize {}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

fn has_rule(findings: &[bg_engine::validate::Finding], rule: &str) -> bool {
    findings.iter().any(|finding| finding.rule == rule)
}

#[test]
fn production_alpha_has_complete_static_release_records_without_claiming_runtime_acceptance() {
    let manifest = recipe();
    let findings = validate_public_alpha_at(&manifest, &release_root())
        .expect("production release records should load");
    assert!(findings.is_empty(), "{findings:#?}");

    let acceptance = fs::read_to_string(release_root().join("acceptance.toml")).unwrap();
    assert!(acceptance.contains("kind = \"runtime-test\""));
    assert!(acceptance.contains("status = \"pending\""));
    assert!(!acceptance.contains("live_acceptance = true"));
}

#[test]
fn selected_default_with_no_artifact_is_release_blocking() {
    let mut manifest = recipe();
    manifest.artifacts.remove("dlcmerger-2.1");

    let findings = validate_public_alpha_at(&manifest, &release_root()).unwrap();
    assert!(
        !findings.is_empty(),
        "a missing selected artifact must block public-alpha evaluation"
    );
}

#[test]
fn selected_component_identified_as_a_fail_stub_is_release_blocking() {
    let mut manifest = recipe();
    manifest.mods.get_mut("dlcmerger").unwrap().components[0].name =
        "FAIL stub retained from the migration".to_owned();

    let findings = validate_public_alpha_at(&manifest, &release_root()).unwrap();
    assert!(has_rule(&findings, RULE_RELEASE_FAIL_STUB));
}

#[test]
fn approved_omitted_component_is_not_treated_as_selected() {
    let mut manifest = recipe();
    let component = manifest
        .mods
        .get_mut("chriz-bg-modpack")
        .unwrap()
        .components
        .iter_mut()
        .find(|component| component.id == 400)
        .unwrap();
    component.name = "FAIL stub intentionally omitted from this alpha".to_owned();

    let findings = validate_public_alpha_at(&manifest, &release_root()).unwrap();
    assert!(!has_rule(&findings, RULE_RELEASE_FAIL_STUB));
}

#[test]
fn every_resolved_run_requires_accepted_static_evidence() {
    let temp = scratch_release();
    let path = temp.path().join("acceptance.toml");
    edit_toml(&path, |root| {
        root["evidence"]
            .as_array_mut()
            .unwrap()
            .retain(|record| record["subject_id"].as_str() != Some("dlcmerger-bg1"));
    });

    let findings = validate_public_alpha_at(&recipe(), temp.path()).unwrap();
    assert!(has_rule(&findings, RULE_RELEASE_STATIC_EVIDENCE));
}

#[test]
fn unknown_pending_and_failed_evidence_never_become_accepted() {
    for status in ["unknown", "pending", "failed"] {
        let temp = scratch_release();
        let path = temp.path().join("acceptance.toml");
        edit_toml(&path, |root| {
            let record = root["evidence"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|record| record["subject_id"].as_str() == Some("dlcmerger-bg1"))
                .unwrap();
            record["status"] = toml::Value::String(status.to_owned());
        });

        let findings = validate_public_alpha_at(&recipe(), temp.path()).unwrap();
        assert!(
            has_rule(&findings, RULE_RELEASE_STATIC_EVIDENCE),
            "status {status:?} unexpectedly satisfied static acceptance"
        );
    }
}

#[test]
fn post_eet_run_requires_explicit_tail_approval() {
    let temp = scratch_release();
    let path = temp.path().join("known-limitations.toml");
    edit_toml(&path, |root| {
        root["approved_tail_runs"]
            .as_array_mut()
            .unwrap()
            .retain(|run| run.as_str() != Some("buffbot-bg2"));
    });

    let findings = validate_public_alpha_at(&recipe(), temp.path()).unwrap();
    assert!(has_rule(&findings, RULE_RELEASE_TAIL));
}

#[test]
fn blocked_default_requires_an_explicit_omission_record() {
    let temp = scratch_release();
    let path = temp.path().join("known-limitations.toml");
    edit_toml(&path, |root| {
        root["omissions"]
            .as_array_mut()
            .unwrap()
            .retain(|record| record["feature_id"].as_str() != Some("feature:bg1npc:component-160"));
    });

    let findings = validate_public_alpha_at(&recipe(), temp.path()).unwrap();
    assert!(has_rule(&findings, RULE_RELEASE_OMISSION));
}

#[test]
fn omitted_default_requires_christopher_approval_reason_and_visible_limitation() {
    for field in ["approved_by", "reason", "user_facing_limitation"] {
        let temp = scratch_release();
        let path = temp.path().join("known-limitations.toml");
        edit_toml(&path, |root| {
            let record = root["omissions"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|record| {
                    record["feature_id"].as_str() == Some("feature:bg1npc:component-160")
                })
                .unwrap();
            record[field] = toml::Value::String(String::new());
        });

        let findings = validate_public_alpha_at(&recipe(), temp.path()).unwrap();
        assert!(
            has_rule(&findings, RULE_RELEASE_OMISSION_APPROVAL),
            "blank {field:?} unexpectedly passed"
        );
    }
}

#[test]
fn mandatory_blocked_tail_fixes_are_recorded_as_limitations_not_forgotten() {
    let limitations: toml::Value =
        toml::from_str(&fs::read_to_string(release_root().join("known-limitations.toml")).unwrap())
            .unwrap();
    let ids = limitations["omissions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|record| record["feature_id"].as_str())
        .collect::<Vec<_>>();
    assert!(ids.contains(&"feature:chriz-bg-modpack:component-400"));
    assert!(ids.contains(&"feature:chriz-bg-modpack:component-430"));
}

#[test]
fn non_release_fixture_does_not_require_production_release_records() {
    let manifest =
        Manifest::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest"))
            .expect("fixture recipe should load");
    let missing = tempfile::tempdir()
        .unwrap()
        .path()
        .join("no-release-records");

    let findings = validate_public_alpha_at(&manifest, &missing).unwrap();
    assert!(findings.is_empty());
}

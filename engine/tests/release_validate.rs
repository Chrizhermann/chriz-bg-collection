use std::fs;
use std::path::{Path, PathBuf};

use bg_engine::cli::{validate_recipe, ValidationProfile};
use bg_engine::manifest::{Decision, InvocationMode, Readiness};
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
fn both_bundled_profiles_pass_the_public_alpha_gate_the_app_runs() {
    // The app validates whichever bundled profile it uses with the public-alpha profile
    // before it plans an installation, and that gate blocks on warnings as well as errors.
    // The release records judge the chris-recommended selection only.
    for profile in ["manifest", "recipes/curated-full-current"] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(profile);
        let report = validate_recipe(&root, ValidationProfile::PublicAlpha)
            .unwrap_or_else(|error| panic!("{profile}: {error:?}"));
        assert!(
            report.findings.is_empty(),
            "{profile}: {:#?}",
            report.findings
        );
    }
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
fn unselected_ui_with_unsupported_invocation_is_release_blocking() {
    let mut manifest = recipe();
    let baseline =
        bg_engine::recipe_view::evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert!(baseline
        .plan
        .components_for("game-interface-infinity-bg2")
        .is_none());
    manifest
        .mods
        .get_mut("infinity-ui")
        .unwrap()
        .invocation_mode = InvocationMode::ExplicitTp2;

    let findings = validate_public_alpha_at(&manifest, &release_root()).unwrap();
    assert!(
        findings.iter().any(|finding| {
            finding.rule == "release-invocation-mode"
                && finding.severity == bg_engine::validate::Severity::Error
                && finding.message.contains("infinity-ui")
                && finding.message.contains("setup-name")
        }),
        "{findings:#?}"
    );
    assert!(
        !has_rule(
            &bg_engine::validate::validate(&manifest),
            "release-invocation-mode",
        ),
        "general authoring validation must keep its explicit-TP2 support"
    );
}

#[test]
fn production_invocation_gate_applies_even_without_release_records() {
    let manifest =
        Manifest::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest"))
            .expect("fixture recipe should load");
    assert_eq!(
        manifest.mods["eefixpack"].invocation_mode,
        InvocationMode::ExplicitTp2
    );
    let temp = tempfile::tempdir().unwrap();

    let findings =
        validate_public_alpha_at(&manifest, &temp.path().join("no-release-records")).unwrap();
    assert!(
        findings.iter().any(|finding| {
            finding.rule == "release-invocation-mode" && finding.message.contains("eefixpack")
        }),
        "{findings:#?}"
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

const HGO: &str = "mod:hidden-gameplay-options";
const HGO_DEFAULT: &str = "feature:hiddengameplayoptions:component-10";

fn set_preset(manifest: &mut Manifest, feature_id: &str, value: &str) {
    manifest
        .presets
        .get_mut("chris-recommended")
        .expect("public alpha preset")
        .selections
        .insert(feature_id.to_owned(), value.to_owned());
}

fn omitted(findings: &[bg_engine::validate::Finding], feature_id: &str) -> bool {
    let quoted = format!("{feature_id:?}");
    findings
        .iter()
        .any(|finding| finding.rule == RULE_RELEASE_OMISSION && finding.message.contains(&quoted))
}

#[test]
fn defaults_beneath_an_unticked_optional_parent_need_no_omission_record() {
    let mut manifest = recipe();
    set_preset(&mut manifest, HGO, "off");
    // The preset still asks for the defaults beneath the unticked switch; they only show what
    // ticking it would add.
    let evaluation =
        bg_engine::recipe_view::evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert!(evaluation.normalized_selection.features[HGO_DEFAULT]);
    assert!(!evaluation.view.control(HGO_DEFAULT).unwrap().selected);

    let findings = validate_public_alpha_at(&manifest, &release_root()).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn a_default_that_drops_out_under_a_ticked_optional_parent_needs_an_omission_record() {
    let mut manifest = recipe();
    let feature = manifest
        .collection
        .features
        .iter_mut()
        .find(|feature| feature.id == HGO_DEFAULT)
        .unwrap();
    feature.readiness = Readiness::Blocked;
    feature.unavailable_reason = Some("Held back for this check.".to_owned());

    let findings = validate_public_alpha_at(&manifest, &release_root()).unwrap();
    assert!(omitted(&findings, HGO_DEFAULT), "{findings:#?}");
}

#[test]
fn outcomes_beneath_a_switched_off_default_parent_still_need_omission_records() {
    let mut manifest = recipe();
    // Default children of a default mod, and a mandatory child of another default mod.
    set_preset(&mut manifest, "mod:bg1npc", "off");
    set_preset(&mut manifest, "mod:paina", "off");
    let paina_child = manifest
        .collection
        .features
        .iter()
        .find(|feature| {
            feature.parent.as_deref() == Some("mod:paina")
                && feature.decision == Decision::Mandatory
        })
        .expect("Pai'Na mandatory child")
        .id
        .clone();

    let findings = validate_public_alpha_at(&manifest, &release_root()).unwrap();
    assert!(
        omitted(&findings, "feature:bg1npc:component-0"),
        "{findings:#?}"
    );
    assert!(omitted(&findings, &paina_child), "{findings:#?}");
}

#[test]
fn an_unticked_optional_grandparent_covers_a_default_parent_and_its_defaults() {
    let mut manifest = recipe();
    let position = manifest
        .collection
        .features
        .iter()
        .position(|feature| feature.id == HGO)
        .unwrap();
    let mut grandparent = manifest.collection.features[position].clone();
    grandparent.id = "test:optional-grandparent".to_owned();
    grandparent.title = "Optional grandparent".to_owned();
    let parent = &mut manifest.collection.features[position];
    parent.decision = Decision::Default;
    parent.parent = Some(grandparent.id.clone());
    manifest.collection.features.insert(position, grandparent);

    let findings = validate_public_alpha_at(&manifest, &release_root()).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");

    // Ticked, the default parent and its defaults are requested and planned again.
    set_preset(&mut manifest, "test:optional-grandparent", "on");
    let evaluation =
        bg_engine::recipe_view::evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert!(evaluation.view.control(HGO_DEFAULT).unwrap().selected);
    let findings = validate_public_alpha_at(&manifest, &release_root()).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
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
    // October 13: alpha.11's reader-level scroll policy 430 needs EEex instead of
    // Tweaks Anthology 2170, so it is a ready default and no longer an omission.
    assert!(!ids.contains(&"feature:chriz-bg-modpack:component-430"));
}

#[test]
fn non_release_fixture_does_not_require_production_release_records() {
    let mut manifest =
        Manifest::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest"))
            .expect("fixture recipe should load");
    for mod_file in manifest.mods.values_mut() {
        mod_file.invocation_mode = InvocationMode::SetupName;
    }
    let missing = tempfile::tempdir()
        .unwrap()
        .path()
        .join("no-release-records");

    let findings = validate_public_alpha_at(&manifest, &missing).unwrap();
    assert!(findings.is_empty());
}

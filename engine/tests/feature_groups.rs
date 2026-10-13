//! Feature groups (Challenge Mode): schema stability, validation, derived state, and the
//! group-to-member change expansion.

use std::collections::BTreeMap;
use std::path::PathBuf;

use bg_engine::digest::{plan_digest, selection_digest, sha256_bytes};
use bg_engine::manifest::{
    Collection, Component, ComponentRef, Decision, Feature, FeatureGroup, InvocationMode, Phase,
    Readiness, Run,
};
use bg_engine::recipe_view::{
    evaluate, evaluate_preset, feature_group_changes, FeatureGroupState, FeatureGroupTarget,
    SelectionEvaluation,
};
use bg_engine::resolve::Selection;
use bg_engine::validate::{
    validate, Finding, Severity, RULE_FEATURE_GROUP_CONTENT, RULE_FEATURE_GROUP_IDS,
    RULE_FEATURE_GROUP_MEMBERS, RULE_FEATURE_GROUP_MEMBER_CONFLICTS,
    RULE_FEATURE_GROUP_MEMBER_DECISIONS,
};
use bg_engine::Manifest;

const RUN: &str = "grouprun";

fn engine_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture() -> Manifest {
    Manifest::load(&engine_dir().join("tests/fixtures/manifest")).unwrap()
}

fn feature(id: &str, decision: Decision, component: u32) -> Feature {
    Feature {
        id: id.to_owned(),
        title: format!("Title {id}"),
        description: format!("Description for {id}"),
        category: "difficulty".to_owned(),
        source_label: None,
        group_label: None,
        choice_group: None,
        decision,
        readiness: Readiness::Ready,
        unavailable_reason: None,
        parent: None,
        components: vec![ComponentRef {
            run_id: RUN.to_owned(),
            component,
        }],
        requires: Vec::new(),
        requires_any: Vec::new(),
        conflicts: Vec::new(),
        inputs: Vec::new(),
    }
}

fn group(id: &str, members: &[&str]) -> FeatureGroup {
    FeatureGroup {
        id: id.to_owned(),
        title: "Challenge Mode".to_owned(),
        description: "Stricter rules for a harder game.".to_owned(),
        members: members.iter().map(|member| (*member).to_owned()).collect(),
    }
}

/// Fixture plus one run whose components are owned by:
/// - `a`, `b`, `c`: optional members of `challenge-mode` (`c` requires the optional `gate`);
/// - `gate` (optional), `normal` (default), `always` (mandatory), `excluded`, and two
///   choice-group alternatives.
fn grouped_manifest() -> Manifest {
    let mut manifest = fixture();
    let components = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut installer = manifest.mods["eefixpack"].clone();
    installer.id = RUN.to_owned();
    installer.name = RUN.to_owned();
    installer.tp2 = format!("{RUN}/{RUN}.tp2");
    installer.invocation_mode = InvocationMode::ExplicitTp2;
    installer.components = components
        .iter()
        .map(|id| Component {
            id: *id,
            name: format!("Component {id}"),
            stdin: None,
            prompts: Vec::new(),
        })
        .collect();
    manifest.mods.insert(RUN.to_owned(), installer);
    let artifact = manifest.artifacts.get_mut("eefixpack").unwrap();
    artifact.archive.publish_roots.push(RUN.to_owned());
    artifact.archive.tp2_paths.push(format!("{RUN}/{RUN}.tp2"));
    manifest.collection.runs.push(Run {
        run_id: RUN.to_owned(),
        mod_id: RUN.to_owned(),
        phase: Phase::Bg2Preparation,
        components: components.to_vec(),
        args: Vec::new(),
        postconditions: Vec::new(),
    });

    let mut c = feature("c", Decision::Optional, 3);
    c.requires = vec!["gate".to_owned()];
    let mut first_choice = feature("choice-one", Decision::Optional, 8);
    let mut second_choice = feature("choice-two", Decision::Optional, 9);
    for (choice, other) in [
        (&mut first_choice, "choice-two"),
        (&mut second_choice, "choice-one"),
    ] {
        choice.choice_group = Some("pick".to_owned());
        choice.group_label = Some("Pick one".to_owned());
        choice.conflicts = vec![bg_engine::manifest::Conflict {
            feature_id: other.to_owned(),
            reason: "Choose one.".to_owned(),
        }];
    }
    manifest.collection.features.extend([
        feature("a", Decision::Optional, 1),
        feature("b", Decision::Optional, 2),
        c,
        feature("gate", Decision::Optional, 4),
        feature("normal", Decision::Default, 5),
        feature("always", Decision::Mandatory, 6),
        feature("excluded", Decision::Excluded, 7),
        first_choice,
        second_choice,
    ]);
    manifest.collection.feature_groups = vec![group("challenge-mode", &["a", "b", "c"])];
    manifest
}

fn selection(choices: &[(&str, &str)]) -> Selection {
    Selection {
        platform: "windows".to_owned(),
        choices: choices
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect(),
    }
}

fn with_changes(base: &Selection, changes: &BTreeMap<String, bool>) -> Selection {
    let mut selection = base.clone();
    for (feature, selected) in changes {
        selection.set_feature(feature, *selected);
    }
    selection
}

fn group_errors(findings: &[Finding]) -> Vec<(&'static str, String)> {
    findings
        .iter()
        .filter(|finding| {
            finding.severity == Severity::Error && finding.rule.starts_with("feature-group")
        })
        .map(|finding| (finding.rule, finding.message.clone()))
        .collect()
}

fn assert_group_error(manifest: &Manifest, rule: &str, fragment: &str) {
    let findings = validate(manifest);
    let errors = group_errors(&findings);
    assert!(
        errors
            .iter()
            .any(|(found, message)| *found == rule && message.contains(fragment)),
        "expected {rule} containing {fragment:?}, got {errors:?}"
    );
}

fn controls_selected(evaluation: &SelectionEvaluation) -> BTreeMap<String, bool> {
    evaluation
        .view
        .controls
        .iter()
        .map(|control| (control.id.clone(), control.selected))
        .collect()
}

// --- Schema and identity stability -------------------------------------------------------

/// Digests of recipes without groups, captured from the engine before feature groups existed
/// (commit 536e216). Equal values prove that the new optional field changes neither the frozen
/// collection serialization nor any selection or plan identity of existing recipes and receipts.
const FIXTURE_COLLECTION_JSON_SHA256: &str =
    "44599ccd32f8c391c5a60deb808a602efa359c83757298786163c1f56450e481";
const FIXTURE_COLLECTION_TOML_SHA256: &str =
    "725c8b18315ae492c4550c1d0e3c1be5c0194afe728c6b7c10508aa4c7b28ceb";
const FIXTURE_SELECTION_SHA256: &str =
    "871c4de396ee3a57f5c7a2b11634c0ecbbfff4f10a5f35347a03a0d435c25479";
const FIXTURE_PLAN_SHA256: &str =
    "6f23bb10da82b8a196d50cdfbe0a86f583cc4df842d62052e624fb9b14bd5c90";

#[test]
fn recipe_without_groups_serializes_and_digests_exactly_as_before() {
    let manifest = fixture();
    assert!(manifest.collection.feature_groups.is_empty());
    let json = serde_json::to_vec(&manifest.collection).unwrap();
    let toml_text = toml::to_string(&manifest.collection).unwrap();
    assert!(!String::from_utf8_lossy(&json).contains("feature_groups"));
    assert!(!toml_text.contains("feature_groups"));
    assert_eq!(sha256_bytes(&json), FIXTURE_COLLECTION_JSON_SHA256);
    assert_eq!(
        sha256_bytes(toml_text.as_bytes()),
        FIXTURE_COLLECTION_TOML_SHA256
    );

    let evaluation = evaluate_preset(&manifest, "recommended", "windows").unwrap();
    assert!(evaluation.groups.is_empty());
    assert!(evaluation.view.groups.is_empty());
    let evaluation_json = serde_json::to_string(&evaluation).unwrap();
    assert!(!evaluation_json.contains("\"groups\""), "{evaluation_json}");
    assert_eq!(
        selection_digest(&evaluation.normalized_selection).unwrap(),
        FIXTURE_SELECTION_SHA256
    );
    assert_eq!(plan_digest(&evaluation.plan).unwrap(), FIXTURE_PLAN_SHA256);
}

#[test]
fn production_recommended_identity_is_unchanged_by_the_challenge_mode_group() {
    // (recipe, collection JSON without the group, Recommended selection, plan)
    // Re-pinned on October 13 when EET's desktop shortcut (EET 100) became an
    // unchecked option, and for the October 13 intake (Modpack alpha.11 / SoD
    // Remix v0.6.13 rows); the group still changes nothing but the group table.
    for (dir, collection_sha, selection_sha, plan_sha) in [
        (
            "../manifest",
            "2088ab998856c0b83abdbc8afb3c39fd6ca5eafb60fa459041e646872b8da564",
            "fc2fa9d5e7c32c8b2264f5b24d01dbe4242ee47c6a5fcf9b86ba4f74508138f3",
            "c6f81e58afd69c54845d682e7d136df2f4d1754cd310dfeb717cc596e6603fc2",
        ),
        (
            "../recipes/curated-full-current",
            "7921b38ae60c08c53c5f7c8f97ab5cdf6a437a4b01cdda585bfff59cc1a58ca0",
            "56271d3d028381a5a246fe3a9db561ddd20d41d00f833168a774612154e7c104",
            "d74ac8362ac601b484d8d6b96391196f9cbacd7474a005ff5286f0c9d2a78849",
        ),
    ] {
        let manifest = Manifest::load(&engine_dir().join(dir)).unwrap();
        let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert_eq!(
            selection_digest(&evaluation.normalized_selection).unwrap(),
            selection_sha,
            "{dir}"
        );
        assert_eq!(plan_digest(&evaluation.plan).unwrap(), plan_sha, "{dir}");
        // The group is the only change to the frozen collection.
        let mut without_groups: Collection = manifest.collection.clone();
        assert_eq!(without_groups.feature_groups.len(), 1, "{dir}");
        without_groups.feature_groups.clear();
        assert_eq!(
            sha256_bytes(&serde_json::to_vec(&without_groups).unwrap()),
            collection_sha,
            "{dir}"
        );
    }
}

#[test]
fn groups_parse_round_trip_and_reject_unknown_fields() {
    let text = r#"
schema = 2
game_build = "2.7.3.0"
runs = []

[[feature_groups]]
id = "challenge-mode"
title = "Challenge Mode"
description = "Harder."
members = ["a", "b"]
"#;
    let collection: Collection = toml::from_str(text).unwrap();
    assert_eq!(
        collection.feature_groups,
        vec![group_with(
            "challenge-mode",
            "Challenge Mode",
            "Harder.",
            &["a", "b"]
        )]
    );
    let round_trip: Collection = toml::from_str(&toml::to_string(&collection).unwrap()).unwrap();
    assert_eq!(round_trip, collection);
    let json: Collection =
        serde_json::from_slice(&serde_json::to_vec(&collection).unwrap()).unwrap();
    assert_eq!(json, collection);

    let without: Collection =
        toml::from_str("schema = 2\ngame_build = \"2.7.3.0\"\nruns = []\n").unwrap();
    assert!(without.feature_groups.is_empty());

    let unknown = text.replace(
        "members = [\"a\", \"b\"]",
        "members = [\"a\"]\ndefault = true",
    );
    assert!(toml::from_str::<Collection>(&unknown).is_err());
}

fn group_with(id: &str, title: &str, description: &str, members: &[&str]) -> FeatureGroup {
    FeatureGroup {
        id: id.to_owned(),
        title: title.to_owned(),
        description: description.to_owned(),
        members: members.iter().map(|member| (*member).to_owned()).collect(),
    }
}

#[test]
fn group_state_never_enters_the_normalized_selection_or_plan() {
    let grouped = grouped_manifest();
    let mut ungrouped = grouped.clone();
    ungrouped.collection.feature_groups.clear();
    for choices in [
        vec![],
        vec![("a", "on")],
        vec![("a", "on"), ("b", "on"), ("gate", "on"), ("c", "on")],
    ] {
        let with_group = evaluate(&grouped, &selection(&choices)).unwrap();
        let without_group = evaluate(&ungrouped, &selection(&choices)).unwrap();
        assert_eq!(
            with_group.normalized_selection,
            without_group.normalized_selection
        );
        assert_eq!(
            selection_digest(&with_group.normalized_selection).unwrap(),
            selection_digest(&without_group.normalized_selection).unwrap()
        );
        assert_eq!(with_group.plan, without_group.plan);
        assert_eq!(with_group.view.controls, without_group.view.controls);
        assert!(!with_group
            .normalized_selection
            .features
            .contains_key("challenge-mode"));
    }
}

// --- Validation ----------------------------------------------------------------------------

#[test]
fn a_valid_group_has_no_findings() {
    let manifest = grouped_manifest();
    assert_eq!(group_errors(&validate(&manifest)), Vec::new());
    bg_engine::validate::check(&manifest).unwrap();
}

#[test]
fn group_ids_must_be_unique_kebab_case() {
    for bad in [
        "",
        "Challenge-Mode",
        "challenge_mode",
        "challenge--mode",
        "-challenge",
        "challenge mode",
        "feature:x",
    ] {
        let mut manifest = grouped_manifest();
        manifest.collection.feature_groups[0].id = bad.to_owned();
        assert_group_error(&manifest, RULE_FEATURE_GROUP_IDS, "kebab-case");
    }
    for good in ["challenge-mode", "mode2", "a-1-b"] {
        let mut manifest = grouped_manifest();
        manifest.collection.feature_groups[0].id = good.to_owned();
        assert_eq!(group_errors(&validate(&manifest)), Vec::new(), "{good}");
    }
    let mut manifest = grouped_manifest();
    manifest
        .collection
        .feature_groups
        .push(group("challenge-mode", &["gate"]));
    assert_group_error(&manifest, RULE_FEATURE_GROUP_IDS, "more than once");
}

#[test]
fn groups_need_title_description_and_members() {
    let mut manifest = grouped_manifest();
    manifest.collection.feature_groups[0].title = " ".to_owned();
    assert_group_error(&manifest, RULE_FEATURE_GROUP_CONTENT, "nonempty title");

    let mut manifest = grouped_manifest();
    manifest.collection.feature_groups[0].description = String::new();
    assert_group_error(
        &manifest,
        RULE_FEATURE_GROUP_CONTENT,
        "nonempty description",
    );

    let mut manifest = grouped_manifest();
    manifest.collection.feature_groups[0].members.clear();
    assert_group_error(&manifest, RULE_FEATURE_GROUP_CONTENT, "at least one member");
}

#[test]
fn members_must_be_unique_existing_and_in_one_group() {
    let mut manifest = grouped_manifest();
    manifest.collection.feature_groups[0]
        .members
        .push("a".to_owned());
    assert_group_error(&manifest, RULE_FEATURE_GROUP_MEMBERS, "more than once");

    let mut manifest = grouped_manifest();
    manifest.collection.feature_groups[0]
        .members
        .push("missing".to_owned());
    assert_group_error(
        &manifest,
        RULE_FEATURE_GROUP_MEMBERS,
        "unknown feature \"missing\"",
    );

    let mut manifest = grouped_manifest();
    manifest
        .collection
        .feature_groups
        .push(group("other-group", &["b"]));
    assert_group_error(&manifest, RULE_FEATURE_GROUP_MEMBERS, "belongs to both");
}

#[test]
fn members_must_be_optional_and_outside_choice_groups() {
    for (member, decision) in [
        ("normal", "default"),
        ("always", "mandatory"),
        ("excluded", "excluded"),
    ] {
        let mut manifest = grouped_manifest();
        manifest.collection.feature_groups[0]
            .members
            .push(member.to_owned());
        assert_group_error(
            &manifest,
            RULE_FEATURE_GROUP_MEMBER_DECISIONS,
            &format!("must be an optional feature, not {decision}"),
        );
    }
    let mut manifest = grouped_manifest();
    manifest.collection.feature_groups[0]
        .members
        .push("choice-one".to_owned());
    assert_group_error(
        &manifest,
        RULE_FEATURE_GROUP_MEMBER_DECISIONS,
        "choice group",
    );
}

fn add_conflict(manifest: &mut Manifest, owner: &str, other: &str) {
    manifest
        .collection
        .features
        .iter_mut()
        .find(|feature| feature.id == owner)
        .unwrap()
        .conflicts
        .push(bg_engine::manifest::Conflict {
            feature_id: other.to_owned(),
            reason: format!("Incompatible with {other}."),
        });
}

#[test]
fn members_must_not_conflict_with_each_other() {
    let expected =
        "feature group \"challenge-mode\" members \"a\" and \"b\" conflict with each other";
    for directions in [
        &[("a", "b")][..],
        &[("b", "a")][..],
        &[("a", "b"), ("b", "a")][..],
    ] {
        let mut manifest = grouped_manifest();
        for (owner, other) in directions {
            add_conflict(&mut manifest, owner, other);
        }
        let conflicts = group_errors(&validate(&manifest))
            .into_iter()
            .filter(|(rule, _)| *rule == RULE_FEATURE_GROUP_MEMBER_CONFLICTS)
            .collect::<Vec<_>>();
        assert_eq!(conflicts.len(), 1, "{directions:?}: {conflicts:?}");
        assert!(conflicts[0].1.contains(expected), "{conflicts:?}");
        assert!(bg_engine::validate::check(&manifest).is_err());
    }

    // A conflict with a feature outside the group stays an ordinary per-feature rule.
    let mut manifest = grouped_manifest();
    add_conflict(&mut manifest, "a", "gate");
    add_conflict(&mut manifest, "gate", "b");
    assert_eq!(group_errors(&validate(&manifest)), Vec::new());
}

#[test]
fn presets_and_selections_cannot_name_a_group() {
    let mut manifest = grouped_manifest();
    manifest
        .presets
        .get_mut("recommended")
        .unwrap()
        .selections
        .insert("challenge-mode".to_owned(), "on".to_owned());
    let error = evaluate_preset(&manifest, "recommended", "windows").unwrap_err();
    assert!(
        error
            .to_string()
            .contains("unknown semantic selection \"challenge-mode\""),
        "{error}"
    );
}

// --- Derived state ---------------------------------------------------------------------------

#[test]
fn state_is_off_on_or_mixed_over_available_members() {
    let manifest = grouped_manifest();
    let off = evaluate(&manifest, &selection(&[])).unwrap();
    let state = off.group("challenge-mode").unwrap();
    assert_eq!(state.state, FeatureGroupState::Off);
    assert!(state.selected_members.is_empty());
    assert_eq!(state.available_members, ["a", "b"]);
    assert_eq!(state.unavailable_members.len(), 1);
    assert_eq!(state.unavailable_members[0].feature_id, "c");
    // The member reason is the ordinary feature reason, not a group-specific one.
    assert_eq!(
        Some(state.unavailable_members[0].reason.as_str()),
        off.view.control("c").unwrap().unavailable_reason.as_deref()
    );
    assert_eq!(state.unavailable_members[0].reason, "Requires Title gate.");
    assert_eq!(state.disabled_reason, None);
    assert_eq!(off.view.groups.len(), 1);
    assert_eq!(off.view.groups[0].members, ["a", "b", "c"]);

    let mixed = evaluate(&manifest, &selection(&[("a", "on")])).unwrap();
    let state = mixed.group("challenge-mode").unwrap();
    assert_eq!(state.state, FeatureGroupState::Mixed);
    assert_eq!(state.selected_members, ["a"]);

    // On means every *available* member; the unavailable `c` does not hold the group in mixed.
    let on = evaluate(&manifest, &selection(&[("a", "on"), ("b", "on")])).unwrap();
    assert_eq!(
        on.group("challenge-mode").unwrap().state,
        FeatureGroupState::On
    );

    // Once its requirement is met, `c` becomes available and the group is mixed again.
    let gated = evaluate(
        &manifest,
        &selection(&[("a", "on"), ("b", "on"), ("gate", "on")]),
    )
    .unwrap();
    let state = gated.group("challenge-mode").unwrap();
    assert_eq!(state.available_members, ["a", "b", "c"]);
    assert_eq!(state.state, FeatureGroupState::Mixed);
    let all = evaluate(
        &manifest,
        &selection(&[("a", "on"), ("b", "on"), ("gate", "on"), ("c", "on")]),
    )
    .unwrap();
    assert_eq!(
        all.group("challenge-mode").unwrap().state,
        FeatureGroupState::On
    );
    assert_eq!(
        all.group("challenge-mode").unwrap().selected_members,
        ["a", "b", "c"]
    );

    // A requested-but-unavailable member is not counted as selected.
    let requested = evaluate(&manifest, &selection(&[("c", "on")])).unwrap();
    let state = requested.group("challenge-mode").unwrap();
    assert_eq!(state.state, FeatureGroupState::Off);
    assert!(state.selected_members.is_empty());
}

#[test]
fn group_with_no_available_member_is_disabled_with_a_reason() {
    let mut manifest = grouped_manifest();
    manifest.collection.feature_groups = vec![group("challenge-mode", &["c"])];
    let evaluation = evaluate(&manifest, &selection(&[])).unwrap();
    let state = evaluation.group("challenge-mode").unwrap();
    assert_eq!(state.state, FeatureGroupState::Off);
    assert!(state.available_members.is_empty());
    assert_eq!(
        state.disabled_reason.as_deref(),
        Some("No Challenge Mode change is available with the current choices.")
    );
    let error = feature_group_changes(
        &manifest,
        &selection(&[]),
        "challenge-mode",
        FeatureGroupTarget::On,
    )
    .unwrap_err();
    assert!(error.to_string().contains("cannot be turned on"), "{error}");
    // Turning a disabled group off is still a valid, explicit request.
    assert_eq!(
        feature_group_changes(
            &manifest,
            &selection(&[]),
            "challenge-mode",
            FeatureGroupTarget::Off
        )
        .unwrap(),
        BTreeMap::from([("c".to_owned(), false)])
    );
}

// --- Change expansion ------------------------------------------------------------------------

#[test]
fn on_selects_only_available_members_and_off_deselects_every_member() {
    let manifest = grouped_manifest();
    let base = selection(&[("a", "on")]);
    let on =
        feature_group_changes(&manifest, &base, "challenge-mode", FeatureGroupTarget::On).unwrap();
    assert_eq!(
        on,
        BTreeMap::from([("a".to_owned(), true), ("b".to_owned(), true)])
    );
    let off =
        feature_group_changes(&manifest, &base, "challenge-mode", FeatureGroupTarget::Off).unwrap();
    assert_eq!(
        off,
        BTreeMap::from([
            ("a".to_owned(), false),
            ("b".to_owned(), false),
            ("c".to_owned(), false)
        ])
    );

    let before = evaluate(&manifest, &base).unwrap();
    let after_on = evaluate(&manifest, &with_changes(&base, &on)).unwrap();
    assert_eq!(
        after_on.group("challenge-mode").unwrap().state,
        FeatureGroupState::On
    );
    let after_off = evaluate(&manifest, &with_changes(&base, &off)).unwrap();
    assert_eq!(
        after_off.group("challenge-mode").unwrap().state,
        FeatureGroupState::Off
    );
    let members = ["a", "b", "c"];
    for after in [&after_on, &after_off] {
        for (id, selected) in controls_selected(&before) {
            if !members.contains(&id.as_str()) {
                assert_eq!(controls_selected(after)[&id], selected, "{id} changed");
            }
        }
    }
    // `c` stays unavailable and unselected; it is never forced on.
    assert!(!after_on.view.control("c").unwrap().selected);
    assert!(!after_on.normalized_selection.features["c"]);
}

#[test]
fn an_individual_member_change_never_changes_another_member() {
    let manifest = grouped_manifest();
    let base = selection(&[("gate", "on")]);
    let on =
        feature_group_changes(&manifest, &base, "challenge-mode", FeatureGroupTarget::On).unwrap();
    let all_on = with_changes(&base, &on);
    let mut one_off = all_on.clone();
    one_off.set_feature("b", false);
    let evaluation = evaluate(&manifest, &one_off).unwrap();
    let state = evaluation.group("challenge-mode").unwrap();
    assert_eq!(state.state, FeatureGroupState::Mixed);
    assert_eq!(state.selected_members, ["a", "c"]);
    // Clicking a mixed group selects every available member again.
    let again = feature_group_changes(
        &manifest,
        &one_off,
        "challenge-mode",
        FeatureGroupTarget::On,
    )
    .unwrap();
    assert_eq!(again.values().filter(|selected| **selected).count(), 3);
}

#[test]
fn unknown_group_lists_the_valid_ids() {
    let manifest = grouped_manifest();
    let error = feature_group_changes(
        &manifest,
        &selection(&[]),
        "hard-mode",
        FeatureGroupTarget::On,
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains("unknown feature group \"hard-mode\""),
        "{error}"
    );
    assert!(error.contains("valid groups: challenge-mode"), "{error}");

    let error = feature_group_changes(
        &fixture(),
        &selection(&[]),
        "hard-mode",
        FeatureGroupTarget::Off,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("defines no feature groups"), "{error}");
}

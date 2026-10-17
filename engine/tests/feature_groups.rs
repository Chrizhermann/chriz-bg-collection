//! Feature groups (Challenge Mode): schema stability, validation, derived state, and the
//! group-to-member change expansion, including choice-group variants.

use std::collections::BTreeMap;
use std::path::PathBuf;

use bg_engine::digest::{plan_digest, selection_digest, sha256_bytes};
use bg_engine::manifest::{
    Collection, Component, ComponentRef, Conflict, Decision, Feature, FeatureGroup,
    FeatureGroupVariant, InvocationMode, Phase, Readiness, Run,
};
use bg_engine::recipe_view::{
    evaluate, evaluate_preset, feature_group_changes, FeatureGroupState, FeatureGroupTarget,
    SelectionEvaluation,
};
use bg_engine::resolve::Selection;
use bg_engine::validate::{
    validate, Finding, Severity, RULE_FEATURE_GROUP_CONTENT, RULE_FEATURE_GROUP_IDS,
    RULE_FEATURE_GROUP_MEMBERS, RULE_FEATURE_GROUP_MEMBER_CONFLICTS,
    RULE_FEATURE_GROUP_MEMBER_DECISIONS, RULE_FEATURE_GROUP_VARIANTS,
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
        variants: Vec::new(),
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
            config_files: Vec::new(),
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
        choice.conflicts = vec![Conflict {
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

/// Digests of a recipe with a group but no variants, and of one evaluation of it, captured from
/// the engine before variants existed (commit be09eb9). Equal values prove that the new optional
/// field changes neither the frozen serialization of existing groups nor what the app receives.
const GROUPED_COLLECTION_JSON_SHA256: &str =
    "c911b70941cecf0e0e922305f4f5cf01eee102cf709ea62038fdf9489bc9296d";
const GROUPED_COLLECTION_TOML_SHA256: &str =
    "177098c538e06cd151d1a7830475294f9c7a05b7a6369b3f87035834dab445e2";
const GROUPED_EVALUATION_JSON_SHA256: &str =
    "556ff6dc4d8361a8433ebf97f35d1a8e4c4642bb775d80f3c2da0821057e9992";

#[test]
fn a_group_without_variants_serializes_and_evaluates_exactly_as_before_variants() {
    let manifest = grouped_manifest();
    assert!(manifest.collection.feature_groups[0].variants.is_empty());
    let json = serde_json::to_vec(&manifest.collection).unwrap();
    let toml_text = toml::to_string(&manifest.collection).unwrap();
    assert_eq!(sha256_bytes(&json), GROUPED_COLLECTION_JSON_SHA256);
    assert_eq!(
        sha256_bytes(toml_text.as_bytes()),
        GROUPED_COLLECTION_TOML_SHA256
    );
    let evaluation = evaluate(&manifest, &selection(&[("a", "on"), ("gate", "on")])).unwrap();
    assert_eq!(evaluation.groups[0].state, FeatureGroupState::Mixed);
    assert_eq!(
        sha256_bytes(&serde_json::to_vec(&evaluation).unwrap()),
        GROUPED_EVALUATION_JSON_SHA256
    );
}

#[test]
fn production_recommended_identity_is_unchanged_by_the_challenge_mode_group() {
    // (recipe, collection JSON without the group, Recommended selection, plan)
    // Re-pinned on October 13 when EET's desktop shortcut (EET 100) became an
    // unchecked option, and for the October 13 intake (Modpack alpha.11 / SoD
    // Remix v0.6.13 rows); the group still changes nothing but the group table.
    // The plans changed again only for the published October 13 artifact pins
    // (Modpack alpha.11, SoD 0.6.13, Rebalance 0.8.2, Bardic balance.7): with the
    // previous pins restored, both plans reproduce their earlier digests.
    // Re-pinned for the October 14 intake: Modpack alpha.12 (230-234), Rebalance
    // v0.8.3 (330-332), SCS 3010/3021 and, in the curated recipe, the Tweaks
    // Anthology 7649ced snapshot (3220/3500/3510/2760, Challenge Mode's 2760).
    // The public-alpha collection was re-pinned once more when its potion choice got the
    // curated recipe's "Potion uniqueness" label (display only; without it, the old value).
    // The curated plan was re-pinned for Bardic Wonders v2.9c-balance.8: only the four
    // Bardic runs' artifact id changes; with balance.7 restored, the plan is the old digest.
    // Re-pinned on October 17 when Unfinished Business became optional and unselected.
    // In both recipes the new Recommended plan equals the previous plan minus the
    // ub-bg2 run (its 18 former defaults); no other run changes.
    // Re-pinned on October 18 for EEex v1.3.1: in both recipes the Recommended plan differs
    // only in the eeex-bg2 run's artifact id (with v1.2.0 restored, the previous digests
    // return) and the collection only in the EEex feature description.
    for (dir, collection_sha, selection_sha, plan_sha) in [
        (
            "../manifest",
            "7e63251498dde16f20b9ac6f590ad66beedb5d7a2cb6ee2a1168eec0eff4255d",
            "004f6b8b5075641f3caaa3bafa9db5e6f69e8f0285524be014a484e6c4b3df29",
            "9e202110b6bfbc5a1352376c7281223a1801a75c48aabf4f18f515d42e9f96e3",
        ),
        (
            "../recipes/curated-full-current",
            "56f8ecd6761c59b8bcd5c80c5b6b851e6c3b369e6c40bf338b344a22e9ca9be5",
            "fe99c02a54aa7aca228559b538570bdbc1929d68a4ec127f3f1f89f090a3b94c",
            "a7e6bf5aa7d2473cbbe9861dcf25f549dfe1025b74613293649c79433a80a045",
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
        variants: Vec::new(),
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
        .push(Conflict {
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

// --- Choice-group variants ---------------------------------------------------------------------

const MILD: &str = "potions-mild";
const STRICT: &str = "potions-strict";
const WILD: &str = "potions-wild";

fn variant(choice_group: &str, option: &str) -> FeatureGroupVariant {
    FeatureGroupVariant {
        choice_group: choice_group.to_owned(),
        option: option.to_owned(),
        label: None,
    }
}

/// `grouped_manifest` plus a `potions` choice whose options conflict pairwise: `potions-mild`
/// is the recipe default, `potions-strict` and `potions-wild` are optional, and `potions-wild`
/// requires `gate`. Challenge Mode picks `potions-strict`.
fn variant_manifest() -> Manifest {
    let mut manifest = grouped_manifest();
    let installer = manifest.mods.get_mut(RUN).unwrap();
    for id in [10, 11, 12] {
        installer.components.push(Component {
            id,
            name: format!("Component {id}"),
            stdin: None,
            prompts: Vec::new(),
            config_files: Vec::new(),
        });
    }
    manifest
        .collection
        .runs
        .iter_mut()
        .find(|run| run.run_id == RUN)
        .unwrap()
        .components
        .extend([10, 11, 12]);
    let options = [
        (MILD, Decision::Default, 10),
        (STRICT, Decision::Optional, 11),
        (WILD, Decision::Optional, 12),
    ];
    for (id, decision, component) in options {
        let mut option = feature(id, decision, component);
        option.choice_group = Some("potions".to_owned());
        option.group_label = Some("Potions".to_owned());
        option.conflicts = options
            .iter()
            .filter(|(other, _, _)| *other != id)
            .map(|(other, _, _)| Conflict {
                feature_id: (*other).to_owned(),
                reason: "Choose one potion rule.".to_owned(),
            })
            .collect();
        if id == WILD {
            option.requires = vec!["gate".to_owned()];
        }
        manifest.collection.features.push(option);
    }
    manifest.collection.feature_groups[0].variants = vec![variant("potions", STRICT)];
    manifest
}

fn set_variant(manifest: &mut Manifest, variants: Vec<FeatureGroupVariant>) {
    manifest.collection.feature_groups[0].variants = variants;
}

fn feature_mut<'a>(manifest: &'a mut Manifest, id: &str) -> &'a mut Feature {
    manifest
        .collection
        .features
        .iter_mut()
        .find(|feature| feature.id == id)
        .unwrap()
}

#[test]
fn variants_parse_round_trip_and_reject_unknown_fields() {
    let text = r#"
schema = 2
game_build = "2.7.3.0"
runs = []

[[feature_groups]]
id = "challenge-mode"
title = "Challenge Mode"
description = "Harder."
members = ["a"]
variants = [
  { choice_group = "potions", option = "potions-strict", label = "Potions: strict" },
  { choice_group = "pace", option = "pace-fast" },
]
"#;
    let collection: Collection = toml::from_str(text).unwrap();
    let group = &collection.feature_groups[0];
    assert_eq!(
        group.variants,
        vec![
            FeatureGroupVariant {
                label: Some("Potions: strict".to_owned()),
                ..variant("potions", "potions-strict")
            },
            variant("pace", "pace-fast"),
        ]
    );
    let round_trip: Collection = toml::from_str(&toml::to_string(&collection).unwrap()).unwrap();
    assert_eq!(round_trip, collection);
    let json: Collection =
        serde_json::from_slice(&serde_json::to_vec(&collection).unwrap()).unwrap();
    assert_eq!(json, collection);

    // A group without variants serializes exactly as before variants existed.
    let mut plain = collection.clone();
    plain.feature_groups[0].variants.clear();
    assert!(!toml::to_string(&plain).unwrap().contains("variants"));
    assert!(!serde_json::to_string(&plain).unwrap().contains("variants"));
    // An unlabeled variant serializes without a label.
    assert!(
        !serde_json::to_string(&collection.feature_groups[0].variants[1])
            .unwrap()
            .contains("label")
    );

    let unknown = text.replace(
        r#"{ choice_group = "pace", option = "pace-fast" }"#,
        r#"{ choice_group = "pace", option = "pace-fast", default = "pace-slow" }"#,
    );
    assert!(toml::from_str::<Collection>(&unknown).is_err());
}

#[test]
fn a_valid_variant_has_no_findings_even_without_members() {
    let manifest = variant_manifest();
    assert_eq!(group_errors(&validate(&manifest)), Vec::new());
    bg_engine::validate::check(&manifest).unwrap();

    let mut only_variants = variant_manifest();
    only_variants.collection.feature_groups[0].members.clear();
    assert_eq!(group_errors(&validate(&only_variants)), Vec::new());

    let mut empty = variant_manifest();
    empty.collection.feature_groups[0].members.clear();
    set_variant(&mut empty, Vec::new());
    assert_group_error(
        &empty,
        RULE_FEATURE_GROUP_CONTENT,
        "at least one member or variant",
    );
}

/// A variant validation case: label, recipe change, expected rule and message fragment.
type VariantCase = (
    &'static str,
    Box<dyn Fn(&mut Manifest)>,
    &'static str,
    &'static str,
);

#[test]
fn variants_must_pick_a_non_default_option_of_an_existing_switchable_choice() {
    let cases: Vec<VariantCase> = vec![
        (
            "unknown choice group",
            Box::new(|m| set_variant(m, vec![variant("missing", STRICT)])),
            RULE_FEATURE_GROUP_VARIANTS,
            "unknown choice group \"missing\"",
        ),
        (
            "one-option choice group",
            Box::new(|m| {
                feature_mut(m, MILD).choice_group = None;
                feature_mut(m, WILD).choice_group = None;
            }),
            RULE_FEATURE_GROUP_VARIANTS,
            "which has only one option",
        ),
        (
            "unknown option",
            Box::new(|m| set_variant(m, vec![variant("potions", "missing")])),
            RULE_FEATURE_GROUP_VARIANTS,
            "unknown feature \"missing\"",
        ),
        (
            "option of another choice group",
            Box::new(|m| set_variant(m, vec![variant("potions", "choice-one")])),
            RULE_FEATURE_GROUP_VARIANTS,
            "is not an option of choice group \"potions\"",
        ),
        (
            "recipe default",
            Box::new(|m| set_variant(m, vec![variant("potions", MILD)])),
            RULE_FEATURE_GROUP_VARIANTS,
            "is the recipe default of choice group \"potions\"",
        ),
        (
            "mandatory alternative",
            Box::new(|m| feature_mut(m, WILD).decision = Decision::Mandatory),
            RULE_FEATURE_GROUP_VARIANTS,
            "option \"potions-wild\" is mandatory",
        ),
        (
            "mandatory option",
            Box::new(|m| feature_mut(m, STRICT).decision = Decision::Mandatory),
            RULE_FEATURE_GROUP_VARIANTS,
            "must be an optional feature, not mandatory",
        ),
        (
            "same choice group twice in one group",
            Box::new(|m| {
                set_variant(
                    m,
                    vec![variant("potions", STRICT), variant("potions", WILD)],
                );
            }),
            RULE_FEATURE_GROUP_VARIANTS,
            "more than once",
        ),
        (
            "same choice group in two groups",
            Box::new(|m| {
                let mut other = group("hard-mode", &["gate"]);
                other.variants = vec![variant("potions", WILD)];
                m.collection.feature_groups.push(other);
            }),
            RULE_FEATURE_GROUP_VARIANTS,
            "has variants in both feature groups \"challenge-mode\" and \"hard-mode\"",
        ),
        (
            "empty label",
            Box::new(|m| {
                set_variant(
                    m,
                    vec![FeatureGroupVariant {
                        label: Some(" ".to_owned()),
                        ..variant("potions", STRICT)
                    }],
                );
            }),
            RULE_FEATURE_GROUP_CONTENT,
            "has an empty label",
        ),
        (
            "option conflicting with a member",
            Box::new(|m| add_conflict(m, STRICT, "a")),
            RULE_FEATURE_GROUP_MEMBER_CONFLICTS,
            "member \"a\" and variant option \"potions-strict\" conflict with each other",
        ),
    ];
    for (label, mutate, rule, fragment) in cases {
        let mut manifest = variant_manifest();
        mutate(&mut manifest);
        let errors = group_errors(&validate(&manifest));
        assert!(
            errors
                .iter()
                .any(|(found, message)| *found == rule && message.contains(fragment)),
            "{label}: expected {rule} containing {fragment:?}, got {errors:?}"
        );
        assert!(bg_engine::validate::check(&manifest).is_err(), "{label}");
    }
}

#[test]
fn a_variant_counts_like_a_member_in_the_derived_state() {
    let manifest = variant_manifest();
    let base = selection(&[]);
    let off = evaluate(&manifest, &base).unwrap();
    let state = off.group("challenge-mode").unwrap();
    assert_eq!(state.state, FeatureGroupState::Off);
    assert_eq!(state.variants.len(), 1);
    assert_eq!(state.variants[0].choice_group, "potions");
    assert_eq!(state.variants[0].option, STRICT);
    assert!(!state.variants[0].selected);
    assert!(state.variants[0].available);
    assert_eq!(state.variants[0].unavailable_reason, None);
    assert!(off.view.control(MILD).unwrap().selected);

    // The view carries everything a client needs to switch and to explain the variant.
    let view = &off.view.groups[0].variants[0];
    assert_eq!(view.label, format!("Title {STRICT}"));
    assert_eq!(view.options, [MILD, STRICT, WILD]);
    assert_eq!(view.recipe_default, [MILD]);

    // Members on but the variant not picked is mixed; so is the variant alone.
    let members_only = evaluate(&manifest, &selection(&[("a", "on"), ("b", "on")])).unwrap();
    assert_eq!(
        members_only.group("challenge-mode").unwrap().state,
        FeatureGroupState::Mixed
    );
    let variant_only = evaluate(&manifest, &selection(&[(MILD, "off"), (STRICT, "on")])).unwrap();
    let state = variant_only.group("challenge-mode").unwrap();
    assert_eq!(state.state, FeatureGroupState::Mixed);
    assert!(state.variants[0].selected);

    let on = evaluate(
        &manifest,
        &selection(&[("a", "on"), ("b", "on"), (MILD, "off"), (STRICT, "on")]),
    )
    .unwrap();
    assert_eq!(
        on.group("challenge-mode").unwrap().state,
        FeatureGroupState::On
    );

    // A labelled variant shows its label.
    let mut labelled = variant_manifest();
    set_variant(
        &mut labelled,
        vec![FeatureGroupVariant {
            label: Some("Potions: strict".to_owned()),
            ..variant("potions", STRICT)
        }],
    );
    let evaluation = evaluate(&labelled, &base).unwrap();
    assert_eq!(
        evaluation.view.groups[0].variants[0].label,
        "Potions: strict"
    );
}

#[test]
fn on_picks_the_variant_option_and_off_restores_the_recipe_default() {
    let manifest = variant_manifest();
    let base = selection(&[]);
    let on =
        feature_group_changes(&manifest, &base, "challenge-mode", FeatureGroupTarget::On).unwrap();
    assert_eq!(
        on,
        BTreeMap::from([
            ("a".to_owned(), true),
            ("b".to_owned(), true),
            (MILD.to_owned(), false),
            (STRICT.to_owned(), true),
            (WILD.to_owned(), false),
        ])
    );
    let on_selection = with_changes(&base, &on);
    let after_on = evaluate(&manifest, &on_selection).unwrap();
    let state = after_on.group("challenge-mode").unwrap();
    assert_eq!(state.state, FeatureGroupState::On);
    assert!(state.variants[0].selected);
    assert!(after_on.view.control(STRICT).unwrap().selected);
    assert!(!after_on.view.control(MILD).unwrap().selected);

    let off = feature_group_changes(
        &manifest,
        &on_selection,
        "challenge-mode",
        FeatureGroupTarget::Off,
    )
    .unwrap();
    assert_eq!(
        off,
        BTreeMap::from([
            ("a".to_owned(), false),
            ("b".to_owned(), false),
            ("c".to_owned(), false),
            (MILD.to_owned(), true),
            (STRICT.to_owned(), false),
            (WILD.to_owned(), false),
        ])
    );
    let after_off = evaluate(&manifest, &with_changes(&on_selection, &off)).unwrap();
    let before = evaluate(&manifest, &base).unwrap();
    assert_eq!(
        after_off.group("challenge-mode").unwrap().state,
        FeatureGroupState::Off
    );
    assert_eq!(after_off.normalized_selection, before.normalized_selection);
    assert_eq!(after_off.plan, before.plan);

    // No feature outside the members and the variant's choice group changes either way.
    let touched = ["a", "b", "c", MILD, STRICT, WILD];
    for after in [&after_on, &after_off] {
        for (id, selected) in controls_selected(&before) {
            if !touched.contains(&id.as_str()) {
                assert_eq!(controls_selected(after)[&id], selected, "{id} changed");
            }
        }
    }

    // Off restores the recipe default even after another option was picked by hand.
    let wild = selection(&[("gate", "on"), (MILD, "off"), (WILD, "on")]);
    let restored = evaluate(
        &manifest,
        &with_changes(
            &wild,
            &feature_group_changes(&manifest, &wild, "challenge-mode", FeatureGroupTarget::Off)
                .unwrap(),
        ),
    )
    .unwrap();
    assert!(restored.view.control(MILD).unwrap().selected);
    assert!(!restored.view.control(WILD).unwrap().selected);

    // A choice without a recipe default is restored to no option at all.
    let mut no_default = variant_manifest();
    feature_mut(&mut no_default, MILD).decision = Decision::Optional;
    let off = feature_group_changes(
        &no_default,
        &base,
        "challenge-mode",
        FeatureGroupTarget::Off,
    )
    .unwrap();
    for option in [MILD, STRICT, WILD] {
        assert!(!off[option], "{option}");
    }
}

#[test]
fn a_hand_change_after_on_makes_the_group_mixed() {
    let manifest = variant_manifest();
    let base = selection(&[]);
    let on =
        feature_group_changes(&manifest, &base, "challenge-mode", FeatureGroupTarget::On).unwrap();
    let mut changed = with_changes(&base, &on);
    // Picking the recommended option again by hand, exactly as the choice control does.
    changed.set_feature(MILD, true);
    changed.set_feature(STRICT, false);
    let evaluation = evaluate(&manifest, &changed).unwrap();
    let state = evaluation.group("challenge-mode").unwrap();
    assert_eq!(state.state, FeatureGroupState::Mixed);
    assert!(!state.variants[0].selected);
    assert_eq!(state.selected_members, ["a", "b"]);
    // Clicking the mixed group picks the variant again.
    let again = feature_group_changes(
        &manifest,
        &changed,
        "challenge-mode",
        FeatureGroupTarget::On,
    )
    .unwrap();
    assert!(again[STRICT]);
    assert!(!again[MILD]);
}

#[test]
fn an_unavailable_variant_is_explained_and_never_forced() {
    let mut manifest = variant_manifest();
    set_variant(&mut manifest, vec![variant("potions", WILD)]);
    let base = selection(&[]);
    let evaluation = evaluate(&manifest, &base).unwrap();
    let state = evaluation.group("challenge-mode").unwrap();
    assert!(!state.variants[0].available);
    assert!(!state.variants[0].selected);
    assert_eq!(
        state.variants[0].unavailable_reason.as_deref(),
        Some("Requires Title gate.")
    );
    let on =
        feature_group_changes(&manifest, &base, "challenge-mode", FeatureGroupTarget::On).unwrap();
    for option in [MILD, STRICT, WILD] {
        assert!(!on.contains_key(option), "{option} was switched");
    }
    let after = evaluate(&manifest, &with_changes(&base, &on)).unwrap();
    // The unavailable variant does not hold the group in mixed, like an unavailable member.
    assert_eq!(
        after.group("challenge-mode").unwrap().state,
        FeatureGroupState::On
    );
    assert!(after.view.control(MILD).unwrap().selected);

    // Its prerequisite makes it available, and the group then picks it.
    let gated = selection(&[("gate", "on")]);
    let on =
        feature_group_changes(&manifest, &gated, "challenge-mode", FeatureGroupTarget::On).unwrap();
    assert!(on[WILD]);
    assert!(!on[MILD]);

    // With nothing else available the group is disabled, and only switching it off is allowed.
    manifest.collection.feature_groups[0].members = vec!["c".to_owned()];
    let state = evaluate(&manifest, &base).unwrap();
    assert_eq!(
        state
            .group("challenge-mode")
            .unwrap()
            .disabled_reason
            .as_deref(),
        Some("No Challenge Mode change is available with the current choices.")
    );
    let error = feature_group_changes(&manifest, &base, "challenge-mode", FeatureGroupTarget::On)
        .unwrap_err();
    assert!(error.to_string().contains("cannot be turned on"), "{error}");
    assert!(
        feature_group_changes(&manifest, &base, "challenge-mode", FeatureGroupTarget::Off).is_ok()
    );
}

#[test]
fn a_saved_selection_from_before_variants_keeps_its_choices_and_shows_mixed() {
    // What switching the group on stored before the variant existed: the members, and the
    // recommended potion rule untouched.
    let with_variant = variant_manifest();
    let mut without_variant = with_variant.clone();
    set_variant(&mut without_variant, Vec::new());
    let saved_before = evaluate(&without_variant, &selection(&[("a", "on"), ("b", "on")]))
        .unwrap()
        .normalized_selection;
    assert_eq!(
        evaluate(&without_variant, &saved_before.to_selection())
            .unwrap()
            .group("challenge-mode")
            .unwrap()
            .state,
        FeatureGroupState::On
    );

    let reloaded = evaluate(&with_variant, &saved_before.to_selection()).unwrap();
    // Nothing is changed silently: the stored choices and the plan stay exactly as saved.
    assert_eq!(reloaded.normalized_selection, saved_before);
    assert_eq!(
        selection_digest(&reloaded.normalized_selection).unwrap(),
        selection_digest(&saved_before).unwrap()
    );
    assert_eq!(
        reloaded.plan,
        evaluate(&without_variant, &saved_before.to_selection())
            .unwrap()
            .plan
    );
    assert!(reloaded.view.control(MILD).unwrap().selected);
    let state = reloaded.group("challenge-mode").unwrap();
    assert_eq!(state.state, FeatureGroupState::Mixed);
    assert!(!state.variants[0].selected);

    // A selection saved with the variant picked reloads as on, unchanged.
    let on = feature_group_changes(
        &with_variant,
        &saved_before.to_selection(),
        "challenge-mode",
        FeatureGroupTarget::On,
    )
    .unwrap();
    let saved_after = evaluate(
        &with_variant,
        &with_changes(&saved_before.to_selection(), &on),
    )
    .unwrap()
    .normalized_selection;
    let reloaded = evaluate(&with_variant, &saved_after.to_selection()).unwrap();
    assert_eq!(reloaded.normalized_selection, saved_after);
    assert_eq!(
        reloaded.group("challenge-mode").unwrap().state,
        FeatureGroupState::On
    );
}

#[test]
fn variants_never_enter_the_normalized_selection_or_plan() {
    let with_variant = variant_manifest();
    let mut without_variant = with_variant.clone();
    set_variant(&mut without_variant, Vec::new());
    for choices in [
        vec![],
        vec![(MILD, "off"), (STRICT, "on")],
        vec![("a", "on"), ("gate", "on"), (MILD, "off"), (WILD, "on")],
    ] {
        let first = evaluate(&with_variant, &selection(&choices)).unwrap();
        let second = evaluate(&without_variant, &selection(&choices)).unwrap();
        assert_eq!(first.normalized_selection, second.normalized_selection);
        assert_eq!(first.plan, second.plan);
        assert_eq!(first.view.controls, second.view.controls);
    }
}

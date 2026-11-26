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
        core_components: None,
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

fn recommended_digests(manifest: &Manifest, evaluation: &SelectionEvaluation) -> [String; 3] {
    // The frozen collection predates the group and the component dependency metadata.
    let mut collection = manifest.collection.clone();
    assert_eq!(collection.feature_groups.len(), 1);
    collection.feature_groups.clear();
    collection.component_needs.clear();
    for feature in &mut collection.features {
        feature.core_components = None;
    }
    [
        sha256_bytes(&serde_json::to_vec(&collection).unwrap()),
        selection_digest(&evaluation.normalized_selection).unwrap(),
        plan_digest(&evaluation.plan).unwrap(),
    ]
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
    // Re-pinned on October 20 for Modpack alpha.13: with alpha.12 restored and EET join XP
    // 239 removed from the mod, the late-companion run, the features and the preset, all six
    // previous digests return in both recipes.
    // Re-pinned on October 22 for the 18 default player voicepacks (their runs, features
    // and preset entries); without them all six October 20 digests return:
    // manifest 01da4486…, cc79fa3c…, b7d2694f…; curated 0dd26563…, 55c33c0b…, 4e62e7d9….
    // Re-pinned on October 23 for SoD Remix v0.6.15 (artifact pin, the Imoen choice
    // 292/293/294 in the SoD run and features, and 292 in the preset); the October 22
    // digests were manifest b0d24b97…, 4085f958…, ea9dcc56…; curated 219dc492…, 0dbd118d…,
    // ee04b38a….
    // Re-pinned on October 23 for BuffBot v1.10.1: in both recipes only the Recommended plan
    // changes, in the buffbot-bg2 run's artifact id. With v1.10.0 restored, the previous plan
    // digests return (manifest 69a7e052…, curated 34f2dae7…). Remote Console v0.2.1 is not
    // in the Recommended plan and changes no digest here.
    // Re-pinned on October 23 for the optional quest mods (six runs, their features and
    // preset entries) and BG1 Unfinished Business becoming optional and unselected. In both
    // recipes the new Recommended plan equals the previous plan minus the bg1ub-bg1 run (its
    // 16 former defaults): with BG1UB's decisions and preset entries restored, the previous
    // plan digests return although the quest mods stay; with the quest mods removed as well,
    // all six previous digests return (manifest bfc610ef…, 204329ad…, a9b52fae…; curated
    // b6360c99…, 16a7e517…, acc45f71…).
    // Re-pinned on October 25 when Christopher moved three of those mods into existing
    // categories (Icewind Dale Voice Pack to audio, PPE to visuals, Thalantyr Item Upgrade to
    // items): only the collection digests change, in those features' category and, in the
    // curated recipe, the generated group label; the previous collection digests were manifest
    // da00f885…, curated 122fb040….
    // Re-pinned on October 26 for the alpha.22 pin batch: Rebalance v0.8.5 with default Chris
    // Revisions 340, Spell Revisions v4.21-chriz.7 with its SR/RR package, BuffBot v1.11.0 and
    // Artisan's Kitpack chriz.3, plus EEex Attack Info 9, offered unchecked. With 340 and 9
    // removed (features, run components and the preset entry), the collection and selection
    // digests return in both recipes, and the plans differ from the previous ones only in the
    // ten updated runs' artifact ids (manifest 2c107ce8…, curated 76003acc…); with the previous
    // pins restored as well, all six previous digests return (manifest d98f3592…, 87b3d5fc…,
    // f1ae4719…; curated 7175787c…, 18657bda…, b8c01c67…).
    // Re-pinned on October 26 for Modpack alpha.14 in the same batch, with alpha.13's default
    // 135 and 460 and optional 652 (Christopher, 2026-10-25). With those three removed
    // (features, run components, preset entries and 651's conflict edge), the collection and
    // selection digests return in both recipes, and the plans differ only in the six selected
    // Modpack runs' artifact ids (manifest 27ade74c…, curated 44268f93…); with alpha.13
    // restored as well, all six previous digests return (manifest b1f25835…, c9354b83…,
    // e9ca1fbd…; curated c9107bcb…, d63e6c3b…, 8f5ae74a…).
    // Re-pinned on October 27 when Tweaks 2292 (druids level on the cleric XP table) became
    // optional and unselected (Christopher, 2026-10-27). Only the curated recipe changes: the
    // feature's decision, its preset entry and 2292 in the cdtweaks-bg2 run of the plan. With
    // 2292 restored as a default, the previous curated digests return (bfa199cc…, 90d4ffb4…,
    // 9e88b9c5…).
    // Re-pinned on October 27 for Artisan's Kitpack chriz.4 (all three installers): only the
    // plans change, in the selected Kitpack runs' artifact id. With chriz.3 restored, the
    // previous plan digests return (manifest acea4e17…, curated 2a08b21b…).
    // Re-pinned on October 28 for BG Rebalance v0.8.6 with the default Time Stop and Haste
    // effect clocks 360 after 340 (Christopher, 2026-10-27): its feature, run component and
    // preset entry change all six digests. With 360 switched off, the Recommended plans differ
    // from the chriz.4 ones only in the Rebalance run's artifact id; with v0.8.5 restored and
    // 360 removed, the six chriz.4 digests return (manifest ea31c07f…, d27b9fa5…, 3f1671d8…;
    // curated 54993bc2…, 3c42bcea…, 5ed8b9b7…).
    // Re-pinned on October 29 for BG Rebalance v0.8.7, which renames 360 to True Time Stop –
    // fair Haste & Slow: the collections change in 360's title and the plans in the Rebalance
    // run's artifact id; the selections are unchanged. With the old title restored the previous
    // collection digests return (manifest 820fab4c…, curated 9a8d8095…), and with v0.8.6
    // restored the previous plan digests (manifest f4ab1fd8…, curated c18baa9d…).
    // Re-pinned on October 29 for Artisan's Kitpack tweak 1010000 (weapon-usability sweep,
    // Christopher, 2026-10-26) as a default at the end of the late tweak run: its feature,
    // run component and preset entry change all six digests. Without them the six previous
    // digests return (manifest b16c7f80…, b1f467ef…, 4fd9d65f…; curated a9ce4259…,
    // c2684cfe…, 2614b16c…).
    // Re-pinned on October 29 for SoD Remix v0.6.16: in both recipes only the Recommended
    // plan changes, in the chriz-sod-remix-bg2 run's artifact id; its components and the
    // selection stay. With v0.6.15 restored, the previous plan digests return (manifest
    // 29a3cb46…, curated e59b795b…).
    // Re-pinned on October 29 for Sirene NPC for BG:EE as a default (Christopher, 2026-10-29):
    // its run after Kitpack main with sirene-bg2 moved there from after Yeslick, its six
    // features and its three preset entries change all six digests. With the run, the
    // features and the preset entries removed and sirene-bg2 back after Yeslick, the six
    // previous digests return (manifest f0fd7718…, 1cb14fad…, 7c78ecb2…; curated b633893e…,
    // cb328676…, 87f1ab67…).
    // Re-pinned on October 29 for BG Rebalance v0.8.8 with Druid weapons 410 and Alaghor of
    // Clangeddin weapons 411 as defaults (Christopher, 2026-10-29): their features, run
    // components and preset entries change all six digests, and the plans also the Rebalance
    // run's artifact id. With both removed and v0.8.7 restored, the six previous digests
    // return (manifest 2b77e6d2…, 90703460…, 616da1fb…; curated d4413a74…, e9994330…,
    // b9e1d118…).
    // Re-pinned on October 29 for Artisan's Kitpack chriz.5 with the katana fix for later mods
    // 7205 as a default (Christopher, 2026-10-29): its feature, run component and preset entry
    // change all six digests, and the plans also the selected Kitpack runs' artifact id. With
    // 7205 removed and chriz.4 restored, the six previous digests return (manifest ece76967…,
    // f6ad7fd6…, 55b18383…; curated 455b2e6b…, fa83cc45…, 5e6ff963…).
    // Re-pinned on October 29 for BuffBot v1.12.1, which goes into the alpha.22 release
    // candidate (Christopher, 2026-10-29): in both recipes only the Recommended plan changes,
    // in the buffbot-bg2 run's artifact id. With v1.11.0 restored, the previous plan digests
    // return (manifest 59f7b6fc…, curated 5dd76d84…).
    // Re-pinned on November 1 for BuffBot v1.12.6, which goes into alpha.22 and into Chriz
    // Easy IWD (Christopher, 2026-11-01): again only the Recommended plan changes, in the
    // buffbot-bg2 run's artifact id. With v1.12.1 restored, the previous plan digests return
    // (manifest 8b323328…, curated d0b7318d…).
    // Re-pinned on November 1 for issue 77 (Christopher's decision of 2026-11-01): the
    // manifest's artisanskitpack-main-bg2 and randomiser-bg2 runs follow their TP2s'
    // declaration order, as the curated recipe already did. Only the manifest's collection
    // and plan digests change, in those two runs' component order; its selection and all
    // curated digests stay. With the old order restored, the previous manifest digests
    // return (collection f338fc11…, plan 9e1d3c91…).
    // Re-pinned on November 2 for Artisan's Kitpack chriz.6 (white werewolf crash fix) and
    // BG Rebalance v0.8.9 (Pierce Magic and Spellstrike texts), which go into 0.1.0
    // (Christopher, 2026-11-02): in both recipes only the Recommended plan changes, in the
    // selected Kitpack runs' and the Rebalance run's artifact ids; components, order and
    // the selection stay. With chriz.5 and v0.8.8 restored, the previous plan digests return
    // (manifest 75caa696…, curated 1de1d6c6…).
    // Re-pinned on November 3 when BG1 NPC Project left the defaults (Christopher,
    // 2026-11-03): mod:bg1npc is optional and off in both recipes, so its feature changes
    // the collection digests, its preset entry the selection digests, and the plans lose
    // the bg1npc-bg1 run and the Kivan quest guard Modpack 130. With the mod back to default
    // and on, all six previous digests return (manifest b0fd4879…, 2447350d…, 94416a3d…;
    // curated bb95f458…, 7cf6a3b5…, 3cf7e8f4…).
    // Re-pinned on November 4 for BG Rebalance v0.8.10 (Flail of the Ages 302 without Spell
    // Revisions), which goes into 0.1.2 (Christopher, 2026-11-04): in both recipes only the
    // Recommended plan changes, in the Rebalance run's artifact id; components, order and the
    // selection stay. With v0.8.9 restored, the previous plan digests return (manifest
    // 9a20fbbf…, curated ee3d8e3b…).
    // Re-pinned on November 4 when Tweaks 3000 (maximum HP on level-up) and 3010 (Maximum HP
    // Creatures) became optional and unselected, for 0.1.2 (Christopher, 2026-11-04). Only the
    // curated recipe changes: the two features' decisions, their preset entries and both
    // components in the cdtweaks-bg2 run of the plan. With both restored as defaults, the
    // previous curated digests return (28812dc1…, 7a1ba97d…, 3cd844e7…).
    // Re-pinned on November 8 for BuffBot v1.16.1, SoD Remix v0.6.17, BG Rebalance v0.8.11,
    // Modpack alpha.16 and the 18 Voicepacks v1.2.0 release. In both recipes only the
    // Recommended plan changes, in those runs' artifact ids; components, order and the
    // selection stay. With their prior pins restored, the November 8 preceding plan digests
    // return (manifest 666e2cbf…, curated da37cc38…).
    // Re-pinned on November 16 for Chriz Overhead Text v0.1.0 (Christopher, 2026-11-16):
    // feature:chriz-overhead-text:component-0 is on by default in both recipes, adding its
    // feature to the collection digests, its preset entry to the selection digests, and
    // chriz-overhead-text-bg2 [0] before eeexremote-bg2 in both plans. Without that intake,
    // the November 8 digests return (manifest c90f8f1b…, 25301534…, 8dd9cf31…; curated
    // 4dcb1fd4…, 999046f1…, d5f6d9b9…).
    // Re-pinned on November 16 for BuffBot v1.16.2 (complete Italian translation): in both
    // recipes only the Recommended plan changes, in the BuffBot run's artifact id; components,
    // order and the selection stay. With v1.16.1 restored, the preceding November 16 plan
    // digests return (manifest ca0104b6…, curated dbad60a8…).
    // Re-pinned on November 18 for the save/rest fixes in Artisan chriz.7 (both BG
    // recipes) and Bardic Wonders balance.9 (curated only). Restoring just those
    // artifact IDs reproduces the previously frozen plan digests.
    // Rebalance v0.8.12 then added Round Clock to both Recommended selections.
    // Removing 361 and restoring v0.8.11 must reproduce all six PR #151 digests.
    // Re-pinned on November 22 for Modpack alpha.17 (component 680 added after 670 as
    // a default in chriz-bg-modpack-bg2). Removing 680 and restoring alpha.16 must reproduce
    // all six November 18 Round Clock digests before reverting Round Clock.
    // Re-pinned on November 22 for Chriz Multiplayer Fix v0.1.0 (Christopher, 2026-11-22):
    // its components 0 and 1 are on by default in both recipes, adding two features to the
    // collection digests, their preset entries to the selection digests, and
    // chriz-multiplayer-fix-bg2 [0, 1] after chriz-overhead-text-bg2 in both plans. Without
    // its run, features, preset entries, mod and artifact, the alpha.17 digests return.
    // Re-pinned on November 22 for Christopher's decision that Modpack 680 is on only with
    // Challenge Mode: it is optional, a group member, off in Recommended and has a player
    // title and description. The group is not part of these digests; the collection and selection
    // digests change, and the plans lose chriz-bg-modpack-bg2's 680. Undoing the decision
    // reproduces the digests of the PR #161 merge (manifest 23437d33…, curated e682d3e4…).
    // Pinned on November 23 to Modpack v0.2.0-alpha.18 (PR #24/#25): adds default component 470
    // (RAW specialist spell-scroll learning fix, requiring EEex) after 460; undoing 470 and
    // restoring the alpha.17 artifact reproduces PR #163's digests (`challenge_680_digests`).
    // Pinned on November 27 to Modpack v0.2.0-alpha.19 (PR #26): adds default BG1 early-party XP
    // boost 695 (requiring EEex and EET) after 680; undoing 695 and restoring alpha.18 reproduces
    // PR #164's digests (`alpha18_digests`).
    // Pinned on November 27 to Modpack v0.2.0-alpha.20 (PR #28): adds default Watcher's Keep
    // teleporters 697 and lingering room-effects cleanup 698 (requiring EEex) after 695; undoing
    // 697 and 698 and restoring alpha.19 reproduces PR #165's digests (`alpha19_digests`).
    const ROUND_CLOCK: &str = "feature:chriz-bg-rebalance:component-361";
    const SCROLL_LEARN_FIX: &str = "feature:chriz-bg-modpack:component-470";
    const BG1_EARLY_PARTY_XP: &str = "feature:chriz-bg-modpack:component-695";
    const WK_INSTANT_TELEPORTERS: &str = "feature:chriz-bg-modpack:component-697";
    const AREA_EFFECTS_CLEAR: &str = "feature:chriz-bg-modpack:component-698";
    const EET_WORLD_PROGRESSION: &str = "feature:chriz-bg-modpack:component-680";
    const EET_WORLD_PROGRESSION_DEFAULT_TITLE: &str =
        "Story-consistent EET campaign boundaries and SoD backtracking";
    const EET_WORLD_PROGRESSION_DEFAULT_DESCRIPTION: &str = "Closes cross-campaign backtracking \
        exploits between BG1, Siege of Dragonspear, Shadows of Amn and Throne of Bhaal (including \
        returning from Watcher's Keep to Amn once ToB begins and post-SoD travel reopening) while \
        keeping BG1, SoD, SoA and ToB new-game starts intact. Keeps the SoD Ch.8-10 road open \
        until the Dragonspear assault and fixes abandoned camp companion stranding, player \
        storage chest forwarding, Baeloth's pit cleanup, camp sentries, siege audio and duplicate \
        Jegg.";
    const MULTIPLAYER_FIX: [&str; 2] = [
        "feature:chriz-multiplayer-fix:component-0",
        "feature:chriz-multiplayer-fix:component-1",
    ];
    for (
        dir,
        current_digests,
        alpha19_digests,
        alpha18_digests,
        challenge_680_digests,
        default_680_digests,
        alpha17_digests,
        combined_digests,
        save_rest_digests,
        previous_plan_sha,
    ) in [
        (
            "../manifest",
            [
                "41f31769dab2ddcd1d1ef21b6c71e3b8e3343dd89237fa4e138485da36f501c4",
                "37114c01f40eb14dec3b3d2b508fbce6276a0bb307afe954c4a6db9376a9a9cb",
                "bc405030d36ab0dacdc10e467707d5ed2871faed17999961bf5bd2b4c66f784d",
            ],
            [
                "3964ac1c075bab2ca1a26ea55fadf9731ccbf4b7eda288a78df50b0dfb10a202",
                "881475e4cd9e2ac28248c2506ba5601e567e0c57d736edc70da1e4ee294f1ad2",
                "4ab7ad90ab467a6e80d212bd893009252d59a29e019e2308ad971c86930eee76",
            ],
            [
                "80881462bdf7ee08758adadef2d4605977b3cfd6b261c6b9f7325e9f49e584eb",
                "e0f0d3c1562194c3dd3194f059d2c21dfea44d69d1696a7e14c3ab9ba27c188a",
                "95764a05a21987ec9e0698da176339f4df76f14571390c38bb027e19719ef9db",
            ],
            [
                "71d08c0bbd6f74843319914bb07b31ded5fcd5b3476e59e182e26b40b82c2e61",
                "6114ffd05035f4f683cfead67e2b59db4b27b2035630664bb422701c6324639a",
                "ce64b750dbf1f418cd1b317a98add694e0781fe5c1b6f62608eb584f49b6f140",
            ],
            [
                "23437d33a7bb60528a677629dd7475c4fbf8b1e9afa464d2d65ead62d54346ef",
                "d67a959c418cef23b60fbc3302207fb5c538758e605c8d7b1a3a04f67edc4729",
                "d708db23e832fadd29b17b0a787bea90582376df632c388c64e75c4b426add5c",
            ],
            [
                "e365e33ed0508e8fcac5a9f6316df82240e8553d403c3b5db9fe03f20ddb9ac7",
                "2bf1a5290d4b18ee42938068fb7a36f2c5865213720929deffeca75b36af9b31",
                "c7f615a0ca7e06f8f7acd55410732535069bec28c16a85de2afb678b82c51b95",
            ],
            [
                "5d980b9b5ce440a69c0cc8bd55a61cd6a54699a60f21824a5791caf91896c7f6",
                "a467fe44b7e2773ee29595ce69cc7023de0bb0b8dbd307b45efa0ff906819f29",
                "bfcaa87e74c0f7941dc78ae5783655025a53cdf20684adec55b8558a9a3e8b88",
            ],
            [
                "228825ab02bfd91190dc6ce3ae909fd28863a8936d11526ad21dc588390f5479",
                "637bff40a2fde3a6af3938b2e25eed38bed3357739566a9ea4cca5f507637151",
                "21679388a966523c853019bcfd652a22f598ce4002c98cb8e317ad19a4739941",
            ],
            "794f3ecfb9b8d399741e8f68c424e133351cc76010dcaa82466c19306e9ea606",
        ),
        (
            "../recipes/curated-full-current",
            [
                "522606393675e1a209848bf8bf5d3e7c88d0cfa0615fe1f92c86117d0f3b5d61",
                "32f218d0591e669f6d26404260fc5ce5e3379273be29298b389c6dcf64fda830",
                "5c992067a94fa8a7bbe7339444b23074649759c066c813a2cdf0a82713690fd6",
            ],
            [
                "3dddff853ced98bc99588291e35e708bb5f81c2d45d25bc49e0cc857e5689dd4",
                "fef2de60f7ae3d3b9e3e060a1de2dc717d210de32c6dee529e861ff3b83102b1",
                "a6770737746fba01ebf950e2581450094a449a1357604a97038361cbd42e9aab",
            ],
            [
                "22ad1ba80c8dbd413f9f8432267d5fb22c5989631ba894a5aff30fab6ec10f60",
                "bf21a58279bd24249560b828142a4941a087c701e3f7f8dbaa53bf2fa2dba569",
                "e0568f4eacfe9210616df3b2fca7a6023c483a015cc81b2388df5afa4b5671ab",
            ],
            [
                "7df5adb11a5c19240275cfa17615c311200914e800e16ee49aa4752986cfb9f1",
                "58b7e3f1cacf98619e6edbdd56b83bbf183dccba8070c454e82e335067ccf948",
                "6c6a6b75d59fa86ae8c0e195d8155ad7758fe947d6a62dd7db57590a8eda3871",
            ],
            [
                "e682d3e4a81c00798bd231d2ed53d75db4ffde625975c456e3d42d71e98e5b5d",
                "dd00a7fd4946b5a781bd4a71541808e48ca41e781aa52fd60142dc1a129f9e83",
                "6d6628bd564fe9e1f42ad77ebe1ef6609cc191ea9a652a66fa3c06c0e2f0ed2b",
            ],
            [
                "6393e51ddc023070c11eecf062e0271904e193fa9d78f4041d0eb758ff531d55",
                "5c0a62bad494e7f643d0e94790aeb1c156cf42cb8a73372fb655d84889627d57",
                "96b35d4be83ef5ce3fab21e3ba787a6cca6b0b35cb1562bd66ffdc45432aa282",
            ],
            [
                "0fa3abae7fbe94cd5bba2bc84d0074212891f4cd0ce88379989d34e8a66339a5",
                "227d2834ad89600e48e767c6bfc4ece1a9d59293eabdc1829bb6dc1491cafa36",
                "d1164d2569cb60d83513e8fea8fda0a1a3e88af506a269788e7a2453e6d2b256",
            ],
            [
                "fab413456c438136699bd56485b8f834e12f3b3c6a1ba340966c3668495ff8a4",
                "071398d8982d27c0316a63cf7edebecc97a87a2feb4388c44adae46099c04171",
                "5728d3574cb2656fecd6bdd073948e0ab83642bc312a0b47baab138bf0ebc3a0",
            ],
            "0c3da570ec67625115f0b28f97e7bb313d10efabc3c408373a6607997287a8ad",
        ),
    ] {
        let mut manifest = Manifest::load(&engine_dir().join(dir)).unwrap();
        let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert_eq!(
            recommended_digests(&manifest, &evaluation),
            current_digests,
            "{dir}"
        );

        // Undoing Modpack alpha.20 (removing 697 and 698 and restoring alpha.19) must reproduce
        // PR #165's digests (`alpha19_digests`).
        let modpack = manifest.mods.get_mut("chriz-bg-modpack").unwrap();
        assert_eq!(modpack.artifact_id, "chriz-bg-modpack-0.2.0-alpha.20");
        modpack.artifact_id = "chriz-bg-modpack-0.2.0-alpha.19".to_owned();
        let components_before = modpack.components.len();
        modpack
            .components
            .retain(|component| !matches!(component.id, 697 | 698));
        assert_eq!(modpack.components.len() + 2, components_before);
        let modpack_run = manifest
            .collection
            .runs
            .iter_mut()
            .find(|run| run.run_id == "chriz-bg-modpack-bg2")
            .unwrap();
        let run_components_before = modpack_run.components.len();
        modpack_run
            .components
            .retain(|&id| !matches!(id, 697 | 698));
        assert_eq!(modpack_run.components.len() + 2, run_components_before);
        let features_before = manifest.collection.features.len();
        manifest.collection.features.retain(|feature| {
            feature.id != WK_INSTANT_TELEPORTERS && feature.id != AREA_EFFECTS_CLEAR
        });
        assert_eq!(manifest.collection.features.len() + 2, features_before);
        let recommended = manifest.presets.get_mut("chris-recommended").unwrap();
        assert!(recommended
            .selections
            .remove(WK_INSTANT_TELEPORTERS)
            .is_some());
        assert!(recommended.selections.remove(AREA_EFFECTS_CLEAR).is_some());
        let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert_eq!(
            recommended_digests(&manifest, &evaluation),
            alpha19_digests,
            "{dir}"
        );

        // Undoing Modpack alpha.19 (removing 695 and restoring alpha.18) must reproduce
        // PR #164's digests (`alpha18_digests`).
        let modpack = manifest.mods.get_mut("chriz-bg-modpack").unwrap();
        assert_eq!(modpack.artifact_id, "chriz-bg-modpack-0.2.0-alpha.19");
        modpack.artifact_id = "chriz-bg-modpack-0.2.0-alpha.18".to_owned();
        let components_before = modpack.components.len();
        modpack.components.retain(|component| component.id != 695);
        assert_eq!(modpack.components.len() + 1, components_before);
        let modpack_run = manifest
            .collection
            .runs
            .iter_mut()
            .find(|run| run.run_id == "chriz-bg-modpack-bg2")
            .unwrap();
        let run_components_before = modpack_run.components.len();
        modpack_run.components.retain(|&id| id != 695);
        assert_eq!(modpack_run.components.len() + 1, run_components_before);
        let features_before = manifest.collection.features.len();
        manifest
            .collection
            .features
            .retain(|feature| feature.id != BG1_EARLY_PARTY_XP);
        assert_eq!(manifest.collection.features.len() + 1, features_before);
        assert!(manifest
            .presets
            .get_mut("chris-recommended")
            .unwrap()
            .selections
            .remove(BG1_EARLY_PARTY_XP)
            .is_some());
        let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert_eq!(
            recommended_digests(&manifest, &evaluation),
            alpha18_digests,
            "{dir}"
        );

        // Undoing Modpack alpha.18 (removing 470 and restoring alpha.17) must reproduce
        // PR #163's digests (`challenge_680_digests`).
        let modpack = manifest.mods.get_mut("chriz-bg-modpack").unwrap();
        assert_eq!(modpack.artifact_id, "chriz-bg-modpack-0.2.0-alpha.18");
        modpack.artifact_id = "chriz-bg-modpack-0.2.0-alpha.17".to_owned();
        let components_before = modpack.components.len();
        modpack.components.retain(|component| component.id != 470);
        assert_eq!(modpack.components.len() + 1, components_before);
        let modpack_run = manifest
            .collection
            .runs
            .iter_mut()
            .find(|run| run.run_id == "chriz-bg-modpack-bg2")
            .unwrap();
        let run_components_before = modpack_run.components.len();
        modpack_run.components.retain(|&id| id != 470);
        assert_eq!(modpack_run.components.len() + 1, run_components_before);
        let features_before = manifest.collection.features.len();
        manifest
            .collection
            .features
            .retain(|feature| feature.id != SCROLL_LEARN_FIX);
        assert_eq!(manifest.collection.features.len() + 1, features_before);
        assert!(manifest
            .presets
            .get_mut("chris-recommended")
            .unwrap()
            .selections
            .remove(SCROLL_LEARN_FIX)
            .is_some());
        let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert_eq!(
            recommended_digests(&manifest, &evaluation),
            challenge_680_digests,
            "{dir}"
        );

        // Christopher, 2026-11-22: Modpack 680 is on only with Challenge Mode. Undoing that
        // decision (default again, PR #161's title and description, on in Recommended) must reproduce
        // the digests of the merge of PR #161 into this branch.
        let world_progression = manifest
            .collection
            .features
            .iter_mut()
            .find(|feature| feature.id == EET_WORLD_PROGRESSION)
            .unwrap();
        assert_eq!(world_progression.decision, Decision::Optional);
        world_progression.decision = Decision::Default;
        world_progression.title = EET_WORLD_PROGRESSION_DEFAULT_TITLE.to_owned();
        world_progression.description = EET_WORLD_PROGRESSION_DEFAULT_DESCRIPTION.to_owned();
        // Group members must be optional; the group is not part of these digests.
        let members = &mut manifest.collection.feature_groups[0].members;
        let count = members.len();
        members.retain(|id| id != EET_WORLD_PROGRESSION);
        assert_eq!(members.len() + 1, count);
        assert!(manifest
            .presets
            .get_mut("chris-recommended")
            .unwrap()
            .selections
            .insert(EET_WORLD_PROGRESSION.to_owned(), "on".to_owned())
            .is_none());
        let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert_eq!(
            recommended_digests(&manifest, &evaluation),
            default_680_digests,
            "{dir}"
        );

        assert!(manifest.mods.remove("chriz-multiplayer-fix").is_some());
        assert!(manifest
            .artifacts
            .remove("chriz-multiplayer-fix-0.1.0")
            .is_some());
        let runs = manifest.collection.runs.len();
        manifest
            .collection
            .runs
            .retain(|run| run.run_id != "chriz-multiplayer-fix-bg2");
        assert_eq!(manifest.collection.runs.len() + 1, runs);
        let features = manifest.collection.features.len();
        manifest
            .collection
            .features
            .retain(|feature| !MULTIPLAYER_FIX.contains(&feature.id.as_str()));
        assert_eq!(manifest.collection.features.len() + 2, features);
        let selections = &mut manifest
            .presets
            .get_mut("chris-recommended")
            .unwrap()
            .selections;
        for feature in MULTIPLAYER_FIX {
            assert!(selections.remove(feature).is_some());
        }
        let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert_eq!(
            recommended_digests(&manifest, &evaluation),
            alpha17_digests,
            "{dir}"
        );

        let modpack = manifest.mods.get_mut("chriz-bg-modpack").unwrap();
        assert_eq!(modpack.artifact_id, "chriz-bg-modpack-0.2.0-alpha.17");
        modpack.artifact_id = "chriz-bg-modpack-0.2.0-alpha.16".to_owned();
        modpack.components.retain(|component| component.id != 680);
        let modpack_run = manifest
            .collection
            .runs
            .iter_mut()
            .find(|run| run.run_id == "chriz-bg-modpack-bg2")
            .unwrap();
        assert_eq!(modpack_run.components.pop(), Some(680));
        manifest
            .collection
            .features
            .retain(|feature| feature.id != EET_WORLD_PROGRESSION);
        assert!(manifest
            .presets
            .get_mut("chris-recommended")
            .unwrap()
            .selections
            .remove(EET_WORLD_PROGRESSION)
            .is_some());

        let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert_eq!(
            recommended_digests(&manifest, &evaluation),
            combined_digests,
            "{dir}"
        );

        let rebalance = manifest.mods.get_mut("chriz-bg-rebalance").unwrap();
        assert_eq!(rebalance.artifact_id, "chriz-bg-rebalance-0.8.12");
        rebalance.artifact_id = "chriz-bg-rebalance-0.8.11".to_owned();
        rebalance.components.retain(|component| component.id != 361);
        let run = manifest
            .collection
            .runs
            .iter_mut()
            .find(|run| run.run_id == "chriz-bg-rebalance-bg2")
            .unwrap();
        assert_eq!(run.components.pop(), Some(361));
        manifest
            .collection
            .features
            .retain(|feature| feature.id != ROUND_CLOCK);
        manifest
            .presets
            .get_mut("chris-recommended")
            .unwrap()
            .selections
            .remove(ROUND_CLOCK);

        let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert_eq!(
            recommended_digests(&manifest, &evaluation),
            save_rest_digests,
            "{dir}"
        );
        let mut previous_plan = evaluation.plan;
        for run in &mut previous_plan.runs {
            let (current, previous) = match run.mod_id.as_str() {
                "artisanskitpack" | "artisanskitpack-npc" | "artisanskitpack-tweak" => (
                    "artisans-kitpack-4.81a-dev.85928f9-chriz.7",
                    "artisans-kitpack-4.81a-dev.85928f9-chriz.6",
                ),
                "bardicwonders" => (
                    "bardicwonders-v2.9c-balance.9",
                    "bardicwonders-v2.9c-balance.8",
                ),
                _ => continue,
            };
            assert_eq!(run.artifact_id, current, "{dir}: {}", run.run_id);
            run.artifact_id = previous.to_owned();
        }
        assert_eq!(
            plan_digest(&previous_plan).unwrap(),
            previous_plan_sha,
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

fn requires(manifest: &mut Manifest, owner: &str, required: &[&str]) {
    feature_mut(manifest, owner).requires = required
        .iter()
        .map(|required| (*required).to_owned())
        .collect();
}

fn planned(evaluation: &SelectionEvaluation, component: u32) -> bool {
    evaluation
        .plan
        .components_for(RUN)
        .is_some_and(|components| components.contains(&component))
}

/// Switches the group on from `base` in one step and returns the evaluation afterwards.
fn switch_on(
    manifest: &Manifest,
    base: &Selection,
) -> (BTreeMap<String, bool>, SelectionEvaluation) {
    let changes =
        feature_group_changes(manifest, base, "challenge-mode", FeatureGroupTarget::On).unwrap();
    let evaluation = evaluate(manifest, &with_changes(base, &changes)).unwrap();
    (changes, evaluation)
}

#[test]
fn a_member_that_requires_another_member_comes_on_with_the_group_in_any_order() {
    for members in [["a", "b"], ["b", "a"]] {
        let mut manifest = grouped_manifest();
        requires(&mut manifest, "b", &["a"]);
        manifest.collection.feature_groups = vec![group("challenge-mode", &members)];
        let base = selection(&[]);

        let off = evaluate(&manifest, &base).unwrap();
        let state = off.group("challenge-mode").unwrap();
        assert_eq!(state.state, FeatureGroupState::Off, "{members:?}");
        assert_eq!(state.available_members, members, "{members:?}");
        assert!(state.unavailable_members.is_empty(), "{members:?}");
        assert_eq!(state.disabled_reason, None, "{members:?}");
        // The member's own control is unchanged: alone, `b` still needs `a`.
        let control = off.view.control("b").unwrap();
        assert_eq!(
            control.unavailable_reason.as_deref(),
            Some("Requires Title a."),
            "{members:?}"
        );
        assert!(!control.interactive, "{members:?}");

        let (changes, on) = switch_on(&manifest, &base);
        assert_eq!(
            changes,
            BTreeMap::from([("a".to_owned(), true), ("b".to_owned(), true)]),
            "{members:?}"
        );
        let state = on.group("challenge-mode").unwrap();
        assert_eq!(state.state, FeatureGroupState::On, "{members:?}");
        assert_eq!(state.selected_members, members, "{members:?}");
        assert!(planned(&on, 1) && planned(&on, 2), "{members:?}");

        // Switching off deselects both and restores the starting plan.
        let off_changes = feature_group_changes(
            &manifest,
            &with_changes(&base, &changes),
            "challenge-mode",
            FeatureGroupTarget::Off,
        )
        .unwrap();
        let back = evaluate(
            &manifest,
            &with_changes(&with_changes(&base, &changes), &off_changes),
        )
        .unwrap();
        assert_eq!(back.plan, off.plan, "{members:?}");
        assert_eq!(
            back.group("challenge-mode").unwrap().state,
            FeatureGroupState::Off,
            "{members:?}"
        );
    }
}

#[test]
fn a_chain_of_member_requirements_comes_on_with_one_switch() {
    // c needs b, b needs a; `c` no longer needs the outside `gate`.
    for members in [["a", "b", "c"], ["c", "b", "a"], ["b", "c", "a"]] {
        let mut manifest = grouped_manifest();
        requires(&mut manifest, "b", &["a"]);
        requires(&mut manifest, "c", &["b"]);
        manifest.collection.feature_groups = vec![group("challenge-mode", &members)];
        let base = selection(&[]);
        let state = evaluate(&manifest, &base)
            .unwrap()
            .group("challenge-mode")
            .unwrap()
            .clone();
        assert_eq!(state.available_members, members, "{members:?}");
        let (changes, on) = switch_on(&manifest, &base);
        assert_eq!(changes.len(), 3, "{members:?}");
        assert!(changes.values().all(|selected| *selected), "{members:?}");
        let state = on.group("challenge-mode").unwrap();
        assert_eq!(state.state, FeatureGroupState::On, "{members:?}");
        assert_eq!(state.selected_members, members, "{members:?}");
        assert!(
            planned(&on, 1) && planned(&on, 2) && planned(&on, 3),
            "{members:?}"
        );
    }
}

#[test]
fn a_requirement_outside_the_group_still_keeps_a_member_unavailable() {
    // b needs the member a; c needs the outside `gate`, which is off.
    let mut manifest = grouped_manifest();
    requires(&mut manifest, "b", &["a"]);
    let base = selection(&[]);
    let evaluation = evaluate(&manifest, &base).unwrap();
    let state = evaluation.group("challenge-mode").unwrap();
    assert_eq!(state.available_members, ["a", "b"]);
    assert_eq!(state.unavailable_members.len(), 1);
    assert_eq!(state.unavailable_members[0].feature_id, "c");
    assert_eq!(state.unavailable_members[0].reason, "Requires Title gate.");
    let (changes, on) = switch_on(&manifest, &base);
    assert!(!changes.contains_key("c"));
    let state = on.group("challenge-mode").unwrap();
    assert_eq!(state.state, FeatureGroupState::On);
    assert_eq!(state.selected_members, ["a", "b"]);
    assert!(!planned(&on, 3));
    assert!(!on.normalized_selection.features["gate"]);
}

#[test]
fn a_member_needing_an_unavailable_member_stays_unavailable_too() {
    // a conflicts with the player's own `gate`; b needs a. Existing choices win.
    let mut manifest = grouped_manifest();
    requires(&mut manifest, "b", &["a"]);
    add_conflict(&mut manifest, "a", "gate");
    manifest.collection.feature_groups = vec![group("challenge-mode", &["a", "b"])];
    let base = selection(&[("gate", "on")]);
    let evaluation = evaluate(&manifest, &base).unwrap();
    let state = evaluation.group("challenge-mode").unwrap();
    assert!(state.available_members.is_empty());
    let reasons = state
        .unavailable_members
        .iter()
        .map(|member| (member.feature_id.as_str(), member.reason.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        reasons,
        [("a", "Incompatible with gate."), ("b", "Requires Title a.")]
    );
    assert_eq!(
        state.disabled_reason.as_deref(),
        Some("No Challenge Mode change is available with the current choices.")
    );
    assert!(
        feature_group_changes(&manifest, &base, "challenge-mode", FeatureGroupTarget::On).is_err()
    );
}

#[test]
fn a_member_reason_is_the_one_left_after_the_other_members_come_on() {
    // b needs a and conflicts with the outside `gate`, which needs a too. The player asked for
    // `gate`; once the group turns a on, `gate` takes effect and b reports the conflict.
    let mut manifest = grouped_manifest();
    requires(&mut manifest, "b", &["a"]);
    requires(&mut manifest, "gate", &["a"]);
    add_conflict(&mut manifest, "b", "gate");
    manifest.collection.feature_groups = vec![group("challenge-mode", &["a", "b"])];
    let base = selection(&[("gate", "on")]);
    let evaluation = evaluate(&manifest, &base).unwrap();
    assert_eq!(
        evaluation
            .view
            .control("b")
            .unwrap()
            .unavailable_reason
            .as_deref(),
        Some("Requires Title a.")
    );
    let state = evaluation.group("challenge-mode").unwrap();
    assert_eq!(state.available_members, ["a"]);
    assert_eq!(state.unavailable_members[0].feature_id, "b");
    assert_eq!(
        state.unavailable_members[0].reason,
        "Incompatible with gate."
    );
    let (changes, on) = switch_on(&manifest, &base);
    assert_eq!(changes, BTreeMap::from([("a".to_owned(), true)]));
    let state = on.group("challenge-mode").unwrap();
    assert_eq!(state.state, FeatureGroupState::On);
    assert!(planned(&on, 4) && !planned(&on, 2));
}

#[test]
fn only_the_prerequisite_ticked_by_hand_leaves_the_group_mixed() {
    let mut manifest = grouped_manifest();
    requires(&mut manifest, "b", &["a"]);
    manifest.collection.feature_groups = vec![group("challenge-mode", &["a", "b"])];
    let base = selection(&[("a", "on")]);
    let evaluation = evaluate(&manifest, &base).unwrap();
    let state = evaluation.group("challenge-mode").unwrap();
    assert_eq!(state.state, FeatureGroupState::Mixed);
    assert_eq!(state.selected_members, ["a"]);
    assert_eq!(state.available_members, ["a", "b"]);
    let (_, on) = switch_on(&manifest, &base);
    assert_eq!(
        on.group("challenge-mode").unwrap().state,
        FeatureGroupState::On
    );
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

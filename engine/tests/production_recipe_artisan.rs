use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveRootRule, Decision, Phase, Readiness, SourceKind,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

// chriz.3 (October 26 intake) restores the sorcerer class weapons (chriz.2) and keeps the
// kit weapons of Eldritch Knight and Arcane Trickster sorcerers at a level-up; against
// chriz.1 the three TP2s change only their VERSION. chriz.4 (October 27 intake) grants
// monks quarterstaves after Monk Revisions and restricts shamans and late-added kits from
// the start; its TP2s add the shared weapon grant library and a table pass at the end of
// components, but no component, so components, order and this route are unchanged.
const ARTIFACT_ID: &str = "artisans-kitpack-4.81a-dev.85928f9-chriz.4";
const ARTIFACT_SHA256: &str = "8d08fc2be48c45b1d17a79abc010b085868696fb00f045c75d057f9c72a73336";
const RELEASE_TAG: &str = "v4.81a-dev.85928f9-chriz.4";

const MAIN_AUTHORED: &[u32] = &[
    1, 2, 20000, 20001, 8001, 8101, 8002, 8004, 10002, 10001, 10003, 10004, 1003, 1006, 1004, 1005,
    1007, 1000, 1001, 1008, 1009, 1100, 2000, 2010, 2011, 2012, 2002, 3000, 3010, 3003, 3011, 3004,
    3001, 3002, 3005, 5100, 5110, 5001, 5002, 7004, 7006, 7001, 7002, 7003, 7005, 9001,
];
const MAIN_RECOMMENDED: &[u32] = &[
    1, 2, 20000, 20001, 8001, 8002, 8004, 10002, 10001, 10003, 10004, 1003, 1006, 1004, 1005, 1007,
    1000, 1001, 1008, 1009, 1100, 2000, 2010, 2011, 2012, 2002, 3000, 3010, 3003, 3011, 3004, 3001,
    3002, 3005, 5100, 5110, 5001, 5002, 7004, 7006, 7001, 7002, 7003, 7005, 9001,
];
const NPC_EARLY_AUTHORED: &[u32] = &[
    1101, 2001, 3101, 3102, 5101, 5102, 7101, 7102, 7104, 21001, 9101, 10004, 20002, 99001,
];
const NPC_RECOMMENDED: &[u32] = &[1101, 2001, 3101, 3102, 5102, 7102, 21001, 9101, 20002];
const TWEAK_EARLY: &[u32] = &[20101, 1209, 7203];
// October 29: the weapon-usability sweep 1010000 (Christopher, 2026-10-26) is the last
// component of the late run, in its TP2 position after 8204.
const TWEAK_LATE: &[u32] = &[12012, 3202, 3302, 8204, 1010000];
const SWEEP: &str = "feature:artisanskitpack-tweak:component-1010000";

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn freezes_the_upstream_based_release_identity_for_all_three_installers() {
    let manifest = recipe();
    let artifact = &manifest.artifacts[ARTIFACT_ID];
    assert_eq!(artifact.version, RELEASE_TAG.trim_start_matches('v'));
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(artifact.source.reference, RELEASE_TAG);
    assert_eq!(
        artifact.source.url,
        format!(
            "https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/releases/download/{RELEASE_TAG}/ArtisansKitpack-Chriz-Balance-Patch-{RELEASE_TAG}.zip"
        )
    );
    assert_eq!(artifact.source.expected_length, Some(89_682_621));
    assert_eq!(artifact.source.sha256, ARTIFACT_SHA256);
    assert_eq!(artifact.archive.root_rule, ArchiveRootRule::Direct);
    assert_eq!(
        artifact.archive.publish_roots,
        [
            "ArtisansKitpack",
            "ArtisansKitpack_npc",
            "ArtisansKitpack_tweak"
        ]
    );
    assert_eq!(
        artifact.archive.tp2_paths,
        [
            "ArtisansKitpack/ArtisansKitpack.TP2",
            "ArtisansKitpack_npc/ArtisansKitpack_npc.TP2",
            "ArtisansKitpack_tweak/ArtisansKitpack_tweak.TP2",
        ]
    );

    for (mod_id, tp2) in [
        ("artisanskitpack", "ArtisansKitpack/ArtisansKitpack.TP2"),
        (
            "artisanskitpack-npc",
            "ArtisansKitpack_npc/ArtisansKitpack_npc.TP2",
        ),
        (
            "artisanskitpack-tweak",
            "ArtisansKitpack_tweak/ArtisansKitpack_tweak.TP2",
        ),
    ] {
        let installer = &manifest.mods[mod_id];
        assert_eq!(installer.artifact_id, ARTIFACT_ID, "{mod_id}");
        assert_eq!(installer.tp2, tp2, "{mod_id}");
        assert_eq!(installer.weidu_artifact_id, "weidu-249-amd64", "{mod_id}");
    }
    let tweaks = &manifest.mods["artisanskitpack-tweak"];
    assert!(tweaks
        .components
        .iter()
        .any(|component| component.id == 51010));
    let cleanup = tweaks
        .components
        .iter()
        .find(|component| component.id == 20101)
        .unwrap();
    assert_eq!(
        cleanup.name,
        "Archer: Remove legacy Manyshot from mod-added items"
    );
    let cleanup_feature = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:artisanskitpack-tweak:component-20101")
        .unwrap();
    assert_eq!(cleanup_feature.title, "Remove old Manyshot effects");
    assert!(cleanup_feature.description.contains("Aimed Shot"));
}

#[test]
fn keeps_the_reviewed_split_positions_and_dependency_safe_recommended_route() {
    let manifest = recipe();
    let runs = &manifest.collection.runs;
    let position = |id: &str| {
        runs.iter()
            .position(|run| run.run_id == id)
            .unwrap_or_else(|| panic!("missing run {id}"))
    };

    let ascension = position("ascension-bg2");
    let main = position("artisanskitpack-main-bg2");
    // October 29: both Sirene mods install between Kitpack main and the Kitpack NPC
    // installer, so that they skip their own Martyr kit (Kitpack 3005 provides it).
    let sirene_bg1 = position("sirene-bg1-bg2");
    let sirene_bg2 = position("sirene-bg2");
    let npcs = position("artisanskitpack-npc-bg2");
    let iep = position("iepbanters-bg2");
    let crossmod = position("crossmodbg2-bg2");
    let early_tweaks = position("artisanskitpack-tweak-early-bg2");
    let hq = position("hq-soundclips-bg2ee-bg2");
    let randomiser = position("randomiser-bg2");
    let scs = position("stratagems-bg2");
    let late_tweaks = position("artisanskitpack-tweak-late-bg2");
    let late_npc = position("artisanskitpack-npc-late-bg2");
    let eet_end = position("eet-end-bg2");
    assert!(
        ascension < main
            && main < sirene_bg1
            && sirene_bg1 < sirene_bg2
            && sirene_bg2 < npcs
            && npcs < iep
            && iep < crossmod
            && crossmod < early_tweaks
            && early_tweaks < hq
            && hq < randomiser
            && randomiser < scs
            && scs < late_tweaks
            && late_tweaks < late_npc
            && late_npc < eet_end
    );
    for id in [
        "artisanskitpack-main-bg2",
        "artisanskitpack-npc-bg2",
        "artisanskitpack-tweak-early-bg2",
        "artisanskitpack-tweak-late-bg2",
        "artisanskitpack-npc-late-bg2",
    ] {
        assert_eq!(runs[position(id)].phase, Phase::Main, "{id}");
    }
    assert_eq!(runs[main].components, MAIN_AUTHORED);
    assert_eq!(runs[npcs].components, NPC_EARLY_AUTHORED);
    assert_eq!(runs[early_tweaks].components, TWEAK_EARLY);
    assert_eq!(
        runs[late_tweaks].components,
        [51010, 12012, 3202, 3302, 8204, 1010000]
    );
    assert_eq!(runs[late_npc].components, [200010]);

    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert_eq!(
        evaluation.plan.components_for("artisanskitpack-main-bg2"),
        Some(MAIN_RECOMMENDED)
    );
    assert_eq!(
        evaluation.plan.components_for("artisanskitpack-npc-bg2"),
        Some(NPC_RECOMMENDED)
    );
    assert_eq!(
        evaluation
            .plan
            .components_for("artisanskitpack-tweak-early-bg2"),
        Some(TWEAK_EARLY)
    );
    assert_eq!(
        evaluation
            .plan
            .components_for("artisanskitpack-tweak-late-bg2"),
        Some(TWEAK_LATE)
    );
    assert_eq!(
        evaluation
            .plan
            .components_for("artisanskitpack-npc-late-bg2"),
        None
    );

    for id in [
        "feature:artisanskitpack:component-8101",
        "feature:artisanskitpack-npc:component-10004",
    ] {
        let control = evaluation.view.control(id).expect("conflicting control");
        assert_eq!(control.readiness, Readiness::Ready, "{id}");
        assert!(!control.selected, "{id}");
        assert!(!control.interactive, "{id}");
        assert_eq!(
            control.unavailable_reason.as_deref(),
            Some("Unavailable with Spell Revisions."),
            "{id}"
        );
    }
    let red_wizard = evaluation
        .view
        .control("feature:artisanskitpack-npc:component-5102")
        .expect("Red Wizard control");
    assert_eq!(red_wizard.readiness, Readiness::Ready);
    assert!(red_wizard.selected);
    assert!(red_wizard.interactive);
    assert_eq!(red_wizard.unavailable_reason, None);
    let garrick = evaluation
        .view
        .control("feature:artisanskitpack-npc:component-99001")
        .expect("blocked Garrick control");
    assert_eq!(garrick.readiness, Readiness::Blocked);
    assert!(!garrick.selected);
    assert!(!garrick.interactive);
    assert!(garrick.unavailable_reason.is_some());
    for excluded in [30001, 300010] {
        assert!(evaluation
            .plan
            .runs
            .iter()
            .all(|run| { !run.components.contains(&excluded) }));
    }
}

#[test]
fn the_weapon_usability_sweep_runs_last_after_item_mods_when_c0pr_cl_exists() {
    // Tweak 1010000 installs only when C0PR#CL.SPL exists. In the pinned chriz.4 source the
    // generator that creates it (eeex/profs_eeex.tpa) is included by Eldritch Knight 20000
    // with EEex, by Arcane Trickster sorcerer thief equipment 8204 (which requires EEex),
    // and by Samurai 1010 and Favored Soul 30001, which the collection does not offer.
    for directory in ["manifest", "recipes/curated-full-current"] {
        let manifest = Manifest::load(&recipe_root().join("..").join(directory)).unwrap();
        let component = manifest.mods["artisanskitpack-tweak"]
            .components
            .iter()
            .find(|component| component.id == 1010000)
            .unwrap_or_else(|| panic!("{directory}: 1010000 must be authored"));
        assert_eq!(component.name, "Update EEex-based Weapon Usability Changes");
        let feature = manifest
            .collection
            .features
            .iter()
            .find(|feature| feature.id == SWEEP)
            .unwrap_or_else(|| panic!("{directory}: missing {SWEEP}"));
        assert_eq!(feature.decision, Decision::Default, "{directory}");
        assert_eq!(feature.readiness, Readiness::Ready, "{directory}");
        assert_eq!(
            feature.parent.as_deref(),
            Some("mod:artisanskitpack-tweak"),
            "{directory}"
        );
        assert_eq!(feature.requires, ["feature:eeex:mandatory-components"]);
        assert_eq!(
            feature.requires_any,
            [
                "feature:artisanskitpack:component-20000",
                "feature:artisanskitpack-tweak:component-8204",
            ],
            "{directory}"
        );
        assert!(feature.conflicts.is_empty(), "{directory}");

        // It sweeps weapons that runs before it added, so it follows every item mod the
        // lab examples come from (IWDification's two-handed axes in the curated recipe,
        // the SCS Drow Katana in both) and the Sirene mod.
        let runs = &manifest.collection.runs;
        let position = |id: &str| runs.iter().position(|run| run.run_id == id);
        let late = position("artisanskitpack-tweak-late-bg2").unwrap();
        assert_eq!(runs[late].components.last(), Some(&1010000), "{directory}");
        for earlier in [
            "artisanskitpack-main-bg2",
            "sirene-bg2",
            "artisanskitpack-npc-bg2",
            "iepbanters-bg2",
            "crossmodbg2-bg2",
            "stratagems-bg2",
        ] {
            assert!(position(earlier).unwrap() < late, "{directory}: {earlier}");
        }
        if directory != "manifest" {
            for earlier in ["cdtweaks-bg2", "iwdification-bg2", "bardicwonders-bg2"] {
                assert!(position(earlier).unwrap() < late, "{directory}: {earlier}");
            }
        }

        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert_eq!(
            manifest.presets["chris-recommended"]
                .selections
                .get(SWEEP)
                .map(String::as_str),
            Some("on"),
            "{directory}"
        );
        let sweeps = |result: &bg_engine::recipe_view::SelectionEvaluation| {
            result
                .plan
                .components_for("artisanskitpack-tweak-late-bg2")
                .is_some_and(|components| components.contains(&1010000))
        };
        assert!(sweeps(&baseline), "{directory}");
        for (eldritch_knight, arcane_trickster, expected) in [
            (true, false, true),
            (false, true, true),
            (false, false, false),
        ] {
            let mut selection = baseline.normalized_selection.to_selection();
            selection.set_feature("feature:artisanskitpack:component-20000", eldritch_knight);
            selection.set_feature(
                "feature:artisanskitpack-tweak:component-8204",
                arcane_trickster,
            );
            let result = evaluate(&manifest, &selection).unwrap();
            assert_eq!(
                sweeps(&result),
                expected,
                "{directory}: 20000={eldritch_knight} 8204={arcane_trickster}"
            );
        }
        let mut without_eeex = baseline.normalized_selection.to_selection();
        without_eeex.set_feature("mod:eeex", false);
        assert!(
            !sweeps(&evaluate(&manifest, &without_eeex).unwrap()),
            "{directory}"
        );
    }
}

#[test]
fn preserves_optional_controls_choices_and_external_dependency_gates() {
    let manifest = recipe();
    let feature = |id: &str| {
        manifest
            .collection
            .features
            .iter()
            .find(|feature| feature.id == id)
            .unwrap_or_else(|| panic!("missing feature {id}"))
    };

    for id in [
        "feature:artisanskitpack-npc:component-5101",
        "feature:artisanskitpack-npc:component-7101",
        "feature:artisanskitpack-npc:component-200010",
    ] {
        assert_eq!(feature(id).decision, Decision::Optional, "{id}");
        assert_eq!(feature(id).readiness, Readiness::Ready, "{id}");
    }
    let (left, right) = (
        "feature:artisanskitpack-npc:component-7101",
        "feature:artisanskitpack-npc:component-7102",
    );
    assert!(feature(left)
        .conflicts
        .iter()
        .any(|rule| rule.feature_id == right));
    assert!(feature(right)
        .conflicts
        .iter()
        .any(|rule| rule.feature_id == left));
    for id in [
        "feature:artisanskitpack:component-8004",
        "feature:artisanskitpack:component-10003",
        "feature:artisanskitpack:component-10004",
        "feature:artisanskitpack-tweak:component-3202",
        "feature:artisanskitpack-tweak:component-3302",
        "feature:artisanskitpack-tweak:component-8204",
    ] {
        assert!(
            feature(id)
                .requires
                .iter()
                .any(|required| required == "feature:eeex:mandatory-components"),
            "{id}"
        );
    }

    let mut disabled = Selection::defaults("windows");
    disabled.set_feature("mod:artisanskitpack", false);
    let evaluation = evaluate(&manifest, &disabled).unwrap();
    assert!(evaluation
        .plan
        .runs
        .iter()
        .all(|run| !run.mod_id.starts_with("artisanskitpack")));
}

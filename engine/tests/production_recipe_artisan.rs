use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveRootRule, Decision, Phase, Readiness, SourceKind,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

const ARTIFACT_ID: &str = "artisans-kitpack-chriz-v1.3.1";
const ARTIFACT_SHA256: &str = "a97bd3a83b8f120bd70a24ec26770b5ad5737669d0b6e548631827df1adda31f";
const RELEASE_COMMIT: &str = "ac718614991e34b4f720807bec5edc96266c6c5e";

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
const NPC_RECOMMENDED: &[u32] = &[1101, 2001, 3101, 3102, 7102, 21001, 9101, 20002];
const TWEAK_EARLY: &[u32] = &[20101, 1209, 7203];
const TWEAK_LATE: &[u32] = &[12012, 3202, 3302, 8204];

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn freezes_the_immutable_chriz_v1_3_1_archive_for_all_three_installers() {
    let manifest = recipe();
    let artifact = &manifest.artifacts[ARTIFACT_ID];
    assert_eq!(artifact.version, "chriz-v1.3.1");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubCommitZip);
    assert_eq!(artifact.source.reference, RELEASE_COMMIT);
    assert_eq!(
        artifact.source.url,
        format!(
            "https://github.com/Chrizhermann/The-Artisan-s-Kitpack-Chriz-Balance-Patch/archive/{RELEASE_COMMIT}.zip"
        )
    );
    assert_eq!(artifact.source.expected_length, Some(90_042_157));
    assert_eq!(artifact.source.sha256, ARTIFACT_SHA256);
    assert_eq!(artifact.archive.root_rule, ArchiveRootRule::SingleWrapper);
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
            && main < npcs
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
    assert_eq!(runs[late_tweaks].components, TWEAK_LATE);
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
        "feature:artisanskitpack-npc:component-5102",
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

use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveRootRule, Decision, Phase, Readiness, SourceKind,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

const SOD_REMIX_COMPONENTS: &[u32] = &[
    100, 110, 120, 130, 140, 150, 145, 160, 170, 180, 175, 185, 190, 195, 210, 197, 187, 200, 215,
    220, 225, 245, 230, 240, 250, 255, 260, 270, 280, 900,
];

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn freezes_the_reviewed_sod_remix_release() {
    let manifest = recipe();
    let artifact = &manifest.artifacts["chriz-sod-remix-0.6.4"];
    assert_eq!(artifact.version, "0.6.4");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(
        artifact.source.url,
        "https://github.com/Chrizhermann/chriz-sod-rebalance/releases/download/v0.6.4/chriz-sod-remix-v0.6.4.zip"
    );
    assert_eq!(artifact.source.reference, "v0.6.4");
    assert_eq!(
        artifact.source.expected_filename.as_deref(),
        Some("chriz-sod-remix-v0.6.4.zip")
    );
    assert_eq!(artifact.source.expected_length, Some(1_459_461));
    assert_eq!(
        artifact.source.sha256,
        "560168af4de06aaa17419213801447863be58f3f74454470c314b03edad79db3"
    );
    assert_eq!(artifact.archive.root_rule, ArchiveRootRule::Direct);
    assert_eq!(
        artifact.archive.publish_roots,
        ["chriz-sod-remix", "setup-chriz-sod-remix.tp2"]
    );
    assert_eq!(
        artifact.archive.tp2_paths,
        [
            "chriz-sod-remix/setup-chriz-sod-remix.tp2",
            "setup-chriz-sod-remix.tp2"
        ]
    );
    assert_eq!(
        manifest.mods["chriz-sod-remix"].tp2,
        "setup-chriz-sod-remix.tp2"
    );
}

#[test]
fn authors_sod_remix_as_one_ready_default_post_eet_bundle_before_buffbot() {
    let manifest = recipe();
    let run_ids = manifest
        .collection
        .runs
        .iter()
        .map(|run| run.run_id.as_str())
        .collect::<Vec<_>>();
    let eet_end = run_ids.iter().position(|id| *id == "eet-end-bg2").unwrap();
    let sod = run_ids
        .iter()
        .position(|id| *id == "chriz-sod-remix-bg2")
        .unwrap();
    let buffbot = run_ids.iter().position(|id| *id == "buffbot-bg2").unwrap();
    assert!(eet_end < sod && sod < buffbot);
    let run = &manifest.collection.runs[sod];
    assert_eq!(run.phase, Phase::PostEetEnd);
    assert_eq!(run.components, SOD_REMIX_COMPONENTS);

    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert_eq!(
        evaluation.plan.components_for("chriz-sod-remix-bg2"),
        Some(SOD_REMIX_COMPONENTS),
        "the corrected release must resolve the complete bundle"
    );
    let parent = evaluation
        .view
        .control("mod:chriz-sod-remix")
        .expect("SoD Remix parent");
    assert_eq!(parent.decision, Decision::Default);
    assert_eq!(parent.readiness, Readiness::Ready);
    assert!(parent.selected);
    assert!(parent.interactive);
    assert_eq!(parent.unavailable_reason, None);
    assert_eq!(
        manifest
            .collection
            .features
            .iter()
            .filter(|feature| feature.id.starts_with("feature:chriz-sod-remix:"))
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        ["feature:chriz-sod-remix:mandatory-components"]
    );

    let bundle = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:chriz-sod-remix:mandatory-components")
        .expect("SoD Remix bundle");
    assert_eq!(bundle.decision, Decision::Mandatory);
    assert_eq!(bundle.readiness, Readiness::Ready);

    let mut selection = Selection::defaults("windows");
    selection.set_feature("mod:chriz-sod-remix", false);
    let disabled = evaluate(&manifest, &selection).unwrap();
    assert_eq!(disabled.plan.components_for("chriz-sod-remix-bg2"), None);
}

#[test]
fn keeps_bg_rebalance_visible_but_blocked_until_its_dependencies_are_available() {
    let manifest = recipe();
    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let parent = evaluation
        .view
        .control("mod:chriz-bg-rebalance")
        .expect("BG Rebalance parent");

    assert_eq!(parent.decision, Decision::Default);
    assert_eq!(parent.readiness, Readiness::Blocked);
    assert!(!parent.selected);
    assert!(!parent.interactive);
    assert_eq!(
        parent.unavailable_reason.as_deref(),
        Some("A reviewed BG Rebalance release is not yet pinned for the public alpha.")
    );
    assert!(evaluation
        .plan
        .runs
        .iter()
        .all(|run| run.mod_id != "chriz-bg-rebalance"));
    assert!(!manifest.mods.contains_key("chriz-bg-rebalance"));
    assert!(!manifest
        .collection
        .features
        .iter()
        .any(|feature| feature.id.starts_with("feature:chriz-bg-rebalance:")));
}

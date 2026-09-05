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
const BG_REBALANCE_COMPONENTS: &[u32] = &[100, 101, 120, 121, 400, 401, 404, 405, 407, 408];
const TEMPUS_COMPONENTS: &[u32] = &[400, 401, 404, 405, 407, 408];

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn freezes_the_reviewed_sod_remix_release() {
    let manifest = recipe();
    let artifact = &manifest.artifacts["chriz-sod-remix-0.6.5"];
    assert_eq!(artifact.version, "0.6.5");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(
        artifact.source.url,
        "https://github.com/Chrizhermann/chriz-sod-rebalance/releases/download/v0.6.5/chriz-sod-remix-v0.6.5.zip"
    );
    assert_eq!(artifact.source.reference, "v0.6.5");
    assert_eq!(
        artifact.source.expected_filename.as_deref(),
        Some("chriz-sod-remix-v0.6.5.zip")
    );
    assert_eq!(artifact.source.expected_length, Some(1_460_498));
    assert_eq!(
        artifact.source.sha256,
        "e964507612730d0c44c0ea155291a1935ee8a6355cc83566be6a14f069e9d601"
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
        "chriz-sod-remix/setup-chriz-sod-remix.tp2"
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
fn freezes_the_reviewed_bg_rebalance_release() {
    let manifest = recipe();
    let artifact = &manifest.artifacts["chriz-bg-rebalance-0.3.1"];
    assert_eq!(artifact.version, "0.3.1");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(
        artifact.source.url,
        "https://github.com/Chrizhermann/chriz-bg-rebalance/releases/download/v0.3.1/chriz-bg-rebalance-v0.3.1.zip"
    );
    assert_eq!(artifact.source.reference, "v0.3.1");
    assert_eq!(
        artifact.source.expected_filename.as_deref(),
        Some("chriz-bg-rebalance-v0.3.1.zip")
    );
    assert_eq!(artifact.source.expected_length, Some(1_364_012));
    assert_eq!(
        artifact.source.sha256,
        "729be99e91f9fa2c9044783bf300998b987011a390f407e8cd0b6d4e9dc507bb"
    );
    assert_eq!(artifact.archive.root_rule, ArchiveRootRule::Direct);
    assert_eq!(
        artifact.archive.publish_roots,
        ["chriz-bg-rebalance", "setup-chriz-bg-rebalance.tp2"]
    );
    assert_eq!(artifact.archive.tp2_paths, ["setup-chriz-bg-rebalance.tp2"]);
    assert_eq!(
        manifest.mods["chriz-bg-rebalance"].tp2,
        "setup-chriz-bg-rebalance.tp2"
    );
}

#[test]
fn authors_bg_rebalance_as_late_independent_fixes_and_one_atomic_tempus_bundle() {
    let manifest = recipe();
    let run_ids = manifest
        .collection
        .runs
        .iter()
        .map(|run| run.run_id.as_str())
        .collect::<Vec<_>>();
    let hgo = run_ids
        .iter()
        .position(|id| *id == "hiddengameplayoptions-bg2")
        .unwrap();
    let rebalance = run_ids
        .iter()
        .position(|id| *id == "chriz-bg-rebalance-bg2")
        .unwrap();
    let remote = run_ids
        .iter()
        .position(|id| *id == "eeexremote-bg2")
        .unwrap();
    assert!(hgo < rebalance && rebalance < remote);
    let run = &manifest.collection.runs[rebalance];
    assert_eq!(run.phase, Phase::PostEetEnd);
    assert_eq!(run.components, BG_REBALANCE_COMPONENTS);

    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert_eq!(
        evaluation.plan.components_for("chriz-bg-rebalance-bg2"),
        Some(BG_REBALANCE_COMPONENTS)
    );
    let parent = evaluation
        .view
        .control("mod:chriz-bg-rebalance")
        .expect("BG Rebalance parent");
    assert_eq!(parent.decision, Decision::Default);
    assert_eq!(parent.readiness, Readiness::Ready);
    assert!(parent.selected);
    assert!(parent.interactive);
    assert_eq!(parent.unavailable_reason, None);

    let features = manifest
        .collection
        .features
        .iter()
        .filter(|feature| feature.id.starts_with("feature:chriz-bg-rebalance:"))
        .map(|feature| {
            (
                feature.id.as_str(),
                feature.decision,
                feature.components.len(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        features,
        [
            (
                "feature:chriz-bg-rebalance:mandatory-components",
                Decision::Mandatory,
                2
            ),
            (
                "feature:chriz-bg-rebalance:component-101",
                Decision::Default,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-121",
                Decision::Default,
                1
            ),
            (
                "feature:chriz-bg-rebalance:tempus-bundle",
                Decision::Default,
                6
            ),
        ]
    );

    let mut no_tempus = Selection::defaults("windows");
    no_tempus.set_feature("feature:chriz-bg-rebalance:tempus-bundle", false);
    let no_tempus = evaluate(&manifest, &no_tempus).unwrap();
    assert_eq!(
        no_tempus.plan.components_for("chriz-bg-rebalance-bg2"),
        Some(&[100, 101, 120, 121][..])
    );

    let mut disabled = Selection::defaults("windows");
    disabled.set_feature("mod:chriz-bg-rebalance", false);
    let disabled = evaluate(&manifest, &disabled).unwrap();
    assert_eq!(disabled.plan.components_for("chriz-bg-rebalance-bg2"), None);

    let tempus = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:chriz-bg-rebalance:tempus-bundle")
        .expect("atomic Tempus bundle");
    assert_eq!(
        tempus
            .components
            .iter()
            .map(|component| component.component)
            .collect::<Vec<_>>(),
        TEMPUS_COMPONENTS
    );
}

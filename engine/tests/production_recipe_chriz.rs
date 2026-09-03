use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveRootRule, Decision, Phase, Readiness, SourceKind,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

const SOD_REMIX_COMPONENTS: &[u32] = &[
    100, 110, 120, 130, 140, 150, 145, 160, 170, 180, 175, 185, 190, 195, 197, 187, 200, 210, 215,
    220, 225, 245, 230, 240, 250, 255, 260, 270, 280, 900,
];

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn freezes_the_reviewed_sod_remix_commit_archive() {
    let manifest = recipe();
    let artifact = &manifest.artifacts["chriz-sod-remix-d0ac9800bc544e0cb4723bf7e7c78cca02cbaae4"];
    assert_eq!(artifact.version, "0.6.3");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubCommitZip);
    assert_eq!(
        artifact.source.url,
        "https://codeload.github.com/Chrizhermann/chriz-sod-rebalance/zip/d0ac9800bc544e0cb4723bf7e7c78cca02cbaae4"
    );
    assert_eq!(
        artifact.source.reference,
        "d0ac9800bc544e0cb4723bf7e7c78cca02cbaae4"
    );
    assert_eq!(artifact.source.expected_length, Some(534_577));
    assert_eq!(
        artifact.source.sha256,
        "40613ae53f966599be28713e2b4bb4b3ab17b91c5b5b37931bccc788c9e30534"
    );
    assert_eq!(artifact.archive.root_rule, ArchiveRootRule::SingleWrapper);
    assert_eq!(artifact.archive.publish_roots, ["chriz-sod-remix"]);
    assert_eq!(
        artifact.archive.tp2_paths,
        ["chriz-sod-remix/setup-chriz-sod-remix.tp2"]
    );
}

#[test]
fn authors_sod_remix_as_one_blocked_default_post_eet_bundle_before_buffbot() {
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
        None,
        "the known forward REQUIRE_COMPONENT must block the bundle"
    );
    let parent = evaluation
        .view
        .control("mod:chriz-sod-remix")
        .expect("SoD Remix parent");
    assert_eq!(parent.decision, Decision::Default);
    assert_eq!(parent.readiness, Readiness::Blocked);
    assert!(!parent.selected);
    assert!(!parent.interactive);
    let unavailable_reason = parent.unavailable_reason.as_deref().unwrap();
    assert!(unavailable_reason.contains("197"));
    assert!(unavailable_reason.contains("210"));
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

    let mut selection = Selection::defaults("windows");
    selection.set_feature("mod:chriz-sod-remix", true);
    let forced_bundle = evaluate(&manifest, &selection).unwrap();
    assert_eq!(
        forced_bundle.plan.components_for("chriz-sod-remix-bg2"),
        None
    );
    assert!(forced_bundle.findings.iter().any(|finding| {
        finding.rule == "blocked-default-omitted" && finding.feature_id == "mod:chriz-sod-remix"
    }));
}

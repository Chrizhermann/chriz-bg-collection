use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveRootRule, Decision, Phase, Readiness, SourceKind,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

const COMMIT: &str = "661927e56e28435541a3128568cd4ee6d5d80164";

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn freezes_the_reviewed_eeex_remote_console_commit() {
    let manifest = recipe();
    let artifact = &manifest.artifacts["eeexremote-0.2.0"];
    assert_eq!(artifact.version, "0.2.0");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubCommitZip);
    assert_eq!(artifact.source.reference, COMMIT);
    assert_eq!(
        artifact.source.url,
        format!("https://codeload.github.com/Chrizhermann/eeex-remote-console/zip/{COMMIT}")
    );
    assert_eq!(artifact.source.expected_length, Some(26_360));
    assert_eq!(
        artifact.source.sha256,
        "d634b1e8f0a253bb1100b924d9a400c2cc7eb74b0d919881739b8cc680a7483a"
    );
    assert_eq!(artifact.archive.root_rule, ArchiveRootRule::SingleWrapper);
    assert_eq!(
        artifact.archive.publish_roots,
        ["eeexremote", "setup-eeexremote.tp2"]
    );
    assert_eq!(artifact.archive.tp2_paths, ["setup-eeexremote.tp2"]);
}

#[test]
fn offers_remote_console_as_an_experimental_advanced_tool_before_buffbot() {
    let manifest = recipe();
    let runs = &manifest.collection.runs;
    let position = |id: &str| {
        runs.iter()
            .position(|run| run.run_id == id)
            .unwrap_or_else(|| panic!("missing run {id}"))
    };
    assert!(position("eet-end-bg2") < position("eeexremote-bg2"));
    assert!(position("hiddengameplayoptions-bg2") < position("eeexremote-bg2"));
    assert_eq!(position("eeexremote-bg2") + 1, position("buffbot-bg2"));

    let run = &runs[position("eeexremote-bg2")];
    assert_eq!(run.phase, Phase::PostEetEnd);
    assert_eq!(run.components, [0]);

    let installer = &manifest.mods["eeexremote"];
    assert_eq!(installer.language, 0);
    assert_eq!(installer.tp2, "setup-eeexremote.tp2");
    assert_eq!(installer.components.len(), 1);
    assert!(installer.components[0].prompts.is_empty());

    let feature = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:eeexremote:component-0")
        .expect("EEex Remote Console feature");
    assert_eq!(feature.decision, Decision::Optional);
    assert_eq!(feature.readiness, Readiness::Experimental);
    assert_eq!(feature.requires, ["feature:eeex:mandatory-components"]);
    assert!(feature.description.contains("arbitrary Lua"));

    let recommended = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert_eq!(recommended.plan.components_for("eeexremote-bg2"), None);

    let mut selected = Selection::defaults("windows");
    selected.set_feature("feature:eeexremote:component-0", true);
    assert_eq!(
        evaluate(&manifest, &selected)
            .unwrap()
            .plan
            .components_for("eeexremote-bg2"),
        Some(&[0][..])
    );
}

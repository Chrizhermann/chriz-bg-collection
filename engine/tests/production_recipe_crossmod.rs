use std::path::PathBuf;

use bg_engine::manifest::{AcquisitionPolicy, ArchiveRootRule, Decision, Phase, SourceKind};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn freezes_the_official_crossmod_v30_iemod() {
    let manifest = recipe();
    let artifact = &manifest.artifacts["crossmodbg2-30"];
    assert_eq!(artifact.version, "30");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(artifact.source.reference, "v30");
    assert_eq!(
        artifact.source.url,
        "https://github.com/Gibberlings3/Crossmod_Banter_Pack_for_Baldurs_Gate_II/releases/download/v30/crossmod-banter-pack-for-baldurs-gate-ii-v30.iemod"
    );
    assert_eq!(artifact.source.expected_length, Some(3_715_445));
    assert_eq!(
        artifact.source.sha256,
        "6b1555d1cf6f146f13a0c12a32008f5b586d3e063265387f42fca679ef8fe3e3"
    );
    assert_eq!(artifact.archive.root_rule, ArchiveRootRule::Direct);
    assert_eq!(artifact.archive.publish_roots, ["crossmodbg2"]);
    assert_eq!(
        artifact.archive.tp2_paths,
        ["crossmodbg2/setup-crossmodbg2.tp2"]
    );
}

#[test]
fn installs_crossmod_defaults_after_current_npcs_and_before_rules_mods() {
    let manifest = recipe();
    let runs = &manifest.collection.runs;
    let position = |id: &str| {
        runs.iter()
            .position(|run| run.run_id == id)
            .unwrap_or_else(|| panic!("missing run {id}"))
    };
    assert!(position("sirene-bg2") < position("crossmodbg2-bg2"));
    assert!(position("ascension-bg2") < position("crossmodbg2-bg2"));

    let run = &runs[position("crossmodbg2-bg2")];
    assert_eq!(run.phase, Phase::Main);
    assert_eq!(run.components, [0, 1, 2]);

    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert_eq!(
        evaluation.plan.components_for("crossmodbg2-bg2"),
        Some(&[0, 1, 2][..])
    );
    for component in 0..=2 {
        let feature = evaluation
            .view
            .control(&format!("feature:crossmodbg2:component-{component}"))
            .unwrap();
        assert_eq!(feature.decision, Decision::Default);
        assert!(feature.selected);
        assert!(feature.interactive);
    }

    let tob = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:crossmodbg2:component-1")
        .unwrap();
    assert_eq!(tob.requires, ["feature:crossmodbg2:component-0"]);

    let mut selection = Selection::defaults("windows");
    selection.set_feature("feature:crossmodbg2:component-2", false);
    let without_conflicts = evaluate(&manifest, &selection).unwrap();
    assert_eq!(
        without_conflicts.plan.components_for("crossmodbg2-bg2"),
        Some(&[0, 1][..])
    );
}

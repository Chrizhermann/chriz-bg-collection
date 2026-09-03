use std::collections::BTreeSet;
use std::path::PathBuf;

use bg_engine::manifest::{AcquisitionPolicy, ArchiveRootRule, Decision, Phase, SourceKind};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

const IEP_AUTHORED: &[u32] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const IEP_RECOMMENDED: &[u32] = &[0, 1, 2, 3, 4, 5, 8];

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn freezes_the_official_iepbanters_v59_iemod() {
    let manifest = recipe();
    let artifact = &manifest.artifacts["iepbanters-5.9"];
    assert_eq!(artifact.version, "5.9");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(artifact.source.reference, "v5.9");
    assert_eq!(
        artifact.source.url,
        "https://github.com/Spellhold-Studios/IEP-Extended-Banters/releases/download/v5.9/iep-extended-banters-v5.9.iemod"
    );
    assert_eq!(artifact.source.expected_length, Some(2_545_102));
    assert_eq!(
        artifact.source.sha256,
        "a79b06ec3b70f680da323703a0a5ef7fb59fa01bef80e371c518d1bebe4375e8"
    );
    assert_eq!(artifact.archive.root_rule, ArchiveRootRule::Direct);
    assert_eq!(artifact.archive.publish_roots, ["iepbanters"]);
    assert_eq!(artifact.archive.tp2_paths, ["iepbanters/iepbanters.tp2"]);
}

#[test]
fn installs_iepbanters_before_crossmod_with_one_default_interval() {
    let manifest = recipe();
    let runs = &manifest.collection.runs;
    let position = |id: &str| {
        runs.iter()
            .position(|run| run.run_id == id)
            .unwrap_or_else(|| panic!("missing run {id}"))
    };
    assert!(position("eet-initialize-bg2") < position("iepbanters-bg2"));
    assert!(position("sirene-bg2") < position("iepbanters-bg2"));
    assert!(position("iepbanters-bg2") < position("crossmodbg2-bg2"));
    assert!(position("iepbanters-bg2") < position("eet-end-bg2"));
    let run = &runs[position("iepbanters-bg2")];
    assert_eq!(run.phase, Phase::Main);
    assert_eq!(run.components, IEP_AUTHORED);
    assert!(manifest.mods["iepbanters"]
        .components
        .iter()
        .all(|component| component.prompts.is_empty()));

    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert_eq!(
        evaluation.plan.components_for("iepbanters-bg2"),
        Some(IEP_RECOMMENDED)
    );

    let interval_ids = (6..=11)
        .map(|component| format!("feature:iepbanters:component-{component}"))
        .collect::<BTreeSet<_>>();
    for component in 6..=11 {
        let id = format!("feature:iepbanters:component-{component}");
        let feature = manifest
            .collection
            .features
            .iter()
            .find(|feature| feature.id == id)
            .unwrap();
        assert_eq!(
            feature.decision,
            if component == 8 {
                Decision::Default
            } else {
                Decision::Optional
            }
        );
        let conflicts = feature
            .conflicts
            .iter()
            .map(|conflict| conflict.feature_id.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            conflicts,
            interval_ids
                .iter()
                .filter(|other| **other != id)
                .cloned()
                .collect()
        );
    }

    let mut selection = Selection::defaults("windows");
    selection.set_feature("feature:iepbanters:component-8", false);
    selection.set_feature("feature:iepbanters:component-6", true);
    let ten_minutes = evaluate(&manifest, &selection).unwrap();
    assert_eq!(
        ten_minutes.plan.components_for("iepbanters-bg2"),
        Some(&[0, 1, 2, 3, 4, 5, 6][..])
    );
}

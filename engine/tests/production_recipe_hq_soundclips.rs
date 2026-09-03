use std::path::PathBuf;

use bg_engine::manifest::{AcquisitionPolicy, ArchiveRootRule, Decision, Phase, SourceKind};
use bg_engine::recipe_view::evaluate_preset;
use bg_engine::Manifest;

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn freezes_the_official_hq_soundclips_v13_windows_archive() {
    let manifest = recipe();
    let artifact = &manifest.artifacts["hq-soundclips-bg2ee-1.3"];
    assert_eq!(artifact.version, "1.3");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(artifact.source.reference, "v1.3");
    assert_eq!(
        artifact.source.url,
        "https://github.com/Argent77/HQ-SoundClips-BG2EE/releases/download/v1.3/win-A7-HQ-SoundClips-BG2EE-v1.3.zip"
    );
    assert_eq!(artifact.source.expected_length, Some(206_363_285));
    assert_eq!(
        artifact.source.sha256,
        "712af8ab21048c0a1dbba7257c33b3dc6beba12d86d4d4c7a1b80982adc6ee63"
    );
    assert_eq!(artifact.archive.root_rule, ArchiveRootRule::Direct);
    assert_eq!(artifact.archive.publish_roots, ["HQ_SoundClips_BG2EE"]);
    assert_eq!(
        artifact.archive.tp2_paths,
        ["HQ_SoundClips_BG2EE/HQ_SoundClips_BG2EE.tp2"]
    );
}

#[test]
fn installs_hq_soundclips_after_content_and_before_scs_by_default() {
    let manifest = recipe();
    let runs = &manifest.collection.runs;
    let position = |id: &str| {
        runs.iter()
            .position(|run| run.run_id == id)
            .unwrap_or_else(|| panic!("missing run {id}"))
    };
    assert!(position("crossmodbg2-bg2") < position("hq-soundclips-bg2ee-bg2"));
    assert!(position("ascension-bg2") < position("hq-soundclips-bg2ee-bg2"));
    assert!(position("hq-soundclips-bg2ee-bg2") < position("stratagems-bg2"));
    assert!(position("hq-soundclips-bg2ee-bg2") < position("eet-end-bg2"));

    let run = &runs[position("hq-soundclips-bg2ee-bg2")];
    assert_eq!(run.phase, Phase::Main);
    assert_eq!(run.components, [0]);

    let installer = &manifest.mods["hq-soundclips-bg2ee"];
    assert_eq!(installer.language, 0);
    assert_eq!(installer.components.len(), 1);
    assert_eq!(installer.components[0].id, 0);
    assert!(installer.components[0].prompts.is_empty());

    let feature = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:hq-soundclips-bg2ee:component-0")
        .expect("HQ Soundclips feature");
    assert_eq!(feature.decision, Decision::Default);

    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert_eq!(
        evaluation.plan.components_for("hq-soundclips-bg2ee-bg2"),
        Some(&[0][..])
    );
}

use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveRootRule, Decision, Phase, Readiness, SourceKind,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

const MODPACK_COMPONENTS: &[u32] = &[
    110, 130, 140, 170, 190, 192, 193, 194, 195, 196, 197, 198, 220, 221, 222, 223, 400, 410, 430,
    440, 450, 610,
];

const RECOMMENDED_COMPONENTS: &[u32] = &[
    110, 130, 140, 170, 190, 192, 193, 194, 195, 196, 197, 198, 220, 221, 410, 440, 450, 610,
];

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn freezes_the_public_modpack_alpha_without_publishing_its_bundled_weidu() {
    let manifest = recipe();
    let artifact = &manifest.artifacts["chriz-bg-modpack-0.2.0-alpha.5"];
    assert_eq!(artifact.version, "0.2.0-alpha.5");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(artifact.source.reference, "v0.2.0-alpha.5");
    assert_eq!(
        artifact.source.url,
        "https://github.com/Chrizhermann/chriz-bg-modpack/releases/download/v0.2.0-alpha.5/chriz-bg-modpack-v0.2.0-alpha.5.zip"
    );
    assert_eq!(
        artifact.source.expected_filename.as_deref(),
        Some("chriz-bg-modpack-v0.2.0-alpha.5.zip")
    );
    assert_eq!(artifact.source.expected_length, Some(1_335_026));
    assert_eq!(
        artifact.source.sha256,
        "2278c839f60e019bedba355cb794176a052d24b68db2851248af926580840b33"
    );
    assert_eq!(artifact.archive.root_rule, ArchiveRootRule::Direct);
    assert_eq!(
        artifact.archive.publish_roots,
        ["chriz-bg-modpack", "setup-chriz-bg-modpack.tp2"]
    );
    assert_eq!(artifact.archive.tp2_paths, ["setup-chriz-bg-modpack.tp2"]);
    assert!(artifact
        .archive
        .publish_roots
        .iter()
        .all(|root| !root.ends_with(".exe")));
}

#[test]
fn authors_all_approved_components_between_remote_console_and_buffbot() {
    let manifest = recipe();
    let runs = &manifest.collection.runs;
    let position = |id: &str| {
        runs.iter()
            .position(|run| run.run_id == id)
            .unwrap_or_else(|| panic!("missing run {id}"))
    };
    let remote = position("eeexremote-bg2");
    let modpack = position("chriz-bg-modpack-bg2");
    let spellbooks = position("spell-rev-npc-spellbooks-bg2");
    let buffbot = position("buffbot-bg2");
    assert_eq!(remote + 1, modpack);
    assert_eq!(modpack + 1, spellbooks);
    assert_eq!(spellbooks + 1, buffbot);
    assert_eq!(buffbot, runs.len() - 1);

    let run = &runs[modpack];
    assert_eq!(run.phase, Phase::PostEetEnd);
    assert_eq!(run.components, MODPACK_COMPONENTS);

    let installer = &manifest.mods["chriz-bg-modpack"];
    assert_eq!(installer.artifact_id, "chriz-bg-modpack-0.2.0-alpha.5");
    assert_eq!(installer.tp2, "setup-chriz-bg-modpack.tp2");
    assert_eq!(installer.language, 0);
    assert_eq!(installer.weidu_artifact_id, "weidu-249-amd64");
    assert_eq!(
        installer
            .components
            .iter()
            .map(|component| component.id)
            .collect::<Vec<_>>(),
        MODPACK_COMPONENTS
    );
    assert!(installer
        .components
        .iter()
        .all(|component| component.prompts.is_empty()));
}

#[test]
fn recommended_preset_selects_ready_defaults_and_keeps_missing_prerequisites_visible() {
    let manifest = recipe();
    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    assert_eq!(
        evaluation.plan.components_for("chriz-bg-modpack-bg2"),
        Some(RECOMMENDED_COMPONENTS)
    );

    let parent = evaluation
        .view
        .control("mod:chriz-bg-modpack")
        .expect("modpack parent");
    assert_eq!(parent.decision, Decision::Default);
    assert_eq!(parent.readiness, Readiness::Ready);
    assert!(parent.selected);

    for component in [400, 430] {
        let id = format!("feature:chriz-bg-modpack:component-{component}");
        let control = evaluation
            .view
            .control(&id)
            .unwrap_or_else(|| panic!("missing control {id}"));
        assert_eq!(control.readiness, Readiness::Blocked, "{id}");
        assert!(!control.selected, "{id}");
        assert!(!control.interactive, "{id}");
        assert!(control.unavailable_reason.is_some(), "{id}");
    }

    for component in [110, 190, 192, 193, 194, 195, 196, 197, 198, 220, 221, 610] {
        let id = format!("feature:chriz-bg-modpack:component-{component}");
        let control = evaluation
            .view
            .control(&id)
            .unwrap_or_else(|| panic!("missing control {id}"));
        assert_eq!(control.decision, Decision::Default, "{id}");
        assert_eq!(control.readiness, Readiness::Ready, "{id}");
        assert!(control.selected, "{id}");
    }

    for component in [130, 140, 170, 410, 440, 450] {
        let id = format!("feature:chriz-bg-modpack:component-{component}");
        let control = evaluation
            .view
            .control(&id)
            .unwrap_or_else(|| panic!("missing control {id}"));
        assert_eq!(control.decision, Decision::Mandatory, "{id}");
        assert_eq!(control.readiness, Readiness::Ready, "{id}");
        assert!(control.selected, "{id}");
    }

    for component in [222, 223] {
        let id = format!("feature:chriz-bg-modpack:component-{component}");
        let control = evaluation
            .view
            .control(&id)
            .unwrap_or_else(|| panic!("missing control {id}"));
        assert_eq!(control.decision, Decision::Optional, "{id}");
        assert_eq!(control.readiness, Readiness::Ready, "{id}");
        assert!(!control.selected, "{id}");
    }

    let utility_xp = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:chriz-bg-modpack:component-610")
        .expect("utility XP feature");
    assert_eq!(
        utility_xp.requires,
        ["feature:eeex:mandatory-components", "mod:eet-end"]
    );

    // The base manifest includes the approved SoD skip (910) added after alpha.10.
    // Its plan is distinct from the 434-component curated runtime recipe.
    assert_eq!(
        evaluation
            .plan
            .runs
            .iter()
            .map(|run| run.components.len())
            .sum::<usize>(),
        345
    );
}

#[test]
fn hexxat_choices_are_mutually_exclusive_and_shadowdancer_is_the_default() {
    let manifest = recipe();
    for component in [221, 222, 223] {
        let feature = manifest
            .collection
            .features
            .iter()
            .find(|feature| feature.id == format!("feature:chriz-bg-modpack:component-{component}"))
            .unwrap_or_else(|| panic!("missing Hexxat component {component}"));
        let conflicts = feature
            .conflicts
            .iter()
            .map(|conflict| conflict.feature_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(conflicts.len(), 3);
        for other in [221, 222, 223] {
            if component != other {
                assert!(conflicts
                    .contains(&format!("feature:chriz-bg-modpack:component-{other}").as_str()));
            }
        }
        assert!(conflicts.contains(&"feature:artisanskitpack-npc:component-7104"));
    }

    let artisan_hexxat = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:artisanskitpack-npc:component-7104")
        .expect("Artisan Hexxat option");
    assert_eq!(artisan_hexxat.decision, Decision::Optional);
    for component in [221, 222, 223] {
        assert!(artisan_hexxat.conflicts.iter().any(|conflict| {
            conflict.feature_id == format!("feature:chriz-bg-modpack:component-{component}")
        }));
    }

    let defaults = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
    let artisan_control = defaults
        .view
        .control("feature:artisanskitpack-npc:component-7104")
        .expect("Artisan Hexxat control");
    assert!(!artisan_control.selected);
    assert!(!artisan_control.interactive);
    assert!(artisan_control
        .unavailable_reason
        .as_deref()
        .expect("active alternative reason")
        .contains("Shadowdancer"));

    let mut fighter_thief = Selection::defaults("windows");
    fighter_thief.set_feature("feature:chriz-bg-modpack:component-221", false);
    fighter_thief.set_feature("feature:chriz-bg-modpack:component-222", true);
    let evaluation = evaluate(&manifest, &fighter_thief).unwrap();
    let components = evaluation
        .plan
        .components_for("chriz-bg-modpack-bg2")
        .expect("modpack run");
    assert!(!components.contains(&221));
    assert!(components.contains(&222));
    assert!(!components.contains(&223));

    let mut from_artisan = Selection::defaults("windows");
    from_artisan.set_feature("feature:chriz-bg-modpack:component-221", false);
    from_artisan.set_feature("feature:artisanskitpack-npc:component-7104", true);
    let artisan_evaluation = evaluate(&manifest, &from_artisan).unwrap();
    assert!(artisan_evaluation
        .plan
        .components_for("artisanskitpack-npc-bg2")
        .expect("Artisan NPC run")
        .contains(&7104));

    from_artisan.set_feature("feature:artisanskitpack-npc:component-7104", false);
    from_artisan.set_feature("feature:chriz-bg-modpack:component-221", true);
    let shadowdancer_evaluation = evaluate(&manifest, &from_artisan).unwrap();
    assert!(shadowdancer_evaluation
        .plan
        .components_for("chriz-bg-modpack-bg2")
        .expect("modpack run")
        .contains(&221));
    assert!(!shadowdancer_evaluation
        .plan
        .components_for("artisanskitpack-npc-bg2")
        .expect("Artisan NPC run")
        .contains(&7104));

    let mut no_artisan = Selection::defaults("windows");
    no_artisan.set_feature("mod:artisanskitpack-npc", false);
    let no_artisan_evaluation = evaluate(&manifest, &no_artisan).unwrap();
    assert!(no_artisan_evaluation
        .plan
        .components_for("chriz-bg-modpack-bg2")
        .expect("modpack run remains independent")
        .contains(&221));
}

#[test]
fn modpack_can_be_disabled_and_conditional_fixes_follow_their_dependencies() {
    let manifest = recipe();
    let mut defaults = Selection::defaults("windows");
    defaults.set_feature("mod:chriz-bg-modpack", false);
    assert_eq!(
        evaluate(&manifest, &defaults)
            .unwrap()
            .plan
            .components_for("chriz-bg-modpack-bg2"),
        None
    );

    let fade_fix = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:chriz-bg-modpack:component-110")
        .expect("Fade fix feature");
    assert!(fade_fix
        .conflicts
        .iter()
        .any(|conflict| conflict.feature_id == "feature:fade:component-2"));

    let yeslick_fix = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:chriz-bg-modpack:component-410")
        .expect("Yeslick/Keldorn fix feature");
    assert!(yeslick_fix
        .requires
        .iter()
        .any(|required| required == "feature:yeslicknpc:component-1"));
    assert!(yeslick_fix
        .requires
        .iter()
        .any(|required| required == "feature:stratagems:component-3540"));
    assert!(yeslick_fix
        .conflicts
        .iter()
        .any(|conflict| conflict.feature_id == "feature:stratagems:component-3541"));

    let mut alternate_dispel = Selection::defaults("windows");
    alternate_dispel.set_feature("mod:chriz-bg-modpack", true);
    alternate_dispel.set_feature("mod:fade", true);
    alternate_dispel.set_feature("feature:chriz-bg-modpack:component-110", true);
    alternate_dispel.set_feature("feature:yeslicknpc:component-1", true);
    alternate_dispel.set_feature("mod:stratagems", true);
    alternate_dispel.set_feature("feature:stratagems:component-3540", false);
    alternate_dispel.set_feature("feature:stratagems:component-3541", true);
    let evaluation = evaluate(&manifest, &alternate_dispel).unwrap();
    let components = evaluation
        .plan
        .components_for("chriz-bg-modpack-bg2")
        .expect("remaining selected modpack components");
    assert!(!components.contains(&410));
}

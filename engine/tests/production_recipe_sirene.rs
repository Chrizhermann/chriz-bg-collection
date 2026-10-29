//! Sirene NPC for BG:EE (October 29 intake, Christopher 2026-10-29): the EET-ready commit
//! archive, the curated components and the run's place in both recipes.
use std::collections::BTreeSet;
use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveRootRule, Decision, Phase, Readiness, SourceKind,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::Manifest;

const PROFILES: [&str; 2] = ["manifest", "recipes/curated-full-current"];
const RUN: &str = "sirene-bg1-bg2";
const COMMIT: &str = "1c8600029a9ac31395a61b5d4d03f98337dcf83e";
const ARTIFACT_ID: &str = "sirene-bg1-1c8600029a9ac31395a61b5d4d03f98337dcf83e";
// Main component, the curated portrait (Alternate 1: Sirick, Light Armor) and the four
// class subcomponents; the other portraits 1, 2, 4 and 5 are not offered.
const OFFERED: &[u32] = &[0, 3, 6, 7, 8, 9];
const RECOMMENDED: &[u32] = &[0, 3, 6];
const CLASSES: [u32; 4] = [6, 7, 8, 9];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn load(profile: &str) -> Manifest {
    Manifest::load(&repo_root().join(profile))
        .unwrap_or_else(|error| panic!("{profile} should load: {error:?}"))
}

fn feature_id(component: u32) -> String {
    format!("feature:sirene-bg1:component-{component}")
}

#[test]
fn pins_the_eet_ready_master_commit_archive_in_both_recipes() {
    // Release v3.1 (2024-04) supports BG:EE only; master since 2025-09 also installs on EET.
    for profile in PROFILES {
        let manifest = load(profile);
        let artifact = &manifest.artifacts[ARTIFACT_ID];
        assert_eq!(artifact.version, COMMIT, "{profile}");
        assert_eq!(
            artifact.acquisition,
            AcquisitionPolicy::FetchOnly,
            "{profile}"
        );
        assert_eq!(
            artifact.source.kind,
            SourceKind::GithubCommitZip,
            "{profile}"
        );
        assert_eq!(artifact.source.reference, COMMIT, "{profile}");
        assert_eq!(
            artifact.source.url,
            format!("https://codeload.github.com/TheArtisanBG/Sirene-BG1/zip/{COMMIT}"),
            "{profile}"
        );
        assert_eq!(
            artifact.source.expected_filename.as_deref(),
            Some(format!("Sirene-BG1-{COMMIT}.zip").as_str()),
            "{profile}"
        );
        assert_eq!(
            artifact.source.expected_length,
            Some(3_734_521),
            "{profile}"
        );
        assert_eq!(
            artifact.source.sha256,
            "f46b035395c4310103e9be2d1623a113e6f3aa66f520624c30c7f355518f4eb9",
            "{profile}"
        );
        assert_eq!(
            artifact.archive.root_rule,
            ArchiveRootRule::SingleWrapper,
            "{profile}"
        );
        assert_eq!(artifact.archive.publish_roots, ["Sirene"], "{profile}");
        assert_eq!(
            artifact.archive.tp2_paths,
            ["Sirene/Setup-Sirene.tp2"],
            "{profile}"
        );

        let installer = &manifest.mods["sirene-bg1"];
        assert_eq!(installer.artifact_id, ARTIFACT_ID, "{profile}");
        assert_eq!(installer.tp2, "Sirene/Setup-Sirene.tp2", "{profile}");
        assert_eq!(installer.weidu_artifact_id, "weidu-249-amd64", "{profile}");
        assert_eq!(
            installer
                .components
                .iter()
                .map(|component| component.id)
                .collect::<Vec<_>>(),
            OFFERED,
            "{profile}"
        );
    }
}

#[test]
fn installs_after_kitpack_main_and_before_sirene_bg2_the_kitpack_npcs_and_the_banter_mods() {
    for profile in PROFILES {
        let manifest = load(profile);
        let runs = &manifest.collection.runs;
        let position = |id: &str| {
            runs.iter()
                .position(|run| run.run_id == id)
                .unwrap_or_else(|| panic!("{profile}: missing run {id}"))
        };
        let sirene_bg1 = position(RUN);
        let sirene_bg2 = position("sirene-bg2");
        // Both Sirene mods skip their own Martyr kit once Kitpack Martyr 3005 is
        // installed, so the Kitpack's player-selectable Martyr stays the only one.
        assert!(
            position("artisanskitpack-main-bg2") < sirene_bg1,
            "{profile}"
        );
        // The BG2 mod's EET continuity block needs the BG1 mod's c0sirene.cre.
        assert!(sirene_bg1 < sirene_bg2, "{profile}");
        assert!(
            sirene_bg2 < position("artisanskitpack-npc-bg2"),
            "{profile}"
        );
        // IEP Banters and Crossmod detect the NPCs that are already installed.
        for later in ["iepbanters-bg2", "crossmodbg2-bg2", "eet-end-bg2"] {
            assert!(sirene_bg1 < position(later), "{profile}: {later}");
            assert!(sirene_bg2 < position(later), "{profile}: {later}");
        }
        let run = &runs[sirene_bg1];
        assert_eq!(run.mod_id, "sirene-bg1", "{profile}");
        assert_eq!(run.phase, Phase::Main, "{profile}");
        assert_eq!(run.components, OFFERED, "{profile}");

        let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        let plan = &evaluation.plan;
        assert_eq!(plan.components_for(RUN), Some(RECOMMENDED), "{profile}");
        assert_eq!(
            plan.components_for("sirene-bg2"),
            Some(&[0, 2, 5][..]),
            "{profile}"
        );
        assert!(
            plan.components_for("artisanskitpack-main-bg2")
                .is_some_and(|components| components.contains(&3005)),
            "{profile}: Recommended installs the Kitpack's Martyr"
        );
        let planned = |id: &str| {
            plan.runs
                .iter()
                .position(|run| run.run_id == id)
                .unwrap_or_else(|| panic!("{profile}: {id} is not planned"))
        };
        assert!(
            planned("artisanskitpack-main-bg2") < planned(RUN),
            "{profile}"
        );
        assert!(planned(RUN) < planned("sirene-bg2"), "{profile}");
        assert!(
            planned("sirene-bg2") < planned("artisanskitpack-npc-bg2"),
            "{profile}"
        );
    }
}

#[test]
fn offers_the_curated_portrait_and_one_class_at_a_time() {
    for profile in PROFILES {
        let manifest = load(profile);
        let features = manifest
            .collection
            .features
            .iter()
            .filter(|feature| {
                feature
                    .components
                    .iter()
                    .any(|component| component.run_id == RUN)
            })
            .collect::<Vec<_>>();
        let offered = features
            .iter()
            .flat_map(|feature| {
                feature
                    .components
                    .iter()
                    .map(|component| component.component)
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(offered, OFFERED.iter().copied().collect(), "{profile}");

        for component in OFFERED {
            let id = feature_id(*component);
            let feature = features
                .iter()
                .find(|feature| feature.id == id)
                .unwrap_or_else(|| panic!("{profile}: missing {id}"));
            let expected = if RECOMMENDED.contains(component) {
                Decision::Default
            } else {
                Decision::Optional
            };
            assert_eq!(feature.decision, expected, "{profile}: {id}");
            assert_eq!(feature.readiness, Readiness::Ready, "{profile}: {id}");
            if *component == 0 {
                assert_eq!(feature.parent, None, "{profile}");
                assert!(feature.title.contains("Siege of Dragonspear"), "{profile}");
            } else {
                assert_eq!(feature.parent, Some(feature_id(0)), "{profile}: {id}");
            }
            if CLASSES.contains(component) {
                let conflicts = feature
                    .conflicts
                    .iter()
                    .map(|conflict| conflict.feature_id.clone())
                    .collect::<BTreeSet<_>>();
                let others = CLASSES
                    .iter()
                    .filter(|other| *other != component)
                    .map(|other| feature_id(*other))
                    .collect::<BTreeSet<_>>();
                assert_eq!(conflicts, others, "{profile}: {id}");
                if profile != "manifest" {
                    assert_eq!(
                        feature.choice_group.as_deref(),
                        Some("choice:sirene-bg1:choose-an-alternate-class-for-sirene"),
                        "{profile}: {id}"
                    );
                }
            }
        }

        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        for component in OFFERED {
            let control = baseline.view.control(&feature_id(*component)).unwrap();
            let default = RECOMMENDED.contains(component);
            assert_eq!(control.selected, default, "{profile}: {component}");
            if default {
                assert!(control.interactive, "{profile}: {component}");
            } else if profile != "manifest" {
                // The curated choice group swaps the True Paladin for the picked class.
                assert_eq!(
                    control.choice_available,
                    Some(true),
                    "{profile}: {component}"
                );
            }
        }

        // An alternate class replaces the True Paladin.
        for class in [7, 8, 9] {
            let mut selection = baseline.normalized_selection.to_selection();
            selection.set_feature(&feature_id(6), false);
            selection.set_feature(&feature_id(class), true);
            let result = evaluate(&manifest, &selection).unwrap();
            assert_eq!(
                result.plan.components_for(RUN),
                Some(&[0, 3, class][..]),
                "{profile}: class {class}"
            );
        }

        // Without the NPC, neither her portrait nor her class installs.
        let mut selection = baseline.normalized_selection.to_selection();
        selection.set_feature(&feature_id(0), false);
        let result = evaluate(&manifest, &selection).unwrap();
        assert_eq!(result.plan.components_for(RUN), None, "{profile}");
        assert_eq!(
            result.plan.components_for("sirene-bg2"),
            Some(&[0, 2, 5][..]),
            "{profile}"
        );
    }
}

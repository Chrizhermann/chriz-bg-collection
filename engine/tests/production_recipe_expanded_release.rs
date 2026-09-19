//! Static checks for the September 16 release runs as authored in the bundled
//! `manifest/` (public-alpha) profile. The curated generator copies these run
//! definitions verbatim; this test covers the fallback profile itself, using the
//! same preset evaluator as release validation. Not game or installer acceptance.

use std::collections::BTreeSet;
use std::path::PathBuf;

use bg_engine::manifest::{Phase, SourceKind};
use bg_engine::recipe_view::{evaluate, evaluate_preset, SelectionEvaluation};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

const SRCB: &str = "srcb-rr-compat-bg2";
const PRE: &str = "chriz-bg-modpack-pre-continuity-bg2";
const CONTINUITY: &str = "chriz-bg-modpack-continuity-bg2";
const SAFANA: &str = "safana-bg2";
const LATE: &str = "chriz-bg-modpack-late-companions-bg2";
const LIGHTNING: &str = "spell-rev-lightning-bg2";

fn recipe() -> Manifest {
    Manifest::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest"))
        .expect("production recipe should load")
}

fn recommended(manifest: &Manifest) -> SelectionEvaluation {
    evaluate_preset(manifest, "chris-recommended", "windows").unwrap()
}

/// The recommended preset with explicit feature overrides.
fn with(manifest: &Manifest, overrides: &[(&str, bool)]) -> SelectionEvaluation {
    let mut selection = Selection {
        platform: "windows".to_owned(),
        choices: manifest.presets["chris-recommended"].selections.clone(),
    };
    for (feature, on) in overrides {
        selection.set_feature(feature, *on);
    }
    evaluate(manifest, &selection).unwrap()
}

fn selected(evaluation: &SelectionEvaluation) -> BTreeSet<(String, u32)> {
    evaluation
        .plan
        .runs
        .iter()
        .flat_map(|run| {
            run.components
                .iter()
                .map(|component| (run.run_id.clone(), *component))
        })
        .collect()
}

fn position(manifest: &Manifest, run_id: &str) -> usize {
    manifest
        .collection
        .runs
        .iter()
        .position(|run| run.run_id == run_id)
        .unwrap_or_else(|| panic!("missing run {run_id}"))
}

#[test]
fn authors_the_six_runs_exactly() {
    let manifest = recipe();
    let expected: [(&str, &str, Phase, &[u32]); 6] = [
        (SRCB, "srcb-rr-compat", Phase::Main, &[0]),
        (
            PRE,
            "chriz-bg-modpack",
            Phase::Main,
            &[
                110, 140, 170, 188, 190, 192, 193, 194, 195, 196, 197, 198, 220, 221, 222, 223,
            ],
        ),
        (CONTINUITY, "chriz-bg-modpack", Phase::Main, &[199]),
        (SAFANA, "safana", Phase::PostEetEnd, &[0]),
        (LATE, "chriz-bg-modpack", Phase::PostEetEnd, &[189, 620]),
        (LIGHTNING, "spell-rev", Phase::PostEetEnd, &[80, 81]),
    ];
    for (run_id, mod_id, phase, components) in expected {
        let run = &manifest.collection.runs[position(&manifest, run_id)];
        assert_eq!(run.mod_id, mod_id, "{run_id}");
        assert_eq!(run.phase, phase, "{run_id}");
        assert_eq!(run.components, components, "{run_id}");
    }
    let modpack = &manifest.collection.runs[position(&manifest, "chriz-bg-modpack-bg2")];
    for component in &manifest.collection.runs[position(&manifest, PRE)].components {
        assert!(!modpack.components.contains(component), "{component}");
    }
}

#[test]
fn recommended_preset_resolves_the_reviewed_components() {
    let manifest = recipe();
    let plan = recommended(&manifest).plan;
    assert_eq!(plan.components_for(SRCB), Some(&[0][..]));
    assert_eq!(
        plan.components_for(PRE),
        Some(&[110, 140, 170, 188, 190, 192, 193, 194, 195, 196, 197, 198, 220, 221][..])
    );
    assert_eq!(plan.components_for(CONTINUITY), Some(&[199][..]));
    assert_eq!(plan.components_for(SAFANA), Some(&[0][..]));
    assert_eq!(plan.components_for(LATE), Some(&[189, 620][..]));
    assert_eq!(plan.components_for(LIGHTNING), Some(&[80][..]));
}

#[test]
fn dependency_and_opt_out_behaviour_is_bounded() {
    let manifest = recipe();
    let base = selected(&recommended(&manifest));

    let no_sr = selected(&with(&manifest, &[("mod:spell-rev", false)]));
    assert!(no_sr.iter().all(|(run, _)| run != SRCB && run != LIGHTNING));

    for (first, second, expected) in [
        (true, true, true),
        (true, false, true),
        (false, true, true),
        (false, false, false),
    ] {
        let rr = selected(&with(
            &manifest,
            &[
                ("feature:rr:component-11", first),
                ("feature:rr:component-12", second),
            ],
        ));
        assert_eq!(rr.contains(&(SRCB.to_owned(), 0)), expected);
    }

    let alternative = with(
        &manifest,
        &[
            ("feature:spell-rev:component-80", false),
            ("feature:spell-rev:component-81", true),
        ],
    );
    assert_eq!(alternative.plan.components_for(LIGHTNING), Some(&[81][..]));
    let classic = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:spell-rev:component-80")
        .unwrap();
    assert!(classic
        .conflicts
        .iter()
        .any(|rule| rule.feature_id == "feature:spell-rev:component-81"));

    let no_safana = selected(&with(&manifest, &[("mod:safana", false)]));
    assert_eq!(
        base.difference(&no_safana)
            .cloned()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([(SAFANA.to_owned(), 0), (LATE.to_owned(), 189)])
    );

    let vanilla_yeslick = selected(&with(
        &manifest,
        &[
            ("feature:yeslicknpc:component-0", true),
            ("feature:yeslicknpc:component-1", false),
        ],
    ));
    assert!(!vanilla_yeslick.contains(&(PRE.to_owned(), 188)));
    assert!(vanilla_yeslick.contains(&("yeslicknpc-bg2".to_owned(), 0)));
    assert!(vanilla_yeslick.contains(&(CONTINUITY.to_owned(), 199)));
}

#[test]
fn install_order_keeps_the_reviewed_anchors() {
    let manifest = recipe();
    let at = |run: &str| position(&manifest, run);
    assert!(at("rr-bg2") < at(SRCB));
    assert!(at("spell-rev-core-bg2") < at(SRCB));
    assert!(at(SRCB) < at("stratagems-bg2"));
    assert!(at(PRE) < at(CONTINUITY));
    assert_eq!(at(CONTINUITY) + 1, at("eet-end-bg2"));
    assert!(at("eet-end-bg2") < at(SAFANA));
    assert!(at("chriz-sod-remix-bg2") < at(LATE));
    assert!(at(SAFANA) < at(LATE));
    assert!(at(LATE) < at("spell-rev-npc-spellbooks-bg2"));
    assert!(at("spell-rev-npc-spellbooks-bg2") < at(LIGHTNING));
    assert!(at(LIGHTNING) < at("buffbot-bg2"));
    assert_eq!(
        manifest.collection.runs.last().unwrap().run_id,
        "buffbot-bg2"
    );
}

#[test]
fn freezes_the_reviewed_release_sources() {
    let manifest = recipe();
    for (mod_id, artifact_id, reference, length, sha256) in [
        (
            "srcb-rr-compat",
            "srcb-rr-compat-4.21-chriz.5",
            "v4.21-chriz.5",
            1_450_569,
            "c7958b467f9197cbe364791796e15678b95f11020d27b14e45ed471b0ff6968d",
        ),
        (
            "chriz-bg-modpack",
            "chriz-bg-modpack-0.2.0-alpha.7",
            "v0.2.0-alpha.7",
            1_367_203,
            "f134085e8220a4222190981f173034653efe7aa92d47caa8430682a1352d1c19",
        ),
        (
            "safana",
            "safana-in-amn-v05",
            "v05",
            1_097_005,
            "29672c878cfb8dbec35d106e4baa4559b1eded55b035e80fdbf831f08f69cad6",
        ),
        (
            "spell-rev",
            "spell-revisions-4.21-chriz.5",
            "v4.21-chriz.5",
            10_244_415,
            "fdbb5cf9c48ece39f051b7120c537f530eb21ecedb780aa65e5ebda2e1191075",
        ),
    ] {
        assert_eq!(manifest.mods[mod_id].artifact_id, artifact_id);
        let artifact = &manifest.artifacts[artifact_id];
        assert_eq!(artifact.source.reference, reference, "{artifact_id}");
        assert_eq!(
            artifact.source.expected_length,
            Some(length),
            "{artifact_id}"
        );
        assert_eq!(artifact.source.sha256, sha256, "{artifact_id}");
        assert!(matches!(
            artifact.source.kind,
            SourceKind::GithubRelease | SourceKind::GithubTagArchive
        ));
    }
}

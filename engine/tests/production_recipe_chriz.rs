use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveRootRule, Decision, Phase, Readiness, SourceKind,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

const SOD_REMIX_COMPONENTS: &[u32] = &[
    100, 110, 115, 120, 130, 135, 140, 150, 145, 160, 170, 180, 175, 185, 190, 195, 210, 197, 187,
    200, 215, 220, 225, 245, 230, 240, 250, 255, 256, 260, 265, 266, 270, 280, 290, 900, 910,
];
const SOD_REMIX_RUN_COMPONENTS: &[u32] = &[
    100, 110, 115, 120, 130, 135, 140, 150, 145, 160, 170, 180, 175, 185, 190, 195, 210, 197, 187,
    200, 215, 220, 225, 245, 230, 240, 250, 255, 256, 257, 260, 265, 266, 270, 280, 290, 900, 910,
];
const BG_REBALANCE_COMPONENTS: &[u32] = &[
    100, 101, 111, 120, 121, 200, 210, 211, 400, 401, 404, 405, 407, 408, 302, 310, 130, 220, 320,
];
const BG_REBALANCE_RUN_COMPONENTS: &[u32] = &[
    100, 101, 110, 111, 120, 121, 200, 201, 202, 210, 211, 301, 400, 401, 404, 405, 407, 408, 302,
    420, 421, 310, 130, 220, 320,
];
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
    assert_eq!(
        manifest.mods["chriz-sod-remix"].artifact_id,
        "chriz-sod-remix-0.6.12"
    );
    let artifact = &manifest.artifacts["chriz-sod-remix-0.6.12"];
    assert_eq!(artifact.version, "0.6.12");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(
        artifact.source.url,
        "https://github.com/Chrizhermann/chriz-sod-rebalance/releases/download/v0.6.12/chriz-sod-remix-v0.6.12.zip"
    );
    assert_eq!(artifact.source.reference, "v0.6.12");
    assert_eq!(
        artifact.source.expected_filename.as_deref(),
        Some("chriz-sod-remix-v0.6.12.zip")
    );
    assert_eq!(artifact.source.expected_length, Some(2_959_604));
    assert_eq!(
        artifact.source.sha256,
        "8c1a18debed45530d629efa19aa6c6d3b34317751d89683104b0c0306e67cd97"
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
    assert_eq!(run.components, SOD_REMIX_RUN_COMPONENTS);

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
        [
            "feature:chriz-sod-remix:mandatory-components",
            "feature:chriz-sod-remix:component-257"
        ]
    );
    let extra_challenge = evaluation
        .view
        .control("feature:chriz-sod-remix:component-257")
        .expect("SoD Remix optional Extra Challenge");
    assert_eq!(extra_challenge.decision, Decision::Optional);
    assert_eq!(extra_challenge.readiness, Readiness::Ready);
    assert!(!extra_challenge.selected);

    let bundle = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:chriz-sod-remix:mandatory-components")
        .expect("SoD Remix bundle");
    assert_eq!(bundle.decision, Decision::Mandatory);
    assert_eq!(bundle.readiness, Readiness::Ready);
    assert_eq!(parent.requires, ["mod:eet-end"]);
    for prerequisite in [110, 140, 150, 160] {
        assert!(run.components.contains(&prerequisite));
    }
    assert!(!run.components.contains(&291));
    assert!(
        !run.components.contains(&258),
        "fresh component 256 includes the bridge timer; 258 is only a retrofit"
    );

    let mut selection = Selection::defaults("windows");
    selection.set_feature("mod:chriz-sod-remix", false);
    let disabled = evaluate(&manifest, &selection).unwrap();
    assert_eq!(disabled.plan.components_for("chriz-sod-remix-bg2"), None);
}

#[test]
fn freezes_the_reviewed_bg_rebalance_release() {
    let manifest = recipe();
    let artifact_id = &manifest.mods["chriz-bg-rebalance"].artifact_id;
    assert_eq!(artifact_id, "chriz-bg-rebalance-0.8.1");
    let artifact = &manifest.artifacts[artifact_id];
    assert_eq!(artifact.version, "0.8.1");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(
        artifact.source.url,
        "https://github.com/Chrizhermann/chriz-bg-rebalance/releases/download/v0.8.1/chriz-bg-rebalance-v0.8.1-windows.zip"
    );
    assert_eq!(artifact.source.reference, "v0.8.1");
    assert_eq!(
        artifact.source.expected_filename.as_deref(),
        Some("chriz-bg-rebalance-v0.8.1-windows.zip")
    );
    assert_eq!(artifact.source.expected_length, Some(1_753_775));
    assert_eq!(
        artifact.source.sha256,
        "5d6cc9dfec8779e1e4534da686dc2f3849bcb795b24981c0930d95918efccf0c"
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
    assert_eq!(run.components, BG_REBALANCE_RUN_COMPONENTS);

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
                "feature:chriz-bg-rebalance:component-301",
                Decision::Optional,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-302",
                Decision::Default,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-130",
                Decision::Default,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-200",
                Decision::Default,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-201",
                Decision::Optional,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-202",
                Decision::Optional,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-210",
                Decision::Default,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-211",
                Decision::Default,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-220",
                Decision::Default,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-310",
                Decision::Default,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-320",
                Decision::Default,
                1
            ),
            (
                "feature:chriz-bg-rebalance:bard-spell-ceilings",
                Decision::Optional,
                2
            ),
            (
                "feature:chriz-bg-rebalance:tempus-bundle",
                Decision::Default,
                6
            ),
            (
                "feature:chriz-bg-rebalance:component-110",
                Decision::Optional,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-111",
                Decision::Default,
                1
            ),
        ]
    );

    let mut no_tempus = Selection::defaults("windows");
    no_tempus.set_feature("feature:chriz-bg-rebalance:tempus-bundle", false);
    let no_tempus = evaluate(&manifest, &no_tempus).unwrap();
    assert_eq!(
        no_tempus.plan.components_for("chriz-bg-rebalance-bg2"),
        Some(&[100, 101, 111, 120, 121, 200, 210, 211, 302, 310, 130, 220, 320][..])
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

#[test]
fn defaults_wing_buffet_spacing_with_scs_and_an_independent_optout() {
    let manifest = recipe();
    let feature_id = "feature:chriz-bg-rebalance:component-111";
    let run_id = "chriz-bg-rebalance-bg2";
    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let wing_buffet = evaluation.view.control(feature_id).unwrap();
    assert_eq!(wing_buffet.decision, Decision::Default);
    assert_eq!(wing_buffet.readiness, Readiness::Ready);
    assert!(wing_buffet.selected);
    assert!(wing_buffet.interactive);
    assert_eq!(
        wing_buffet.requires,
        ["feature:stratagems:mandatory-components"]
    );
    let lethal_melee = evaluation
        .view
        .control("feature:chriz-bg-rebalance:component-110")
        .unwrap();
    assert_eq!(lethal_melee.decision, Decision::Optional);
    assert!(!lethal_melee.selected);

    let mut selection = Selection::defaults("windows");
    let defaults = evaluate(&manifest, &selection).unwrap();
    assert_eq!(
        defaults.plan.components_for(run_id),
        Some(BG_REBALANCE_COMPONENTS)
    );
    selection.set_feature(feature_id, false);
    let opted_out = evaluate(&manifest, &selection).unwrap();
    assert_eq!(
        opted_out.plan.components_for(run_id),
        Some(
            &[
                100, 101, 120, 121, 200, 210, 211, 400, 401, 404, 405, 407, 408, 302, 310, 130,
                220, 320
            ][..]
        )
    );

    let mut no_scs = Selection::defaults("windows");
    no_scs.set_feature("mod:stratagems", false);
    let no_scs = evaluate(&manifest, &no_scs).unwrap();
    assert!(!no_scs.plan.components_for(run_id).unwrap().contains(&111));
}

#[test]
fn hardiness_choices_are_exclusive_and_extra_protection_requires_a_physical_choice() {
    const FORTY: &str = "feature:chriz-bg-rebalance:component-200";
    const THIRTY: &str = "feature:chriz-bg-rebalance:component-201";
    const EXTRA: &str = "feature:chriz-bg-rebalance:component-202";
    const RUN: &str = "chriz-bg-rebalance-bg2";

    for directory in ["manifest", "recipes/curated-full-current"] {
        let manifest = Manifest::load(&recipe_root().join("..").join(directory)).unwrap();
        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert!(baseline.view.control(FORTY).unwrap().selected);
        assert!(!baseline.view.control(THIRTY).unwrap().selected);
        assert!(!baseline.view.control(EXTRA).unwrap().selected);
        assert!(manifest
            .collection
            .features
            .iter()
            .all(|feature| feature.id != "feature:chriz-bg-rebalance:component-311"));

        for (forty, thirty, extra, expected) in [
            (true, false, false, &[200][..]),
            (false, true, false, &[201][..]),
            (true, false, true, &[200, 202][..]),
            (false, true, true, &[201, 202][..]),
            (false, false, false, &[][..]),
            (false, false, true, &[][..]),
            (true, true, false, &[][..]),
            (true, true, true, &[][..]),
        ] {
            let mut selection = baseline.normalized_selection.to_selection();
            selection.set_feature(FORTY, forty);
            selection.set_feature(THIRTY, thirty);
            selection.set_feature(EXTRA, extra);
            let result = evaluate(&manifest, &selection).unwrap();
            let selected: Vec<_> = result
                .plan
                .components_for(RUN)
                .unwrap_or_default()
                .iter()
                .copied()
                .filter(|component| [200, 201, 202].contains(component))
                .collect();
            assert_eq!(
                selected, expected,
                "{directory}: 40%={forty}, 30%={thirty}, extra={extra}"
            );
            for (feature, component) in [(FORTY, 200), (THIRTY, 201), (EXTRA, 202)] {
                assert_eq!(
                    result.view.control(feature).unwrap().selected,
                    expected.contains(&component),
                    "{directory}: {feature} UI and installed plan must agree"
                );
            }
        }

        for disabled in [
            "mod:spell-rev",
            "feature:spell-rev:component-65",
            "mod:chriz-bg-rebalance",
        ] {
            for physical in [FORTY, THIRTY] {
                let mut selection = baseline.normalized_selection.to_selection();
                selection.set_feature(FORTY, false);
                selection.set_feature(THIRTY, false);
                selection.set_feature(physical, true);
                selection.set_feature(EXTRA, true);
                selection.set_feature(disabled, false);
                let result = evaluate(&manifest, &selection).unwrap();
                assert!(result
                    .plan
                    .components_for(RUN)
                    .unwrap_or_default()
                    .iter()
                    .all(|component| ![200, 201, 202].contains(component)));
                for feature in [FORTY, THIRTY, EXTRA] {
                    assert!(
                        !result.view.control(feature).unwrap().selected,
                        "{directory}: {feature} must not override disabled {disabled}"
                    );
                }
            }
        }
    }
}

#[test]
fn approved_balance_defaults_respect_sr_and_eeex_prerequisites() {
    for directory in ["manifest", "recipes/curated-full-current"] {
        let manifest = Manifest::load(&recipe_root().join("..").join(directory)).unwrap();
        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        let run = "chriz-bg-rebalance-bg2";
        for component in [130, 200, 210, 211, 220, 310, 320] {
            assert!(baseline
                .plan
                .components_for(run)
                .unwrap()
                .contains(&component));
        }
        let mut no_sr = baseline.normalized_selection.to_selection();
        no_sr.set_feature("mod:spell-rev", false);
        let no_sr = evaluate(&manifest, &no_sr).unwrap();
        assert!(no_sr
            .plan
            .components_for(run)
            .unwrap()
            .iter()
            .all(|component| ![200, 201, 202, 210, 211, 220].contains(component)));
        assert!(no_sr.plan.components_for(run).unwrap().contains(&310));

        let mut no_eeex = baseline.normalized_selection.to_selection();
        no_eeex.set_feature("mod:eeex", false);
        let no_eeex = evaluate(&manifest, &no_eeex).unwrap();
        let components = no_eeex.plan.components_for(run).unwrap();
        assert!(!components.contains(&210));
        assert!(!components.contains(&310));
        for independent in [130, 200, 211, 220, 320] {
            assert!(components.contains(&independent));
        }
    }
}

#[test]
fn bard_spell_ceilings_are_an_unchecked_eeex_choice_in_both_profiles() {
    const BASE: &str = "feature:chriz-bg-rebalance:bard-spell-ceilings";
    const RUN: &str = "chriz-bg-rebalance-bg2";
    for directory in ["manifest", "recipes/curated-full-current"] {
        let manifest = Manifest::load(&recipe_root().join("..").join(directory)).unwrap();
        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert_eq!(
            manifest.presets["chris-recommended"].selections[BASE], "off",
            "{directory}"
        );
        let control = baseline.view.control(BASE).unwrap();
        assert_eq!(control.decision, Decision::Optional, "{directory}");
        assert!(!control.selected && control.interactive, "{directory}");
        assert_eq!(control.requires, ["feature:eeex:mandatory-components"]);
        assert_eq!(
            baseline.plan.components_for(RUN),
            Some(BG_REBALANCE_COMPONENTS),
            "{directory}"
        );

        let mut selection = baseline.normalized_selection.to_selection();
        selection.set_feature(BASE, true);
        let enabled = evaluate(&manifest, &selection).unwrap();
        let mut expected = BG_REBALANCE_COMPONENTS.to_vec();
        let after_flail = expected.iter().position(|c| *c == 302).unwrap() + 1;
        expected.splice(after_flail..after_flail, [420, 421]);
        assert_eq!(
            enabled.plan.components_for(RUN),
            Some(&expected[..]),
            "{directory}"
        );
        let mut others = enabled.plan.clone();
        others.runs.retain(|run| run.run_id != RUN);
        let mut baseline_others = baseline.plan.clone();
        baseline_others.runs.retain(|run| run.run_id != RUN);
        assert_eq!(others, baseline_others, "{directory}");

        selection.set_feature("mod:eeex", false);
        let no_eeex = evaluate(&manifest, &selection).unwrap();
        let components = no_eeex.plan.components_for(RUN).unwrap();
        assert!(!components.contains(&420) && !components.contains(&421));
    }
}

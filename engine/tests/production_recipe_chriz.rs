use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveRootRule, Decision, Phase, Readiness, SourceKind,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

/// The October 13 intake adds the mandatory v0.6.13 rows 177, 198 and 285 in TP2 order.
/// The October 23 intake (v0.6.15) adds the Imoen in Shadows of Amn choice after 910;
/// Recommended installs its default 292. v0.6.16 (October 29 intake) changes 290 and the
/// timer-name reads of 265 and six more components; components and order stay the same.
const SOD_REMIX_COMPONENTS: &[u32] = &[
    100, 110, 115, 120, 130, 135, 140, 150, 145, 160, 170, 180, 175, 177, 185, 190, 195, 210, 197,
    198, 187, 200, 215, 220, 225, 245, 230, 240, 250, 255, 256, 260, 265, 266, 270, 280, 285, 290,
    900, 910, 292,
];
const SOD_REMIX_RUN_COMPONENTS: &[u32] = &[
    100, 110, 115, 120, 130, 135, 140, 150, 145, 160, 170, 180, 175, 177, 185, 190, 195, 210, 197,
    198, 187, 200, 215, 220, 225, 245, 230, 240, 250, 255, 256, 257, 260, 265, 266, 270, 280, 285,
    290, 900, 910, 292, 293, 294,
];
/// Christopher, 2026-10-16: three choices, the vanilla BG2 thief-to-mage dual (292) by default.
const IMOEN_CHOICE: &str = "sod-remix-imoen-soa";
const IMOEN_DUAL: &str = "feature:chriz-sod-remix:component-292";
const IMOEN_AS_TAKEN: &str = "feature:chriz-sod-remix:component-293";
const IMOEN_SOD_LEVEL_DUAL: &str = "feature:chriz-sod-remix:component-294";
// v0.8.3 (October 14) declares potion uniqueness 330/331 and Heroism 332 last;
// 331 (Genius still stacks) and 332 are defaults, 330 is the optional strict variant.
// v0.8.5 (October 26 intake) declares Chris Revisions 340 after them, a default.
// v0.8.6 (October 28 intake) declares the Time Stop and Haste effect clocks 360 after 340,
// a default (Christopher, 2026-10-27); v0.8.7 (October 29 intake) renames 360 to True Time
// Stop – fair Haste & Slow and keeps every component and the order. v0.8.8 (October 29)
// declares Druid weapons 410 and Alaghor of Clangeddin weapons 411 between 302 and 420, both
// defaults (Christopher, 2026-10-29).
const BG_REBALANCE_COMPONENTS: &[u32] = &[
    100, 101, 111, 120, 121, 200, 210, 211, 400, 401, 404, 405, 407, 408, 302, 410, 411, 420, 421,
    310, 130, 220, 320, 331, 332, 340, 360,
];
const BG_REBALANCE_RUN_COMPONENTS: &[u32] = &[
    100, 101, 110, 111, 120, 121, 200, 201, 202, 210, 211, 301, 400, 401, 404, 405, 407, 408, 302,
    410, 411, 420, 421, 310, 130, 220, 320, 330, 331, 332, 340, 360,
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
        "chriz-sod-remix-0.6.16"
    );
    let artifact = &manifest.artifacts["chriz-sod-remix-0.6.16"];
    assert_eq!(artifact.version, "0.6.16");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(
        artifact.source.url,
        "https://github.com/Chrizhermann/chriz-sod-rebalance/releases/download/v0.6.16/chriz-sod-remix-v0.6.16.zip"
    );
    assert_eq!(artifact.source.reference, "v0.6.16");
    assert_eq!(
        artifact.source.expected_filename.as_deref(),
        Some("chriz-sod-remix-v0.6.16.zip")
    );
    assert_eq!(artifact.source.expected_length, Some(3_055_655));
    assert_eq!(
        artifact.source.sha256,
        "7da50d206e4f0d50dd94d5e60fd985290a245d2dc3bed32068e1eaad0219c2e6"
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
            "feature:chriz-sod-remix:component-257",
            IMOEN_DUAL,
            IMOEN_AS_TAKEN,
            IMOEN_SOD_LEVEL_DUAL,
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
fn offers_imoen_in_shadows_of_amn_as_one_choice_with_the_vanilla_dual_by_default() {
    let manifest = recipe();
    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    for (id, decision, selected) in [
        (IMOEN_DUAL, Decision::Default, true),
        (IMOEN_AS_TAKEN, Decision::Optional, false),
        (IMOEN_SOD_LEVEL_DUAL, Decision::Optional, false),
    ] {
        let control = evaluation.view.control(id).expect("Imoen choice option");
        assert_eq!(control.decision, decision, "{id}");
        assert_eq!(control.readiness, Readiness::Ready, "{id}");
        assert_eq!(control.selected, selected, "{id}");
        assert_eq!(control.choice_group.as_deref(), Some(IMOEN_CHOICE), "{id}");
    }

    let sod_components = |changes: &[(&str, bool)]| {
        let mut selection = Selection::defaults("windows");
        for (id, selected) in changes {
            selection.set_feature(id, *selected);
        }
        let evaluation = evaluate(&manifest, &selection).unwrap();
        evaluation
            .plan
            .components_for("chriz-sod-remix-bg2")
            .unwrap()
            .iter()
            .copied()
            .filter(|component| [292, 293, 294].contains(component))
            .collect::<Vec<_>>()
    };
    assert_eq!(sod_components(&[]), [292]);
    assert_eq!(
        sod_components(&[(IMOEN_DUAL, false), (IMOEN_AS_TAKEN, true)]),
        [293]
    );
    assert_eq!(
        sod_components(&[(IMOEN_DUAL, false), (IMOEN_SOD_LEVEL_DUAL, true)]),
        [294]
    );
    // Only the SoD-level dual-class needs EEex; the default does not.
    let dual = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == IMOEN_SOD_LEVEL_DUAL)
        .unwrap();
    assert_eq!(
        dual.requires,
        [
            "feature:chriz-sod-remix:mandatory-components",
            "feature:eeex:mandatory-components"
        ]
    );
    assert_eq!(sod_components(&[("mod:eeex", false)]), [292]);
}

#[test]
fn freezes_the_reviewed_bg_rebalance_release() {
    let manifest = recipe();
    let artifact_id = &manifest.mods["chriz-bg-rebalance"].artifact_id;
    assert_eq!(artifact_id, "chriz-bg-rebalance-0.8.8");
    let artifact = &manifest.artifacts[artifact_id];
    assert_eq!(artifact.version, "0.8.8");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(
        artifact.source.url,
        "https://github.com/Chrizhermann/chriz-bg-rebalance/releases/download/v0.8.8/chriz-bg-rebalance-v0.8.8-windows.zip"
    );
    assert_eq!(artifact.source.reference, "v0.8.8");
    assert_eq!(
        artifact.source.expected_filename.as_deref(),
        Some("chriz-bg-rebalance-v0.8.8-windows.zip")
    );
    assert_eq!(artifact.source.expected_length, Some(1_839_187));
    assert_eq!(
        artifact.source.sha256,
        "dec0544b3cc0f2197b5791e89cf0eb73d2cdb563d3ae35c254d0da719d89a2e3"
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
            // v0.8.3: strict potion uniqueness 330 is the optional alternative to 331.
            (
                "feature:chriz-bg-rebalance:component-330",
                Decision::Optional,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-331",
                Decision::Default,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-332",
                Decision::Default,
                1
            ),
            // v0.8.5: Chris Revisions 340 (Christopher, 2026-10-25).
            (
                "feature:chriz-bg-rebalance:component-340",
                Decision::Default,
                1
            ),
            // v0.8.6: Time Stop and Haste effect clocks 360 (Christopher, 2026-10-27), named
            // True Time Stop – fair Haste & Slow since v0.8.7.
            (
                "feature:chriz-bg-rebalance:component-360",
                Decision::Default,
                1
            ),
            // v0.8.8: Druid weapons 410 and Alaghor of Clangeddin weapons 411 (Christopher,
            // 2026-10-29).
            (
                "feature:chriz-bg-rebalance:component-410",
                Decision::Default,
                1
            ),
            (
                "feature:chriz-bg-rebalance:component-411",
                Decision::Default,
                1
            ),
            (
                "feature:chriz-bg-rebalance:bard-spell-ceilings",
                Decision::Default,
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
        Some(
            &[
                100, 101, 111, 120, 121, 200, 210, 211, 302, 410, 411, 420, 421, 310, 130, 220,
                320, 331, 332, 340, 360
            ][..]
        )
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
                100, 101, 120, 121, 200, 210, 211, 400, 401, 404, 405, 407, 408, 302, 410, 411,
                420, 421, 310, 130, 220, 320, 331, 332, 340, 360
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
fn bard_spell_ceilings_are_a_default_eeex_choice_in_both_profiles() {
    const BASE: &str = "feature:chriz-bg-rebalance:bard-spell-ceilings";
    const RUN: &str = "chriz-bg-rebalance-bg2";
    const BARDIC: &str = "bardicwonders-progression-bg2";
    for directory in ["manifest", "recipes/curated-full-current"] {
        let manifest = Manifest::load(&recipe_root().join("..").join(directory)).unwrap();
        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert_eq!(
            manifest.presets["chris-recommended"].selections[BASE], "on",
            "{directory}"
        );
        let control = baseline.view.control(BASE).unwrap();
        assert_eq!(control.decision, Decision::Default, "{directory}");
        assert!(control.selected && control.interactive, "{directory}");
        assert_eq!(control.requires, ["feature:eeex:mandatory-components"]);
        assert_eq!(
            baseline.plan.components_for(RUN),
            Some(BG_REBALANCE_COMPONENTS),
            "{directory}"
        );

        // Opting out removes exactly 420/421 (and the dependent Bardic adapter run).
        let mut selection = baseline.normalized_selection.to_selection();
        selection.set_feature(BASE, false);
        let opted_out = evaluate(&manifest, &selection).unwrap();
        let expected = BG_REBALANCE_COMPONENTS
            .iter()
            .copied()
            .filter(|c| ![420, 421].contains(c))
            .collect::<Vec<_>>();
        assert_eq!(
            opted_out.plan.components_for(RUN),
            Some(&expected[..]),
            "{directory}"
        );
        assert_eq!(opted_out.plan.components_for(BARDIC), None, "{directory}");
        let mut others = opted_out.plan.clone();
        others.runs.retain(|run| run.run_id != RUN);
        let mut baseline_others = baseline.plan.clone();
        baseline_others
            .runs
            .retain(|run| run.run_id != RUN && run.run_id != BARDIC);
        assert_eq!(others, baseline_others, "{directory}");

        let mut no_eeex = baseline.normalized_selection.to_selection();
        no_eeex.set_feature("mod:eeex", false);
        let no_eeex = evaluate(&manifest, &no_eeex).unwrap();
        let components = no_eeex.plan.components_for(RUN).unwrap();
        assert!(!components.contains(&420) && !components.contains(&421));
        assert_eq!(no_eeex.plan.components_for(BARDIC), None, "{directory}");
    }
}

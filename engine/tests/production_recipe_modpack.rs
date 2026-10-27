use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveRootRule, Decision, Phase, Readiness, SourceKind,
};
use bg_engine::recipe_view::{evaluate, evaluate_preset};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

const MODPACK_COMPONENTS: &[u32] = &[
    110, 130, 135, 140, 170, 188, 189, 190, 192, 193, 194, 195, 196, 197, 198, 199, 220, 221, 222,
    223, 224, 225, 230, 231, 232, 233, 234, 400, 410, 430, 440, 450, 460, 610, 611, 620, 630, 640,
    235, 236, 237, 238, 239, 660, 650, 651, 652,
];

/// The September 16 intake split the single post-EET modpack run into the
/// companion preparation before continuity, the continuity run before EET_end,
/// the unchanged post-EET slot and the late companions after Safana.
/// The October 14 intake (Modpack alpha.12) adds companion stats 230-234 after
/// the class/kit conversions, in TP2 order.
const PRE_CONTINUITY_COMPONENTS: &[u32] = &[
    110, 140, 170, 188, 190, 192, 193, 194, 195, 196, 197, 198, 220, 221, 222, 223, 224, 225, 230,
    231, 232, 233, 234,
];

const CONTINUITY_COMPONENTS: &[u32] = &[199];

/// The October 13 intake (Modpack alpha.11 candidate) moves scroll policy 430 into
/// its own final run and adds Degrodel 660 after 630 in TP2 order. The October 26
/// alpha.14 pin adds alpha.13's Dorn ambush fix 135 after 130 and its party AI
/// Defensive Stance fix 460 after 450, in TP2 order.
const POST_EET_END_COMPONENTS: &[u32] = &[130, 135, 400, 410, 440, 450, 460, 610, 611, 630, 660];

const RECOMMENDED_PRE_CONTINUITY_COMPONENTS: &[u32] = &[
    110, 140, 170, 188, 190, 192, 193, 194, 195, 196, 197, 198, 220, 221, 230, 231, 232, 233, 234,
];

const RECOMMENDED_COMPONENTS: &[u32] = &[130, 135, 410, 440, 450, 460, 611, 660];

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[test]
fn freezes_the_public_modpack_alpha_without_publishing_its_bundled_weidu() {
    let manifest = recipe();
    let artifact_id = &manifest.mods["chriz-bg-modpack"].artifact_id;
    assert_eq!(artifact_id, "chriz-bg-modpack-0.2.0-alpha.14");
    let artifact = &manifest.artifacts[artifact_id];
    assert!(!manifest
        .artifacts
        .contains_key("chriz-bg-modpack-0.2.0-alpha.5"));
    assert_eq!(artifact.version, "0.2.0-alpha.14");
    assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
    assert_eq!(artifact.source.kind, SourceKind::GithubRelease);
    assert_eq!(artifact.source.reference, "v0.2.0-alpha.14");
    assert_eq!(
        artifact.source.url,
        "https://github.com/Chrizhermann/chriz-bg-modpack/releases/download/v0.2.0-alpha.14/chriz-bg-modpack-v0.2.0-alpha.14.zip"
    );
    assert_eq!(
        artifact.source.expected_filename.as_deref(),
        Some("chriz-bg-modpack-v0.2.0-alpha.14.zip")
    );
    assert_eq!(artifact.source.expected_length, Some(10_550_312));
    assert_eq!(
        artifact.source.sha256,
        "45c767b21342d1c7c23ded6cd52aef29b4054daf9cc949613d8f6620207a356c"
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
    let safana = position("safana-bg2");
    let late_companions = position("chriz-bg-modpack-late-companions-bg2");
    let campaign_xp = position("chriz-bg-modpack-campaign-xp-bg2");
    let spellbooks = position("spell-rev-npc-spellbooks-bg2");
    let lightning = position("spell-rev-lightning-bg2");
    let baeloth = position("chriz-bg-modpack-baeloth-bg2");
    let edwin = position("chriz-bg-modpack-edwin-bg2");
    let buffbot = position("buffbot-bg2");
    let scrolls = position("chriz-bg-modpack-scrolls-bg2");
    assert_eq!(remote + 1, modpack);
    assert_eq!(modpack + 1, safana);
    assert_eq!(safana + 1, late_companions);
    assert_eq!(late_companions + 1, campaign_xp);
    assert_eq!(campaign_xp + 1, spellbooks);
    assert_eq!(spellbooks + 1, lightning);
    assert_eq!(lightning + 1, baeloth);
    // October 22: the 18 sound-only player voicepacks run between Baeloth and BuffBot.
    assert_eq!(baeloth + 1, position("voicepack-neera-bg2"));
    assert_eq!(baeloth + 18, position("voicepack-garithos-bg2"));
    assert_eq!(position("voicepack-garithos-bg2") + 1, buffbot);
    // 430 hashes the final scrolls and spells at install time, so it runs last.
    assert_eq!(buffbot + 1, scrolls);
    assert_eq!(position("artisanskitpack-npc-bg2") + 1, edwin);
    assert!(edwin < spellbooks);
    assert_eq!(scrolls, runs.len() - 1);

    let run = &runs[modpack];
    assert_eq!(run.phase, Phase::PostEetEnd);
    assert_eq!(run.components, POST_EET_END_COMPONENTS);

    let pre_continuity = &runs[position("chriz-bg-modpack-pre-continuity-bg2")];
    assert_eq!(pre_continuity.phase, Phase::Main);
    assert_eq!(pre_continuity.components, PRE_CONTINUITY_COMPONENTS);
    let continuity = &runs[position("chriz-bg-modpack-continuity-bg2")];
    assert_eq!(continuity.phase, Phase::Main);
    assert_eq!(continuity.components, CONTINUITY_COMPONENTS);
    let late = &runs[late_companions];
    assert_eq!(late.phase, Phase::PostEetEnd);
    // October 20: Modpack alpha.13's EET join XP 239 follows 238 in TP2 order and precedes 650.
    assert_eq!(late.components, [189, 237, 238, 239, 620]);
    assert_eq!(runs[campaign_xp].phase, Phase::PostEetEnd);
    // October 26: alpha.13's chapter caps 652 follow 651, the alternative they exclude.
    assert_eq!(runs[campaign_xp].components, [650, 651, 652]);
    assert_eq!(runs[scrolls].phase, Phase::PostEetEnd);
    assert_eq!(runs[scrolls].components, [430]);
    assert_eq!(runs[edwin].phase, Phase::Main);
    assert_eq!(runs[edwin].components, [236]);
    assert_eq!(runs[baeloth].phase, Phase::PostEetEnd);
    assert_eq!(runs[baeloth].components, [235]);

    let installer = &manifest.mods["chriz-bg-modpack"];
    assert_eq!(installer.artifact_id, "chriz-bg-modpack-0.2.0-alpha.14");
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
        evaluation
            .plan
            .components_for("chriz-bg-modpack-pre-continuity-bg2"),
        Some(RECOMMENDED_PRE_CONTINUITY_COMPONENTS)
    );
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

    // Evandra's recruitment XP 238 needs her core, which only the generated full
    // recipe supplies; this narrower fallback keeps it optional and blocked.
    for component in [400, 238] {
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

    // October 13: reader-level scrolls 430 (EEex, no longer TA 2170), Fade's
    // recruitment XP 237 and Degrodel 660 are deselectable defaults.
    // October 14: companion stats 230-234 (Modpack alpha.12) are defaults too.
    // October 20: so is EET join XP 239 (Modpack alpha.13).
    // October 26: and alpha.13's Dorn ambush fix 135 and party AI fix 460 (alpha.14 pin).
    for component in [
        110, 135, 190, 192, 193, 194, 195, 196, 197, 198, 220, 221, 230, 231, 232, 233, 234, 236,
        237, 239, 430, 460, 611, 660,
    ] {
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

    for component in [222, 223, 235, 610] {
        let id = format!("feature:chriz-bg-modpack:component-{component}");
        let control = evaluation
            .view
            .control(&id)
            .unwrap_or_else(|| panic!("missing control {id}"));
        assert_eq!(control.decision, Decision::Optional, "{id}");
        assert_eq!(control.readiness, Readiness::Ready, "{id}");
        assert!(!control.selected, "{id}");
    }

    // Campaign XP caps 650/651 and chapter caps 652 ship experimental, optional and unchecked.
    for component in [650, 651, 652] {
        let id = format!("feature:chriz-bg-modpack:component-{component}");
        let control = evaluation
            .view
            .control(&id)
            .unwrap_or_else(|| panic!("missing control {id}"));
        assert_eq!(control.decision, Decision::Optional, "{id}");
        assert_eq!(control.readiness, Readiness::Experimental, "{id}");
        assert!(!control.selected, "{id}");
    }
    assert_eq!(
        evaluation
            .plan
            .components_for("chriz-bg-modpack-scrolls-bg2"),
        Some(&[430][..])
    );
    assert_eq!(
        evaluation
            .plan
            .components_for("chriz-bg-modpack-campaign-xp-bg2"),
        None
    );

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

    // The base manifest includes the approved SoD skip (910) added after alpha.10,
    // Red Wizard 5102 (c907b38), the Acton Balthis 25 opt-out (9c66cee) and the
    // September 16 intake (cb07eca), and September 26 Emotion exclusivity (301).
    // October 7 adds seven Rebalance defaults (130/200/210/211/220/310/320)
    // and Edwin 236. Optional Baeloth 235, Hardiness 201/202 remain unselected.
    // Next-release Emotion 301 opt-in policy removes one default from each plan.
    // October 13 default bard ceilings add Rebalance 420/421 (and Bardic 3010
    // in the curated plan only). EET's desktop shortcut (100) leaves both plans.
    // The October 13 intake adds Modpack 237/430/660 and SoD 177/198/285 here
    // (Evandra's 238 only in the curated plan).
    // The October 14 intake adds Modpack 230-234, Rebalance 331/332 and SCS 3021
    // (and Tweaks 3220/3500/3510 in the curated plan only).
    // October 17 makes Unfinished Business optional and unselected, which removes
    // its 18 former defaults from both plans.
    // The October 20 intake adds Modpack EET join XP 239 to both plans.
    // The October 22 intake adds the 18 default player voicepacks to both plans.
    // The October 23 intake (SoD Remix v0.6.15) adds default Imoen choice 292 to both.
    // The October 23 quest-mod intake makes BG1 Unfinished Business optional and
    // unselected, which removes its 16 former defaults from both plans.
    // The October 26 intake adds default Rebalance Chris Revisions 340 to both plans;
    // EEex Attack Info 9 is offered unchecked. Its Modpack alpha.14 pin adds default
    // 135 and 460 to both plans; chapter caps 652 are offered unchecked.
    // The October 28 intake (Rebalance v0.8.6) adds default Time Stop and Haste effect
    // clocks 360 to both plans.
    // This 372-component base plan is distinct from the 469-component curated plan.
    assert_eq!(
        evaluation
            .plan
            .runs
            .iter()
            .map(|run| run.components.len())
            .sum::<usize>(),
        372
    );
}

#[test]
fn chapter_caps_652_and_the_soa_cap_651_are_alternatives_on_top_of_650() {
    // Alpha.13's TP2 refuses 651 with 652 installed and 652 with 651 installed; both
    // need 650. Both recipes encode that as a conflict in each direction.
    let run = "chriz-bg-modpack-campaign-xp-bg2";
    for directory in ["manifest", "recipes/curated-full-current"] {
        let manifest = Manifest::load(&recipe_root().join("..").join(directory)).unwrap();
        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        assert_eq!(baseline.plan.components_for(run), None, "{directory}");
        for component in [651, 652] {
            let id = format!("feature:chriz-bg-modpack:component-{component}");
            let other = format!(
                "feature:chriz-bg-modpack:component-{}",
                651 + 652 - component
            );
            let feature = manifest
                .collection
                .features
                .iter()
                .find(|feature| feature.id == id)
                .unwrap_or_else(|| panic!("{directory}: missing {id}"));
            assert_eq!(feature.requires, ["feature:chriz-bg-modpack:component-650"]);
            assert_eq!(
                feature
                    .conflicts
                    .iter()
                    .map(|conflict| conflict.feature_id.as_str())
                    .collect::<Vec<_>>(),
                [other.as_str()],
                "{directory}: {id}"
            );
        }
        for (soa, chapters, expected) in [
            (true, false, vec![650, 651]),
            (false, true, vec![650, 652]),
            (false, false, vec![650]),
            (true, true, vec![650]),
        ] {
            let mut selection = baseline.normalized_selection.to_selection();
            selection.set_feature("feature:chriz-bg-modpack:component-650", true);
            selection.set_feature("feature:chriz-bg-modpack:component-651", soa);
            selection.set_feature("feature:chriz-bg-modpack:component-652", chapters);
            let result = evaluate(&manifest, &selection).unwrap();
            assert_eq!(
                result.plan.components_for(run).map(<[u32]>::to_vec),
                Some(expected),
                "{directory}: 651={soa} 652={chapters}"
            );
        }
        // Without 650 neither cap policy installs.
        let mut selection = baseline.normalized_selection.to_selection();
        selection.set_feature("feature:chriz-bg-modpack:component-652", true);
        let result = evaluate(&manifest, &selection).unwrap();
        assert_eq!(result.plan.components_for(run), None, "{directory}");
    }
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
        .components_for("chriz-bg-modpack-pre-continuity-bg2")
        .expect("modpack companion run");
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
        .components_for("chriz-bg-modpack-pre-continuity-bg2")
        .expect("modpack companion run")
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
        .components_for("chriz-bg-modpack-pre-continuity-bg2")
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

#[test]
fn story_xp_is_default_and_appearance_is_an_independent_optional_choice() {
    for directory in ["manifest", "recipes/curated-full-current"] {
        let manifest = Manifest::load(&recipe_root().join("..").join(directory)).unwrap();
        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        let run = "chriz-bg-modpack-bg2";
        let components = baseline.plan.components_for(run).unwrap();
        assert!(components.contains(&611));
        assert!(!components.contains(&610));
        assert!(!components.contains(&630));
        for (old, story, expected) in [
            (true, false, vec![610]),
            (false, true, vec![611]),
            (false, false, vec![]),
            (true, true, vec![]),
        ] {
            let mut selection = baseline.normalized_selection.to_selection();
            selection.set_feature("feature:chriz-bg-modpack:component-610", old);
            selection.set_feature("feature:chriz-bg-modpack:component-611", story);
            let result = evaluate(&manifest, &selection).unwrap();
            let actual: Vec<_> = result
                .plan
                .components_for(run)
                .unwrap()
                .iter()
                .copied()
                .filter(|c| [610, 611].contains(c))
                .collect();
            assert_eq!(actual, expected);
        }
        let mut selection = baseline.normalized_selection.to_selection();
        selection.set_feature("feature:chriz-bg-modpack:component-630", true);
        assert!(!evaluate(&manifest, &selection)
            .unwrap()
            .plan
            .components_for(run)
            .unwrap()
            .contains(&630));
        selection.set_feature("feature:randomiser:component-570", false);
        let appearance = evaluate(&manifest, &selection).unwrap();
        assert!(appearance.plan.components_for(run).unwrap().contains(&630));
        assert!(appearance
            .plan
            .components_for("randomiser-bg2")
            .unwrap()
            .contains(&1100));
        assert!(
            appearance
                .view
                .control("feature:chriz-bg-modpack:component-611")
                .unwrap()
                .selected
        );
        selection.set_feature("mod:eeex", false);
        let without_eeex = evaluate(&manifest, &selection).unwrap();
        assert!(!without_eeex
            .plan
            .components_for(run)
            .unwrap()
            .iter()
            .any(|c| [610, 611, 630].contains(c)));
        assert!(evaluate_preset(&manifest, "chris-recommended", "linux").is_err());
        let positions = |id: &str| {
            manifest
                .collection
                .runs
                .iter()
                .position(|r| r.run_id == id)
                .unwrap()
        };
        for earlier in [
            "stratagems-bg2",
            "randomiser-bg2",
            "eet-end-bg2",
            "chriz-bg-rebalance-bg2",
            "chriz-sod-remix-bg2",
        ] {
            assert!(positions(earlier) < positions(run));
        }
    }
}

#[test]
fn edwin_follows_the_selected_conversion_and_baeloth_remains_an_optional_late_choice() {
    for directory in ["manifest", "recipes/curated-full-current"] {
        let manifest = Manifest::load(&recipe_root().join("..").join(directory)).unwrap();
        let baseline = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
        let edwin_run = "chriz-bg-modpack-edwin-bg2";
        let baeloth_run = "chriz-bg-modpack-baeloth-bg2";
        assert_eq!(baseline.plan.components_for(edwin_run), Some(&[236][..]));
        assert_eq!(baseline.plan.components_for(baeloth_run), None);

        let mut challenge = baseline.normalized_selection.to_selection();
        challenge.set_feature("feature:chriz-bg-modpack:component-235", true);
        let challenge = evaluate(&manifest, &challenge).unwrap();
        assert_eq!(challenge.plan.components_for(baeloth_run), Some(&[235][..]));

        for disabled in [
            "feature:artisanskitpack-npc:component-5102",
            "mod:artisanskitpack-npc",
            "mod:chriz-bg-modpack",
        ] {
            let mut selection = baseline.normalized_selection.to_selection();
            selection.set_feature(disabled, false);
            selection.set_feature("feature:chriz-bg-modpack:component-236", true);
            let result = evaluate(&manifest, &selection).unwrap();
            assert_eq!(
                result.plan.components_for(edwin_run),
                None,
                "{directory}: {disabled}"
            );
        }

        let position = |run_id: &str| {
            manifest
                .collection
                .runs
                .iter()
                .position(|run| run.run_id == run_id)
                .unwrap()
        };
        assert_eq!(position("artisanskitpack-npc-bg2") + 1, position(edwin_run));
        assert!(position(edwin_run) < position("spell-rev-npc-spellbooks-bg2"));
        assert!(position("spell-rev-npc-spellbooks-bg2") < position(baeloth_run));
        assert!(position(baeloth_run) < position("buffbot-bg2"));
    }
}

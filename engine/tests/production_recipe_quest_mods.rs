//! Static checks for the October 23 quest-mod intake in both recipe profiles: NTotSC at
//! upstream master f0ef953, Ascalon's Questpack v7.0, Black Hearts v2.05, Thalantyr Item
//! Upgrade v4.5.1, the Icewind Dale Voice Pack 1.2.0 and PPE at DaftHunk commit 417226ea,
//! with BG1 Unfinished Business made optional. Selection, conflict and order checks only;
//! not a game installation or gameplay acceptance.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveKind, ArchiveRootRule, Decision, InvocationMode, Phase, Readiness,
    SourceKind,
};
use bg_engine::recipe_view::{evaluate, SelectionEvaluation};
use bg_engine::resolve::Selection;
use bg_engine::Manifest;

struct Pin {
    mod_id: &'static str,
    run_id: &'static str,
    artifact_id: &'static str,
    kind: SourceKind,
    url: &'static str,
    reference: &'static str,
    length: u64,
    sha256: &'static str,
    archive: ArchiveKind,
    root_rule: ArchiveRootRule,
    publish_root: &'static str,
    tp2: &'static str,
    language: u32,
    /// Offered components in TP2 declaration order.
    components: &'static [u32],
}

const NTOTSC: &str = "ntotsc-bg2";
const AC_QUEST: &str = "ac-quest-bg2";
const BLACKHEARTS: &str = "blackhearts-bg2";
const THALAN: &str = "thalan-bg2";
const IWD_VOICES: &str = "zg-iwd-voices-bg2";
const PPE: &str = "ppe-bg2";
const CONTINUITY: &str = "chriz-bg-modpack-continuity-bg2";
const BLACKHEARTS_FEATURE: &str = "feature:blackhearts:component-1";
const SOD_REMIX_CORE: &str = "feature:chriz-sod-remix:mandatory-components";

const PINS: &[Pin] = &[
    Pin {
        mod_id: "ntotsc",
        run_id: NTOTSC,
        artifact_id: "ntotsc-f0ef95323630bc38df7dd4c969a2b48a71ebec82",
        kind: SourceKind::GithubCommitZip,
        url: "https://codeload.github.com/Spellhold-Studios/NTotSC/zip/f0ef95323630bc38df7dd4c969a2b48a71ebec82",
        reference: "f0ef95323630bc38df7dd4c969a2b48a71ebec82",
        length: 135_854_624,
        sha256: "b507d4b833676c5d8924f549bb8cfef408c18cd21ea61612dd93290773b48e9d",
        archive: ArchiveKind::Zip,
        root_rule: ArchiveRootRule::SingleWrapper,
        publish_root: "ntotsc",
        tp2: "ntotsc/ntotsc.tp2",
        language: 0,
        components: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
    },
    Pin {
        mod_id: "ac-quest",
        run_id: AC_QUEST,
        artifact_id: "ac-quest-7.0",
        kind: SourceKind::GithubRelease,
        url: "https://github.com/Gitjas/Ascalons_Questpack/releases/download/v7.0/ascalons-questpack-7.0.iemod",
        reference: "v7.0",
        length: 20_936_689,
        sha256: "87c2feb30c8968c7097913cb540eb1dd5025f311186c01b1100151af02447564",
        archive: ArchiveKind::Iemod,
        root_rule: ArchiveRootRule::Direct,
        publish_root: "AC_QUEST",
        tp2: "AC_QUEST/ac_quest.TP2",
        // English is the second language in the TP2.
        language: 1,
        components: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    },
    Pin {
        mod_id: "blackhearts",
        run_id: BLACKHEARTS,
        artifact_id: "blackhearts-2.05",
        kind: SourceKind::GithubRelease,
        url: "https://github.com/A-Wizard-Did-It/Black-Hearts/releases/download/v2.05/BlackHearts-v2.05.iemod",
        reference: "v2.05",
        length: 38_645_056,
        sha256: "a1a6257a2c21e19c87778bd34dba5c50224fdc3767b9fdbb2e41561f50340301",
        archive: ArchiveKind::Iemod,
        root_rule: ArchiveRootRule::Direct,
        publish_root: "Blackhearts",
        tp2: "Blackhearts/Blackhearts.tp2",
        language: 0,
        // 20005 is the BG2:EE-without-EET variant and is not offered.
        components: &[1],
    },
    Pin {
        mod_id: "thalan",
        run_id: THALAN,
        artifact_id: "thalan-4.5.1",
        kind: SourceKind::GithubRelease,
        url: "https://github.com/CrevsDaak/thalan/releases/download/v4.5.1/thalantyr-item-upgrade-v4.5.1.iemod",
        reference: "v4.5.1",
        length: 833_151,
        sha256: "ac5a2dff1feb41ee0e5d7a2e029c8a4985957aaef9c689558dbc6ed09cf019d3",
        archive: ArchiveKind::Iemod,
        root_rule: ArchiveRootRule::Direct,
        publish_root: "thalan",
        tp2: "thalan/thalan.tp2",
        language: 0,
        components: &[0],
    },
    Pin {
        mod_id: "zg-iwd-voices",
        run_id: IWD_VOICES,
        artifact_id: "zg-iwd-voices-1.2.0",
        kind: SourceKind::GithubRelease,
        url: "https://github.com/GraionDilach/Icewind-Dale-Voice-Pack-For-EE-2.6/releases/download/version-1.2.0/zg-iwd-voices-1.2.0.iemod",
        reference: "version-1.2.0",
        length: 28_942_218,
        sha256: "7677e0d8fdef8422ec6424e29054e21bad7792ae07dc482331022b30bf9edefa",
        archive: ArchiveKind::Iemod,
        root_rule: ArchiveRootRule::Direct,
        publish_root: "zg_iwd_voices",
        tp2: "zg_iwd_voices/zg_iwd_voices.tp2",
        language: 0,
        components: &[0],
    },
    Pin {
        mod_id: "ppe",
        run_id: PPE,
        artifact_id: "ppe-417226ea6642b8c682b2d049d0ec6e86f38b7f47",
        kind: SourceKind::GithubCommitZip,
        url: "https://codeload.github.com/DaftHunk/Portraits-Portraits-Everywhere/zip/417226ea6642b8c682b2d049d0ec6e86f38b7f47",
        reference: "417226ea6642b8c682b2d049d0ec6e86f38b7f47",
        length: 387_318_340,
        sha256: "fc250193d9cc69823032529fc4dac1d66f3765e5d9a9877f28c269dc5654b539",
        archive: ArchiveKind::Zip,
        root_rule: ArchiveRootRule::SingleWrapper,
        publish_root: "PPE",
        tp2: "PPE/setup-PPE.tp2",
        language: 0,
        // 310 and 320 (Viconia and Jaheira, BG1 style) are expected to fail on EET.
        components: &[
            0, 100, 150, 10, 20, 25, 300, 301, 302, 311, 312, 322, 330, 332, 340, 342, 350, 30, 40,
        ],
    },
];

/// The decided defaults inside the otherwise unchecked quest mods.
const DEFAULTS: &[(&str, u32)] = &[
    (NTOTSC, 0),
    (AC_QUEST, 2),
    (AC_QUEST, 5),
    (AC_QUEST, 10),
    (PPE, 0),
];

/// Every either-or choice: (choice group, run, options in authored order, default option).
const CHOICES: &[(&str, &str, &[u32], Option<u32>)] = &[
    (
        "choice:ntotsc:reduce-number-of-shaman-cloaks",
        NTOTSC,
        &[13, 14],
        None,
    ),
    ("choice:ntotsc:spell-scrolls", NTOTSC, &[15, 16], None),
    ("choice:ac-quest:jumper", AC_QUEST, &[2, 3], Some(2)),
    (
        "choice:ac-quest:of-wolves-and-men",
        AC_QUEST,
        &[5, 6],
        Some(5),
    ),
    (
        "choice:ac-quest:the-serpents-of-abbathor",
        AC_QUEST,
        &[10, 11],
        Some(10),
    ),
    ("choice:ppe:category-portraits", PPE, &[100, 150], None),
    (
        "choice:ppe:change-imoen-portraits",
        PPE,
        &[300, 301, 302],
        None,
    ),
    (
        "choice:ppe:change-viconia-portraits",
        PPE,
        &[311, 312],
        None,
    ),
    ("choice:ppe:change-minsc-portraits", PPE, &[330, 332], None),
    ("choice:ppe:change-edwin-portraits", PPE, &[340, 342], None),
];

fn profiles() -> [(&'static str, Manifest); 2] {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    ["manifest", "recipes/curated-full-current"].map(|profile| {
        let manifest = Manifest::load(&root.join(profile))
            .unwrap_or_else(|error| panic!("{profile} recipe should load: {error}"));
        (profile, manifest)
    })
}

fn feature_id(run_id: &str, component: u32) -> String {
    let mod_id = run_id.strip_suffix("-bg2").unwrap();
    format!("feature:{mod_id}:component-{component}")
}

/// The recommended preset with explicit feature overrides.
fn with(manifest: &Manifest, overrides: &[(String, bool)]) -> SelectionEvaluation {
    let mut selection = Selection {
        platform: "windows".to_owned(),
        choices: manifest.presets["chris-recommended"].selections.clone(),
    };
    for (feature, on) in overrides {
        selection.set_feature(feature, *on);
    }
    evaluate(manifest, &selection).unwrap()
}

fn on(ids: &[&str]) -> Vec<(String, bool)> {
    ids.iter().map(|id| ((*id).to_owned(), true)).collect()
}

fn planned(evaluation: &SelectionEvaluation, run_id: &str) -> Vec<u32> {
    evaluation
        .plan
        .components_for(run_id)
        .map(<[u32]>::to_vec)
        .unwrap_or_default()
}

fn quest_runs() -> BTreeSet<&'static str> {
    PINS.iter().map(|pin| pin.run_id).collect()
}

#[test]
fn pins_the_reviewed_quest_mod_archives_and_offered_component_menus() {
    for (profile, manifest) in profiles() {
        for pin in PINS {
            let artifact = &manifest.artifacts[pin.artifact_id];
            assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly);
            assert_eq!(artifact.source.kind, pin.kind, "{profile} {}", pin.mod_id);
            assert_eq!(artifact.source.url, pin.url);
            assert_eq!(artifact.source.reference, pin.reference);
            assert_eq!(artifact.source.expected_length, Some(pin.length));
            assert_eq!(artifact.source.sha256, pin.sha256);
            assert_eq!(artifact.archive.kind, pin.archive);
            assert_eq!(artifact.archive.root_rule, pin.root_rule);
            // Only the mod folder is published: never a bundled setup executable.
            assert_eq!(artifact.archive.publish_roots, [pin.publish_root]);
            assert_eq!(artifact.archive.tp2_paths, [pin.tp2]);

            let installer = &manifest.mods[pin.mod_id];
            assert_eq!(installer.artifact_id, pin.artifact_id);
            assert_eq!(installer.tp2, pin.tp2);
            assert_eq!(installer.language, pin.language, "{profile} {}", pin.mod_id);
            assert_eq!(installer.weidu_artifact_id, "weidu-249-amd64");
            assert_eq!(installer.invocation_mode, InvocationMode::SetupName);
            let menu = installer
                .components
                .iter()
                .map(|component| component.id)
                .collect::<Vec<_>>();
            assert_eq!(menu, pin.components, "{profile} {} menu", pin.mod_id);
            assert!(installer
                .components
                .iter()
                .all(|component| component.prompts.is_empty()));

            let runs = manifest
                .collection
                .runs
                .iter()
                .filter(|run| run.mod_id == pin.mod_id)
                .collect::<Vec<_>>();
            assert_eq!(runs.len(), 1, "{profile} {}", pin.mod_id);
            assert_eq!(runs[0].run_id, pin.run_id);
            assert_eq!(runs[0].phase, Phase::Main);
            assert_eq!(runs[0].components, pin.components);
            assert!(runs[0].args.is_empty());
        }
    }
}

#[test]
fn quest_mods_and_bg1ub_are_experimental_options_left_out_of_every_default_plan() {
    let quest_runs = quest_runs();
    for (profile, manifest) in profiles() {
        let mut defaults = BTreeSet::new();
        for feature in &manifest.collection.features {
            let quest = feature
                .components
                .iter()
                .any(|component| quest_runs.contains(component.run_id.as_str()))
                || ["mod:ntotsc", "mod:ac-quest", "mod:ppe"].contains(&feature.id.as_str());
            if quest {
                assert_eq!(feature.category, "quest-mods", "{profile} {}", feature.id);
                assert_eq!(feature.readiness, Readiness::Experimental, "{}", feature.id);
                match feature.decision {
                    Decision::Default => {
                        for component in &feature.components {
                            defaults.insert((component.run_id.clone(), component.component));
                        }
                        // A decided default only describes what ticking its mod adds.
                        assert!(feature.parent.is_some(), "{}", feature.id);
                    }
                    Decision::Optional => {}
                    other => panic!("{profile}: {} is {other:?}", feature.id),
                }
            }
            if feature.id == "mod:bg1ub" || feature.parent.as_deref() == Some("mod:bg1ub") {
                assert_eq!(feature.decision, Decision::Optional, "{}", feature.id);
            }
        }
        let expected = DEFAULTS
            .iter()
            .map(|(run, component)| ((*run).to_owned(), *component))
            .collect::<BTreeSet<_>>();
        assert_eq!(defaults, expected, "{profile}");
        for parent in ["mod:ntotsc", "mod:ac-quest", "mod:ppe", "mod:bg1ub"] {
            let feature = manifest
                .collection
                .features
                .iter()
                .find(|feature| feature.id == parent)
                .unwrap_or_else(|| panic!("{profile}: missing {parent}"));
            assert_eq!(feature.decision, Decision::Optional, "{parent}");
            assert_ne!(
                manifest.presets["chris-recommended"]
                    .selections
                    .get(parent)
                    .map(String::as_str),
                Some("on"),
                "{profile}: {parent}"
            );
        }

        let recommended = with(&manifest, &[]);
        let default_selection = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
        for evaluation in [&recommended, &default_selection] {
            for run_id in quest_runs.iter().copied().chain(["bg1ub-bg1"]) {
                assert_eq!(
                    evaluation.plan.components_for(run_id),
                    None,
                    "{profile}: {run_id}"
                );
            }
        }
        for control in ["mod:ntotsc", "mod:ac-quest", "mod:ppe", "mod:bg1ub"] {
            let control = recommended.view.control(control).unwrap();
            assert!(!control.selected && control.interactive, "{}", control.id);
        }

        // Ticking a mod alone selects only its decided defaults.
        for (parent, run_id, expected) in [
            ("mod:ntotsc", NTOTSC, &[0][..]),
            ("mod:ac-quest", AC_QUEST, &[2, 5, 10][..]),
            ("mod:ppe", PPE, &[0][..]),
            ("feature:thalan:component-0", THALAN, &[0][..]),
            ("feature:zg-iwd-voices:component-0", IWD_VOICES, &[0][..]),
        ] {
            let evaluation = with(&manifest, &on(&[parent]));
            assert_eq!(
                planned(&evaluation, run_id),
                expected,
                "{profile}: {parent}"
            );
            let others = |runs: &[bg_engine::resolve::PlannedRun]| {
                runs.iter()
                    .filter(|run| run.run_id != run_id)
                    .cloned()
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                others(&evaluation.plan.runs),
                others(&recommended.plan.runs),
                "{profile}: {parent} changes another run"
            );
        }
    }
}

#[test]
fn every_quest_mod_choice_resolves_to_exactly_one_side() {
    for (profile, manifest) in profiles() {
        let features = manifest
            .collection
            .features
            .iter()
            .map(|feature| (feature.id.as_str(), feature))
            .collect::<BTreeMap<_, _>>();
        let mut authored = BTreeMap::<String, Vec<String>>::new();
        for feature in &manifest.collection.features {
            if let Some(group) = &feature.choice_group {
                if feature.category == "quest-mods" {
                    authored
                        .entry(group.clone())
                        .or_default()
                        .push(feature.id.clone());
                }
            }
        }
        assert_eq!(
            authored.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            CHOICES.iter().map(|(group, ..)| *group).collect(),
            "{profile}"
        );

        for (group, run_id, options, default) in CHOICES {
            let ids = options
                .iter()
                .map(|component| feature_id(run_id, *component))
                .collect::<Vec<_>>();
            assert_eq!(authored[*group], ids, "{profile} {group}");
            let parent = features[ids[0].as_str()].parent.clone().unwrap();
            for (id, option) in ids.iter().zip(options.iter()) {
                let feature = features[id.as_str()];
                assert_eq!(feature.parent.as_deref(), Some(parent.as_str()));
                // Every option conflicts with every sibling, in both directions.
                let conflicts = feature
                    .conflicts
                    .iter()
                    .map(|conflict| conflict.feature_id.clone())
                    .collect::<BTreeSet<_>>();
                let siblings = ids
                    .iter()
                    .filter(|other| *other != id)
                    .cloned()
                    .collect::<BTreeSet<_>>();
                assert_eq!(conflicts, siblings, "{profile} {id}");
                let expected = if *default == Some(*option) {
                    Decision::Default
                } else {
                    Decision::Optional
                };
                assert_eq!(feature.decision, expected, "{profile} {id}");
            }

            // Component 0 stays on: NTotSC 13/14 and PPE 100/150 require it.
            let base = vec![(parent.clone(), true), (feature_id(run_id, 0), true)];
            let ticked = with(&manifest, &base);
            let chosen = planned(&ticked, run_id)
                .into_iter()
                .filter(|component| options.contains(component))
                .collect::<Vec<_>>();
            assert_eq!(
                chosen,
                default.iter().copied().collect::<Vec<_>>(),
                "{group}"
            );

            for option in *options {
                let mut overrides = base.clone();
                overrides.extend(
                    options
                        .iter()
                        .map(|other| (feature_id(run_id, *other), other == option)),
                );
                let chosen = planned(&with(&manifest, &overrides), run_id)
                    .into_iter()
                    .filter(|component| options.contains(component))
                    .collect::<Vec<_>>();
                assert_eq!(chosen, [*option], "{profile} {group} {option}");
            }

            // Requesting every side at once never plans two of them.
            let mut overrides = base.clone();
            overrides.extend(
                options
                    .iter()
                    .map(|option| (feature_id(run_id, *option), true)),
            );
            let both = planned(&with(&manifest, &overrides), run_id)
                .into_iter()
                .filter(|component| options.contains(component))
                .count();
            assert!(both <= 1, "{profile} {group} planned {both} options");
        }
    }
}

#[test]
fn ntotsc_never_plans_nim_or_svlast_together_with_the_bg1ub_encounters() {
    for (profile, manifest) in profiles() {
        for (ntotsc, bg1ub, untick) in [
            (
                4,
                30,
                "untick Nim Furlwing Encounter under BG1 Unfinished Business",
            ),
            (
                6,
                32,
                "untick Svlast, the Fallen Paladin Encounter under BG1 Unfinished Business",
            ),
        ] {
            let quest = format!("feature:ntotsc:component-{ntotsc}");
            let encounter = format!("feature:bg1ub:component-{bg1ub}");
            for bg1ub_mod in [false, true] {
                for encounter_on in [false, true] {
                    let overrides = vec![
                        ("mod:ntotsc".to_owned(), true),
                        ("feature:ntotsc:component-0".to_owned(), true),
                        (quest.clone(), true),
                        ("mod:bg1ub".to_owned(), bg1ub_mod),
                        (encounter.clone(), encounter_on),
                    ];
                    let evaluation = with(&manifest, &overrides);
                    let has_quest = planned(&evaluation, NTOTSC).contains(&ntotsc);
                    let has_encounter = planned(&evaluation, "bg1ub-bg1").contains(&bg1ub);
                    assert!(
                        !(has_quest && has_encounter),
                        "{profile} {quest} {encounter}"
                    );
                    // BG1UB is preselected: its encounter wins whenever it is selected.
                    let bg1ub_selected = bg1ub_mod && encounter_on;
                    assert_eq!(has_encounter, bg1ub_selected, "{profile} {encounter}");
                    assert_eq!(has_quest, !bg1ub_selected, "{profile} {quest}");
                    let control = evaluation.view.control(&quest).unwrap();
                    if bg1ub_selected {
                        let reason = control.unavailable_reason.as_deref().unwrap();
                        assert!(reason.contains(untick), "{profile}: {reason}");
                    } else {
                        assert!(control.selected, "{profile} {quest}");
                    }
                }
            }
        }
    }
}

#[test]
fn ppe_310_and_320_are_never_planned() {
    for (profile, manifest) in profiles() {
        assert!(manifest
            .collection
            .features
            .iter()
            .all(|feature| feature
                .components
                .iter()
                .all(|component| component.run_id != PPE
                    || ![310, 320].contains(&component.component))));
        // Tick every PPE option, taking each portrait choice in turn.
        let singles = manifest
            .collection
            .features
            .iter()
            .filter(|feature| {
                feature.parent.as_deref() == Some("mod:ppe") && feature.choice_group.is_none()
            })
            .map(|feature| (feature.id.clone(), true))
            .collect::<Vec<_>>();
        for pick in 0..3 {
            let mut overrides = vec![("mod:ppe".to_owned(), true)];
            overrides.extend(singles.iter().cloned());
            for (_, run_id, options, _) in CHOICES.iter().filter(|(_, run, ..)| *run == PPE) {
                let chosen = options[pick.min(options.len() - 1)];
                overrides.extend(
                    options
                        .iter()
                        .map(|option| (feature_id(run_id, *option), *option == chosen)),
                );
            }
            let plan = planned(&with(&manifest, &overrides), PPE);
            assert!(!plan.contains(&310) && !plan.contains(&320), "{profile}");
            // The core, seven single options and one option of each of the five choices.
            assert_eq!(plan.len(), 13, "{profile} {plan:?}");
        }
        // Category portraits need the core component.
        let without_core = with(
            &manifest,
            &[
                ("mod:ppe".to_owned(), true),
                ("feature:ppe:component-0".to_owned(), false),
                ("feature:ppe:component-100".to_owned(), true),
            ],
        );
        assert!(!planned(&without_core, PPE).contains(&100), "{profile}");
    }
}

#[test]
fn black_hearts_is_unavailable_while_the_sod_remix_is_selected() {
    for (profile, manifest) in profiles() {
        let feature = manifest
            .collection
            .features
            .iter()
            .find(|feature| feature.id == BLACKHEARTS_FEATURE)
            .unwrap();
        assert_eq!(feature.parent, None);
        assert_eq!(feature.conflicts.len(), 1);
        assert_eq!(feature.conflicts[0].feature_id, SOD_REMIX_CORE);
        let reason = &feature.conflicts[0].reason;
        assert!(reason.starts_with("Unavailable with CHRIZ-SOD-REMIX"));

        let ticked = with(&manifest, &on(&[BLACKHEARTS_FEATURE]));
        assert_eq!(ticked.plan.components_for(BLACKHEARTS), None, "{profile}");
        let control = ticked.view.control(BLACKHEARTS_FEATURE).unwrap();
        assert!(!control.selected);
        assert_eq!(control.unavailable_reason.as_deref(), Some(reason.as_str()));

        // The declaration is the only blocker: without the Remix, Black Hearts resolves.
        let without_remix = with(
            &manifest,
            &[
                (BLACKHEARTS_FEATURE.to_owned(), true),
                ("mod:chriz-sod-remix".to_owned(), false),
            ],
        );
        assert_eq!(
            without_remix.plan.components_for(BLACKHEARTS),
            Some(&[1][..])
        );
        assert_eq!(
            without_remix.plan.components_for("chriz-sod-remix-bg2"),
            None
        );
    }
}

#[test]
fn quest_mod_runs_keep_the_reviewed_order_in_recipe_and_plan() {
    for (profile, manifest) in profiles() {
        let mut everything = vec![
            ("mod:bg1ub".to_owned(), true),
            ("feature:bg1ub:component-0".to_owned(), true),
            (BLACKHEARTS_FEATURE.to_owned(), true),
        ];
        everything.extend(on(&[
            "mod:ntotsc",
            "mod:ac-quest",
            "mod:ppe",
            "feature:thalan:component-0",
            "feature:zg-iwd-voices:component-0",
        ]));
        let with_remix = with(&manifest, &everything);
        let mut without_remix_overrides = everything.clone();
        without_remix_overrides.push(("mod:chriz-sod-remix".to_owned(), false));
        let without_remix = with(&manifest, &without_remix_overrides);

        let recipe_order = manifest
            .collection
            .runs
            .iter()
            .map(|run| run.run_id.as_str())
            .collect::<Vec<_>>();
        let with_remix_order = with_remix
            .plan
            .runs
            .iter()
            .map(|run| run.run_id.as_str())
            .collect::<Vec<_>>();
        let without_remix_order = without_remix
            .plan
            .runs
            .iter()
            .map(|run| run.run_id.as_str())
            .collect::<Vec<_>>();
        for (name, order) in [
            ("recipe", &recipe_order),
            ("plan", &with_remix_order),
            ("plan without the Remix", &without_remix_order),
        ] {
            let at = |run: &str| {
                order
                    .iter()
                    .position(|id| *id == run)
                    .unwrap_or_else(|| panic!("{profile} {name}: missing {run}"))
            };
            let present = |run: &str| order.contains(&run);
            let eet_end = at("eet-end-bg2");
            for pin in PINS {
                if present(pin.run_id) {
                    assert!(at(pin.run_id) < eet_end, "{profile} {name}: {}", pin.run_id);
                }
            }
            assert_eq!(present(BLACKHEARTS), name != "plan", "{profile} {name}");
            // NTotSC and Ascalon's follow EET and both fixpacks, precede the NPC mods.
            assert!(at("eet-initialize-bg2") < at(NTOTSC));
            assert!(at("eefixpack-bg2") < at(NTOTSC));
            assert!(at(NTOTSC) < at(AC_QUEST));
            assert!(at(AC_QUEST) < at("rr-bg2"));
            if present(BLACKHEARTS) {
                assert!(at(AC_QUEST) < at(BLACKHEARTS));
                assert!(at(BLACKHEARTS) < at("rr-bg2"));
            }
            // The IWD Voice Pack refuses an unmerged sod-dlc.zip.
            assert!(at("dlcmerger-bg1") < at(IWD_VOICES));
            assert!(at("eet-initialize-bg2") < at(IWD_VOICES));
            // Thalantyr after NTotSC, before the Item Randomiser, SCS and Tweaks Anthology.
            assert!(at(NTOTSC) < at(THALAN));
            assert!(at(THALAN) < at("randomiser-bg2"));
            assert!(at(THALAN) < at("stratagems-bg2"));
            if present("cdtweaks-bg2") {
                assert!(at(THALAN) < at("cdtweaks-bg2"));
            }
            for earlier in [NTOTSC, AC_QUEST] {
                assert!(at(earlier) < at("randomiser-bg2"));
                assert!(at(earlier) < at("stratagems-bg2"));
            }
            // PPE after every other main-phase run except Modpack continuity 199, which
            // immediately precedes EET_end.
            assert_eq!(at(PPE) + 1, at(CONTINUITY), "{profile} {name}");
            assert_eq!(at(CONTINUITY) + 1, eet_end, "{profile} {name}");
            for run in [
                "randomiser-bg2",
                "stratagems-bg2",
                "chriz-bg-modpack-pre-continuity-bg2",
            ] {
                assert!(at(run) < at(PPE), "{profile} {name}: {run}");
            }
        }
        // Every main-phase run of the recipe other than continuity precedes PPE.
        let ppe = recipe_order.iter().position(|id| *id == PPE).unwrap();
        for (index, run) in manifest.collection.runs.iter().enumerate() {
            if run.phase == Phase::Main && run.run_id != CONTINUITY {
                assert!(index <= ppe, "{profile}: {} after PPE", run.run_id);
            }
        }
    }
}

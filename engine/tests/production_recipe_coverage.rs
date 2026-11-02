use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bg_engine::manifest::{Decision, Readiness};
use bg_engine::recipe_view::evaluate_preset;
use bg_engine::Manifest;
use serde::Deserialize;

#[derive(Deserialize)]
struct CurationMap {
    targets: Vec<CurationTarget>,
}

#[derive(Deserialize)]
struct CurationTarget {
    rows: Vec<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn recipe() -> Manifest {
    Manifest::load(&repo_root().join("manifest")).expect("production recipe should load")
}

fn decision_for(catalog: &str, component: u32) -> String {
    let path = repo_root()
        .join("docs/curation/components")
        .join(format!("{catalog}.md"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read curation catalog {}: {error}", path.display()));
    let prefix = format!("| {component} |");
    let row = text
        .lines()
        .find(|line| line.starts_with(&prefix))
        .unwrap_or_else(|| panic!("missing {catalog}:{component} in {}", path.display()));
    row.split('|')
        .nth(6)
        .expect("decision column")
        .trim()
        .to_owned()
}

fn catalog_for_mod(mod_id: &str) -> &'static str {
    match mod_id {
        "dlcmerger" => "DLCMERGER",
        "eefixpack" => "EEFIXPACK",
        "bg1ub" => "BG1UB",
        "bg1npc" => "BG1NPC",
        "eet" => "EET",
        "eet-end" => "EET_END",
        "eeex" => "EEEX",
        "eeexremote" => "EEEXREMOTE",
        "chriz-bg-modpack" => "CHRIZ-BG-MODPACK",
        "chriz-bg-rebalance" => "CHRIZ-BG-REBALANCE",
        "bubb-spell-menu" => "BUBB_SPELL_MENU_EXTENDED",
        "bggo" => "BGGO",
        "rr" => "RR",
        "fade" => "FADE",
        "paina" => "PAINA",
        "sarahtob" => "SARAHTOB",
        "ub" => "UB",
        "xan" => "XAN",
        "yeslicknpc" => "YESLICKNPC",
        "sirene-bg2" => "SIRENE_BG2",
        "sirene-bg1" => "SIRENE_BG1",
        "spell-rev" => "SPELL_REV",
        "srcb-rr-compat" => "SRCB_RR_COMPAT",
        "ascension" => "ASCENSION",
        "artisanskitpack" => "ARTISANSKITPACK",
        "artisanskitpack-npc" => "ARTISANSKITPACK_NPC",
        "artisanskitpack-tweak" => "ARTISANSKITPACK_TWEAK",
        "stratagems" => "STRATAGEMS",
        "randomiser" => "RANDOMISER",
        "buffbot" => "BUFFBOT",
        "chriz-sod-remix" => "CHRIZ-SOD-REMIX",
        "crossmodbg2" => "CROSSMODBG2",
        "hidden-gameplay-options" => "HIDDENGAMEPLAYOPTIONS",
        "hq-soundclips-bg2ee" => "HQ_SOUNDCLIPS_BG2EE",
        "iepbanters" => "IEPBANTERS",
        "safana" => "SAFANA",
        "voicepack-neera" => "CD_NEERA",
        "voicepack-safana-bg1" => "CD_SAFANA",
        "voicepack-safana-sod" => "CD_SAFSOD",
        "voicepack-jaheira-bg1" => "CD_JAHEBG1",
        "voicepack-jaheira-bg2" => "CD_JAHEBG2",
        "voicepack-yeslick" => "CD_YESLICK",
        "voicepack-edwin-bg1" => "CD_EDWNBG1",
        "voicepack-edwin-bg2" => "CD_EDWNBG2",
        "voicepack-aerie" => "CD_AERIE",
        "voicepack-imoen-bg1" => "CD_IMOBG1",
        "voicepack-imoen-bg2" => "CD_IMOBG2",
        "voicepack-sarevok" => "CD_SAREVOK",
        "voicepack-amelyssan" => "CD_AMELY",
        "voicepack-gromnir-il-khan" => "CD_GROMN",
        "voicepack-ajantis" => "CD_AJANTI",
        "voicepack-viconia-bg1" => "CD_VICBG1",
        "voicepack-viconia-bg2" => "CD_VICBG2",
        "voicepack-garithos" => "CD_GARITH",
        "ntotsc" => "NTOTSC",
        "ac-quest" => "AC_QUEST",
        "blackhearts" => "BLACKHEARTS",
        "thalan" => "THALAN",
        "zg-iwd-voices" => "ZG_IWD_VOICES",
        "ppe" => "PPE",
        other => panic!("production run has unknown curation catalog: {other}"),
    }
}

#[test]
fn every_resolved_component_maps_to_a_nonblank_curated_row() {
    let root = repo_root();
    let map_text = std::fs::read_to_string(root.join("manifest/curation-map.toml")).unwrap();
    let curation_map: CurationMap = toml::from_str(&map_text).unwrap();
    let mapped_rows = curation_map
        .targets
        .into_iter()
        .flat_map(|target| target.rows)
        .collect::<BTreeSet<_>>();
    let manifest = recipe();
    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let run_mods = manifest
        .collection
        .runs
        .iter()
        .map(|run| (run.run_id.as_str(), run.mod_id.as_str()))
        .collect::<BTreeMap<_, _>>();

    for run in &evaluation.plan.runs {
        let catalog = catalog_for_mod(run_mods[run.run_id.as_str()]);
        for component in &run.components {
            let row = format!("{catalog}:{component}");
            assert!(
                mapped_rows.contains(&row),
                "resolved row {row} is not mapped"
            );
            let decision = decision_for(catalog, *component);
            assert!(!decision.is_empty(), "resolved blank curation row {row}");
        }
    }
}

#[test]
fn every_offered_quest_mod_and_bg1ub_component_maps_to_a_nonblank_curated_row() {
    // October 23: the quest mods and BG1 Unfinished Business are optional and unchecked
    // in Recommended, so the resolved-plan check above does not reach their components.
    let root = repo_root();
    let map_text = std::fs::read_to_string(root.join("manifest/curation-map.toml")).unwrap();
    let curation_map: CurationMap = toml::from_str(&map_text).unwrap();
    let mapped_rows = curation_map
        .targets
        .into_iter()
        .flat_map(|target| target.rows)
        .collect::<BTreeSet<_>>();
    let manifest = recipe();
    let run_mods = manifest
        .collection
        .runs
        .iter()
        .map(|run| (run.run_id.as_str(), run.mod_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let offered = [
        "ntotsc",
        "ac-quest",
        "blackhearts",
        "thalan",
        "zg-iwd-voices",
        "ppe",
        "bg1ub",
    ];
    let mut checked = BTreeSet::new();
    for feature in &manifest.collection.features {
        for component in &feature.components {
            let mod_id = run_mods[component.run_id.as_str()];
            if !offered.contains(&mod_id) {
                continue;
            }
            let catalog = catalog_for_mod(mod_id);
            let row = format!("{catalog}:{}", component.component);
            assert!(
                mapped_rows.contains(&row),
                "offered row {row} is not mapped"
            );
            let decision = decision_for(catalog, component.component);
            assert!(!decision.is_empty(), "offered blank curation row {row}");
            checked.insert(row);
        }
    }
    // 52 offered quest-mod rows (PPE 310/320 and Black Hearts 20005 are not offered)
    // and the 16 BG1UB rows.
    assert_eq!(checked.len(), 68);
}

#[test]
fn blocked_and_blank_rows_never_resolve_and_no_legacy_fix_installer_is_present() {
    let manifest = recipe();
    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let resolved = evaluation
        .plan
        .runs
        .iter()
        .flat_map(|run| {
            run.components
                .iter()
                .map(move |component| (run.run_id.as_str(), *component))
        })
        .collect::<BTreeSet<_>>();

    for feature in &manifest.collection.features {
        if feature.readiness == Readiness::Blocked || feature.decision == Decision::Excluded {
            assert!(
                feature.components.iter().all(|component| {
                    !resolved.contains(&(component.run_id.as_str(), component.component))
                }),
                "blocked/excluded feature {:?} resolved",
                feature.id
            );
        }
    }
    assert!(!resolved.contains(&("dlcmerger-bg1", 10)));
    assert_eq!(
        manifest.mods.keys().cloned().collect::<Vec<_>>(),
        [
            "ac-quest",
            "artisanskitpack",
            "artisanskitpack-npc",
            "artisanskitpack-tweak",
            "ascension",
            "bg1npc",
            "bg1ub",
            "bggo",
            "blackhearts",
            "bubb-spell-menu",
            "buffbot",
            "chriz-bg-modpack",
            "chriz-bg-rebalance",
            "chriz-sod-remix",
            "crossmodbg2",
            "dlcmerger",
            "dragonspear-ui",
            "eeex",
            "eeexremote",
            "eefixpack",
            "eet",
            "eet-end",
            "eet-gui",
            "fade",
            "hidden-gameplay-options",
            "hq-soundclips-bg2ee",
            "iepbanters",
            "infinity-ui",
            "leui",
            "leui-bg1ee",
            "leui-sod",
            "ntotsc",
            "paina",
            "ppe",
            "randomiser",
            "rr",
            "safana",
            "sarahtob",
            "sirene-bg1",
            "sirene-bg2",
            "spell-rev",
            "srcb-rr-compat",
            "stratagems",
            "thalan",
            "ub",
            "voicepack-aerie",
            "voicepack-ajantis",
            "voicepack-amelyssan",
            "voicepack-edwin-bg1",
            "voicepack-edwin-bg2",
            "voicepack-garithos",
            "voicepack-gromnir-il-khan",
            "voicepack-imoen-bg1",
            "voicepack-imoen-bg2",
            "voicepack-jaheira-bg1",
            "voicepack-jaheira-bg2",
            "voicepack-neera",
            "voicepack-safana-bg1",
            "voicepack-safana-sod",
            "voicepack-sarevok",
            "voicepack-viconia-bg1",
            "voicepack-viconia-bg2",
            "voicepack-yeslick",
            "xan",
            "yeslicknpc",
            "zg-iwd-voices",
        ]
    );
    let collection = std::fs::read_to_string(repo_root().join("manifest/collection.toml")).unwrap();
    assert!(!collection.contains("FAIL"));
    assert!(!collection.contains("local-fix"));
}

#[test]
fn recommended_preset_explicitly_keeps_every_default_core_control_desired() {
    let manifest = recipe();
    let preset = &manifest.presets["chris-recommended"];
    for feature in &manifest.collection.features {
        if feature.decision == Decision::Default {
            assert_eq!(
                preset.selections.get(&feature.id).map(String::as_str),
                Some("on"),
                "preset must explicitly enable {:?}",
                feature.id
            );
        }
    }
    assert_eq!(
        preset
            .selections
            .get("feature:bg1npc:component-160")
            .map(String::as_str),
        Some("on")
    );
}

#[test]
fn eet_end_is_the_core_tail_anchor_and_only_the_scroll_policy_follows_buffbot() {
    let manifest = recipe();
    let last_before_post_eet = manifest
        .collection
        .runs
        .iter()
        .rfind(|run| run.phase != bg_engine::manifest::Phase::PostEetEnd)
        .expect("one run before post-EET tail");
    assert_eq!(last_before_post_eet.run_id, "eet-end-bg2");
    assert_eq!(last_before_post_eet.mod_id, "eet-end");
    // Modpack scroll policy 430 hashes the final scrolls and the spells they cast,
    // so its own run follows BuffBot and every other spell/item writer (October 13).
    let (last, rest) = manifest.collection.runs.split_last().expect("one run");
    assert_eq!(last.run_id, "chriz-bg-modpack-scrolls-bg2");
    assert_eq!(last.components, [430]);
    let buffbot = rest.last().expect("BuffBot before the scroll policy");
    assert_eq!(buffbot.run_id, "buffbot-bg2");
    assert_eq!(buffbot.mod_id, "buffbot");
}

#[test]
fn legacy_source_ledger_points_at_the_same_frozen_eefixpack_and_eet_bytes() {
    let ledger = std::fs::read_to_string(repo_root().join("manifest/mod-sources.tsv")).unwrap();
    let rows = ledger
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields = line.split('\t').collect::<Vec<_>>();
            (fields.len() >= 9).then_some((fields[0], (fields[7], fields[8])))
        })
        .collect::<BTreeMap<_, _>>();

    assert_eq!(
        rows["EEFIXPACK"].0,
        "https://github.com/Gibberlings3/EE_Fixpack/releases/download/Beta_2/enhanced-edition-fixpack-beta-2.iemod"
    );
    for id in ["EET", "EET_END"] {
        assert_eq!(
            rows[id].0,
            "https://codeload.github.com/Gibberlings3/EET/zip/74e91d72bca5d073fa11c1d088b90d7ff0c7105d"
        );
    }
    assert_eq!(
        rows["EET"].1,
        "EET-74e91d72bca5d073fa11c1d088b90d7ff0c7105d"
    );
}

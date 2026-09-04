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
        "spell-rev" => "SPELL_REV",
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
            "artisanskitpack",
            "artisanskitpack-npc",
            "artisanskitpack-tweak",
            "ascension",
            "bg1npc",
            "bg1ub",
            "bggo",
            "bubb-spell-menu",
            "buffbot",
            "chriz-bg-modpack",
            "chriz-sod-remix",
            "crossmodbg2",
            "dlcmerger",
            "eeex",
            "eeexremote",
            "eefixpack",
            "eet",
            "eet-end",
            "fade",
            "hidden-gameplay-options",
            "hq-soundclips-bg2ee",
            "iepbanters",
            "paina",
            "randomiser",
            "rr",
            "sarahtob",
            "sirene-bg2",
            "spell-rev",
            "stratagems",
            "ub",
            "xan",
            "yeslicknpc",
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
fn eet_end_is_the_core_tail_anchor_and_buffbot_is_absolute_last() {
    let manifest = recipe();
    let last_before_post_eet = manifest
        .collection
        .runs
        .iter()
        .rfind(|run| run.phase != bg_engine::manifest::Phase::PostEetEnd)
        .expect("one run before post-EET tail");
    assert_eq!(last_before_post_eet.run_id, "eet-end-bg2");
    assert_eq!(last_before_post_eet.mod_id, "eet-end");
    let last = manifest.collection.runs.last().expect("one run");
    assert_eq!(last.run_id, "buffbot-bg2");
    assert_eq!(last.mod_id, "buffbot");
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

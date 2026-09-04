use std::collections::BTreeMap;
use std::path::PathBuf;

use bg_engine::manifest::{
    AcquisitionPolicy, ArchiveKind, ArchiveRootRule, Decision, Phase, Readiness, SourceKind,
};
use bg_engine::recipe_view::evaluate_preset;
use bg_engine::Manifest;

const EEEX_COMPONENTS: &[u32] = &[0, 1, 2, 3, 4, 5, 6, 7, 8];
const RR_AUTHORED_COMPONENTS: &[u32] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 999];
const RR_DEFAULT_COMPONENTS: &[u32] = &[0, 3, 7, 8, 11, 12, 999];
const UB_AUTHORED_COMPONENTS: &[u32] = &[
    0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 17, 18, 20, 21, 22, 23, 24, 25,
];
const UB_DEFAULT_COMPONENTS: &[u32] = &[
    0, 1, 2, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 17, 18, 20, 21, 25,
];
const ASCENSION_COMPONENTS: &[u32] = &[
    0, 10, 20, 30, 50, 60, 1000, 1100, 1200, 1300, 1400, 1500, 2000, 2100, 2300, 2400,
];
const HGO_COMPONENTS: &[u32] = &[
    10, 11, 12, 13, 14, 16, 17, 18, 19, 20, 22, 23, 24, 25, 27, 28, 29, 30, 32, 33, 34, 35, 36, 37,
    38, 39, 40, 200, 300, 301, 302, 304, 101, 102, 103,
];

fn recipe_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../manifest")
}

fn recipe() -> Manifest {
    Manifest::load(&recipe_root()).expect("production recipe should load")
}

#[derive(Clone)]
struct ExpectedArtifact<'a> {
    version: &'a str,
    source_kind: SourceKind,
    url: &'a str,
    reference: &'a str,
    filename: &'a str,
    length: u64,
    sha256: &'a str,
    archive_kind: ArchiveKind,
    root_rule: ArchiveRootRule,
    publish_roots: &'a [&'a str],
    tp2_paths: &'a [&'a str],
}

#[test]
fn pins_the_reviewed_official_artifacts_from_eeex_through_ascension() {
    let manifest = recipe();
    let expected = BTreeMap::from([
        (
            "eeex-1.2.0",
            ExpectedArtifact {
                version: "1.2.0",
                source_kind: SourceKind::GithubRelease,
                url: "https://github.com/Bubb13/EEex/releases/download/v1.2.0/eeex-v1.2.0.zip",
                reference: "v1.2.0",
                filename: "eeex-v1.2.0.zip",
                length: 10_346_381,
                sha256: "ac2d81bc5e6fdba04a686338a969507e5beca9cb082b2809a556e0874e690a88",
                archive_kind: ArchiveKind::Zip,
                root_rule: ArchiveRootRule::Direct,
                publish_roots: &["EEex"],
                tp2_paths: &["EEex/EEex.tp2"],
            },
        ),
        (
            "bubb-spell-menu-5.2",
            ExpectedArtifact {
                version: "5.2",
                source_kind: SourceKind::GithubTagArchive,
                url: "https://codeload.github.com/Bubb13/Bubbs-Spell-Menu-Extended/zip/refs/tags/v5.2",
                reference: "v5.2",
                filename: "Bubbs-Spell-Menu-Extended-5.2.zip",
                length: 1_432_756,
                sha256: "ac5380e99a534dd96200e97fa4358ca22ff2738266570d95195eac85d8f82e12",
                archive_kind: ArchiveKind::Zip,
                root_rule: ArchiveRootRule::SingleWrapper,
                publish_roots: &["bubb_spell_menu_extended"],
                tp2_paths: &["bubb_spell_menu_extended/bubb_spell_menu_extended.tp2"],
            },
        ),
        (
            "bggo-3.6",
            ExpectedArtifact {
                version: "3.6",
                source_kind: SourceKind::GithubRelease,
                url: "https://github.com/Spellhold-Studios/Baldurs-Gate-Graphical-Overhaul/releases/download/v3.6/baldurs-gate-graphical-overhaul-v3.6.zip",
                reference: "v3.6",
                filename: "baldurs-gate-graphical-overhaul-v3.6.zip",
                length: 1_459_899_654,
                sha256: "59e01a3dcf48b141fe2e24dc5df1906c5031cedf22662b697d13756609d3d635",
                archive_kind: ArchiveKind::Zip,
                root_rule: ArchiveRootRule::Direct,
                publish_roots: &["bggo"],
                tp2_paths: &["bggo/bggo.tp2"],
            },
        ),
        (
            "hidden-gameplay-options-5.2",
            ExpectedArtifact {
                version: "5.2",
                source_kind: SourceKind::GithubRelease,
                url: "https://github.com/Argent77/A7-HiddenGameplayOptions/releases/download/v5.2/win-A7-HiddenGameplayOptions-v5.2.zip",
                reference: "v5.2",
                filename: "win-A7-HiddenGameplayOptions-v5.2.zip",
                length: 2_767_061,
                sha256: "a7b173f811b1d2137e9836e8dec7118e0a1927ff85e26956d211e0d1b53c2029",
                archive_kind: ArchiveKind::Zip,
                root_rule: ArchiveRootRule::Direct,
                publish_roots: &["HiddenGameplayOptions"],
                tp2_paths: &["HiddenGameplayOptions/HiddenGameplayOptions.tp2"],
            },
        ),
        (
            "hq-soundclips-bg2ee-1.3",
            ExpectedArtifact {
                version: "1.3",
                source_kind: SourceKind::GithubRelease,
                url: "https://github.com/Argent77/HQ-SoundClips-BG2EE/releases/download/v1.3/win-A7-HQ-SoundClips-BG2EE-v1.3.zip",
                reference: "v1.3",
                filename: "win-A7-HQ-SoundClips-BG2EE-v1.3.zip",
                length: 206_363_285,
                sha256: "712af8ab21048c0a1dbba7257c33b3dc6beba12d86d4d4c7a1b80982adc6ee63",
                archive_kind: ArchiveKind::Zip,
                root_rule: ArchiveRootRule::Direct,
                publish_roots: &["HQ_SoundClips_BG2EE"],
                tp2_paths: &["HQ_SoundClips_BG2EE/HQ_SoundClips_BG2EE.tp2"],
            },
        ),
        (
            "rr-4.92",
            ExpectedArtifact {
                version: "4.92",
                source_kind: SourceKind::GithubRelease,
                url: "https://github.com/Spellhold-Studios/Rogue-Rebalancing/releases/download/v4.92/rr-v492.zip",
                reference: "v4.92",
                filename: "rr-v492.zip",
                length: 4_823_623,
                sha256: "692297f6da8fbfe40eeccfc3ac8fd449c66b2c8e91d3d59b70cd5866a69c503e",
                archive_kind: ArchiveKind::Zip,
                root_rule: ArchiveRootRule::Direct,
                publish_roots: &["rr"],
                tp2_paths: &["rr/setup-rr.tp2"],
            },
        ),
        (
            "fade-5.6",
            ExpectedArtifact {
                version: "5.6",
                source_kind: SourceKind::GithubRelease,
                url: "https://github.com/Spellhold-Studios/Fade-NPC/releases/download/v5.6/Fade-v5.6.zip",
                reference: "v5.6",
                filename: "Fade-v5.6.zip",
                length: 10_000_433,
                sha256: "6d3cd8d528cfad55c8542fa706fc9e7f4f6c27ba6307515dd6f6d0d308ebe3ce",
                archive_kind: ArchiveKind::Zip,
                root_rule: ArchiveRootRule::Direct,
                publish_roots: &["fade"],
                tp2_paths: &["fade/Setup-Fade.tp2"],
            },
        ),
        (
            "paina-1.9",
            ExpectedArtifact {
                version: "1.9",
                source_kind: SourceKind::GithubTagArchive,
                url: "https://codeload.github.com/TheArtisanBG/Pai-Na-NPC-mod-for-BG2-EE/zip/refs/tags/v1.9",
                reference: "v1.9",
                filename: "Pai-Na-NPC-mod-for-BG2-EE-1.9.zip",
                length: 10_772_351,
                sha256: "8b448170062e4dba39bcd23909228cc80131a526fc56ee7215a0a0e96540371c",
                archive_kind: ArchiveKind::Zip,
                root_rule: ArchiveRootRule::SingleWrapper,
                publish_roots: &["paina"],
                tp2_paths: &["paina/Paina.tp2"],
            },
        ),
        (
            "sarahtob-8",
            ExpectedArtifact {
                version: "8",
                source_kind: SourceKind::GithubRelease,
                url: "https://github.com/Gibberlings3/Sarah/releases/download/v8/sarah-npc-romance-mod-for-bg2-tob-v8.iemod",
                reference: "v8",
                filename: "sarah-npc-romance-mod-for-bg2-tob-v8.iemod",
                length: 58_496_710,
                sha256: "061b1da90d8cd58caf7ef3c3f8f4ab225dce2c6b2e528eec656a9465b0d33811",
                archive_kind: ArchiveKind::Iemod,
                root_rule: ArchiveRootRule::Direct,
                publish_roots: &["sarahtob"],
                tp2_paths: &["sarahtob/setup-sarahtob.tp2"],
            },
        ),
        (
            "ub-28",
            ExpectedArtifact {
                version: "28",
                source_kind: SourceKind::GithubRelease,
                url: "https://github.com/Pocket-Plane-Group/UnfinishedBusiness/releases/download/v28/ub-v28.zip",
                reference: "v28",
                filename: "ub-v28.zip",
                length: 6_969_494,
                sha256: "1d5aff32d760c54486eacf09c4904c3cf2cd52cb76ba15c8f050430fd7a73d62",
                archive_kind: ArchiveKind::Zip,
                root_rule: ArchiveRootRule::Direct,
                publish_roots: &["ub"],
                tp2_paths: &["ub/kalah/kalah.tp2", "ub/setup-ub.tp2"],
            },
        ),
        (
            "xan-19",
            ExpectedArtifact {
                version: "19",
                source_kind: SourceKind::GithubRelease,
                url: "https://github.com/Pocket-Plane-Group/Xan_for_BGII/releases/download/v19/Xan_v19.zip",
                reference: "v19",
                filename: "Xan_v19.zip",
                length: 74_295_924,
                sha256: "47cd6622678f56d1e67b7c01579f943bf9ced813a26d3d72bc0866088cf195d7",
                archive_kind: ArchiveKind::Zip,
                root_rule: ArchiveRootRule::Direct,
                publish_roots: &["Xan"],
                tp2_paths: &["Xan/xan.tp2"],
            },
        ),
        (
            "yeslicknpc-5.0",
            ExpectedArtifact {
                version: "5.0",
                source_kind: SourceKind::GithubRelease,
                url: "https://github.com/Spellhold-Studios/Yeslick-NPC/releases/download/v5.0/yeslick-npc-v5.0.zip",
                reference: "v5.0",
                filename: "yeslick-npc-v5.0.zip",
                length: 9_393_233,
                sha256: "d9b403a6feb8638cc70ac778ccaed73f4eadd3b780d1e10605265763fe6414fe",
                archive_kind: ArchiveKind::Zip,
                root_rule: ArchiveRootRule::Direct,
                publish_roots: &["yeslicknpc"],
                tp2_paths: &["yeslicknpc/yeslicknpc.tp2"],
            },
        ),
        (
            "sirene-bg2-00beda909a4a791a35016069b4ebb77f27279723",
            ExpectedArtifact {
                version: "00beda909a4a791a35016069b4ebb77f27279723",
                source_kind: SourceKind::GithubCommitZip,
                url: "https://codeload.github.com/TheArtisanBG/Sirene-NPC-for-BG2-EE/zip/00beda909a4a791a35016069b4ebb77f27279723",
                reference: "00beda909a4a791a35016069b4ebb77f27279723",
                filename: "Sirene-NPC-for-BG2-EE-00beda909a4a791a35016069b4ebb77f27279723.zip",
                length: 33_874_271,
                sha256: "1a6d517b7b02367d35df61931e168af4d9673c978c3bfc553bfdf138bd4cdcf8",
                archive_kind: ArchiveKind::Zip,
                root_rule: ArchiveRootRule::SingleWrapper,
                publish_roots: &["Sirene_BG2"],
                tp2_paths: &["Sirene_BG2/Sirene_BG2.tp2"],
            },
        ),
        (
            "ascension-2.1.0",
            ExpectedArtifact {
                version: "2.1.0",
                source_kind: SourceKind::GithubRelease,
                url: "https://github.com/Gibberlings3/Ascension/releases/download/2.1.0/ascension-2.1.0.zip",
                reference: "2.1.0",
                filename: "ascension-2.1.0.zip",
                length: 7_826_251,
                sha256: "bd0ee14b3770d56104eaa81287c88a8672afa889a35184fb308be12d6d707e77",
                archive_kind: ArchiveKind::Zip,
                root_rule: ArchiveRootRule::Direct,
                publish_roots: &["ascension"],
                tp2_paths: &["ascension/Ascension.tp2"],
            },
        ),
    ]);

    for (id, expected) in expected {
        let artifact = manifest
            .artifacts
            .get(id)
            .unwrap_or_else(|| panic!("missing official artifact {id}"));
        assert_eq!(artifact.version, expected.version, "{id} version");
        assert_eq!(artifact.acquisition, AcquisitionPolicy::FetchOnly, "{id}");
        assert_eq!(artifact.source.kind, expected.source_kind, "{id} kind");
        assert_eq!(artifact.source.url, expected.url, "{id} URL");
        assert_eq!(artifact.source.reference, expected.reference, "{id} ref");
        assert_eq!(
            artifact.source.expected_filename.as_deref(),
            Some(expected.filename),
            "{id} filename"
        );
        assert_eq!(
            artifact.source.expected_length,
            Some(expected.length),
            "{id}"
        );
        assert_eq!(artifact.source.sha256, expected.sha256, "{id} digest");
        assert_eq!(artifact.archive.kind, expected.archive_kind, "{id}");
        assert_eq!(artifact.archive.root_rule, expected.root_rule, "{id}");
        assert_eq!(
            artifact.archive.publish_roots, expected.publish_roots,
            "{id}"
        );
        assert_eq!(artifact.archive.tp2_paths, expected.tp2_paths, "{id}");
        assert_eq!(artifact.provenance.reviewed_on, "2026-09-04", "{id}");
    }
}

#[test]
fn declares_the_reviewed_component_menus_and_late_hgo_run() {
    let manifest = recipe();
    let expected = BTreeMap::from([
        ("eeex", EEEX_COMPONENTS),
        ("hidden-gameplay-options", HGO_COMPONENTS),
        ("rr", RR_AUTHORED_COMPONENTS),
        ("fade", &[0, 2][..]),
        ("paina", &[0][..]),
        ("sarahtob", &[1][..]),
        ("ub", UB_AUTHORED_COMPONENTS),
        ("xan", &[0, 1, 2, 3, 4][..]),
        ("yeslicknpc", &[0, 1][..]),
        ("sirene-bg2", &[0, 2, 5, 6, 7, 8][..]),
        ("ascension", ASCENSION_COMPONENTS),
    ]);
    for (mod_id, expected_components) in expected {
        let actual = manifest.mods[mod_id]
            .components
            .iter()
            .map(|component| component.id)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected_components, "{mod_id} menu order");
    }
    assert_eq!(manifest.mods["bubb-spell-menu"].components[0].id, 0);
    assert_eq!(manifest.mods["bggo"].components[0].id, 0);
    let hgo = manifest
        .collection
        .runs
        .iter()
        .find(|run| run.mod_id == "hidden-gameplay-options")
        .expect("late HGO run");
    assert_eq!(hgo.components, HGO_COMPONENTS);
}

#[test]
fn preserves_xan_wild_mage_tob_only_scope_in_catalog_and_ui() {
    let manifest = recipe();
    let component = manifest.mods["xan"]
        .components
        .iter()
        .find(|component| component.id == 4)
        .expect("Xan component 4 must be authored");
    assert_eq!(component.name, "Change Xan's class to Wild Mage (ToB only)");

    let feature = manifest
        .collection
        .features
        .iter()
        .find(|feature| feature.id == "feature:xan:component-4")
        .expect("Xan component 4 must have a player-facing feature");
    assert_eq!(feature.title, "Make Xan a Wild Mage (ToB only)");
}

#[test]
fn authors_the_reviewed_official_run_order_and_filters_unresolved_duplicates() {
    let manifest = recipe();
    let run_ids = manifest
        .collection
        .runs
        .iter()
        .map(|run| run.run_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        run_ids,
        [
            "dlcmerger-bg1",
            "eefixpack-bg1",
            "bg1ub-bg1",
            "bg1npc-bg1",
            "eefixpack-bg2",
            "eet-initialize-bg2",
            "eeex-bg2",
            "bubb-spell-menu-bg2",
            "bggo-bg2",
            "rr-bg2",
            "fade-bg2",
            "paina-bg2",
            "sarahtob-bg2",
            "ub-bg2",
            "xan-bg2",
            "yeslicknpc-bg2",
            "sirene-bg2",
            "ascension-bg2",
            "spell-rev-core-bg2",
            "artisanskitpack-main-bg2",
            "artisanskitpack-npc-bg2",
            "iepbanters-bg2",
            "crossmodbg2-bg2",
            "artisanskitpack-tweak-early-bg2",
            "hq-soundclips-bg2ee-bg2",
            "randomiser-bg2",
            "stratagems-bg2",
            "artisanskitpack-tweak-late-bg2",
            "artisanskitpack-npc-late-bg2",
            "eet-end-bg2",
            "chriz-sod-remix-bg2",
            "hiddengameplayoptions-bg2",
            "chriz-bg-rebalance-bg2",
            "eeexremote-bg2",
            "chriz-bg-modpack-bg2",
            "spell-rev-npc-spellbooks-bg2",
            "buffbot-bg2",
        ]
    );
    assert!(manifest.collection.runs[6..29]
        .iter()
        .all(|run| run.phase == Phase::Main));
    assert_eq!(manifest.collection.runs[6].components, EEEX_COMPONENTS);
    assert_eq!(
        manifest.collection.runs[9].components,
        RR_AUTHORED_COMPONENTS
    );
    assert_eq!(
        manifest.collection.runs[13].components,
        UB_AUTHORED_COMPONENTS
    );
    assert_eq!(
        manifest.collection.runs[17].components,
        ASCENSION_COMPONENTS
    );
    assert!(!manifest.collection.runs[13].components.contains(&19));
    assert!(!manifest.collection.runs[17].components.contains(&40));
}

#[test]
fn recommended_preset_reproduces_the_proven_official_defaults_except_recorded_omissions() {
    let manifest = recipe();
    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    let plan = &evaluation.plan;

    assert_eq!(plan.components_for("eeex-bg2"), Some(EEEX_COMPONENTS));
    assert!(
        evaluation
            .view
            .control("feature:bubb-spell-menu-extended:component-0")
            .expect("curation-map feature id must be authored")
            .selected
    );
    assert_eq!(plan.components_for("bubb-spell-menu-bg2"), Some(&[0][..]));
    assert_eq!(plan.components_for("bggo-bg2"), Some(&[0][..]));
    assert_eq!(plan.components_for("rr-bg2"), Some(RR_DEFAULT_COMPONENTS));
    assert_eq!(plan.components_for("fade-bg2"), Some(&[0][..]));
    assert_eq!(plan.components_for("paina-bg2"), Some(&[0][..]));
    assert_eq!(plan.components_for("sarahtob-bg2"), Some(&[1][..]));
    assert_eq!(plan.components_for("ub-bg2"), Some(UB_DEFAULT_COMPONENTS));
    assert_eq!(plan.components_for("xan-bg2"), Some(&[0, 1][..]));
    assert_eq!(plan.components_for("yeslicknpc-bg2"), Some(&[1][..]));
    assert_eq!(plan.components_for("sirene-bg2"), Some(&[0, 2, 5][..]));
    assert_eq!(
        plan.components_for("hiddengameplayoptions-bg2"),
        Some(
            &[
                10, 11, 12, 13, 14, 16, 18, 19, 20, 22, 23, 24, 25, 27, 28, 29, 30, 32, 33, 34, 35,
                36, 37, 39, 300, 301, 103,
            ][..]
        )
    );
    assert_eq!(
        plan.components_for("ascension-bg2"),
        Some(ASCENSION_COMPONENTS)
    );
    assert!(plan.runs.iter().all(|run| run.mod_id != "evandra"));
}

#[test]
fn keeps_manual_evandra_and_dependency_blocked_hgo_options_unavailable() {
    let manifest = recipe();
    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();

    for feature_id in [
        "mod:evandra",
        "feature:hiddengameplayoptions:component-38",
        "feature:hiddengameplayoptions:component-40",
    ] {
        let control = evaluation
            .view
            .control(feature_id)
            .unwrap_or_else(|| panic!("missing deferred control {feature_id}"));
        assert_eq!(control.readiness, Readiness::Blocked, "{feature_id}");
        assert!(!control.selected, "{feature_id}");
        assert!(!control.interactive, "{feature_id}");
        assert!(control.unavailable_reason.is_some(), "{feature_id}");
    }
    assert_eq!(
        evaluation.view.control("mod:evandra").unwrap().decision,
        Decision::Default
    );
    let cheat_menu = evaluation
        .view
        .control("feature:hiddengameplayoptions:component-200")
        .unwrap();
    assert_eq!(cheat_menu.readiness, Readiness::Ready);
    assert!(cheat_menu.interactive);
    assert!(!cheat_menu.selected);
}

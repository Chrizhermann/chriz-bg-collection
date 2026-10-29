use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Cursor, Write};
use std::path::PathBuf;

use bg_engine::updates::{
    classify_updates, diff_recipe_packages, ensure_change_coverage, load_release, RecipeChange,
    RecipeDifference, RecipeRelease, SaveApplicability, UpdateDisposition, Urgency,
};
use semver::Version;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Subject kinds that `diff_recipe_packages` emits. A ledger cover of any other kind can
/// never match a recipe difference, so it covers nothing.
const DIFF_SUBJECT_KINDS: [&str; 5] = ["artifact", "feature", "game-profile", "input", "run"];

/// Covers of other kinds in ledgers that are already published. Those ledgers are history
/// and stay unchanged (#7); the entries are harmless, because coverage only requires every
/// real difference to be covered. No new ledger may add one.
const SHIPPED_DEAD_COVERS: [(&str, &str); 4] = [
    ("0.1.0-alpha.1", "release:initial"),
    ("0.1.0-alpha.12", "mod:chriz-sod-remix"),
    ("0.1.0-alpha.12", "release:customization-presentation"),
    ("0.1.0-alpha.13", "mod:chriz-bg-rebalance"),
];

/// Recorded semantic differences of released recipes, as `{version, from_commit,
/// to_commit, subjects}` objects in version order.
fn released_snapshots() -> Vec<serde_json::Value> {
    let snapshots: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/updates/released-recipe-differences.json"
    ))
    .unwrap();
    snapshots.as_array().unwrap().clone()
}

fn snapshot_differences(snapshot: &serde_json::Value) -> Vec<RecipeDifference> {
    snapshot["subjects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|subject| RecipeDifference {
            subject: subject.as_str().unwrap().to_owned(),
            cosmetic_only: false,
        })
        .collect()
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/updates")
        .join(name)
}

fn release(version: &str, supersedes: Option<&str>, app: &str) -> RecipeRelease {
    RecipeRelease {
        schema: 1,
        recipe_id: "chriz-bg-collection".to_owned(),
        version: version.to_owned(),
        published_at: "2026-09-04T00:00:00Z".to_owned(),
        minimum_app_version: app.to_owned(),
        supersedes: supersedes.map(str::to_owned),
        changes: vec![RecipeChange {
            id: format!("change-{}", version.replace('.', "-")),
            title: "A change".to_owned(),
            summary: "Authored guidance.".to_owned(),
            save_applicability: SaveApplicability::NextPlaythrough,
            urgency: Urgency::Informational,
            condition_note: None,
            covers: vec!["feature:example".to_owned()],
        }],
    }
}

fn zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(zip::DateTime::default());
    for (name, bytes) in entries {
        writer.start_file(name, options).unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn loads_strict_ledger_with_all_authored_classifications() {
    let ledger = load_release(&fixture("v0.1.0-alpha.1.toml")).unwrap();
    assert_eq!(ledger.version, "0.1.0-alpha.1");
    assert_eq!(ledger.supersedes.as_deref(), Some("0.1.0-alpha.0"));
    assert_eq!(
        ledger
            .changes
            .iter()
            .map(|change| change.save_applicability)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            SaveApplicability::CurrentSave,
            SaveApplicability::BeforeNpcJoin,
            SaveApplicability::BeforeAreaVisit,
            SaveApplicability::BeforeEvent,
            SaveApplicability::NextPlaythrough,
            SaveApplicability::NewGameOnly,
            SaveApplicability::Unknown,
        ])
    );
}

#[test]
fn ledger_errors_name_path_and_reject_unknown_fields_values_and_duplicates() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("bad.toml");
    let valid = fs::read_to_string(fixture("v0.1.0-alpha.1.toml")).unwrap();
    for broken in [
        format!("{valid}\nunknown_field = true\n"),
        valid.replace("current-save", "invented"),
        valid.replace("id = \"before-npc\"", "id = \"current\""),
    ] {
        fs::write(&path, broken).unwrap();
        let error = load_release(&path).unwrap_err().to_string();
        assert!(error.contains(path.to_str().unwrap()), "{error}");
    }
}

#[test]
fn classifier_preserves_urgency_and_handles_deferred_current_unknown_and_app_prerequisite() {
    let mut next = release("0.1.0-alpha.1", Some("0.1.0-alpha.0"), "0.1.0");
    next.changes[0].urgency = Urgency::Critical;
    assert_eq!(
        classify_updates("0.1.0-alpha.0", "0.1.0", &[next.clone()])
            .unwrap()
            .disposition,
        UpdateDisposition::DeferredForNextPlaythrough
    );
    for (applicability, expected) in [
        (
            SaveApplicability::CurrentSave,
            UpdateDisposition::MayAffectCurrentPlaythrough,
        ),
        (
            SaveApplicability::BeforeNpcJoin,
            UpdateDisposition::MayAffectCurrentPlaythrough,
        ),
        (
            SaveApplicability::BeforeAreaVisit,
            UpdateDisposition::MayAffectCurrentPlaythrough,
        ),
        (
            SaveApplicability::BeforeEvent,
            UpdateDisposition::MayAffectCurrentPlaythrough,
        ),
        (
            SaveApplicability::Unknown,
            UpdateDisposition::UnknownApplicability,
        ),
    ] {
        next.changes[0].save_applicability = applicability;
        next.changes[0].condition_note = applicability
            .needs_guidance_for_test()
            .then(|| "Applies only before the named condition.".to_owned());
        assert_eq!(
            classify_updates("0.1.0-alpha.0", "0.1.0", &[next.clone()])
                .unwrap()
                .disposition,
            expected
        );
    }
    next.minimum_app_version = "0.2.0".to_owned();
    let result = classify_updates("0.1.0-alpha.0", "0.1.0", &[next]).unwrap();
    assert_eq!(result.disposition, UpdateDisposition::AppUpdateRequired);
    assert_eq!(result.minimum_app_version.as_deref(), Some("0.2.0"));
    assert_eq!(result.changes[0].urgency, Urgency::Critical);
}

trait ApplicabilityTestExt {
    fn needs_guidance_for_test(self) -> bool;
}

impl ApplicabilityTestExt for SaveApplicability {
    fn needs_guidance_for_test(self) -> bool {
        matches!(
            self,
            Self::BeforeNpcJoin | Self::BeforeAreaVisit | Self::BeforeEvent
        )
    }
}

#[test]
fn skipped_releases_accumulate_in_chain_order_and_reject_broken_or_duplicate_chains() {
    let first = release("0.1.0-alpha.1", Some("0.1.0-alpha.0"), "0.1.0");
    let second = release("0.1.0-alpha.2", Some("0.1.0-alpha.1"), "0.1.0");
    let result =
        classify_updates("0.1.0-alpha.0", "0.1.0", &[second.clone(), first.clone()]).unwrap();
    assert_eq!(result.changes[0].id, first.changes[0].id);
    assert_eq!(result.changes[1].id, second.changes[0].id);

    let broken = release("0.1.0-alpha.3", Some("0.1.0-alpha.0"), "0.1.0");
    assert!(classify_updates("0.1.0-alpha.1", "0.1.0", &[broken])
        .unwrap_err()
        .to_string()
        .contains("supersedes"));
    let mut duplicate = second;
    duplicate.changes[0].id = first.changes[0].id.clone();
    assert!(classify_updates("0.1.0-alpha.0", "0.1.0", &[first, duplicate]).is_err());
}

#[test]
fn semantic_diff_and_coverage_include_features_artifacts_runs_inputs_and_game_profiles() {
    let before_collection = b"schema=2\ngame_build='2.7'\n[[runs]]\nrun_id='r'\nmod_id='m'\nphase='main'\ncomponents=[0]\nargs=[]\n[[features]]\nid='f'\ntitle='Old'\ndescription='Same'\ncategory='rules'\ndecision='default'\nreadiness='ready'\ninputs=[{kind='boolean',id='i',default=false}]\n";
    let after_collection = b"schema=2\ngame_build='2.7'\n[[runs]]\nrun_id='r'\nmod_id='m'\nphase='main'\ncomponents=[0,1]\nargs=[]\n[[features]]\nid='f'\ntitle='New'\ndescription='Same'\ncategory='rules'\ndecision='default'\nreadiness='ready'\ninputs=[{kind='boolean',id='i',default=true}]\n";
    let before = zip(&[
        ("collection.toml", before_collection),
        ("artifacts/a.toml", b"id='a'\nversion='1'\n"),
        ("game-builds/steam.toml", b"build='1'\n"),
    ]);
    let after = zip(&[
        ("collection.toml", after_collection),
        ("artifacts/a.toml", b"id='a'\nversion='2'\n"),
        ("game-builds/steam.toml", b"build='2'\n"),
    ]);
    let differences = diff_recipe_packages(&before, &after).unwrap();
    assert_eq!(
        differences
            .iter()
            .map(|difference| difference.subject.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "artifact:a",
            "feature:f",
            "game-profile:steam.toml",
            "input:f/i",
            "run:r",
        ])
    );
    assert!(differences
        .iter()
        .find(|difference| difference.subject == "feature:f")
        .is_some_and(|difference| difference.cosmetic_only));
    // This payload pair changes every kind of address, so a new kind fails here until
    // DIFF_SUBJECT_KINDS, which the ledger cover check uses, names it too.
    assert_eq!(
        differences
            .iter()
            .map(|difference| difference.subject.split_once(':').unwrap().0)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(DIFF_SUBJECT_KINDS)
    );

    let mut ledger = release("0.1.0-alpha.1", Some("0.1.0-alpha.0"), "0.1.0");
    ledger.changes[0].covers = differences
        .iter()
        .take(4)
        .map(|difference| difference.subject.clone())
        .collect();
    assert!(ensure_change_coverage(&differences, &ledger).is_err());
    ledger.changes[0].covers = differences
        .iter()
        .map(|difference| difference.subject.clone())
        .collect();
    ensure_change_coverage(&differences, &ledger).unwrap();
}

#[test]
fn no_later_releases_is_up_to_date() {
    let result = classify_updates("0.1.0-alpha.1", "0.1.0", &[]).unwrap();
    assert_eq!(result.disposition, UpdateDisposition::UpToDate);
    assert!(result.changes.is_empty());
}

#[test]
fn alpha14_ledger_covers_armor_and_bardic_update_addresses() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let ledger = load_release(
        &root.join("recipes/curated-full-current/releases/v0.1.0-alpha.14/ledger.toml"),
    )
    .unwrap();
    // Artifact replacement reports both the removed and added identities. The
    // feature diff prefixes the authored ID, which itself starts with feature:.
    let differences = [
        "artifact:klatu-tweaks-1.7.4",
        "run:klatu-armor-thieving-bg2",
        "feature:feature:klatu:component-2150",
        "artifact:bardicwonders-v2.9c-balance.3",
        "artifact:bardicwonders-v2.9c-balance.4",
    ]
    .into_iter()
    .map(|subject| RecipeDifference {
        subject: subject.to_owned(),
        cosmetic_only: false,
    })
    .collect::<Vec<_>>();
    ensure_change_coverage(&differences, &ledger).unwrap();
}

#[test]
fn alpha18_records_emotion_default_and_stronghold_selection_fix_for_new_installs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let ledger = load_release(
        &root.join("recipes/curated-full-current/releases/v0.1.0-alpha.18/ledger.toml"),
    )
    .unwrap();
    let source = load_release(&root.join("manifest/releases/v0.1.0-alpha.18/ledger.toml")).unwrap();
    assert_eq!(ledger, source);
    assert_eq!(ledger.version, "0.1.0-alpha.18");
    assert_eq!(ledger.supersedes.as_deref(), Some("0.1.0-alpha.17"));
    assert_eq!(ledger.minimum_app_version, "0.1.0-alpha.19");
    assert_eq!(ledger.changes.len(), 12);
    let change = &ledger.changes[0];
    assert_eq!(change.id, "emotion-courage-hope-exclusive-default");
    assert_eq!(
        change.save_applicability,
        SaveApplicability::NextPlaythrough
    );
    assert_eq!(change.urgency, Urgency::Recommended);
    let differences = [
        "artifact:chriz-bg-rebalance-0.4.0",
        "artifact:chriz-bg-rebalance-0.6.0",
        "run:chriz-bg-rebalance-bg2",
        "feature:feature:chriz-bg-rebalance:component-301",
    ]
    .into_iter()
    .map(|subject| RecipeDifference {
        subject: subject.to_owned(),
        cosmetic_only: false,
    })
    .collect::<Vec<_>>();
    ensure_change_coverage(&differences, &ledger).unwrap();
    let strongholds = &ledger.changes[1];
    assert_eq!(strongholds.id, "stronghold-selection-exclusions");
    assert_eq!(
        strongholds.save_applicability,
        SaveApplicability::NextPlaythrough
    );
    assert_eq!(strongholds.urgency, Urgency::Recommended);
    let subjects = [1160, 1161]
        .into_iter()
        .chain(1340..=1347)
        .map(|n| format!("feature:feature:cdtweaks:component-{n}"))
        .collect::<Vec<_>>();
    assert_eq!(strongholds.covers, subjects);
    let dependencies = &ledger.changes[2];
    assert_eq!(dependencies.id, "custom-selection-core-dependencies");
    assert_eq!(
        dependencies.save_applicability,
        SaveApplicability::NextPlaythrough
    );
    assert_eq!(dependencies.urgency, Urgency::Recommended);
    let mut dependency_subjects = vec!["feature:mod:stratagems".to_owned()];
    for n in [20, 30, 40, 50, 60, 70]
        .into_iter()
        .flat_map(|start| start..start + 5)
        .chain([110, 112, 113, 114, 131, 241])
    {
        dependency_subjects.push(format!("feature:feature:bg1npc:component-{n}"));
    }
    assert_eq!(dependencies.covers, dependency_subjects);
    for (index, id, subjects) in [
        (
            3,
            "artisan-chriz-1-6-refresh",
            vec![
                "artifact:artisans-kitpack-chriz-v1.5.0",
                "artifact:artisans-kitpack-chriz-v1.6.0",
                "feature:feature:artisanskitpack-tweak:component-20101",
            ],
        ),
        (
            4,
            "bardic-balance-6-refresh",
            vec![
                "artifact:bardicwonders-v2.9c-balance.5",
                "artifact:bardicwonders-v2.9c-balance.6",
            ],
        ),
        (
            5,
            "sod-remix-0-6-12-refresh",
            vec![
                "artifact:chriz-sod-remix-0.6.11",
                "artifact:chriz-sod-remix-0.6.12",
            ],
        ),
        (
            6,
            "buffbot-1-9-0-refresh",
            vec!["artifact:buffbot-1.8.4-alpha", "artifact:buffbot-1.9.0"],
        ),
    ] {
        let change = &ledger.changes[index];
        assert_eq!(change.id, id);
        assert_eq!(
            change.save_applicability,
            SaveApplicability::NextPlaythrough
        );
        assert_eq!(change.urgency, Urgency::Recommended);
        assert_eq!(change.covers, subjects);
        let differences = subjects
            .into_iter()
            .map(|subject| RecipeDifference {
                subject: subject.to_owned(),
                cosmetic_only: false,
            })
            .collect::<Vec<_>>();
        ensure_change_coverage(&differences, &ledger).unwrap();
    }
    let differences = subjects
        .into_iter()
        .map(|subject| RecipeDifference {
            subject,
            cosmetic_only: false,
        })
        .collect::<Vec<_>>();
    ensure_change_coverage(&differences, &ledger).unwrap();
    let summary = classify_updates("0.1.0-alpha.17", "0.1.0-alpha.19", &[ledger]).unwrap();
    assert_eq!(
        summary.disposition,
        UpdateDisposition::DeferredForNextPlaythrough
    );
}

#[test]
fn alpha20_requires_the_feature_group_app_and_waits_for_the_next_playthrough() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let ledger = load_release(
        &root.join("recipes/curated-full-current/releases/v0.1.0-alpha.20/ledger.toml"),
    )
    .unwrap();
    let source = load_release(&root.join("manifest/releases/v0.1.0-alpha.20/ledger.toml")).unwrap();
    assert_eq!(ledger, source);
    assert_eq!(ledger.supersedes.as_deref(), Some("0.1.0-alpha.19"));
    // The recipe adds [[feature_groups]] (Challenge Mode), which alpha.19 apps cannot read.
    let old_app = classify_updates(
        "0.1.0-alpha.19",
        "0.1.0-alpha.19",
        std::slice::from_ref(&ledger),
    )
    .unwrap();
    assert_eq!(old_app.disposition, UpdateDisposition::AppUpdateRequired);
    assert_eq!(
        old_app.minimum_app_version.as_deref(),
        Some("0.1.0-alpha.20")
    );
    let new_app = classify_updates("0.1.0-alpha.19", "0.1.0-alpha.20", &[ledger]).unwrap();
    assert_eq!(new_app.latest_version.as_deref(), Some("0.1.0-alpha.20"));
    assert_eq!(
        new_app.disposition,
        UpdateDisposition::DeferredForNextPlaythrough
    );
}

#[test]
fn alpha22_supersedes_alpha20_requires_the_alpha22_app_and_waits_for_the_next_playthrough() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let curated = root.join("recipes/curated-full-current");
    let ledger = load_release(&curated.join("releases/v0.1.0-alpha.22/ledger.toml")).unwrap();
    let source = load_release(&root.join("manifest/releases/v0.1.0-alpha.22/ledger.toml")).unwrap();
    assert_eq!(ledger, source);
    assert_eq!(ledger.version, "0.1.0-alpha.22");
    assert_eq!(
        bg_engine::cli::recipe_release_version(&curated)
            .unwrap()
            .as_deref(),
        Some("0.1.0-alpha.22")
    );
    // Alpha.21 was an app-only release that shipped the alpha.20 recipe.
    assert!(!curated.join("releases/v0.1.0-alpha.21").exists());
    assert_eq!(ledger.supersedes.as_deref(), Some("0.1.0-alpha.20"));

    // The alpha.21 app cannot use this recipe: its engine rejects Challenge Mode's
    // `variants` and the `config_files` of Tweaks Anthology 3347, and without them its
    // public-alpha validation reports the quest mods' defaults as omitted.
    let old_app = classify_updates(
        "0.1.0-alpha.20",
        "0.1.0-alpha.21",
        std::slice::from_ref(&ledger),
    )
    .unwrap();
    assert_eq!(old_app.disposition, UpdateDisposition::AppUpdateRequired);
    assert_eq!(
        old_app.minimum_app_version.as_deref(),
        Some("0.1.0-alpha.22")
    );
    let new_app = classify_updates(
        "0.1.0-alpha.20",
        "0.1.0-alpha.22",
        std::slice::from_ref(&ledger),
    )
    .unwrap();
    assert_eq!(new_app.latest_version.as_deref(), Some("0.1.0-alpha.22"));
    assert_eq!(
        new_app.disposition,
        UpdateDisposition::DeferredForNextPlaythrough
    );
    assert_eq!(new_app.changes, ledger.changes);
    // Everything in this release is install-time content for new installations.
    assert!(ledger
        .changes
        .iter()
        .all(|change| change.save_applicability == SaveApplicability::NextPlaythrough));

    // An alpha.19 installation that skipped alpha.20 gets both releases once, in order.
    let alpha20 = load_release(&curated.join("releases/v0.1.0-alpha.20/ledger.toml")).unwrap();
    let skipped = classify_updates(
        "0.1.0-alpha.19",
        "0.1.0-alpha.22",
        &[ledger.clone(), alpha20.clone()],
    )
    .unwrap();
    assert_eq!(skipped.latest_version.as_deref(), Some("0.1.0-alpha.22"));
    assert_eq!(
        skipped.disposition,
        UpdateDisposition::DeferredForNextPlaythrough
    );
    assert_eq!(
        skipped.changes,
        [alpha20.changes, ledger.changes.clone()].concat()
    );

    // The ledger covers the recorded alpha.20 -> alpha.22 differences and nothing else.
    // They were taken from payloads that chriz-bg-author package-recipe built from the
    // recipe trees of both commits; e0b5116 is the recipe that app alpha.20 and alpha.21
    // shipped.
    let snapshot = released_snapshots()
        .into_iter()
        .find(|snapshot| snapshot["version"] == "0.1.0-alpha.22")
        .expect("the alpha.22 differences are recorded");
    assert_eq!(snapshot["from_commit"], "e0b5116");
    let differences = snapshot_differences(&snapshot);
    assert_eq!(differences.len(), 229);
    ensure_change_coverage(&differences, &ledger).unwrap();
    assert_eq!(
        ledger
            .changes
            .iter()
            .flat_map(|change| change.covers.iter().map(String::as_str))
            .collect::<BTreeSet<_>>(),
        differences
            .iter()
            .map(|difference| difference.subject.as_str())
            .collect::<BTreeSet<_>>()
    );
}

#[test]
fn coverage_error_names_uncovered_subject() {
    let differences = vec![RecipeDifference {
        subject: "run:missing".to_owned(),
        cosmetic_only: false,
    }];
    let ledger = release("0.1.0-alpha.1", Some("0.1.0-alpha.0"), "0.1.0");
    assert!(ensure_change_coverage(&differences, &ledger)
        .unwrap_err()
        .to_string()
        .contains("run:missing"));
}

#[test]
fn ledger_covers_name_only_subject_kinds_the_recipe_diff_emits() {
    // Authoring check over both profiles. Runtime loading stays tolerant: an installed app
    // must not refuse a published ledger over a cover that merely matches nothing.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let shipped = SHIPPED_DEAD_COVERS
        .iter()
        .map(|(version, cover)| ((*version).to_owned(), (*cover).to_owned()))
        .collect::<BTreeSet<_>>();
    for profile in ["manifest", "recipes/curated-full-current"] {
        let mut ledgers = 0;
        let mut dead = BTreeSet::new();
        for entry in fs::read_dir(root.join(profile).join("releases")).unwrap() {
            let path = entry.unwrap().path().join("ledger.toml");
            if !path.is_file() {
                continue;
            }
            ledgers += 1;
            let ledger = load_release(&path).unwrap();
            for cover in ledger.changes.iter().flat_map(|change| &change.covers) {
                let emitted = cover.split_once(':').is_some_and(|(kind, address)| {
                    DIFF_SUBJECT_KINDS.contains(&kind) && !address.is_empty()
                });
                if !emitted {
                    dead.insert((ledger.version.clone(), cover.clone()));
                }
            }
        }
        // alpha.1, alpha.12 to alpha.20 and alpha.22; published ledgers are never removed.
        assert!(ledgers >= 11, "{profile}: only {ledgers} ledgers found");
        assert_eq!(
            dead, shipped,
            "{profile}: covers that no recipe difference can match (kinds: {DIFF_SUBJECT_KINDS:?})"
        );
    }
}

#[test]
fn released_history_covers_real_differences_and_skipping_versions_does_not_duplicate_changes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    // Captured by comparing the exact recipe Git objects named in the fixture,
    // using diff_recipe_packages' subject rules; not inferred from the ledgers.
    let snapshots = released_snapshots();
    for snapshot in &snapshots {
        let version = snapshot["version"].as_str().unwrap();
        let ledger = load_release(&root.join(format!(
            "recipes/curated-full-current/releases/v{version}/ledger.toml"
        )))
        .unwrap();
        ensure_change_coverage(&snapshot_differences(snapshot), &ledger).unwrap();
    }

    // No release gate can diff against the previous recipe (the app bundles the recipe
    // directory and CI checks out one commit), so this fixture is the coverage gate. Every
    // release from alpha.20 on needs its snapshot, recorded from the previous release's
    // commit; alpha.17 to alpha.19 were never recorded.
    let tracked_from = Version::parse("0.1.0-alpha.20").unwrap();
    let recorded = snapshots
        .iter()
        .map(|snapshot| {
            (
                Version::parse(snapshot["version"].as_str().unwrap()).unwrap(),
                snapshot,
            )
        })
        .filter(|(version, _)| *version >= tracked_from)
        .collect::<BTreeMap<_, _>>();
    let mut released = fs::read_dir(root.join("recipes/curated-full-current/releases"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.join("ledger.toml").is_file())
        .map(|path| load_release(&path.join("ledger.toml")).unwrap().version)
        .map(|version| Version::parse(&version).unwrap())
        .filter(|version| *version >= tracked_from)
        .collect::<Vec<_>>();
    released.sort();
    assert_eq!(
        recorded.keys().cloned().collect::<Vec<_>>(),
        released,
        "every release ledger from alpha.20 on needs its recorded differences"
    );
    for (previous, next) in recorded.values().zip(recorded.values().skip(1)) {
        assert_eq!(
            next["from_commit"], previous["to_commit"],
            "{} must be recorded from the commit that {} shipped",
            next["version"], previous["version"]
        );
    }

    let releases = (14..=18)
        .map(|number| {
            load_release(&root.join(format!(
                "recipes/curated-full-current/releases/v0.1.0-alpha.{number}/ledger.toml"
            )))
            .unwrap()
        })
        .collect::<Vec<_>>();
    let summary = classify_updates("0.1.0-alpha.13", "0.1.0-alpha.19", &releases).unwrap();
    assert_eq!(summary.latest_version.as_deref(), Some("0.1.0-alpha.18"));
    assert_eq!(
        summary.disposition,
        UpdateDisposition::DeferredForNextPlaythrough
    );
    assert_eq!(summary.changes.len(), 30); // September 29 adds pins, UI choices, Modpack and Wing Buffet defaults.
}

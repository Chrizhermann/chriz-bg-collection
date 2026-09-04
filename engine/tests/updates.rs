use std::collections::BTreeSet;
use std::fs;
use std::io::{Cursor, Write};
use std::path::PathBuf;

use bg_engine::updates::{
    classify_updates, diff_recipe_packages, ensure_change_coverage, load_release, RecipeChange,
    RecipeDifference, RecipeRelease, SaveApplicability, UpdateDisposition, Urgency,
};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

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
            .is_conditional_for_test()
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
    fn is_conditional_for_test(self) -> bool;
}

impl ApplicabilityTestExt for SaveApplicability {
    fn is_conditional_for_test(self) -> bool {
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

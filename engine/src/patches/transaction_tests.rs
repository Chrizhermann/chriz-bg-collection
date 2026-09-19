//! Windows production-boundary tests using only disposable synthetic installations.
#![cfg(windows)]

use super::*;
use crate::{
    digest::plan_digest,
    manifest::GameRoot,
    receipt::{
        FinalLogReceipt, FinalReceiptState, InstallReceipt, LogComponentReceipt, ReceiptOutcome,
        ReceiptVersions, RECEIPT_SCHEMA_VERSION,
    },
    recipe_view::NormalizedSelection,
    registry::REGISTRY_SCHEMA_VERSION,
    resolve::InstallPlan,
};
use serde_json::{json, Value};

#[allow(dead_code)]
#[path = "../../tests/support/fakegame.rs"]
mod fakegame;

const AFFECTED: [&str; 5] = [
    "override/bgclatxt.2da",
    "override/clastext.2da",
    "override/sodcltxt.2da",
    "weidu.log",
    "weidu.conf",
];
const ENGINE_NAME: &str = "Patch tests - fixture";
const LOG: &[u8] = b"// Synthetic component evidence\r\n~ArtisansKitpack/ArtisansKitpack.TP2~ #0 #7004 // Assassin: chriz-v1.3.0\r\n";
const KITS: &[u8] = b"2DA V1.0\r\n*\r\nROWNAME LOWER MIXED HELP\r\n0 ASSASIN 10 11 1\r\n";
const CLASSES: &[u8] = b"2DA V1.0\r\n*\r\nLOWER MIXED OTHER DESCSTR EXTRA\r\nASSASSIN 10 11 12 0 unchanged\r\nMAGE 20 21 22 0 unrelated\r\n";

struct Fixture {
    _temp: tempfile::TempDir,
    record: ManagedInstallRecord,
    app_data: PathBuf,
    cache: PathBuf,
    original: BTreeMap<String, Vec<u8>>,
    receipt: Vec<u8>,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("managed");
        fs::create_dir(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let game = root.join("game");
        fakegame::FakeGame::build(&game).unwrap();
        write(
            &game.join("Baldur.exe"),
            b"MZ synthetic marker; never execute",
        );
        write(
            &game.join("engine.lua"),
            format!("engine_name = '{ENGINE_NAME}'\n").as_bytes(),
        );
        write(&game.join("weidu.conf"), b"lang_dir = en_US\r\n");
        write(&game.join("WeiDU.log"), LOG);
        write(&game.join("override/KITLIST.2DA"), KITS);
        for table in description::TABLES {
            write(&game.join("override").join(table), CLASSES);
        }
        let tlk = tiny_tlk();
        write(&game.join("lang/en_US/dialog.tlk"), &tlk);
        write(&game.join("dialog.tlk"), &tlk);
        let save_root = root.join("profile");
        write(
            &save_root.join("save/00001-test/BALDUR.gam"),
            b"synthetic save; immutable",
        );
        let app_data = temp.path().join("app-data");
        fs::create_dir(&app_data).unwrap();
        let cache = temp.path().join("download-cache");
        let install_plan = InstallPlan { runs: Vec::new() };
        let recipe = "0.1.0-alpha.fixture";
        let receipt = InstallReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION,
            install_id: "install-description-test".into(),
            attempt_id: "attempt-complete".into(),
            evidence_attempt_id: "attempt-complete".into(),
            managed_root: root.clone(),
            staged_bg1: root.join("bg1"),
            staged_bg2: game.clone(),
            started_at_millis: 1,
            completed_at_millis: 2,
            outcome: ReceiptOutcome::Succeeded,
            versions: ReceiptVersions {
                application: "0.1.0-alpha.18".into(),
                engine: "0.1.0".into(),
                manifest_schema: 2,
                recipe: recipe.into(),
            },
            source_games: Vec::new(),
            recipe_payload_sha256: "33".repeat(32),
            recipe_envelope_sha256: "44".repeat(32),
            selection_sha256: "55".repeat(32),
            normalized_selection: NormalizedSelection {
                platform: "windows".into(),
                features: BTreeMap::new(),
                inputs: BTreeMap::new(),
            },
            plan_sha256: plan_digest(&install_plan).unwrap(),
            plan: install_plan,
            artifacts: Vec::new(),
            weidu_tools: Vec::new(),
            runs: Vec::new(),
            final_state: Some(FinalReceiptState {
                logs: vec![FinalLogReceipt {
                    target: GameRoot::Bg2,
                    sha256: sha256_bytes(LOG),
                    components: vec![LogComponentReceipt {
                        tp2: "ArtisansKitpack/ArtisansKitpack.TP2".into(),
                        language: 0,
                        component: 7004,
                    }],
                }],
                bg1_engine_name: ENGINE_NAME.into(),
                bg2_engine_name: ENGINE_NAME.into(),
                managed_save_root: save_root.clone(),
                launch_path: game.join("Baldur.exe"),
                verification_summary: "synthetic completed installation".into(),
            }),
        };
        let receipt = serde_json::to_vec_pretty(&receipt).unwrap();
        write(&root.join(".chriz/install-receipt.json"), &receipt);
        let record = ManagedInstallRecord {
            schema_version: REGISTRY_SCHEMA_VERSION,
            install_id: "install-description-test".into(),
            display_name: "Synthetic patch fixture".into(),
            managed_root: root,
            recipe_version: recipe.into(),
            recipe_sha256: "33".repeat(32),
            engine_name: ENGINE_NAME.into(),
            managed_save_root: save_root,
            launch_path: game.join("Baldur.exe"),
            receipt_sha256: sha256_bytes(&receipt),
            completed_at_millis: 2,
        };
        let original = file_bytes(&game);
        Self {
            _temp: temp,
            record,
            app_data,
            cache,
            original,
            receipt,
        }
    }

    fn ctx(&self) -> Context<'_> {
        Context {
            record: &self.record,
            app_data: &self.app_data,
            cache: &self.cache,
        }
    }

    fn game(&self) -> PathBuf {
        self.ctx().game()
    }

    fn assert_original(&self) {
        for (name, bytes) in &self.original {
            assert_eq!(&fs::read(self.game().join(name)).unwrap(), bytes, "{name}");
        }
        assert_eq!(
            fs::read(self.record.managed_root.join(".chriz/install-receipt.json")).unwrap(),
            self.receipt,
            "the base receipt must remain byte-exact"
        );
        assert_eq!(
            fs::read(
                self.record
                    .managed_save_root
                    .join("save/00001-test/BALDUR.gam")
            )
            .unwrap(),
            b"synthetic save; immutable"
        );
    }
}

fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn tiny_tlk() -> Vec<u8> {
    let mut data = vec![0_u8; 18 + 2 * 26];
    data[..8].copy_from_slice(b"TLK V1  ");
    data[10..14].copy_from_slice(&2_u32.to_le_bytes());
    data[14..18].copy_from_slice(&70_u32.to_le_bytes());
    for index in 0..2 {
        let at = 18 + index * 26;
        data[at..at + 2].copy_from_slice(&1_u16.to_le_bytes());
        data[at + 18..at + 22].copy_from_slice(&(index as u32).to_le_bytes());
        data[at + 22..at + 26].copy_from_slice(&1_u32.to_le_bytes());
    }
    data.extend_from_slice(b"AB");
    data
}

fn catalog() -> catalog::Catalog {
    serde_json::from_slice(include_bytes!("../../../patches/catalog.json")).unwrap()
}

fn file_bytes(root: &Path) -> BTreeMap<String, Vec<u8>> {
    files::inventory(root, false)
        .unwrap()
        .into_iter()
        .filter(|(_, stamp)| !stamp.directory)
        .map(|(name, _)| {
            let bytes = fs::read(root.join(&name)).unwrap();
            (name, bytes)
        })
        .collect()
}

/// Creates exactly the same durable pre-write evidence shape as production apply.
/// The production validation and recovery entrypoints remain unchanged.
fn prepared(fixture: &Fixture) -> (PathBuf, Plan, Inventory) {
    let ctx = fixture.ctx();
    let plan = super::plan(&ctx, &catalog()).unwrap();
    let before = files::inventory(&ctx.game(), false).unwrap();
    let dir = ctx.state().join("1000000000000000001");
    let mut backups = BTreeMap::new();
    for name in AFFECTED {
        let bytes = fs::read(ctx.game().join(name)).unwrap();
        write(&dir.join("affected").join(name), &bytes);
        backups.insert(name.to_owned(), sha256_bytes(&bytes));
    }
    let evidence = json!({
        "schema": 1,
        "patch_id": catalog::PATCH_ID,
        "install_id": fixture.record.install_id,
        "managed_root": fixture.record.managed_root,
        "plan": plan,
        "before": before,
        "backups": backups,
    });
    files::json_new(&dir.join("prepared.json"), &evidence).unwrap();
    (dir, plan, before)
}

fn repair_tables(fixture: &Fixture, plan: &Plan) {
    for (name, rows) in &plan.description.expected {
        let text = rows
            .iter()
            .map(|row| row.join(" "))
            .collect::<Vec<_>>()
            .join("\r\n");
        write(
            &fixture.game().join("override").join(name),
            format!("{text}\r\n").as_bytes(),
        );
    }
}

fn publish_applied(fixture: &Fixture, dir: &Path, before: &Inventory) {
    let after = files::inventory(&fixture.game(), false).unwrap();
    let mut hashes = BTreeMap::new();
    let mut created = Vec::new();
    for name in changed(before, &after) {
        assert!(
            allowed(&name),
            "fixture changes must respect the production write set: {name}"
        );
        if !before.contains_key(&name) {
            created.push(name.clone());
        }
        if after.get(&name).is_some_and(|stamp| !stamp.directory) {
            hashes.insert(
                name.clone(),
                files::hash(&fixture.game().join(&name)).unwrap(),
            );
        }
    }
    files::json_new(
        &dir.join("applied.json"),
        &json!({
            "prepared_sha256": files::hash(&dir.join("prepared.json")).unwrap(),
            "post_hashes": hashes,
            "created": created,
            "after": after,
        }),
    )
    .unwrap();
}

fn applied(fixture: &Fixture) -> PathBuf {
    let (dir, plan, before) = prepared(fixture);
    repair_tables(fixture, &plan);
    let mut log = LOG.to_vec();
    log.extend_from_slice(b"~AKCB_KIT_DESCRIPTIONS/setup-AKCB_KIT_DESCRIPTIONS.tp2~ #0 #0 // Description links: 1.0\r\n");
    write(&fixture.game().join("WeiDU.log"), &log);
    // Synthetic fixture bytes are never executed. Evidence records their exact hashes.
    write(
        &fixture
            .game()
            .join(catalog::ADAPTER)
            .join("setup-AKCB_KIT_DESCRIPTIONS.tp2"),
        b"synthetic adapter source",
    );
    write(
        &fixture
            .game()
            .join(catalog::ADAPTER)
            .join("backup/0/BGCLATXT.2DA"),
        CLASSES,
    );
    publish_applied(fixture, &dir, &before);
    dir
}

#[test]
fn available_preview_has_a_stable_review_token_and_does_not_write() {
    let fixture = Fixture::new();
    assert!(verified_log_suffix(&fixture.record.managed_root)
        .unwrap()
        .is_empty());
    let first = inspect(&fixture.ctx(), &catalog()).unwrap();
    assert_eq!(first.state, "available");
    assert_eq!(first.review_token.as_ref().map(String::len), Some(64));
    assert_eq!(first.base_recipe_version, fixture.record.recipe_version);
    assert!(first.applied_patch_ids.is_empty());
    assert!(!first.can_undo && !first.can_restore);
    let second = inspect(&fixture.ctx(), &catalog()).unwrap();
    assert_eq!(first.review_token, second.review_token);
    assert!(!fixture.ctx().state().exists());
    assert!(!fixture.cache.exists());
    assert_eq!(file_bytes(&fixture.game()), fixture.original);
    fixture.assert_original();
}

#[test]
fn changed_protected_file_rejects_apply_before_downloading_or_backing_up() {
    let fixture = Fixture::new();
    let preview = inspect(&fixture.ctx(), &catalog()).unwrap();
    let mut key = fs::read(fixture.game().join("chitin.key")).unwrap();
    key.extend_from_slice(b"changed after review");
    write(&fixture.game().join("chitin.key"), &key);
    let before_apply = file_bytes(&fixture.game());
    let error = apply(
        &fixture.ctx(),
        &catalog(),
        preview.review_token.as_deref().unwrap(),
        true,
        true,
    )
    .unwrap_err();
    assert!(error.contains("changed since"), "{error}");
    assert!(
        !fixture.cache.exists(),
        "no artifact cache or network preparation"
    );
    assert!(
        !fixture.ctx().state().exists(),
        "no transaction preparation"
    );
    assert!(
        !fixture.ctx().backups().exists(),
        "no backups before stale-token rejection"
    );
    assert_eq!(file_bytes(&fixture.game()), before_apply);
}

#[test]
fn unselected_and_already_fixed_installations_offer_no_apply_token() {
    let fixture = Fixture::new();
    write(
        &fixture.game().join("WeiDU.log"),
        b"~ArtisansKitpack/ArtisansKitpack.TP2~ #0 #1 // Other component: chriz-v1.3.0\n",
    );
    let unselected = inspect(&fixture.ctx(), &catalog()).unwrap();
    assert_eq!(unselected.state, "not-selected");
    assert!(unselected.detail.contains("None of the Artisan components"));
    assert!(unselected.review_token.is_none());
    write(&fixture.game().join("WeiDU.log"), LOG);
    let plan = super::plan(&fixture.ctx(), &catalog()).unwrap();
    repair_tables(&fixture, &plan);
    let fixed = inspect(&fixture.ctx(), &catalog()).unwrap();
    assert_eq!(fixed.state, "already-fixed");
    assert!(fixed.review_token.is_none());
    assert!(!fixture.ctx().state().exists());
}

#[test]
fn durable_prepared_transaction_blocks_launch_and_offers_recovery() {
    let fixture = Fixture::new();
    prepared(&fixture);
    assert!(launch_guard(&fixture.record.managed_root).is_err());
    let preview = inspect(&fixture.ctx(), &catalog()).unwrap();
    assert_eq!(preview.state, "needs-recovery");
    assert!(preview.can_restore);
    assert!(preview.review_token.is_none());
    fixture.assert_original();
}

#[test]
fn interrupted_preparation_before_the_durable_barrier_does_not_block_launch() {
    let fixture = Fixture::new();
    let pending = fixture
        .ctx()
        .state()
        .join("1000000000000000001/prepared.pending");
    write(&pending, b"{ incomplete pre-write evidence");
    assert!(launch_guard(&fixture.record.managed_root).is_ok());
    let preview = inspect(&fixture.ctx(), &catalog()).unwrap();
    assert_eq!(preview.state, "available");
    assert!(preview.review_token.is_some());
    fixture.assert_original();
}

#[test]
fn recovery_restores_a_missing_table_and_truncated_configuration() {
    let fixture = Fixture::new();
    let (dir, _, _) = prepared(&fixture);
    fs::remove_file(fixture.game().join("override/BGCLATXT.2DA")).unwrap();
    write(&fixture.game().join("weidu.conf"), b"lang_");
    let preview = inspect(&fixture.ctx(), &catalog()).unwrap();
    assert_eq!(preview.state, "needs-recovery");
    assert!(preview.can_restore);
    let restored = restore(&fixture.ctx(), &catalog(), false).unwrap();
    assert_eq!(restored.state, "available");
    assert!(dir.join("restored.json").is_file());
    assert!(launch_guard(&fixture.record.managed_root).is_ok());
    fixture.assert_original();
}

#[test]
fn recovery_never_deletes_an_unknown_file_added_under_the_adapter() {
    let fixture = Fixture::new();
    let (dir, _, _) = prepared(&fixture);
    let unknown = fixture
        .game()
        .join(catalog::ADAPTER)
        .join("user-added-note.txt");
    write(&unknown, b"user content must survive");
    let result = restore(&fixture.ctx(), &catalog(), false);
    assert!(
        result.is_err(),
        "unknown adapter contents must block automatic deletion"
    );
    assert_eq!(fs::read(&unknown).unwrap(), b"user content must survive");
    assert!(!dir.join("restored.json").exists());
    fixture.assert_original();
}

#[test]
fn undo_restores_original_bytes_and_preserves_unrelated_runtime_files() {
    let fixture = Fixture::new();
    let dir = applied(&fixture);
    let runtime = fixture.game().join("runtime-note.txt");
    write(&runtime, b"runtime-created unrelated data");
    let preview = inspect(&fixture.ctx(), &catalog()).unwrap();
    assert_eq!(preview.state, "applied");
    assert_eq!(preview.applied_patch_ids, [catalog::PATCH_ID]);
    assert!(preview.can_undo);
    assert!(launch_guard(&fixture.record.managed_root).is_ok());
    assert_eq!(
        verified_log_suffix(&fixture.record.managed_root).unwrap(),
        vec![LogComponentReceipt {
            tp2: "akcb_kit_descriptions/setup-akcb_kit_descriptions.tp2".into(),
            language: 0,
            component: 0,
        }]
    );
    assert_eq!(
        restore(&fixture.ctx(), &catalog(), true).unwrap().state,
        "available"
    );
    assert!(dir.join("restored.json").is_file());
    assert!(verified_log_suffix(&fixture.record.managed_root)
        .unwrap()
        .is_empty());
    assert!(!fixture.game().join(catalog::ADAPTER).exists());
    assert_eq!(
        fs::read(runtime).unwrap(),
        b"runtime-created unrelated data"
    );
    fixture.assert_original();
}

#[test]
fn interrupted_undo_is_retryable_after_partial_restore_and_runtime_writes() {
    let fixture = Fixture::new();
    let dir = applied(&fixture);
    let runtime = fixture.game().join("runtime-note.txt");
    write(&runtime, b"unrelated runtime data");
    let before = files::inventory(&fixture.game(), false).unwrap();
    let removable: BTreeMap<String, Option<String>> = before
        .iter()
        .filter(|(name, _)| adapter_scope(name) && allowed(name))
        .map(|(name, stamp)| {
            let hash = if stamp.directory {
                None
            } else {
                Some(files::hash(&fixture.game().join(name)).unwrap())
            };
            (name.clone(), hash)
        })
        .collect();
    files::json_new(
        &dir.join("restoring.json"),
        &json!({
            "prepared_sha256": files::hash(&dir.join("prepared.json")).unwrap(),
            "before": before,
            "removable": removable,
        }),
    )
    .unwrap();
    // Simulate a crash after one original table was restored and one generated file removed.
    let original = fs::read(dir.join("affected/override/bgclatxt.2da")).unwrap();
    write(&fixture.game().join("override/BGCLATXT.2DA"), &original);
    fs::remove_file(
        fixture
            .game()
            .join(catalog::ADAPTER)
            .join("backup/0/BGCLATXT.2DA"),
    )
    .unwrap();
    assert!(launch_guard(&fixture.record.managed_root).is_err());
    let preview = inspect(&fixture.ctx(), &catalog()).unwrap();
    assert_eq!(preview.state, "needs-recovery");
    assert!(
        preview.can_restore,
        "partial undo must retain a usable recovery path"
    );
    let restored = restore(&fixture.ctx(), &catalog(), false).unwrap();
    assert_eq!(restored.state, "available");
    assert_eq!(fs::read(runtime).unwrap(), b"unrelated runtime data");
    assert!(launch_guard(&fixture.record.managed_root).is_ok());
    fixture.assert_original();
}

#[test]
fn verified_log_suffix_rejects_an_extra_unreceipted_component() {
    let fixture = Fixture::new();
    applied(&fixture);
    assert_eq!(
        verified_log_suffix(&fixture.record.managed_root)
            .unwrap()
            .len(),
        1
    );
    let log = fixture.game().join("WeiDU.log");
    let mut bytes = fs::read(&log).unwrap();
    bytes.extend_from_slice(b"~OtherMod/setup.tp2~ #0 #10 // Later unrelated component: v1\n");
    write(&log, &bytes);
    let before = file_bytes(&fixture.game());
    assert!(verified_log_suffix(&fixture.record.managed_root).is_err());
    assert_eq!(file_bytes(&fixture.game()), before);
}

#[test]
fn corrupted_backup_or_prepared_hash_link_blocks_automatic_undo() {
    let fixture = Fixture::new();
    let dir = applied(&fixture);
    write(&dir.join("affected/weidu.log"), b"corrupted backup");
    let before = file_bytes(&fixture.game());
    assert!(restore(&fixture.ctx(), &catalog(), true).is_err());
    assert_eq!(file_bytes(&fixture.game()), before);
    let mut prepared: Value =
        serde_json::from_slice(&fs::read(dir.join("prepared.json")).unwrap()).unwrap();
    prepared["schema"] = json!(2);
    write(
        &dir.join("prepared.json"),
        &serde_json::to_vec(&prepared).unwrap(),
    );
    assert!(launch_guard(&fixture.record.managed_root).is_err());
    assert!(restore(&fixture.ctx(), &catalog(), true).is_err());
    assert_eq!(file_bytes(&fixture.game()), before);
}

/// Explicit local opt-in only. Both inputs must match the production pins; the tool
/// runs only against this disposable fake game, never against the supplied tool directory.
#[test]
#[ignore = "requires CEBG_PILOT_WEIDU and CEBG_PILOT_ADAPTER; no downloads"]
fn pinned_real_weidu_repairs_the_synthetic_game_and_production_undo_restores_it() {
    let tool_source =
        PathBuf::from(std::env::var_os("CEBG_PILOT_WEIDU").expect("explicit tool path"));
    let adapter_source =
        PathBuf::from(std::env::var_os("CEBG_PILOT_ADAPTER").expect("explicit adapter path"));
    crate::weidu::invocation::VerifiedWeidu::verify(&tool_source, catalog::TOOL_HASH).unwrap();
    let fixture = Fixture::new();
    // WeiDU may parse a historical installer's metadata when regenerating its log.
    // This authored stub declares only the synthetic component; it is never installed.
    write(
        &fixture.game().join("ArtisansKitpack/ArtisansKitpack.TP2"),
        b"BACKUP ~ArtisansKitpack/backup~\nAUTHOR ~Synthetic test fixture~\nVERSION ~chriz-v1.3.0~\nBEGIN ~Assassin~ DESIGNATED 7004\nPRINT ~Synthetic metadata only~\n",
    );
    let (dir, plan, _) = prepared(&fixture);
    let tool = dir.join("weidu.exe");
    write(&tool, &fs::read(tool_source).unwrap());
    for (name, length, hash) in catalog::FILES {
        let bytes = files::read(&adapter_source.join(name), 65536).unwrap();
        assert_eq!(bytes.len() as u64, length, "{name}");
        assert_eq!(sha256_bytes(&bytes), hash, "{name}");
        write(&dir.join("payload").join(name), &bytes);
    }
    transaction::run_prepared_fixture(&fixture.ctx(), &dir, &tool).unwrap_or_else(|error| {
        panic!(
            "Production apply failed: {error}\n{}",
            fs::read_to_string(dir.join("output.log")).unwrap_or_default()
        )
    });
    assert!(dir.join("applied.json").is_file());
    description::verify(&plan.description, &tables(&fixture.game()).unwrap()).unwrap();
    let protected = plan
        .hashes
        .iter()
        .filter(|(name, _)| !writable(&name.to_ascii_lowercase()))
        .map(|(name, hash)| (name.clone(), hash.clone()))
        .collect();
    unchanged_hashes(&fixture.game(), &protected).unwrap();
    assert_eq!(
        verified_log_suffix(&fixture.record.managed_root)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        inspect(&fixture.ctx(), &catalog()).unwrap().state,
        "applied"
    );
    restore(&fixture.ctx(), &catalog(), true).unwrap();
    fixture.assert_original();
}

/// End-to-end public entrypoints: real pinned downloads, independent backups,
/// hidden WeiDU, receipt overlay, no-op repeat and Undo. No installed game input.
#[test]
#[ignore = "explicit opt-in network test on a disposable synthetic game"]
fn downloaded_adapter_applies_and_undoes_with_independent_backups() {
    assert_eq!(
        std::env::var("CEBG_PATCH_DOWNLOAD_TEST").as_deref(),
        Ok("1")
    );
    let fixture = Fixture::new();
    write(&fixture.game().join("ArtisansKitpack/ArtisansKitpack.TP2"),
        b"BACKUP ~ArtisansKitpack/backup~\nAUTHOR ~Synthetic test fixture~\nVERSION ~chriz-v1.3.0~\nBEGIN ~Assassin~ DESIGNATED 7004\nPRINT ~Synthetic metadata only~\n");
    let preview = inspect(&fixture.ctx(), &catalog()).unwrap();
    let result = apply(
        &fixture.ctx(),
        &catalog(),
        preview.review_token.as_deref().unwrap(),
        true,
        true,
    );
    let applied = result.unwrap_or_else(|e| {
        let mut logs = String::new();
        for entry in walkdir::WalkDir::new(fixture.ctx().state())
            .into_iter()
            .flatten()
        {
            if entry.file_name() == "output.log" {
                logs.push_str(&fs::read_to_string(entry.path()).unwrap_or_default());
            }
        }
        panic!("Public apply failed: {e}\n{logs}")
    });
    assert_eq!(applied.state, "applied");
    assert!(applied.can_undo);
    assert_eq!(
        verified_log_suffix(&fixture.record.managed_root)
            .unwrap()
            .len(),
        1
    );
    let backups = fs::read_dir(fixture.ctx().backups())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(backups.len(), 1);
    assert_eq!(
        fs::read(backups[0].join("game/override/bgclatxt.2da")).unwrap(),
        CLASSES
    );
    assert_eq!(
        fs::read(backups[0].join("profile/save/00001-test/BALDUR.gam")).unwrap(),
        b"synthetic save; immutable"
    );
    let repeat = apply(
        &fixture.ctx(),
        &catalog(),
        preview.review_token.as_deref().unwrap(),
        true,
        true,
    )
    .unwrap();
    assert_eq!(repeat.state, "applied");
    assert_eq!(fs::read_dir(fixture.ctx().state()).unwrap().count(), 1);
    restore(&fixture.ctx(), &catalog(), true).unwrap();
    assert!(verified_log_suffix(&fixture.record.managed_root)
        .unwrap()
        .is_empty());
    fixture.assert_original();
}

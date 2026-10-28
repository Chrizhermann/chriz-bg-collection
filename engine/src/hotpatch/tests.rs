//! Hotpatch records on disposable synthetic installations. Nothing is downloaded and no
//! installer runs: the tests write what WeiDU would write, then use the production checks,
//! records and restoring.
use super::catalog::{self, Catalog};
use super::transaction::{self, History, Prepared, Sources, State};
use super::*;
use crate::{
    cli::VerifiedInstalledBaseline,
    digest::plan_digest,
    game_files,
    manifest::GameRoot,
    receipt::{
        FinalLogReceipt, FinalReceiptState, InstallReceipt, LogComponentReceipt, ReceiptOutcome,
        ReceiptVersions, RECEIPT_SCHEMA_VERSION,
    },
    recipe_view::evaluate_preset,
    registry::REGISTRY_SCHEMA_VERSION,
};
use std::collections::BTreeMap;
use std::fs;

const ENGINE_NAME: &str = "Hotpatch tests - fixture";
const BASE_LOG: &[u8] =
    b"// Synthetic mod list\r\n~STRATAGEMS/SETUP-STRATAGEMS.TP2~ #0 #1000 // Fixture: v1\r\n";
const ROW: &[u8] =
    b"~CHRIZ_IWD_ENCOUNTERS/SETUP-CHRIZ_IWD_ENCOUNTERS.TP2~ #0 #10 // Belhifet: v0.2.0\r\n";
const SAVE: &str = "save/000000001-Quick-Save/ICEWIND.gam";
const OLD_CREATURE: &[u8] = b"CRE V9.0 synthetic Belhifet before the hotpatch";
const NEW_CREATURE: &[u8] = b"CRE V9.0 synthetic Belhifet after the hotpatch";

const CATALOG: &str = r#"
schema = 1
game = "iwdee"
runtime_files = ["infinityloader.ini", "buffbot_*.log"]

[gate]
recommended_preset = "chris-recommended"
max_changed_mods = 2
free_mods = ["cdtweaks"]
required_mods = ["stratagems"]

[[bases]]
id = "iwd-fixture"
label = "Chriz Easy IWD fixture"
recipe_version = "0.1.0-alpha.fixture"
content_sha256 = "@DIGEST@"

[[hotpatches]]
id = "belhifet-1"
title = "Belhifet fight"
summary = "A harder, fairer Belhifet."
bases = ["iwd-fixture"]
artifact = "chriz-iwd-encounters-0.2.0"
mod = "chriz-iwd-encounters"
components = [10]

[[hotpatches.changes]]
title = "Belhifet"
summary = "Belhifet fights with new tactics."
save_applicability = "before-area-visit"
condition_note = "Saves made before the last fight get it."
"#;

const ARTIFACT: &str = r#"
id = "chriz-iwd-encounters-0.2.0"
name = "Chriz IWD Encounters"
version = "0.2.0"
acquisition = "fetch-only"

[source]
kind = "github-release"
url = "https://example.invalid/chriz-iwd-encounters-v0.2.0.zip"
reference = "v0.2.0"
expected_filename = "chriz-iwd-encounters-v0.2.0.zip"
expected_length = 1234
sha256 = "6666666666666666666666666666666666666666666666666666666666666666"
redirect_hosts = []

[archive]
kind = "zip"
root_rule = "direct"
publish_roots = ["chriz_iwd_encounters"]
tp2_paths = ["chriz_iwd_encounters/setup-chriz_iwd_encounters.tp2"]

[archive.limits]
max_depth = 8
max_entries = 256
max_entry_uncompressed_bytes = 8388608
max_total_uncompressed_bytes = 33554432
max_compression_ratio = 100

[provenance]
homepage = "https://example.invalid/"
license = "GPL-3.0"
url = "https://example.invalid/"
reviewed_on = "2026-10-28"
"#;

const INSTALLER: &str = r#"
id = "chriz-iwd-encounters"
artifact_id = "chriz-iwd-encounters-0.2.0"
name = "Chriz IWD Encounters"
tp2 = "chriz_iwd_encounters/setup-chriz_iwd_encounters.tp2"
language = 0
weidu_artifact_id = "weidu-249-amd64"
invocation_mode = "setup-name"

[[components]]
id = 10
name = "Belhifet"
"#;

struct Fixture {
    temp: tempfile::TempDir,
    record: ManagedInstallRecord,
    receipt: InstallReceipt,
    receipt_bytes: Vec<u8>,
    app_data: PathBuf,
    locks: crate::lock::TargetLockNamespace,
    cache: PathBuf,
    catalog: Catalog,
    package: PathBuf,
    tool: PathBuf,
    /// The finished game folder.
    original: game_files::GameFiles,
}

fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

/// A valid TLK with `entries` strings, as the close checks read it.
fn tlk(entries: u32) -> Vec<u8> {
    let header = 18 + 26 * entries as usize;
    let mut data = vec![0_u8; header];
    data[..8].copy_from_slice(b"TLK V1  ");
    data[10..14].copy_from_slice(&entries.to_le_bytes());
    data[14..18].copy_from_slice(&(header as u32).to_le_bytes());
    for index in 0..entries as usize {
        let at = 18 + index * 26;
        data[at..at + 2].copy_from_slice(&1_u16.to_le_bytes());
        data[at + 18..at + 22].copy_from_slice(&(index as u32).to_le_bytes());
        data[at + 22..at + 26].copy_from_slice(&1_u32.to_le_bytes());
    }
    data.extend((0..entries).map(|index| b'A' + index as u8));
    data
}

fn recipe() -> crate::Manifest {
    crate::Manifest::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../recipes/iwd-current"))
        .unwrap()
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("managed");
        fs::create_dir(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let game = root.join("game");
        write(
            &game.join("InfinityLoader.exe"),
            b"MZ synthetic marker; never execute",
        );
        write(
            &game.join("engine.lua"),
            format!("engine_name = '{ENGINE_NAME}'\n").as_bytes(),
        );
        write(&game.join("weidu.conf"), b"lang_dir = en_US\r\n");
        write(&game.join("WeiDU.log"), BASE_LOG);
        write(&game.join("chitin.key"), b"KEY V1  synthetic");
        write(
            &game.join("InfinityLoader.ini"),
            b"[General]\r\nLogging=0\r\n",
        );
        write(&game.join("lang/en_US/dialog.tlk"), &tlk(2));
        write(&game.join("override/BELHIFET.CRE"), OLD_CREATURE);
        write(&game.join("override/other.itm"), b"ITM V1   unrelated");
        write(&game.join("data/AREA.BIF"), b"BIFFV1  synthetic");
        fs::create_dir_all(game.join("stratagems/backup/1000")).unwrap();
        let save_root = root.join("profile");
        write(&save_root.join(SAVE), b"synthetic save; never changed");

        let (_, evaluation) = recommended();
        let receipt = InstallReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION,
            install_id: "install-hotpatch-test".into(),
            attempt_id: "attempt-complete".into(),
            evidence_attempt_id: "attempt-complete".into(),
            managed_root: root.clone(),
            staged_bg1: None,
            staged_bg2: game.clone(),
            started_at_millis: 1,
            completed_at_millis: 2,
            outcome: ReceiptOutcome::Succeeded,
            versions: ReceiptVersions {
                application: "0.1.0-alpha.1".into(),
                engine: "0.1.0".into(),
                manifest_schema: 2,
                recipe: "0.1.0-alpha.fixture".into(),
            },
            source_games: Vec::new(),
            recipe_payload_sha256: "33".repeat(32),
            recipe_envelope_sha256: "44".repeat(32),
            selection_sha256: "55".repeat(32),
            normalized_selection: evaluation.normalized_selection,
            plan_sha256: plan_digest(&evaluation.plan).unwrap(),
            plan: evaluation.plan,
            artifacts: Vec::new(),
            weidu_tools: Vec::new(),
            runs: Vec::new(),
            final_state: Some(FinalReceiptState {
                logs: vec![FinalLogReceipt {
                    target: GameRoot::Bg2,
                    sha256: sha256_bytes(BASE_LOG),
                    components: vec![LogComponentReceipt {
                        tp2: "stratagems/setup-stratagems.tp2".into(),
                        language: 0,
                        component: 1000,
                    }],
                }],
                bg1_engine_name: None,
                bg2_engine_name: ENGINE_NAME.into(),
                managed_save_root: save_root.clone(),
                launch_path: game.join("InfinityLoader.exe"),
                verification_summary: "synthetic completed installation".into(),
            }),
        };
        let receipt_bytes = serde_json::to_vec_pretty(&receipt).unwrap();
        write(&root.join(".chriz/install-receipt.json"), &receipt_bytes);
        let original = game_files::capture(&game).unwrap();
        write(
            &root.join(game_files::COMPLETED_RECORD),
            &serde_json::to_vec(&original).unwrap(),
        );
        let record = ManagedInstallRecord {
            schema_version: REGISTRY_SCHEMA_VERSION,
            install_id: "install-hotpatch-test".into(),
            display_name: "Synthetic hotpatch fixture".into(),
            managed_root: root,
            recipe_version: "0.1.0-alpha.fixture".into(),
            recipe_sha256: "33".repeat(32),
            engine_name: ENGINE_NAME.into(),
            managed_save_root: save_root,
            launch_path: game.join("InfinityLoader.exe"),
            receipt_sha256: sha256_bytes(&receipt_bytes),
            completed_at_millis: 2,
            recipe_id: None,
        };

        let catalog_dir = temp.path().join("catalog");
        write(
            &catalog_dir.join("catalog.toml"),
            CATALOG
                .replace("@DIGEST@", &recipe_content_digest(&recipe()).unwrap())
                .as_bytes(),
        );
        write(
            &catalog_dir.join("artifacts/chriz-iwd-encounters-0.2.0.toml"),
            ARTIFACT.as_bytes(),
        );
        write(
            &catalog_dir.join("mods/chriz-iwd-encounters.toml"),
            INSTALLER.as_bytes(),
        );
        let catalog = catalog::load(&catalog_dir).unwrap();

        let package = temp.path().join("package");
        write(
            &package.join("chriz_iwd_encounters/setup-chriz_iwd_encounters.tp2"),
            b"BACKUP ~chriz_iwd_encounters/backup~\r\nBEGIN ~Belhifet~ DESIGNATED 10\r\n",
        );
        write(
            &package.join("chriz_iwd_encounters/cre/belhifet.cre"),
            NEW_CREATURE,
        );
        let tool = temp.path().join("tool/weidu.exe");
        write(&tool, b"synthetic stand-in; never executed");
        let app_data = temp.path().join("app-data");
        fs::create_dir(&app_data).unwrap();
        let locks =
            crate::lock::TargetLockNamespace::in_local_application_data(temp.path().join("local"))
                .unwrap();
        Self {
            cache: temp.path().join("cache"),
            temp,
            record,
            receipt,
            receipt_bytes,
            app_data,
            locks,
            catalog,
            package,
            tool,
            original,
        }
    }

    fn ctx(&self) -> Context<'_> {
        Context {
            record: &self.record,
            app_data: &self.app_data,
            locks: &self.locks,
            cache: &self.cache,
        }
    }

    fn root(&self) -> &Path {
        &self.record.managed_root
    }

    fn game(&self) -> PathBuf {
        self.ctx().game()
    }

    fn history(&self) -> History {
        History::load(self.root(), &self.record.install_id).unwrap()
    }

    /// Prepare the next transaction as apply does, after its checks and backups.
    fn prepare(&self) -> (PathBuf, Prepared) {
        let ctx = self.ctx();
        let history = self.history();
        // Apply names records by time; any later name than the existing folders will do.
        let existing = fs::read_dir(ctx.state()).map_or(0, |folders| folders.count());
        let dir = ctx.state().join(format!("{:020}", existing + 1));
        let payload = [
            "chriz_iwd_encounters/setup-chriz_iwd_encounters.tp2",
            "chriz_iwd_encounters/cre/belhifet.cre",
        ]
        .into_iter()
        .map(|name| (name.to_owned(), self.package.join(name)))
        .collect();
        let sources = Sources {
            payload,
            tool: self.tool.clone(),
            tool_sha256: files::hash(&self.tool).unwrap(),
        };
        files::create(&dir.join("identity"), self.record.install_id.as_bytes()).unwrap();
        let prepared = transaction::prepare_record(
            &ctx,
            &self.catalog,
            &dir,
            &self.catalog.hotpatches[0],
            "en_US".into(),
            &game_files::capture(&self.game()).unwrap(),
            &sources,
            history.head(),
        )
        .unwrap();
        (dir, prepared)
    }

    /// What WeiDU writes for the Belhifet component: its backup of the creature it replaces,
    /// the new creature, a new spell, two strings and its row in the mod list.
    fn weidu(&self, dir: &Path, prepared: &Prepared) {
        let game = self.game();
        transaction::place_payload(&self.ctx(), dir, prepared).unwrap();
        write(
            &game.join("chriz_iwd_encounters/backup/10/BELHIFET.CRE"),
            OLD_CREATURE,
        );
        write(
            &game.join("chriz_iwd_encounters/backup/10/UNINSTALL.10"),
            b"override/BELHIFET.CRE\r\n",
        );
        write(&game.join("override/BELHIFET.CRE"), NEW_CREATURE);
        write(&game.join("override/BELHIF1.SPL"), b"SPL V1  new spell");
        write(&game.join("lang/en_US/dialog.tlk"), &tlk(4));
        let mut log = fs::read(game.join("WeiDU.log")).unwrap();
        log.extend_from_slice(ROW);
        write(&game.join("WeiDU.log"), &log);
    }

    fn apply(&self) -> PathBuf {
        let (dir, prepared) = self.prepare();
        self.weidu(&dir, &prepared);
        transaction::record_applied(&self.ctx(), &self.catalog, &dir, &prepared).unwrap();
        dir
    }

    fn now(&self) -> game_files::GameFiles {
        game_files::capture(&self.game()).unwrap()
    }

    /// The game folder is the finished one again, apart from `runtime` names; the receipt and
    /// the save never changed.
    fn assert_original(&self) {
        let left = self
            .original
            .differences(&self.now(), |name| self.catalog.runtime_files.matches(name));
        assert!(left.is_empty(), "{left:?}");
        self.assert_untouched();
    }

    fn assert_untouched(&self) {
        assert_eq!(
            fs::read(self.root().join(".chriz/install-receipt.json")).unwrap(),
            self.receipt_bytes
        );
        assert_eq!(
            fs::read(self.record.managed_save_root.join(SAVE)).unwrap(),
            b"synthetic save; never changed"
        );
    }

    fn baseline(&self) -> VerifiedInstalledBaseline {
        VerifiedInstalledBaseline {
            manifest: recipe(),
            receipt: self.receipt.clone(),
            replacements: BTreeMap::new(),
            recovered: false,
            description_fix: false,
            upgraded: false,
        }
    }

    fn assess(&self, baseline: VerifiedInstalledBaseline) -> gate::Assessment {
        gate::assess_with(&self.ctx(), &self.catalog, &self.history(), Ok(baseline))
    }
}

fn recommended() -> (crate::Manifest, crate::recipe_view::SelectionEvaluation) {
    let manifest = recipe();
    let evaluation = evaluate_preset(&manifest, "chris-recommended", "windows").unwrap();
    (manifest, evaluation)
}

fn row() -> LogComponentReceipt {
    LogComponentReceipt {
        tp2: "chriz_iwd_encounters/setup-chriz_iwd_encounters.tp2".into(),
        language: 0,
        component: 10,
    }
}

#[test]
fn an_applied_hotpatch_is_recorded_and_undo_puts_back_every_byte() {
    let f = Fixture::new();
    let dir = f.apply();
    let history = f.history();
    assert_eq!(history.entries.len(), 1);
    assert_eq!(history.entries[0].state, State::Applied);
    assert_eq!(history.applied_ids(), ["belhifet-1"]);
    assert!(history.undo_target().is_some());
    assert_eq!(
        history.expected_state(f.root()).unwrap(),
        f.now(),
        "the applied record is the game folder as the hotpatch left it"
    );
    assert_eq!(verified_log_suffix(f.root()).unwrap(), [row()]);
    launch_guard(f.root()).unwrap();
    // The overwritten creature is kept in the record itself, not only in WeiDU's backup.
    assert!(dir
        .join("preimages")
        .join(sha256_bytes(OLD_CREATURE))
        .is_file());
    f.assert_untouched();

    let preview = restore(&f.ctx(), &f.catalog, true).unwrap();
    assert!(preview.applied.is_empty(), "{preview:?}");
    f.assert_original();
    let history = f.history();
    assert_eq!(history.entries[0].state, State::Restored);
    assert!(history.applied_ids().is_empty());
    assert!(history.undo_target().is_none());
    assert!(verified_log_suffix(f.root()).unwrap().is_empty());
    assert_eq!(history.expected_state(f.root()).unwrap(), f.original);
}

#[test]
fn a_hotpatch_can_be_applied_again_after_undo() {
    let f = Fixture::new();
    f.apply();
    restore(&f.ctx(), &f.catalog, true).unwrap();
    f.apply();
    let history = f.history();
    assert_eq!(
        history
            .entries
            .iter()
            .map(|entry| entry.state)
            .collect::<Vec<_>>(),
        [State::Restored, State::Applied]
    );
    assert_eq!(verified_log_suffix(f.root()).unwrap(), [row()]);
    restore(&f.ctx(), &f.catalog, true).unwrap();
    f.assert_original();
}

#[test]
fn an_interrupted_hotpatch_blocks_play_until_it_is_restored() {
    let f = Fixture::new();
    let (dir, prepared) = f.prepare();
    // WeiDU stopped half-way: the package and the creature are there, the mod list is not.
    transaction::place_payload(&f.ctx(), &dir, &prepared).unwrap();
    write(
        &f.game().join("chriz_iwd_encounters/backup/10/BELHIFET.CRE"),
        OLD_CREATURE,
    );
    write(&f.game().join("override/BELHIFET.CRE"), NEW_CREATURE);
    assert!(launch_guard(f.root()).is_err());
    assert!(verified_log_suffix(f.root()).is_err());
    let preview = inspect(&f.ctx(), &f.catalog).unwrap();
    assert_eq!(preview.state, "needs-recovery");
    assert!(preview.can_restore);
    assert!(
        restore(&f.ctx(), &f.catalog, true).is_err(),
        "there is nothing to undo yet"
    );

    restore(&f.ctx(), &f.catalog, false).unwrap();
    f.assert_original();
    launch_guard(f.root()).unwrap();
    assert_eq!(f.history().entries[0].state, State::Restored);
}

#[test]
fn a_preparation_without_its_record_is_ignored() {
    let f = Fixture::new();
    let dir = f.ctx().state().join(format!("{:020}", 1));
    // A download failed after the folder was made: no game file changed.
    files::create(&dir.join("identity"), f.record.install_id.as_bytes()).unwrap();
    assert!(f.history().entries.is_empty());
    launch_guard(f.root()).unwrap();
    f.apply();
    assert_eq!(f.history().applied_ids(), ["belhifet-1"]);
}

#[test]
fn changes_outside_the_hotpatch_area_fail_and_are_left_for_a_closer_look() {
    let f = Fixture::new();
    let (dir, prepared) = f.prepare();
    f.weidu(&dir, &prepared);
    write(
        &f.game().join("data/NEW.BIF"),
        b"BIFFV1  biffed by a package",
    );
    let error = transaction::record_applied(&f.ctx(), &f.catalog, &dir, &prepared).unwrap_err();
    assert!(error.contains("outside its area"), "{error}");
    assert!(error.contains("data/new.bif"), "{error}");
    let error = restore(&f.ctx(), &f.catalog, false).unwrap_err();
    assert!(error.contains("closer look"), "{error}");
    assert!(launch_guard(f.root()).is_err());
    f.assert_untouched();
}

#[test]
fn a_changed_key_file_is_outside_the_area_but_can_be_put_back() {
    let f = Fixture::new();
    let (dir, prepared) = f.prepare();
    f.weidu(&dir, &prepared);
    write(
        &f.game().join("chitin.key"),
        b"KEY V1  changed by a package",
    );
    let error = transaction::record_applied(&f.ctx(), &f.catalog, &dir, &prepared).unwrap_err();
    assert!(error.contains("chitin.key"), "{error}");
    restore(&f.ctx(), &f.catalog, false).unwrap();
    f.assert_original();
}

#[test]
fn files_the_game_writes_neither_block_undo_nor_are_put_back() {
    let f = Fixture::new();
    f.apply();
    let ini = b"[General]\r\nLogging=1\r\n";
    write(&f.game().join("InfinityLoader.ini"), ini);
    write(&f.game().join("buffbot_exec.log"), b"cast Haste");
    let preview = inspect(&f.ctx(), &f.catalog).unwrap();
    assert!(preview.can_undo, "{preview:?}");
    restore(&f.ctx(), &f.catalog, true).unwrap();
    f.assert_original();
    assert_eq!(fs::read(f.game().join("InfinityLoader.ini")).unwrap(), ini);
    assert_eq!(
        fs::read(f.game().join("buffbot_exec.log")).unwrap(),
        b"cast Haste"
    );
}

#[test]
fn a_change_after_the_hotpatch_blocks_undo_without_writing_anything() {
    let f = Fixture::new();
    f.apply();
    write(
        &f.game().join("override/other.itm"),
        b"ITM V1   edited by hand",
    );
    let before = f.now();
    let preview = inspect(&f.ctx(), &f.catalog).unwrap();
    assert!(!preview.can_undo);
    let error = restore(&f.ctx(), &f.catalog, true).unwrap_err();
    assert!(error.contains("override/other.itm"), "{error}");
    assert_eq!(f.now(), before);
    assert_eq!(f.history().entries[0].state, State::Applied);
}

#[test]
fn a_replaced_file_without_any_earlier_copy_is_refused() {
    let f = Fixture::new();
    let (dir, prepared) = f.prepare();
    transaction::place_payload(&f.ctx(), &dir, &prepared).unwrap();
    // WeiDU would have kept a copy in its backup folder; here nobody did.
    write(&f.game().join("override/BELHIFET.CRE"), NEW_CREATURE);
    let mut log = BASE_LOG.to_vec();
    log.extend_from_slice(ROW);
    write(&f.game().join("WeiDU.log"), &log);
    let error = transaction::record_applied(&f.ctx(), &f.catalog, &dir, &prepared).unwrap_err();
    assert!(
        error.contains("No earlier copy of override/belhifet.cre"),
        "{error}"
    );
    let error = restore(&f.ctx(), &f.catalog, false).unwrap_err();
    assert!(error.contains("override/belhifet.cre"), "{error}");
}

#[test]
fn the_mod_list_must_grow_by_exactly_the_hotpatch_rows() {
    let f = Fixture::new();
    let (dir, prepared) = f.prepare();
    f.weidu(&dir, &prepared);
    let mut log = BASE_LOG.to_vec();
    log.extend_from_slice(
        b"~CHRIZ_IWD_ENCOUNTERS/SETUP-CHRIZ_IWD_ENCOUNTERS.TP2~ #0 #11 // Another fight\r\n",
    );
    write(&f.game().join("WeiDU.log"), &log);
    let error = transaction::record_applied(&f.ctx(), &f.catalog, &dir, &prepared).unwrap_err();
    assert!(
        error.contains("exactly the hotpatch's components"),
        "{error}"
    );
    restore(&f.ctx(), &f.catalog, false).unwrap();
    f.assert_original();
}

#[test]
fn a_mod_list_edited_after_the_hotpatch_is_not_accepted() {
    let f = Fixture::new();
    f.apply();
    let mut log = fs::read(f.game().join("WeiDU.log")).unwrap();
    log.extend_from_slice(b"// edited by hand\r\n");
    write(&f.game().join("WeiDU.log"), &log);
    let error = verified_log_suffix(f.root()).unwrap_err();
    assert!(error.contains("mod list changed"), "{error}");
}

#[test]
fn the_history_refuses_records_that_were_changed_or_do_not_chain() {
    let f = Fixture::new();
    let dir = f.apply();
    let applied = fs::read(dir.join("applied.json")).unwrap();
    let mut tampered: serde_json::Value = serde_json::from_slice(&applied).unwrap();
    tampered["prepared_sha256"] = "00".repeat(32).into();
    fs::write(dir.join("applied.json"), tampered.to_string()).unwrap();
    assert!(History::load(f.root(), &f.record.install_id).is_err());
    assert!(launch_guard(f.root()).is_err());
    fs::write(dir.join("applied.json"), &applied).unwrap();

    // A later record must name the applied one as its predecessor.
    let mut prepared: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("prepared.json")).unwrap()).unwrap();
    prepared["previous"] = serde_json::Value::Null;
    let later = f.ctx().state().join(format!("{:020}", 2));
    files::create(
        &later.join("prepared.json"),
        prepared.to_string().as_bytes(),
    )
    .unwrap();
    let error = History::load(f.root(), &f.record.install_id).err().unwrap();
    assert!(error.contains("chain"), "{error}");
    fs::remove_dir_all(later).unwrap();
    f.history();

    // Another installation's record is never accepted.
    assert!(History::load(f.root(), "another-installation").is_err());
}

#[test]
fn the_rule_offers_the_recommended_setup_its_next_hotpatch() {
    let f = Fixture::new();
    let assessment = f.assess(f.baseline());
    assert!(assessment.refusals.is_empty(), "{:?}", assessment.refusals);
    assert_eq!(assessment.base.unwrap().id, "iwd-fixture");
    assert_eq!(assessment.next.unwrap().id, "belhifet-1");
    assert_eq!(assessment.current.unwrap(), f.original);

    f.apply();
    let assessment = f.assess(f.baseline());
    assert!(assessment.refusals.is_empty(), "{:?}", assessment.refusals);
    assert!(assessment.next.is_none(), "every hotpatch is applied");
}

#[test]
fn the_rule_refuses_files_changed_by_hand() {
    let f = Fixture::new();
    write(
        &f.game().join("override/other.itm"),
        b"ITM V1   edited by hand",
    );
    fs::remove_file(f.game().join("data/AREA.BIF")).unwrap();
    write(
        &f.game().join("override/MINE.ITM"),
        b"ITM V1   added by hand",
    );
    let assessment = f.assess(f.baseline());
    let refusals = assessment.refusals.join(" ");
    assert!(
        refusals.contains("changed since the installer"),
        "{refusals}"
    );
    for name in [
        "override/other.itm (changed)",
        "data/area.bif (removed)",
        "override/mine.itm (added)",
    ] {
        assert!(refusals.contains(name), "{refusals}");
    }
    // The player still sees what is withheld.
    assert_eq!(assessment.next.unwrap().id, "belhifet-1");
}

#[test]
fn the_rule_refuses_a_changed_collection_and_far_away_choices() {
    let f = Fixture::new();
    let mut baseline = f.baseline();
    let artifact = baseline.manifest.artifacts.values_mut().next().unwrap();
    artifact.version.push_str("-changed");
    let assessment = f.assess(baseline);
    let refusals = assessment.refusals.join(" ");
    assert!(refusals.contains("mod list was changed"), "{refusals}");
    assert!(assessment.base.is_none());
    assert_eq!(assessment.next.unwrap().id, "belhifet-1");

    let mut baseline = f.baseline();
    baseline.receipt.versions.recipe = "0.0.9".into();
    baseline
        .manifest
        .artifacts
        .values_mut()
        .next()
        .unwrap()
        .version
        .push_str("-older");
    // Another collection version simply has no hotpatches.
    let assessment = f.assess(baseline);
    assert!(assessment.refusals.is_empty(), "{:?}", assessment.refusals);
    assert!(assessment.base.is_none());
    assert!(assessment.next.is_none());

    let (manifest, _) = recommended();
    let mut selection = baseline_selection(&manifest);
    selection.set_feature("mod:stratagems", false);
    let evaluation = crate::recipe_view::evaluate(&manifest, &selection).unwrap();
    let mut baseline = f.baseline();
    baseline.receipt.normalized_selection = evaluation.normalized_selection;
    baseline.receipt.plan = evaluation.plan;
    let refusals = f.assess(baseline).refusals.join(" ");
    assert!(refusals.contains("was left out"), "{refusals}");

    let mut baseline = f.baseline();
    baseline.recovered = true;
    let refusals = f.assess(baseline).refusals.join(" ");
    assert!(refusals.contains("repaired"), "{refusals}");

    let refusals = gate::assess_with(
        &f.ctx(),
        &f.catalog,
        &f.history(),
        Err("The installed mod list changed outside CEBG's recorded work.".into()),
    )
    .refusals
    .join(" ");
    assert!(refusals.contains("mod list changed outside"), "{refusals}");
}

#[test]
fn the_rule_refuses_after_other_game_updates() {
    let f = Fixture::new();
    fs::create_dir_all(f.root().join(".chriz/patches/cebg-v1")).unwrap();
    let refusals = f.assess(f.baseline()).refusals.join(" ");
    assert!(refusals.contains("earlier game fix"), "{refusals}");
}

fn baseline_selection(manifest: &crate::Manifest) -> crate::resolve::Selection {
    crate::resolve::Selection {
        platform: "windows".into(),
        choices: manifest.presets["chris-recommended"].selections.clone(),
    }
}

#[test]
fn the_catalog_refuses_packages_that_cannot_install_unattended() {
    let f = Fixture::new();
    let dir = f.temp.path().join("catalog");
    let original = fs::read_to_string(dir.join("mods/chriz-iwd-encounters.toml")).unwrap();
    fs::write(
        dir.join("mods/chriz-iwd-encounters.toml"),
        original.replace("id = 10\n", "id = 11\n"),
    )
    .unwrap();
    assert!(catalog::load(&dir)
        .unwrap_err()
        .contains("does not declare"));
    fs::write(
        dir.join("mods/chriz-iwd-encounters.toml"),
        original.replace(
            "tp2 = \"chriz_iwd_encounters/setup-chriz_iwd_encounters.tp2\"",
            "tp2 = \"other/setup-other.tp2\"",
        ),
    )
    .unwrap();
    assert!(catalog::load(&dir).is_err());
    fs::write(dir.join("mods/chriz-iwd-encounters.toml"), &original).unwrap();
    let text = fs::read_to_string(dir.join("catalog.toml")).unwrap();
    fs::write(
        dir.join("catalog.toml"),
        text.replace("bases = [\"iwd-fixture\"]", "bases = [\"unknown\"]"),
    )
    .unwrap();
    assert!(catalog::load(&dir).unwrap_err().contains("unknown base"));
    fs::write(dir.join("catalog.toml"), &text).unwrap();
    let reloaded = catalog::load(&dir).unwrap();
    assert_eq!(reloaded, f.catalog);
}

#[test]
fn checking_reads_but_never_writes() {
    let f = Fixture::new();
    let before = f.now();
    let preview = inspect(&f.ctx(), &f.catalog).unwrap();
    // Without a verifiable recipe the synthetic installation is refused, and says why.
    assert_eq!(preview.state, "refused", "{preview:?}");
    assert!(!preview.refusals.is_empty());
    assert_eq!(f.now(), before);
    assert!(!f.root().join(".chriz/hotpatches").exists());
    f.assert_untouched();
}

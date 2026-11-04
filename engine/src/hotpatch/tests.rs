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
const VOICE_ROW: &[u8] = b"~VOICEPACK/VOICEPACK.TP2~ #0 #0 // Voice pack: 1.0\r\n";
const SAVE: &str = "save/000000001-Quick-Save/ICEWIND.gam";
const OLD_CREATURE: &[u8] = b"CRE V9.0 synthetic Belhifet before the hotpatch";
const NEW_CREATURE: &[u8] = b"CRE V9.0 synthetic Belhifet after the hotpatch";

const CATALOG: &str = r#"
schema = 1
game = "iwdee"
runtime_files = ["infinityloader.ini", "buffbot_*.log", "override/bfbot_presets/**"]

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

[[updates]]
mod = "buffbot"
title = "BuffBot"
summary = "Brings BuffBot to its newest release."
artifact = "buffbot-1.16.0"
"#;

const BUFFBOT_ARTIFACT: &str = r#"
id = "buffbot-1.16.0"
name = "BuffBot"
version = "1.16.0"
acquisition = "fetch-only"

[source]
kind = "github-release"
url = "https://example.invalid/buffbot-v1.16.0.zip"
reference = "v1.16.0"
expected_filename = "buffbot-v1.16.0.zip"
expected_length = 5797041
sha256 = "ce38d81102f717acf709cc8a7827586f8ab6ff5ff450aaca42f0b5d513da0082"
redirect_hosts = []

[archive]
kind = "zip"
root_rule = "direct"
publish_roots = ["buffbot"]
tp2_paths = ["buffbot/setup-buffbot.tp2"]

[archive.limits]
max_depth = 8
max_entries = 2048
max_entry_uncompressed_bytes = 16777216
max_total_uncompressed_bytes = 67108864
max_compression_ratio = 100

[provenance]
homepage = "https://example.invalid/"
license = "Fetch-only"
url = "https://example.invalid/"
reviewed_on = "2026-11-04"
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
        Self::build(
            BASE_LOG,
            vec![LogComponentReceipt {
                tp2: "stratagems/setup-stratagems.tp2".into(),
                language: 0,
                component: 1000,
            }],
            &|_| {},
        )
    }

    /// A finished installation whose mod list is `log` with the rows `components`; `setup` adds
    /// game files before the installer's record of the folder is taken.
    fn build(log: &[u8], components: Vec<LogComponentReceipt>, setup: &dyn Fn(&Path)) -> Self {
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
        write(&game.join("WeiDU.log"), log);
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
        setup(&game);
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
                    sha256: sha256_bytes(log),
                    components,
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
        write(
            &catalog_dir.join("artifacts/buffbot-1.16.0.toml"),
            BUFFBOT_ARTIFACT.as_bytes(),
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

    /// What a player's voice pack writes after the installer finished: its folder and WeiDU
    /// backup, its sounds, one more string and its row in the mod list.
    fn add_voice_pack(&self) {
        let game = self.game();
        write(
            &game.join("voicepack/voicepack.tp2"),
            b"BACKUP ~weidu_external/voicepack/backup~\r\nBEGIN ~Voice pack~\r\n",
        );
        write(
            &game.join("weidu_external/voicepack/backup/0/UNINSTALL.0"),
            b"lang/en_us/sounds/VOICE01.wav\r\n",
        );
        write(&game.join("lang/en_US/sounds/VOICE01.wav"), b"RIFF voice");
        write(&game.join("override/VOICE01.wav"), b"RIFF voice copy");
        let strings = fs::read(game.join("lang/en_US/dialog.tlk")).unwrap();
        let entries = u32::from_le_bytes(strings[10..14].try_into().unwrap());
        write(&game.join("lang/en_US/dialog.tlk"), &tlk(entries + 1));
        let mut log = fs::read(game.join("WeiDU.log")).unwrap();
        log.extend_from_slice(VOICE_ROW);
        write(&game.join("WeiDU.log"), &log);
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
fn weidu_is_judged_by_its_result_lines_as_in_a_fresh_installation() {
    let f = Fixture::new();
    let (_, prepared) = f.prepare();
    let judge = |after: &[u8], debug: &[u8], stdout: &[u8], code| {
        transaction::finished_as_planned(BASE_LOG, after, debug, stdout, &prepared.package, code)
    };
    let after = [BASE_LOG, ROW].concat();
    let doubled = [BASE_LOG, ROW, ROW].concat();
    let done = b"SUCCESSFULLY INSTALLED Belhifet\r\n";
    let warned = b"INSTALLED WITH WARNINGS Belhifet\r\n";
    judge(&after, done, b"", 0).unwrap();
    // WeiDU may keep its detailed log elsewhere; its console output then carries the result.
    judge(&after, b"WeiDU v 249\r\n", done, 0).unwrap();
    // Warnings count as installed, exactly as in a fresh installation.
    judge(&after, warned, b"", 3).unwrap();
    for (after, debug, code) in [
        (&after[..], &warned[..], 0),
        (&after[..], &done[..], 3),
        (&after[..], &b""[..], 0),
        (
            BASE_LOG,
            &b"NOT INSTALLED DUE TO ERRORS Belhifet\r\n"[..],
            1,
        ),
        (&doubled[..], &done[..], 0),
    ] {
        let error = judge(after, debug, b"", code).unwrap_err();
        assert!(
            error.contains("exactly the hotpatch's components"),
            "{error}"
        );
    }
}

#[test]
fn a_mod_added_after_the_hotpatch_keeps_its_rows_and_blocks_only_its_undo() {
    let f = Fixture::new();
    f.apply();
    let hotpatched = sha256_bytes(&fs::read(f.game().join("WeiDU.log")).unwrap());
    assert_eq!(
        recorded_log_sha256(f.root()).unwrap().as_deref(),
        Some(hotpatched.as_str())
    );
    f.add_voice_pack();
    // The hotpatch's rows stay verified; the launcher names the voice pack's row as extra.
    assert_eq!(verified_log_suffix(f.root()).unwrap(), [row()]);
    // Game updates, which need the exact list, still see the mod list as the hotpatch left it.
    assert_eq!(
        recorded_log_sha256(f.root()).unwrap().as_deref(),
        Some(hotpatched.as_str())
    );
    assert_ne!(
        sha256_bytes(&fs::read(f.game().join("WeiDU.log")).unwrap()),
        hotpatched
    );
    launch_guard(f.root()).unwrap();
    let assessment = f.assess(f.baseline());
    assert!(assessment.refusals.is_empty(), "{:?}", assessment.refusals);
    assert!(
        assessment
            .notes
            .join(" ")
            .contains("do not stop hotpatches"),
        "{:?}",
        assessment.notes
    );
    // Undo would take back what the voice pack changed, so it refuses without writing.
    let before = f.now();
    let error = restore(&f.ctx(), &f.catalog, true).unwrap_err();
    assert!(error.contains("cannot be undone automatically"), "{error}");
    assert_eq!(f.now(), before);
    assert_eq!(f.history().entries[0].state, State::Applied);
}

#[test]
fn a_voice_pack_added_after_installing_stays_through_apply_and_undo() {
    let f = Fixture::new();
    f.add_voice_pack();
    let with_voices = f.now();
    let assessment = f.assess(f.baseline());
    assert!(assessment.refusals.is_empty(), "{:?}", assessment.refusals);
    assert_eq!(assessment.next.unwrap().id, "belhifet-1");
    assert_eq!(assessment.current.unwrap(), with_voices);

    f.apply();
    // The hotpatch is installed after the voice pack.
    let log = fs::read(f.game().join("WeiDU.log")).unwrap();
    assert!(log.ends_with(&[VOICE_ROW, ROW].concat()));
    assert_eq!(verified_log_suffix(f.root()).unwrap(), [row()]);
    let history = f.history();
    assert_eq!(history.expected_state(f.root()).unwrap(), f.now());
    assert!(history.undo_target().is_some());

    // Undo gives back the folder with the voice pack, byte for byte, not the installer's.
    restore(&f.ctx(), &f.catalog, true).unwrap();
    let left = with_voices.differences(&f.now(), |name| f.catalog.runtime_files.matches(name));
    assert!(left.is_empty(), "{left:?}");
    assert_eq!(
        fs::read(f.game().join("lang/en_US/dialog.tlk")).unwrap(),
        tlk(3)
    );
    f.assert_untouched();
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

/// Christopher, 2026-11-04: refusing every installation whose files changed since the installer
/// was far too strict; players add voice packs and other mods.
#[test]
fn the_rule_names_the_players_own_changes_but_never_refuses_them() {
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
    assert!(assessment.refusals.is_empty(), "{:?}", assessment.refusals);
    assert_eq!(assessment.next.unwrap().id, "belhifet-1");
    let notes = assessment.notes.join(" ");
    assert!(notes.contains("do not stop hotpatches"), "{notes}");
    for name in [
        "override/other.itm (changed)",
        "data/area.bif (removed)",
        "override/mine.itm (added)",
    ] {
        assert!(notes.contains(name), "{notes}");
    }

    // A voice pack installed with WeiDU after the installer is no different.
    let f = Fixture::new();
    f.add_voice_pack();
    let assessment = f.assess(f.baseline());
    assert!(assessment.refusals.is_empty(), "{:?}", assessment.refusals);
    let notes = assessment.notes.join(" ");
    for name in [
        "lang/en_us/dialog.tlk (changed)",
        "voicepack (added)",
        "more.",
    ] {
        assert!(notes.contains(name), "{notes}");
    }
}

/// A player may install the hotpatch's own components with the mod's setup (Christopher did with
/// the boss fights, 2026-11-04); the check then says so instead of offering a hotpatch that
/// cannot apply on top.
#[test]
fn components_installed_outside_the_app_are_named_instead_of_offered() {
    let f = Fixture::new();
    let mut log = fs::read(f.game().join("WeiDU.log")).unwrap();
    log.extend_from_slice(ROW);
    write(&f.game().join("WeiDU.log"), &log);
    let assessment = f.assess(f.baseline());
    let refusals = assessment.refusals.join(" ");
    assert!(refusals.contains("already in your game"), "{refusals}");
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
        Err("A mod the installer put in was removed or moved since.".into()),
    )
    .refusals
    .join(" ");
    assert!(refusals.contains("installer put in"), "{refusals}");
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

// Whole-mod updates: BuffBot 1.12.6 -> 1.16.0, with the EEex Remote Console after it. WeiDU's
// re-installation is written by hand, as above for hotpatches.

const BUFFBOT_TP2: &str = "buffbot/setup-buffbot.tp2";
const OLD_TP2: &[u8] = b"BACKUP ~weidu_external/backup/buffbot~\r\nVERSION ~v1.12.6~\r\nLANGUAGE ~English~ ~english~ ~buffbot/lang/english/setup.tra~\r\nLANGUAGE ~German~ ~german~ ~buffbot/lang/german/setup.tra~\r\nBEGIN @100 DESIGNATED 1\r\nBEGIN @111 DESIGNATED 0\r\n";
// The new release lists its languages the other way round: English is number 1 there.
const NEW_TP2: &[u8] = b"BACKUP ~weidu_external/backup/buffbot~\r\nVERSION ~v1.16.0~\r\nLANGUAGE ~German~ ~german~ ~buffbot/lang/german/setup.tra~\r\nLANGUAGE ~English~ ~english~ ~buffbot/lang/english/setup.tra~\r\nBEGIN @100 DESIGNATED 1\r\nBEGIN @111 DESIGNATED 0\r\n";
const OLD_CORE: &[u8] = b"-- BuffBot core 1.12.6";
const NEW_CORE: &[u8] = b"-- BuffBot core 1.16.0";
const OLD_ONLY: &[u8] = b"-- only in 1.12.6";
const NEW_ONLY: &[u8] = b"-- new in 1.16.0";
const REMOTE: &[u8] = b"-- EEex Remote Console";
const PRESET: &str = "override/bfbot_presets/Zyntris.lua";
/// A voice pack a player added after installing, with a name in its own encoding.
const PLAYER_ROW: &[u8] = b"~RONDAK/RONDAK.TP2~ #0 #0 // Rond\xE1k: 1.0\r\n";

fn buffbot_log(language: u32, version: &str, extra: &[u8]) -> Vec<u8> {
    let mut log = format!(
        "// Synthetic mod list\r\n~STRATAGEMS/SETUP-STRATAGEMS.TP2~ #0 #1000 // Fixture: v1\r\n~BUFFBOT/SETUP-BUFFBOT.TP2~ #{language} #1 // BuffBot: EEex LuaJIT Support (auto-detected): v{version}\r\n~BUFFBOT/SETUP-BUFFBOT.TP2~ #{language} #0 // BuffBot: In-Game Buff Automation: v{version}\r\n~SETUP-EEEXREMOTE.TP2~ #0 #0 // EEex Remote Console: v0.2.1\r\n"
    )
    .into_bytes();
    log.extend_from_slice(extra);
    log
}

/// What WeiDU leaves in the list it saw: its notes of the old installation, then the new rows.
fn weidu_log() -> Vec<u8> {
    b"// Synthetic mod list\r\n~STRATAGEMS/SETUP-STRATAGEMS.TP2~ #0 #1000 // Fixture: v1\r\n// Recently Uninstalled: ~BUFFBOT/SETUP-BUFFBOT.TP2~ #0 #1 // BuffBot: EEex LuaJIT Support (auto-detected): v1.16.0\r\n// Recently Uninstalled: ~BUFFBOT/SETUP-BUFFBOT.TP2~ #0 #0 // BuffBot: In-Game Buff Automation: v1.16.0\r\n~BUFFBOT/SETUP-BUFFBOT.TP2~ #1 #1 // BuffBot: EEex LuaJIT Support (auto-detected): v1.16.0\r\n~BUFFBOT/SETUP-BUFFBOT.TP2~ #1 #0 // BuffBot: In-Game Buff Automation: v1.16.0\r\n".to_vec()
}

fn buffbot_rows(extra: &[(&str, u32)]) -> Vec<LogComponentReceipt> {
    [
        ("stratagems/setup-stratagems.tp2", 1000),
        (BUFFBOT_TP2, 1),
        (BUFFBOT_TP2, 0),
        ("setup-eeexremote.tp2", 0),
    ]
    .iter()
    .chain(extra)
    .map(|(tp2, component)| LogComponentReceipt {
        tp2: (*tp2).into(),
        language: 0,
        component: *component,
    })
    .collect()
}

fn buffbot_files(game: &Path) {
    write(&game.join("buffbot/setup-buffbot.tp2"), OLD_TP2);
    write(&game.join("buffbot/BfBotCor.lua"), OLD_CORE);
    write(&game.join("buffbot/BfBotOld.lua"), OLD_ONLY);
    write(&game.join("buffbot/lang/english/setup.tra"), b"@1 = ~old~");
    write(&game.join("buffbot/lang/german/setup.tra"), b"@1 = ~alt~");
    write(&game.join("override/BfBotCor.lua"), OLD_CORE);
    write(&game.join("override/BfBotOld.lua"), OLD_ONLY);
    write(&game.join("override/bfbot_l10n.tra"), b"@1 = ~old~");
    write(
        &game.join("weidu_external/backup/buffbot/0/UNINSTALL.0"),
        b"override\\BfBotCor.lua\r\noverride\\BfBotOld.lua\r\noverride\\bfbot_l10n.tra\r\n",
    );
    write(
        &game.join("weidu_external/backup/buffbot/1/UNINSTALL.1"),
        b"",
    );
    write(
        &game.join("setup-eeexremote.tp2"),
        b"BACKUP ~weidu_external/backup/eeexremote~\r\nBEGIN ~Remote~ DESIGNATED 0\r\n",
    );
    write(&game.join("eeexremote/M_EEexRC.lua"), REMOTE);
    write(&game.join("override/M_EEexRC.lua"), REMOTE);
    write(
        &game.join("weidu_external/backup/eeexremote/0/UNINSTALL.0"),
        b"override\\M_EEexRC.lua\r\n",
    );
}

fn success(count: usize) -> Vec<u8> {
    b"SUCCESSFULLY INSTALLED x\r\n".repeat(count)
}

impl Fixture {
    fn buffbot() -> Self {
        Self::buffbot_with(b"", &[])
    }

    fn buffbot_with(extra_log: &[u8], extra_rows: &[(&str, u32)]) -> Self {
        let f = Self::build(
            &buffbot_log(0, "1.12.6", extra_log),
            buffbot_rows(extra_rows),
            &buffbot_files,
        );
        // Played since: the player exported a preset.
        write(&f.game().join(PRESET), b"-- the player's own preset");
        let release = f.temp.path().join("release");
        write(&release.join("buffbot/setup-buffbot.tp2"), NEW_TP2);
        write(&release.join("buffbot/BfBotCor.lua"), NEW_CORE);
        write(&release.join("buffbot/BfBotNew.lua"), NEW_ONLY);
        write(
            &release.join("buffbot/lang/english/setup.tra"),
            b"@1 = ~new~",
        );
        write(
            &release.join("buffbot/lang/german/setup.tra"),
            b"@1 = ~neu~",
        );
        f
    }

    fn update_entry(&self) -> &catalog::ModUpdate {
        self.catalog.update("buffbot").unwrap()
    }

    fn update_assess(&self) -> update::Assessment {
        update::assess_with(&self.ctx(), self.update_entry(), Ok(self.baseline()))
    }

    /// Plan and prepare the update as apply does, after its checks and backups.
    fn update_prepare(&self) -> (PathBuf, Prepared) {
        let ctx = self.ctx();
        let history = self.history();
        let assessment = self.update_assess();
        assert!(assessment.refusals.is_empty(), "{:?}", assessment.refusals);
        let manifest = recipe();
        let pinned = &manifest.artifacts[&manifest.mods["buffbot"].artifact_id];
        let release = update::release(self.update_entry(), pinned).unwrap();
        let plan = update::plan(
            &ctx,
            self.update_entry(),
            assessment.baseline.as_ref().unwrap(),
            assessment.installed.as_ref().unwrap(),
            &release,
            &self.temp.path().join("release"),
            "en_US".into(),
        )
        .unwrap();
        let existing = fs::read_dir(ctx.state()).map_or(0, |folders| folders.count());
        let dir = ctx.state().join(format!("{:020}", existing + 1));
        files::create(&dir.join("identity"), self.record.install_id.as_bytes()).unwrap();
        let prepared = update::prepare(
            &ctx,
            &self.catalog,
            &dir,
            &plan,
            assessment.current.as_ref().unwrap(),
            &self.tool,
            files::hash(&self.tool).unwrap(),
            history.head(),
        )
        .unwrap();
        (dir, prepared)
    }

    /// What WeiDU's re-installation writes, seeing BuffBot as the last mod: the new release's
    /// files in `override/`, a new uninstall list, the rows with the new version and English
    /// under its new number.
    fn update_weidu(&self, dir: &Path, prepared: &Prepared) {
        update::place_release(&self.ctx(), dir, prepared).unwrap();
        update::hide_later_rows(&self.ctx(), dir, prepared).unwrap();
        let game = self.game();
        let seen = fs::read(game.join("WeiDU.log")).unwrap();
        assert!(
            seen.ends_with(b"BuffBot: In-Game Buff Automation: v1.12.6\r\n"),
            "WeiDU must see BuffBot last: {}",
            String::from_utf8_lossy(&seen)
        );
        write(&game.join("override/BfBotCor.lua"), NEW_CORE);
        fs::remove_file(game.join("override/BfBotOld.lua")).unwrap();
        write(&game.join("override/BfBotNew.lua"), NEW_ONLY);
        write(&game.join("override/bfbot_l10n.tra"), b"@1 = ~new~");
        write(
            &game.join("weidu_external/backup/buffbot/0/UNINSTALL.0"),
            b"override\\BfBotCor.lua\r\noverride\\BfBotNew.lua\r\noverride\\bfbot_l10n.tra\r\n",
        );
        write(&game.join("WeiDU.log"), &weidu_log());
    }

    fn update_apply(&self) -> PathBuf {
        let (dir, prepared) = self.update_prepare();
        self.update_weidu(&dir, &prepared);
        let evidence = transaction::WeiduEvidence {
            exit_code: 0,
            debug: success(2),
            stdout: Vec::new(),
        };
        update::reinstalled_as_planned(
            &fs::read(dir.join("affected/weidu.log")).unwrap(),
            &fs::read(self.game().join("WeiDU.log")).unwrap(),
            &evidence,
            &prepared,
            Some("1.16.0"),
        )
        .unwrap();
        update::put_later_rows_back(&self.ctx(), &dir, &prepared).unwrap();
        update::record_applied(&self.ctx(), &self.catalog, &dir, &prepared).unwrap();
        dir
    }

    fn assert_preset_kept(&self) {
        assert_eq!(
            fs::read(self.game().join(PRESET)).unwrap(),
            b"-- the player's own preset"
        );
    }
}

#[test]
fn a_buffbot_update_is_recorded_and_undo_puts_back_every_byte() {
    let f = Fixture::buffbot();
    let assessment = f.update_assess();
    assert_eq!(assessment.installed.as_ref().unwrap().version, "1.12.6");

    let dir = f.update_apply();
    let prepared: Prepared =
        serde_json::from_slice(&fs::read(dir.join("prepared.json")).unwrap()).unwrap();
    assert_eq!(prepared.package.components, [1, 0]);
    assert_eq!(prepared.package.language, 1, "English by its folder");
    // The old release's files and the overwritten ones are kept in the record itself.
    for name in [
        "buffbot/bfbotold.lua",
        "override/bfbotold.lua",
        "override/bfbotcor.lua",
        "weidu_external/backup/buffbot/0/uninstall.0",
    ] {
        assert!(prepared.saved.contains_key(name), "{name}");
    }
    assert!(
        !prepared.saved.contains_key("override/m_eeexrc.lua"),
        "another mod's file is not touched, so it needs no copy"
    );
    // The mod after BuffBot is where it was; only BuffBot's rows changed.
    assert_eq!(
        fs::read(f.game().join("WeiDU.log")).unwrap(),
        buffbot_log(1, "1.16.0", b"")
    );
    assert_eq!(
        fs::read(f.game().join("override/M_EEexRC.lua")).unwrap(),
        REMOTE
    );
    let history = f.history();
    assert_eq!(history.entries[0].state, State::Applied);
    assert!(history.applied_ids().is_empty(), "an update is no hotpatch");
    let updates = history.applied_updates();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].from_version, "1.12.6");
    assert_eq!(updates[0].to_version, "1.16.0");
    assert_eq!(update::applied(f.root()).unwrap().len(), 1);
    assert_eq!(history.expected_state(f.root()).unwrap(), f.now());
    assert_eq!(
        recorded_log_sha256(f.root()).unwrap(),
        Some(sha256_bytes(&buffbot_log(1, "1.16.0", b"")))
    );
    assert!(!f.game().join("buffbot/BfBotOld.lua").exists());
    assert_eq!(
        fs::read(f.game().join("buffbot/BfBotNew.lua")).unwrap(),
        NEW_ONLY
    );
    launch_guard(f.root()).unwrap();
    f.assert_preset_kept();
    f.assert_untouched();

    let preview = restore(&f.ctx(), &f.catalog, true).unwrap();
    assert!(!preview.can_undo, "{preview:?}");
    f.assert_original();
    f.assert_preset_kept();
    assert_eq!(f.history().entries[0].state, State::Restored);
    assert!(f.history().applied_updates().is_empty());
    assert_eq!(
        fs::read(f.game().join("buffbot/BfBotOld.lua")).unwrap(),
        OLD_ONLY
    );
}

#[test]
fn hotpatches_follow_a_buffbot_update_and_undo_goes_back_one_change_at_a_time() {
    let f = Fixture::buffbot();
    assert!(!update::undo_offered(&f.ctx(), "buffbot").unwrap());
    f.update_apply();
    assert!(update::undo_offered(&f.ctx(), "buffbot").unwrap());
    assert!(!update::undo_offered(&f.ctx(), "other").unwrap());
    let assessment = f.assess(f.baseline());
    assert!(assessment.refusals.is_empty(), "{:?}", assessment.refusals);
    assert_eq!(assessment.next.unwrap().id, "belhifet-1");
    f.apply();
    assert!(
        !update::undo_offered(&f.ctx(), "buffbot").unwrap(),
        "the latest change is the hotpatch"
    );
    let history = f.history();
    assert_eq!(history.applied_ids(), ["belhifet-1"]);
    assert_eq!(history.applied_updates().len(), 1);
    assert_eq!(verified_log_suffix(f.root()).unwrap(), [row()]);

    restore(&f.ctx(), &f.catalog, true).unwrap();
    assert!(update::undo_offered(&f.ctx(), "buffbot").unwrap());
    let history = f.history();
    assert!(history.applied_ids().is_empty());
    assert_eq!(history.applied_updates().len(), 1, "only the hotpatch went");
    assert_eq!(
        fs::read(f.game().join("override/BfBotCor.lua")).unwrap(),
        NEW_CORE
    );
    restore(&f.ctx(), &f.catalog, true).unwrap();
    f.assert_original();
    f.assert_preset_kept();
}

#[test]
fn an_update_leaves_a_hotpatch_and_a_player_mod_after_it_in_place() {
    let f = Fixture::buffbot_with(PLAYER_ROW, &[("rondak/rondak.tp2", 0)]);
    assert!(f.update_assess().refusals.is_empty());
    f.apply();
    let log = [&buffbot_log(0, "1.12.6", PLAYER_ROW)[..], ROW].concat();
    assert_eq!(fs::read(f.game().join("WeiDU.log")).unwrap(), log);

    f.update_apply();
    assert_eq!(
        fs::read(f.game().join("WeiDU.log")).unwrap(),
        [&buffbot_log(1, "1.16.0", PLAYER_ROW)[..], ROW].concat(),
        "the rows after BuffBot come back byte for byte, in their own encoding"
    );
    assert_eq!(verified_log_suffix(f.root()).unwrap(), [row()]);

    restore(&f.ctx(), &f.catalog, true).unwrap();
    assert_eq!(fs::read(f.game().join("WeiDU.log")).unwrap(), log);
    restore(&f.ctx(), &f.catalog, true).unwrap();
    f.assert_original();
}

#[test]
fn hand_changes_stay_through_an_update_and_its_undo() {
    let f = Fixture::buffbot();
    let edited = b"ITM V1   edited by hand";
    write(&f.game().join("override/other.itm"), edited);
    f.update_apply();
    assert_eq!(
        fs::read(f.game().join("override/other.itm")).unwrap(),
        edited
    );
    let assessment = f.assess(f.baseline());
    assert!(assessment.refusals.is_empty(), "{:?}", assessment.refusals);
    restore(&f.ctx(), &f.catalog, true).unwrap();
    assert_eq!(
        fs::read(f.game().join("override/other.itm")).unwrap(),
        edited
    );
    assert_eq!(
        fs::read(f.game().join("WeiDU.log")).unwrap(),
        buffbot_log(0, "1.12.6", b"")
    );
}

#[test]
fn an_interrupted_update_blocks_play_and_restoring_gives_the_whole_mod_list_back() {
    let f = Fixture::buffbot_with(PLAYER_ROW, &[("rondak/rondak.tp2", 0)]);
    let (dir, prepared) = f.update_prepare();
    // WeiDU stopped half-way: the release is there, the mod list ends with BuffBot.
    update::place_release(&f.ctx(), &dir, &prepared).unwrap();
    update::hide_later_rows(&f.ctx(), &dir, &prepared).unwrap();
    assert!(launch_guard(f.root()).is_err());
    restore(&f.ctx(), &f.catalog, false).unwrap();
    f.assert_original();
    assert_eq!(
        fs::read(f.game().join("WeiDU.log")).unwrap(),
        buffbot_log(0, "1.12.6", PLAYER_ROW)
    );
    launch_guard(f.root()).unwrap();
}

#[test]
fn an_update_that_left_the_old_files_is_not_recorded() {
    let f = Fixture::buffbot();
    let (dir, prepared) = f.update_prepare();
    update::place_release(&f.ctx(), &dir, &prepared).unwrap();
    // WeiDU wrote the new version into the mod list but installed nothing.
    write(&f.game().join("WeiDU.log"), &buffbot_log(1, "1.16.0", b""));
    let error = update::record_applied(&f.ctx(), &f.catalog, &dir, &prepared).unwrap_err();
    assert!(error.contains("still the old release's"), "{error}");
    assert!(error.contains("override/bfbotcor.lua"), "{error}");
    restore(&f.ctx(), &f.catalog, false).unwrap();
    f.assert_original();
}

#[test]
fn an_update_may_change_no_other_mods_files() {
    let f = Fixture::buffbot();
    let (dir, prepared) = f.update_prepare();
    f.update_weidu(&dir, &prepared);
    update::put_later_rows_back(&f.ctx(), &dir, &prepared).unwrap();
    write(
        &f.game().join("override/other.itm"),
        b"ITM V1   changed by the run",
    );
    let error = update::record_applied(&f.ctx(), &f.catalog, &dir, &prepared).unwrap_err();
    assert!(error.contains("files of other mods"), "{error}");
    assert!(error.contains("override/other.itm"), "{error}");
}

#[test]
fn changes_outside_the_update_area_fail_and_are_left_for_a_closer_look() {
    let f = Fixture::buffbot();
    let (dir, prepared) = f.update_prepare();
    f.update_weidu(&dir, &prepared);
    update::put_later_rows_back(&f.ctx(), &dir, &prepared).unwrap();
    write(
        &f.game().join("data/NEW.BIF"),
        b"BIFFV1  biffed by an update",
    );
    let error = update::record_applied(&f.ctx(), &f.catalog, &dir, &prepared).unwrap_err();
    assert!(error.contains("outside its area"), "{error}");
    let error = restore(&f.ctx(), &f.catalog, false).unwrap_err();
    assert!(error.contains("closer look"), "{error}");
    assert!(launch_guard(f.root()).is_err());
}

#[test]
fn the_update_is_judged_by_the_mod_list_weidu_saw_and_its_result_lines() {
    let f = Fixture::buffbot();
    let (dir, prepared) = f.update_prepare();
    let before = fs::read(dir.join("affected/weidu.log")).unwrap();
    let judge = |after: &[u8], debug: Vec<u8>, code: i32, version: Option<&str>| {
        update::reinstalled_as_planned(
            &before,
            after,
            &transaction::WeiduEvidence {
                exit_code: code,
                debug,
                stdout: Vec::new(),
            },
            &prepared,
            version,
        )
    };
    let after = weidu_log();
    judge(&after, success(2), 0, Some("1.16.0")).unwrap();
    // WeiDU installs a component it took off for the one before it twice; more results are fine.
    judge(&after, success(3), 0, Some("1.16.0")).unwrap();
    let mut warned = success(1);
    warned.extend_from_slice(b"INSTALLED WITH WARNINGS x\r\n");
    judge(&after, warned.clone(), 3, Some("1.16.0")).unwrap();
    let garbled = [&after[..], b"// Recently Uninstalled: something\r\n"].concat();
    let other = [
        &after[..],
        b"// Recently Uninstalled: ~SETUP-X.TP2~ #0 #0\r\n",
    ]
    .concat();
    let old_version = String::from_utf8(after.clone())
        .unwrap()
        .replace(
            "#1 #1 // BuffBot: EEex LuaJIT Support (auto-detected): v1.16.0",
            "#1 #1 // BuffBot: EEex LuaJIT Support (auto-detected): v1.12.6",
        )
        .into_bytes();
    // English kept its old number, which is German in the new release.
    let old_number = String::from_utf8(after.clone())
        .unwrap()
        .replace("TP2~ #1 #", "TP2~ #0 #")
        .into_bytes();
    let row_lost = String::from_utf8(after.clone())
        .unwrap()
        .replace("~BUFFBOT/SETUP-BUFFBOT.TP2~ #1 #1 // BuffBot: EEex LuaJIT Support (auto-detected): v1.16.0\r\n", "")
        .into_bytes();
    for (after, debug, code, says) in [
        (after.clone(), warned.clone(), 0, "exit code"),
        (after.clone(), success(2), 3, "exit code"),
        (after.clone(), success(1), 0, "results"),
        (
            after.clone(),
            [success(2), b"NOT INSTALLED DUE TO ERRORS x\r\n".to_vec()].concat(),
            0,
            "exit code",
        ),
        (old_version, success(2), 0, "1.16.0"),
        (old_number, success(2), 0, "order"),
        (row_lost, success(2), 0, "uninstalled"),
        (garbled, success(2), 0, "uninstalled"),
        (other, success(2), 0, "uninstalled"),
    ] {
        let error = judge(&after, debug, code, Some("1.16.0")).unwrap_err();
        assert!(error.contains(says), "{says}: {error}");
    }
}

#[test]
fn a_game_without_buffbot_is_offered_nothing() {
    let f = Fixture::new();
    let assessment = f.update_assess();
    assert!(assessment.installed.is_none());
    assert!(assessment.refusals.is_empty(), "{:?}", assessment.refusals);
}

#[test]
fn an_undo_that_stopped_after_a_later_one_was_undone_blocks_play_until_restored() {
    let f = Fixture::buffbot();
    let update = f.update_apply();
    f.apply();
    restore(&f.ctx(), &f.catalog, true).unwrap();
    // Undo of the update stopped right after its restoring record.
    files::json_new(
        &update.join("restoring.json"),
        &serde_json::json!({
            "prepared_sha256": files::hash(&update.join("prepared.json")).unwrap(),
            "undo": true,
        }),
    )
    .unwrap();
    assert!(launch_guard(f.root()).is_err());
    assert_eq!(f.history().unfinished().unwrap().dir, update);
    assert!(restore(&f.ctx(), &f.catalog, true).is_err());
    restore(&f.ctx(), &f.catalog, false).unwrap();
    f.assert_original();
    f.assert_preset_kept();
    launch_guard(f.root()).unwrap();
    assert!(f.history().undo_target().is_none());
}

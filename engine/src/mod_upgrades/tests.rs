//! Transaction boundaries use disposable synthetic data, never installed games.
use super::*;
use transaction::{finish, history, prepare, restore_files};

struct Fixture {
    _temp: tempfile::TempDir,
    record: ManagedInstallRecord,
    app: PathBuf,
    cache: PathBuf,
    original: BTreeMap<String, Vec<u8>>,
}
fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("managed");
        fs::create_dir(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let original = BTreeMap::from([
            ("weidu.log".into(), b"original mod list".to_vec()),
            ("weidu.conf".into(), b"lang_dir = en_US".to_vec()),
            (
                "lang/en_us/dialog.tlk".into(),
                b"synthetic original strings".to_vec(),
            ),
            ("override/bfbotscn.lua".into(), b"old code".to_vec()),
            (
                "override/bfbot_strrefs.txt".into(),
                b"1\n2\n3\n4\n5\n6\n7\n8\n".to_vec(),
            ),
            ("override/bfbotbg.mos".into(), b"old background".to_vec()),
            ("engine.lua".into(), b"immutable engine identity".to_vec()),
            ("override/other-mod.lua".into(), b"unrelated".to_vec()),
            (
                "override/bfbot_presets/player.json".into(),
                b"user settings".to_vec(),
            ),
        ]);
        for (n, b) in &original {
            write(&root.join("game").join(n), b);
        }
        let receipt=br#"{"install_id":"fixture","recipe_payload_sha256":"recipe","selection_sha256":"selection"}"#;
        write(&root.join(".chriz/install-receipt.json"), receipt);
        let record = ManagedInstallRecord {
            schema_version: 1,
            install_id: "fixture".into(),
            display_name: "Test".into(),
            managed_root: root.clone(),
            recipe_version: "fixture".into(),
            recipe_sha256: "recipe".into(),
            engine_name: "Test".into(),
            managed_save_root: root.join("profile"),
            launch_path: root.join("game/Baldur.exe"),
            receipt_sha256: sha256_bytes(receipt),
            completed_at_millis: 1,
        };
        write(
            &record.managed_save_root.join("save/one.gam"),
            b"player save",
        );
        Self {
            app: temp.path().join("app"),
            cache: temp.path().join("cache"),
            _temp: temp,
            record,
            original,
        }
    }
    fn ctx(&self) -> Context<'_> {
        Context {
            record: &self.record,
            app_data: &self.app,
            cache: &self.cache,
            recipe: self._temp.path(),
        }
    }
    fn prepare(&self, id: u32) -> (PathBuf, transaction::Prepared) {
        let ctx = self.ctx();
        let before = files::inventory(&ctx.game(), false).unwrap();
        let hashes = before
            .iter()
            .filter(|(n, s)| {
                !s.directory
                    && (buffbot::writable(n)
                        || buffbot::runtime_mutable(n)
                        || ["engine.lua", "infinityloader.ini"].contains(&n.as_str()))
            })
            .map(|(n, _)| (n.clone(), files::hash(&ctx.game().join(n)).unwrap()))
            .collect();
        let p = Plan {
            install_id: self.record.install_id.clone(),
            managed_root: self.record.managed_root.clone(),
            base_receipt_sha256: self.record.receipt_sha256.clone(),
            recipe_sha256: "recipe".into(),
            selection_sha256: "selection".into(),
            previous: transaction::head(&self.record.managed_root).unwrap(),
            hashes,
        };
        let dir = ctx.state().join(format!("{id:020}"));
        fs::create_dir_all(&dir).unwrap();
        let p = prepare(&ctx, &dir, p, before).unwrap();
        files::json_new(&dir.join("prepared.json"), &p).unwrap();
        (dir, p)
    }
    fn change(&self, name: &str, bytes: &[u8]) {
        write(&self.ctx().game().join(name), bytes);
    }
    fn assert_original(&self) {
        for (name, bytes) in &self.original {
            assert_eq!(
                fs::read(self.ctx().game().join(name)).unwrap(),
                *bytes,
                "{name}"
            );
        }
        assert_eq!(
            files::hash(&self.record.managed_root.join(".chriz/install-receipt.json")).unwrap(),
            self.record.receipt_sha256
        );
        assert_eq!(
            fs::read(self.record.managed_save_root.join("save/one.gam")).unwrap(),
            b"player save"
        );
    }
}

#[test]
fn success_records_partial_version_and_undo_restores_without_changing_base_or_saves() {
    let f = Fixture::new();
    let (dir, p) = f.prepare(1);
    assert!(launch_guard(&f.record.managed_root).is_err());
    f.change("override/bfbotscn.lua", b"new code");
    f.change("weidu.log", b"new mod version");
    finish(&f.ctx(), &dir, &p).unwrap();
    let u = active_upgrade(&f.record.managed_root).unwrap().unwrap();
    assert_eq!(u.version, buffbot::NEW_VERSION);
    assert_eq!(u.log_sha256, sha256_bytes(b"new mod version"));
    restore_files(&f.ctx(), &dir, true).unwrap();
    f.assert_original();
    assert!(active_upgrade(&f.record.managed_root).unwrap().is_none());
    assert_eq!(
        history(&f.record.managed_root).unwrap()[0].state,
        "restored"
    );
    let (next, np) = f.prepare(2);
    finish(&f.ctx(), &next, &np).unwrap();
    assert_eq!(history(&f.record.managed_root).unwrap().len(), 2);
    assert!(transaction::check_restore(&f.ctx(), &dir, true).is_err());
}
#[test]
fn interrupted_update_can_restore_deleted_files_and_remove_only_owned_new_files() {
    let f = Fixture::new();
    let (dir, _) = f.prepare(1);
    fs::remove_file(f.ctx().game().join("override/bfbotscn.lua")).unwrap();
    f.change("buffbot/setup-buffbot.tp2", b"synthetic new source");
    f.change("lang/en_us/dialog.tlk", b"partly written");
    restore_files(&f.ctx(), &dir, false).unwrap();
    f.assert_original();
    assert!(!f.ctx().game().join("buffbot").exists());
    launch_guard(&f.record.managed_root).unwrap();
}
#[test]
fn corrupted_backup_blocks_restore_before_any_overwrite() {
    let f = Fixture::new();
    let (dir, _) = f.prepare(1);
    f.change("override/bfbotscn.lua", b"partial");
    write(&dir.join("affected/weidu.log"), b"corrupt");
    assert!(restore_files(&f.ctx(), &dir, false).is_err());
    assert_eq!(
        fs::read(f.ctx().game().join("override/bfbotscn.lua")).unwrap(),
        b"partial"
    );
}
#[test]
fn unexpected_external_write_prevents_success_and_automatic_restore() {
    let f = Fixture::new();
    let (dir, p) = f.prepare(1);
    f.change("override/other-mod.lua", b"unexpected");
    assert!(finish(&f.ctx(), &dir, &p).is_err());
    assert!(restore_files(&f.ctx(), &dir, false).is_err());
    assert!(!dir.join("applied.json").exists());
}
#[test]
fn later_mod_or_manual_owned_file_change_prevents_undo() {
    let f = Fixture::new();
    let (dir, p) = f.prepare(1);
    finish(&f.ctx(), &dir, &p).unwrap();
    f.change("override/bfbotscn.lua", b"manual edit");
    assert!(restore_files(&f.ctx(), &dir, true).is_err());
    f.change("weidu.log", b"later mod");
    assert!(active_upgrade(&f.record.managed_root).is_err());
}
#[test]
fn played_game_runtime_resources_and_player_settings_are_preserved() {
    let f = Fixture::new();
    let (dir, p) = f.prepare(1);
    finish(&f.ctx(), &dir, &p).unwrap();
    f.change("override/bfbt01.spl", b"new generated preset");
    f.change("override/bfbotbg.mos", b"resized background");
    f.change("override/bfbot_presets/player.json", b"new user settings");
    restore_files(&f.ctx(), &dir, true).unwrap();
    assert_eq!(
        fs::read(f.ctx().game().join("override/bfbt01.spl")).unwrap(),
        b"new generated preset"
    );
    assert_eq!(
        fs::read(f.ctx().game().join("override/bfbot_presets/player.json")).unwrap(),
        b"new user settings"
    );
}
#[test]
fn history_rejects_missing_reordered_or_rebound_records() {
    let f = Fixture::new();
    let (dir, p) = f.prepare(1);
    finish(&f.ctx(), &dir, &p).unwrap();
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("prepared.json")).unwrap()).unwrap();
    value["plan"]["install_id"] = "different".into();
    write(
        &dir.join("prepared.json"),
        &serde_json::to_vec(&value).unwrap(),
    );
    assert!(launch_guard(&f.record.managed_root).is_err());
}
#[test]
fn incomplete_preparation_without_game_writes_does_not_block_play() {
    let f = Fixture::new();
    let dir = f.ctx().state().join("00000000000000000001");
    write(&dir.join("prepared.pending"), b"partial journal");
    launch_guard(&f.record.managed_root).unwrap();
    write(
        &dir.join("applied.pending"),
        b"partial completion with missing evidence",
    );
    assert!(launch_guard(&f.record.managed_root).is_err());
}
/// The loader settings are relied on but never written by the update. Turning the game log on
/// or off for crash reports changes only its line there and keeps Undo available; any other
/// change still blocks it.
#[test]
fn undo_accepts_a_game_log_switch_but_no_other_loader_change() {
    let ini = |log: &str, lua: &str| {
        format!("[General]\r\nExeNames=Baldur.exe\r\nLogFile={log}\r\nLuaLibrary={lua}\r\n")
    };
    let f = Fixture::new();
    f.change("infinityloader.ini", ini("", "lua51.dll").as_bytes());
    let (dir, p) = f.prepare(1);
    assert!(p.plan.hashes.contains_key("infinityloader.ini"));
    finish(&f.ctx(), &dir, &p).unwrap();

    f.change("infinityloader.ini", ini("", "").as_bytes());
    assert_eq!(
        transaction::check_restore(&f.ctx(), &dir, true).unwrap_err(),
        "File changed: infinityloader.ini"
    );
    f.change(
        "infinityloader.ini",
        ini("InfinityLoader.log", "").as_bytes(),
    );
    assert!(transaction::check_restore(&f.ctx(), &dir, true).is_err());
    f.change(
        "infinityloader.ini",
        ini("InfinityLoader.log", "lua51.dll").as_bytes(),
    );
    restore_files(&f.ctx(), &dir, true).unwrap();
    f.assert_original();
    assert_eq!(
        fs::read(f.ctx().game().join("infinityloader.ini")).unwrap(),
        ini("InfinityLoader.log", "lua51.dll").as_bytes()
    );
}
#[test]
fn tail_rejects_later_mods_wrong_versions_languages_or_order() {
    let good=b"~base.tp2~ #0 #1 // Base: 1\n~buffbot/setup-buffbot.tp2~ #0 #1 // Helper: v1.8.3-alpha\n~buffbot/setup-buffbot.tp2~ #0 #0 // Main: v1.8.3-alpha\n";
    verify_tail(good, buffbot::OLD_VERSION).unwrap();
    assert!(verify_tail(good, buffbot::NEW_VERSION).is_err());
    let mut later = good.to_vec();
    later.extend_from_slice(b"~later.tp2~ #0 #0 // later\n");
    assert!(verify_tail(&later, buffbot::OLD_VERSION).is_err());
    assert!(verify_tail(
        &String::from_utf8_lossy(good)
            .replace("#0 #1", "#1 #1")
            .into_bytes(),
        buffbot::OLD_VERSION
    )
    .is_err());
}

#[test]
fn restoring_a_missing_tlk_passes_the_real_restoration_close_check() {
    let f = Fixture::new();
    let (dir, p) = f.prepare(1);
    fs::remove_file(f.ctx().game().join("lang/en_us/dialog.tlk")).unwrap();
    let existing = p
        .plan
        .hashes
        .iter()
        .filter(|(n, _)| f.ctx().game().join(n).exists())
        .map(|(n, h)| (n.clone(), h.clone()))
        .collect();
    transaction::check_restore(&f.ctx(), &dir, false).unwrap();
    crate::patches::close_restore_check(&f.ctx().game(), &existing).unwrap();
    restore_files(&f.ctx(), &dir, false).unwrap();
    f.assert_original();
}

#[test]
fn interrupted_restoration_resumes_from_its_durable_before_record() {
    let f = Fixture::new();
    let (dir, _) = f.prepare(1);
    f.change("weidu.log", b"new version");
    f.change("override/bfbotscn.lua", b"new code");
    let before = files::inventory(&f.ctx().game(), false).unwrap();
    files::json_new(
        &dir.join("restoring.json"),
        &serde_json::json!({
            "prepared_sha256":files::hash(&dir.join("prepared.json")).unwrap(),
            "before":before,"removable":{}
        }),
    )
    .unwrap();
    f.change("weidu.log", &f.original["weidu.log"]);
    assert!(launch_guard(&f.record.managed_root).is_err());
    restore_files(&f.ctx(), &dir, false).unwrap();
    f.assert_original();
}

#[test]
fn undo_accepts_identical_bytes_restored_by_a_later_undone_fix() {
    let f = Fixture::new();
    let (dir, p) = f.prepare(1);
    finish(&f.ctx(), &dir, &p).unwrap();
    f.change(
        "lang/en_us/dialog.tlk",
        &f.original["lang/en_us/dialog.tlk"],
    );
    restore_files(&f.ctx(), &dir, true).unwrap();
    f.assert_original();
}

#[test]
fn review_token_changes_for_owned_bytes_selection_recipe_and_history() {
    let f = Fixture::new();
    let (_, p) = f.prepare(1);
    let expected = token(&p.plan).unwrap();
    let mut changed = p.plan.clone();
    changed.selection_sha256.push('x');
    assert_ne!(token(&changed).unwrap(), expected);
    let mut changed = p.plan.clone();
    changed.recipe_sha256.push('x');
    assert_ne!(token(&changed).unwrap(), expected);
    let mut changed = p.plan.clone();
    changed.previous = Some("new history".into());
    assert_ne!(token(&changed).unwrap(), expected);
    let mut changed = p.plan.clone();
    changed
        .hashes
        .insert("weidu.log".into(), "different".into());
    assert_ne!(token(&changed).unwrap(), expected);
}

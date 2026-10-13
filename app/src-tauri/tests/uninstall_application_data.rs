//! The uninstaller's removal of application data, through the real app executable and, when
//! Tauri's NSIS compiler is available, through the real uninstaller hook.
//!
//! Only development builds of the app are run: they remove nothing but the folders a test
//! declares. A release build would remove the real application data of whoever runs the tests.
#![cfg(all(windows, debug_assertions))]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use chriz_bg_app_lib::application_data::{
    IDENTIFIER, NOT_REMOVED, REFUSED, REMOVED, REMOVE_APPLICATION_DATA,
};

const APP: &str = env!("CARGO_BIN_EXE_chriz-bg-app");
const TEST_PARENTS: &str = "CHRIZ_BG_COLLECTION_TEST_APPLICATION_DATA_PARENTS";

/// Disposable stand-ins for the user's roaming and local application data folders, and a game
/// folder outside them that a junction in the download cache points to.
struct Profile {
    _temp: tempfile::TempDir,
    roaming: PathBuf,
    local: PathBuf,
    game_save: PathBuf,
    neighbours: Vec<PathBuf>,
}

impl Profile {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let roaming = temp.path().join("Roaming");
        let local = temp.path().join("Local");
        let game = temp.path().join("Games").join("Chriz Easy BG");
        fs::create_dir_all(&game).unwrap();
        let game_save = game.join("baldur.sav");
        fs::write(&game_save, b"keep").unwrap();

        fs::create_dir_all(roaming.join(IDENTIFIER)).unwrap();
        fs::write(roaming.join(IDENTIFIER).join("settings.json"), b"{}").unwrap();
        let cache = local.join(IDENTIFIER);
        let mut deepest = cache
            .join("extracted/sha256-v3/ab")
            .join("a".repeat(64))
            .join("c".repeat(64));
        while deepest.as_os_str().len() < 400 {
            deepest = deepest.join("nested-directory-level");
        }
        fs::create_dir_all(&deepest).unwrap();
        fs::write(deepest.join("setup-mod.tp2"), b"BACKUP ~mod/backup~\n").unwrap();
        fs::create_dir_all(cache.join("EBWebView")).unwrap();
        fs::write(cache.join("EBWebView").join("Local State"), b"{}").unwrap();
        // Sorted last, so it is still there when removal stops at a held file.
        junction(&cache.join("z-linked-game"), &game);

        // Other folders in the same parents belong to other programs or to CEBG's install
        // registry, which the uninstaller does not delete.
        let neighbours = vec![
            local.join("Chriz BG Collection").join("registry.json"),
            local.join("Chriz Easy BG").join("chriz-bg-app.exe"),
            roaming.join("dev.example.other").join("settings.json"),
        ];
        for neighbour in &neighbours {
            fs::create_dir_all(neighbour.parent().unwrap()).unwrap();
            fs::write(neighbour, b"keep").unwrap();
        }
        Self {
            _temp: temp,
            roaming,
            local,
            game_save,
            neighbours,
        }
    }

    fn parents(&self) -> std::ffi::OsString {
        std::env::join_paths([&self.roaming, &self.local]).unwrap()
    }

    fn held_file(&self) -> PathBuf {
        let folder = self.local.join(IDENTIFIER).join("a-held");
        fs::create_dir_all(&folder).unwrap();
        let file = folder.join("in-use.dat");
        fs::write(&file, b"in use").unwrap();
        file
    }

    fn assert_untouched_outside(&self) {
        assert_eq!(fs::read(&self.game_save).unwrap(), b"keep");
        for neighbour in &self.neighbours {
            assert_eq!(
                fs::read(neighbour).unwrap(),
                b"keep",
                "{}",
                neighbour.display()
            );
        }
    }

    fn assert_kept(&self) {
        assert!(self
            .roaming
            .join(IDENTIFIER)
            .join("settings.json")
            .is_file());
        assert!(self
            .local
            .join(IDENTIFIER)
            .join("EBWebView")
            .join("Local State")
            .is_file());
        assert!(fs::symlink_metadata(self.local.join(IDENTIFIER).join("z-linked-game")).is_ok());
        self.assert_untouched_outside();
    }

    fn assert_removed(&self) {
        assert!(fs::symlink_metadata(self.roaming.join(IDENTIFIER)).is_err());
        assert!(fs::symlink_metadata(self.local.join(IDENTIFIER)).is_err());
        self.assert_untouched_outside();
    }
}

fn junction(link: &Path, target: &Path) {
    let output = Command::new("cmd.exe")
        .args(["/c", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
}

/// Opens `path` so that it cannot be deleted until the handle is closed.
fn hold(path: &Path) -> fs::File {
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_SHARE_READ: u32 = 1;
    fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(path)
        .unwrap()
}

/// Runs `command` and fails instead of hanging if it does not finish, for example because the
/// app started its window.
fn finish(command: &mut Command) -> Output {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("{command:?} did not finish");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    child.wait_with_output().unwrap()
}

fn run_app(arguments: &[&str], parents: Option<&std::ffi::OsStr>) -> Output {
    let mut command = Command::new(APP);
    command.args(arguments).env_remove(TEST_PARENTS);
    if let Some(parents) = parents {
        command.env(TEST_PARENTS, parents);
    }
    finish(&mut command)
}

#[test]
fn the_uninstaller_request_removes_both_folders_without_following_junctions() {
    let profile = Profile::new();

    let output = run_app(
        &[REMOVE_APPLICATION_DATA, IDENTIFIER],
        Some(&profile.parents()),
    );

    assert_eq!(output.status.code(), Some(REMOVED), "{output:?}");
    profile.assert_removed();
}

#[test]
fn any_other_request_touches_nothing() {
    let profile = Profile::new();
    for arguments in [
        vec![REMOVE_APPLICATION_DATA],
        vec![REMOVE_APPLICATION_DATA, "dev.example.other"],
        vec![REMOVE_APPLICATION_DATA, IDENTIFIER, "--force"],
    ] {
        let output = run_app(&arguments, Some(&profile.parents()));
        assert_eq!(
            output.status.code(),
            Some(REFUSED),
            "{arguments:?}: {output:?}"
        );
        profile.assert_kept();
    }
    // A development build never removes the real application data folders.
    let output = run_app(&[REMOVE_APPLICATION_DATA, IDENTIFIER], None);
    assert_eq!(output.status.code(), Some(REFUSED), "{output:?}");
    profile.assert_kept();
}

#[test]
fn data_that_stays_in_use_is_reported() {
    let profile = Profile::new();
    let held_file = profile.held_file();
    let held = hold(&held_file);

    let output = run_app(
        &[REMOVE_APPLICATION_DATA, IDENTIFIER],
        Some(&profile.parents()),
    );

    drop(held);
    assert_eq!(output.status.code(), Some(NOT_REMOVED), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&profile.local.join(IDENTIFIER).display().to_string()),
        "{stderr}"
    );
    assert!(held_file.is_file());
    assert!(fs::symlink_metadata(profile.roaming.join(IDENTIFIER)).is_err());
    profile.assert_untouched_outside();
}

/// The uninstaller around the hook: the template's own deletion of the application data runs
/// after `NSIS_HOOK_PREUNINSTALL`, against the disposable profile instead of the real folders.
const HARNESS: &str = r#"Unicode true
!include LogicLib.nsh
!include FileFunc.nsh
Name "cebg-uninstall-hook-test"
OutFile "${OUT}"
RequestExecutionLevel user
SilentInstall silent
!define MAINBINARYNAME "chriz-bg-app"
!define PRODUCTNAME "Chriz Easy BG"
!define BUNDLEID "dev.chrizhermann.bgcollection"
Var DeleteAppDataCheckboxState
Var UpdateMode
; The template closes a running app here. The test records that the hook asked for it.
!macro CheckIfAppIsRunning executableName productName
  FileOpen $9 "$EXEDIR\closed-app.txt" a
  FileWrite $9 "${executableName}|${productName}$\r$\n"
  FileClose $9
!macroend
!include "${HOOKS}"
Section
  ${GetParameters} $R0
  ${GetOptions} $R0 "/INSTDIR=" $INSTDIR
  ${GetOptions} $R0 "/CHECKED=" $DeleteAppDataCheckboxState
  ${GetOptions} $R0 "/UPDATE=" $UpdateMode
  ${GetOptions} $R0 "/ROAMING=" $R1
  ${GetOptions} $R0 "/LOCAL=" $R2
  !insertmacro NSIS_HOOK_PREUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    RmDir /r "$R1\${BUNDLEID}"
    RmDir /r "$R2\${BUNDLEID}"
  ${EndIf}
SectionEnd
"#;

/// Runs the real hook in a compiled harness, never the real uninstaller.
#[test]
#[ignore = "requires CEBG_MAKENSIS, the makensis.exe of Tauri's NSIS tools"]
fn the_uninstaller_hook_removes_application_data_through_the_app_and_never_follows_junctions() {
    let makensis = PathBuf::from(std::env::var_os("CEBG_MAKENSIS").expect("set CEBG_MAKENSIS"));
    let temp = tempfile::tempdir().unwrap();
    let hooks = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("windows")
        .join("hooks.nsh");
    let script = temp.path().join("harness.nsi");
    let harness = temp.path().join("harness.exe");
    fs::write(&script, HARNESS).unwrap();
    let compiled = finish(
        Command::new(&makensis)
            .arg("-V2")
            .arg(format!("-DOUT={}", harness.display()))
            .arg(format!("-DHOOKS={}", hooks.display()))
            .arg(&script),
    );
    assert!(compiled.status.success(), "{compiled:?}");

    let run = |profile: &Profile, install: &Path, checked: &str, update: &str| {
        let closed = temp.path().join("closed-app.txt");
        let _ = fs::remove_file(&closed);
        let output = finish(
            Command::new(&harness)
                .env(TEST_PARENTS, profile.parents())
                .arg(format!("/INSTDIR={}", install.display()))
                .arg(format!("/CHECKED={checked}"))
                .arg(format!("/UPDATE={update}"))
                .arg(format!("/ROAMING={}", profile.roaming.display()))
                .arg(format!("/LOCAL={}", profile.local.display())),
        );
        assert!(output.status.success(), "{output:?}");
        fs::read_to_string(closed).ok()
    };
    let installed = temp.path().join("installed");
    fs::create_dir(&installed).unwrap();
    fs::copy(APP, installed.join("chriz-bg-app.exe")).unwrap();

    let profile = Profile::new();
    assert_eq!(
        run(&profile, &installed, "1", "0").as_deref(),
        Some("chriz-bg-app.exe|Chriz Easy BG\r\n")
    );
    profile.assert_removed();

    // Unchecked, or uninstalling for an update: nothing is removed and the app is not asked.
    let profile = Profile::new();
    assert_eq!(run(&profile, &installed, "0", "0"), None);
    profile.assert_kept();
    assert_eq!(run(&profile, &installed, "1", "1"), None);
    profile.assert_kept();

    // When the app cannot remove everything, RMDir /r never runs on what is left, so the
    // junction to the game folder is not followed.
    let held_file = profile.held_file();
    let held = hold(&held_file);
    run(&profile, &installed, "1", "0");
    drop(held);
    assert!(held_file.is_file());
    assert!(fs::symlink_metadata(profile.local.join(IDENTIFIER).join("z-linked-game")).is_ok());
    profile.assert_untouched_outside();

    // The same holds when the app cannot run at all.
    let profile = Profile::new();
    run(&profile, &temp.path().join("missing"), "1", "0");
    profile.assert_kept();
}

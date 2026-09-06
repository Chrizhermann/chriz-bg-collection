//! Narrow, audited compatibility fixes to a staged EET installer, never its archive/cache.

use std::fs;
use std::io::Write;
use std::path::Path;

use serde::Serialize;

use crate::digest::sha256_bytes;
use crate::manifest::GameRoot;

const ARTIFACT_ID: &str = "eet-official-74e91d72bca5d073fa11c1d088b90d7ff0c7105d";
const ARCHIVE_SHA256: &str = "9834a53322b7fe9d8923bfde36c0e6bc8bef54d730d32ac29286b80d6f08287e";
// Full upstream EET/lib/macros.tph, LF bytes, at the exact artifact commit above.
const ORIGINAL_SHA256: &str = "f7c6fc721f05d7bb38149f8f935153f80f6f4b0040df5ea637a5d7b955287168";
const CORRECTED_SHA256: &str = "1bd191c84ff6c890a5df5d8b556349ec54a996812d0d5f50998f9b86d524ef89";

pub(crate) struct EetCompatibilityInput<'a> {
    pub mod_id: &'a str,
    pub artifact_id: &'a str,
    pub artifact_sha256: &'a str,
    pub target: GameRoot,
    pub tp2: &'a str,
    pub components: &'a [u32],
    pub target_root: &'a Path,
}

/// Written into each affected invocation's evidence directory, including old frozen retries.
#[derive(Debug, Serialize)]
pub(crate) struct EetCompatibilityEvidence {
    fix_id: &'static str,
    artifact_id: &'static str,
    archive_sha256: &'static str,
    relative_path: &'static str,
    upstream_lf_sha256: &'static str,
    corrected_lf_sha256: &'static str,
    observed_before_sha256: String,
    observed_after_sha256: String,
    changed: bool,
}

/// Apply only to the known Windows EET core, after source verification/materialization.
/// Neither the original download nor extracted cache is rewritten.
pub(crate) fn prepare(
    input: EetCompatibilityInput<'_>,
) -> Result<Option<EetCompatibilityEvidence>, String> {
    if !cfg!(windows)
        || input.mod_id != "eet"
        || input.target != GameRoot::Bg2
        || !input.components.contains(&0)
        || input.tp2.replace('\\', "/").to_ascii_lowercase() != "eet/eet.tp2"
        || input.artifact_id != ARTIFACT_ID
        || !input.artifact_sha256.eq_ignore_ascii_case(ARCHIVE_SHA256)
    {
        return Ok(None);
    }
    let (before, after) = patch_file(input.target_root, ORIGINAL_SHA256, CORRECTED_SHA256)?;
    Ok(Some(EetCompatibilityEvidence {
        fix_id: "eet-windows-documents-path-v1",
        artifact_id: ARTIFACT_ID,
        archive_sha256: ARCHIVE_SHA256,
        relative_path: "EET/lib/macros.tph",
        upstream_lf_sha256: ORIGINAL_SHA256,
        corrected_lf_sha256: CORRECTED_SHA256,
        changed: before != after,
        observed_before_sha256: before,
        observed_after_sha256: after,
    }))
}

// Upstream match text: EET/lib/macros.tph at 74e91d72bca5d073fa11c1d088b90d7ff0c7105d.
// GET_USER_DIRECTORY was contributed by Argent77 in upstream commit 38db44c2 (2024).
// See THIRD_PARTY_NOTICES.md and docs/audits/2026-09-07-eet-attribution.md for provenance
// and the upstream GPLv3 statement. CEBG's MIT license does not relicense this excerpt.
// CORRECTED_BATCH records CEBG's 2026-09-07 correction to the Windows path handling.
const ORIGINAL_BATCH: &str = r#"@echo off
for /f "tokens=3,*" %%a in ('reg query "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders" /v Personal') do set DOC_DIR=%%a
for /f "delims=" %%a in ('echo %DOC_DIR%') do echo %%a>"${temp_directory}\documents_path.txt""#;

const CORRECTED_BATCH: &str = r#"@echo off
setlocal DisableDelayedExpansion
for /f "tokens=2,*" %%a in ('reg query "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders" /v Personal') do set "DOC_DIR=%%b"
for /f "delims=" %%a in ('echo "%DOC_DIR%"') do >"${temp_directory}\documents_path.txt" echo %%~a
endlocal"#;

fn corrected_bytes(
    input: &[u8],
    original_sha256: &str,
    corrected_sha256: &str,
) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(input)
        .map_err(|error| format!("EET compatibility source is not UTF-8: {error}"))?;
    let normalized = text.replace("\r\n", "\n");
    let crlf = text.contains("\r\n");
    if crlf && normalized.replace('\n', "\r\n") != text {
        return Err(
            "EET compatibility source has unrecognized mixed line endings; file left unchanged"
                .into(),
        );
    }
    let digest = sha256_bytes(normalized.as_bytes());
    if digest == corrected_sha256 {
        return Ok(input.to_vec());
    }
    if digest != original_sha256 || normalized.matches(ORIGINAL_BATCH).count() != 1 {
        return Err(format!("EET Documents-path compatibility fix does not recognize macros.tph (SHA-256 {digest}); file left unchanged"));
    }
    let corrected = normalized.replacen(ORIGINAL_BATCH, CORRECTED_BATCH, 1);
    if sha256_bytes(corrected.as_bytes()) != corrected_sha256 {
        return Err("EET Documents-path compatibility result did not match its reviewed hash; file left unchanged".into());
    }
    Ok(if crlf {
        corrected.replace('\n', "\r\n").into_bytes()
    } else {
        corrected.into_bytes()
    })
}

fn patch_file(
    target_root: &Path,
    original_sha256: &str,
    corrected_sha256: &str,
) -> Result<(String, String), String> {
    let path = target_root.join("EET/lib/macros.tph");
    if !target_root.is_absolute() {
        return Err("EET compatibility target must be absolute".into());
    }
    // Check ancestors as well as the leaf: a junction must not redirect this scoped write.
    for ancestor in path.ancestors() {
        let metadata = fs::symlink_metadata(ancestor).map_err(|error| {
            format!(
                "Inspect EET compatibility path {}: {error}",
                ancestor.display()
            )
        })?;
        #[cfg(windows)]
        let reparse = {
            use std::os::windows::fs::MetadataExt;
            metadata.file_attributes() & 0x400 != 0
        };
        #[cfg(not(windows))]
        let reparse = false;
        if metadata.file_type().is_symlink()
            || reparse
            || (ancestor == path && !metadata.is_file())
            || (ancestor != path && !metadata.is_dir())
        {
            return Err(format!(
                "EET compatibility path is not a direct file/directory: {}",
                ancestor.display()
            ));
        }
    }
    let before = fs::read(&path).map_err(|error| error.to_string())?;
    let after = corrected_bytes(&before, original_sha256, corrected_sha256)?;
    if before != after {
        let mut temporary = tempfile::NamedTempFile::new_in(path.parent().unwrap())
            .map_err(|error| error.to_string())?;
        temporary
            .write_all(&after)
            .map_err(|error| error.to_string())?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|error| error.to_string())?;
        temporary
            .persist(&path)
            .map_err(|error| error.to_string())?;
    }
    Ok((sha256_bytes(&before), sha256_bytes(&after)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (String, String) {
        let original = format!("unrelated prefix\n{ORIGINAL_BATCH}\nunrelated suffix\n");
        let corrected = original.replace(ORIGINAL_BATCH, CORRECTED_BATCH);
        (original, corrected)
    }

    #[test]
    fn eet_compat_changes_only_the_known_batch_and_is_idempotent() {
        let (original, corrected) = fixture();
        let old_hash = sha256_bytes(original.as_bytes());
        let new_hash = sha256_bytes(corrected.as_bytes());
        let first = corrected_bytes(original.as_bytes(), &old_hash, &new_hash).unwrap();
        assert_eq!(first, corrected.as_bytes());
        assert_eq!(
            corrected_bytes(&first, &old_hash, &new_hash).unwrap(),
            first
        );
    }

    #[test]
    fn eet_compat_accepts_crlf_without_changing_other_line_endings() {
        let (original, corrected) = fixture();
        let result = corrected_bytes(
            original.replace('\n', "\r\n").as_bytes(),
            &sha256_bytes(original.as_bytes()),
            &sha256_bytes(corrected.as_bytes()),
        )
        .unwrap();
        assert_eq!(result, corrected.replace('\n', "\r\n").as_bytes());
    }

    #[test]
    fn eet_compat_rejects_unknown_source_even_when_the_batch_matches() {
        let (original, corrected) = fixture();
        assert!(corrected_bytes(
            format!("{original}extra edit").as_bytes(),
            &sha256_bytes(original.as_bytes()),
            &sha256_bytes(corrected.as_bytes())
        )
        .is_err());
    }

    #[test]
    fn eet_compat_fresh_then_retry_keeps_source_cache_unchanged() {
        let (original, corrected) = fixture();
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("staged");
        std::fs::create_dir_all(target.join("EET/lib")).unwrap();
        let cache = temp.path().join("verified-cache-macros.tph");
        std::fs::write(&cache, &original).unwrap();
        std::fs::copy(&cache, target.join("EET/lib/macros.tph")).unwrap();
        let old = sha256_bytes(original.as_bytes());
        let new = sha256_bytes(corrected.as_bytes());
        assert_eq!(
            patch_file(&target, &old, &new).unwrap(),
            (old.clone(), new.clone())
        );
        assert_eq!(patch_file(&target, &old, &new).unwrap(), (new.clone(), new));
        assert_eq!(std::fs::read_to_string(cache).unwrap(), original);
    }

    #[test]
    fn eet_compat_other_mods_components_and_artifacts_do_not_touch_files() {
        let temp = tempfile::tempdir().unwrap();
        for (mod_id, artifact_id, hash, target, tp2, components) in [
            (
                "other",
                ARTIFACT_ID,
                ARCHIVE_SHA256,
                GameRoot::Bg2,
                "EET/EET.tp2",
                &[0][..],
            ),
            (
                "eet",
                "other-version",
                ARCHIVE_SHA256,
                GameRoot::Bg2,
                "EET/EET.tp2",
                &[0][..],
            ),
            (
                "eet",
                ARTIFACT_ID,
                "wrong-hash",
                GameRoot::Bg2,
                "EET/EET.tp2",
                &[0][..],
            ),
            (
                "eet",
                ARTIFACT_ID,
                ARCHIVE_SHA256,
                GameRoot::Bg1,
                "EET/EET.tp2",
                &[0][..],
            ),
            (
                "eet",
                ARTIFACT_ID,
                ARCHIVE_SHA256,
                GameRoot::Bg2,
                "other.tp2",
                &[0][..],
            ),
            (
                "eet",
                ARTIFACT_ID,
                ARCHIVE_SHA256,
                GameRoot::Bg2,
                "EET/EET.tp2",
                &[100][..],
            ),
        ] {
            assert!(prepare(EetCompatibilityInput {
                mod_id,
                artifact_id,
                artifact_sha256: hash,
                target,
                tp2,
                components,
                target_root: temp.path()
            })
            .unwrap()
            .is_none());
        }
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn eet_compat_unknown_file_is_not_modified() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("EET/lib")).unwrap();
        let path = temp.path().join("EET/lib/macros.tph");
        std::fs::write(&path, "user changes").unwrap();
        assert!(patch_file(temp.path(), ORIGINAL_SHA256, CORRECTED_SHA256).is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), "user changes");
    }

    /// Opt-in acceptance against locally verified upstream files, with no network or game.
    #[cfg(windows)]
    #[test]
    #[ignore = "requires CEBG_EET_SOURCE and CEBG_WEIDU_EXE verified local files"]
    fn eet_compat_official_source_and_real_weidu_macro_smoke() {
        use std::process::Command;
        let source = std::env::var_os("CEBG_EET_SOURCE").expect("CEBG_EET_SOURCE");
        let weidu = std::env::var_os("CEBG_WEIDU_EXE").expect("CEBG_WEIDU_EXE");
        assert_eq!(
            sha256_bytes(&std::fs::read(&source).unwrap()),
            ORIGINAL_SHA256
        );
        assert_eq!(
            sha256_bytes(&std::fs::read(&weidu).unwrap()),
            "ad70f5897a6d0ba4b0d226f845a9b14cf345f56cc9697ca8d05cac9fe4932c1a"
        );
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("EET/lib")).unwrap();
        let staged = temp.path().join("EET/lib/macros.tph");
        std::fs::copy(&source, &staged).unwrap();
        for changed in [true, false] {
            let evidence = prepare(EetCompatibilityInput {
                mod_id: "eet",
                artifact_id: ARTIFACT_ID,
                artifact_sha256: ARCHIVE_SHA256,
                target: GameRoot::Bg2,
                tp2: "EET/EET.tp2",
                components: &[0, 100],
                target_root: temp.path(),
            })
            .unwrap()
            .unwrap();
            assert_eq!(evidence.changed, changed);
            assert_eq!(evidence.observed_after_sha256, CORRECTED_SHA256);
        }
        assert_eq!(
            sha256_bytes(&std::fs::read(&source).unwrap()),
            ORIGINAL_SHA256
        );
        let corrected = std::fs::read_to_string(staged).unwrap();
        // Execute the actual first macro, replacing only its registry read with a
        // synthetic output file. No registry keys or real Documents folders change.
        let first_macro = corrected
            .split("DEFINE_ACTION_FUNCTION GET_SYSTEM_ARCH")
            .next()
            .unwrap();
        let query = "reg query \"HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders\" /v Personal";
        let fixture_macro = first_macro.replace(query, "type registry.txt");
        std::fs::write(temp.path().join("macro-under-test.tph"), fixture_macro).unwrap();
        std::fs::write(
            temp.path().join("registry.txt"),
            "\r\nHKEY_TEST\r\n    Personal    REG_EXPAND_SZ    %CEBG_TEST_PROFILE%\\Documents\r\n",
        )
        .unwrap();
        std::fs::write(
            temp.path().join("harness.tp2"),
            r#"BACKUP ~backup~
AUTHOR ~CEBG fixture~
BEGIN ~EET Documents path fixture~
INCLUDE ~macro-under-test.tph~
LAF GET_USER_DIRECTORY STR_VAR temp_directory = ~scratch~ RET user_directory END
PRINT ~CEBG_DOCUMENTS=%user_directory%~
"#,
        )
        .unwrap();
        let output = Command::new(weidu)
            .current_dir(temp.path())
            .args([
                "harness.tp2",
                "--nogame",
                "--force-install-list",
                "0",
                "--no-exit-pause",
                "--noautoupdate",
                "--quick-log",
            ])
            .env("CEBG_TEST_PROFILE", r"C:\Users\Dan & Jane!")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "stdout: {stdout}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            stdout.contains(
                r"CEBG_DOCUMENTS=C:\Users\Dan & Jane!\Documents\Baldur's Gate II - Enhanced Edition"
            ),
            "{stdout}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn eet_compat_batch_preserves_spaces_metacharacters_and_expanded_variables() {
        use std::process::Command;
        let temp = tempfile::tempdir().unwrap();
        let registry_line = temp.path().join("registry.txt");
        let output_path = temp.path().join("documents_path.txt");
        let batch_path = temp.path().join("probe.bat");
        let query = "reg query \"HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders\" /v Personal";
        let batch = CORRECTED_BATCH
            .replace(query, &format!("type \"{}\"", registry_line.display()))
            .replace("${temp_directory}", &temp.path().display().to_string());
        std::fs::write(&batch_path, batch.replace('\n', "\r\n")).unwrap();
        for value in [
            r"C:\Users\Dan Example\Documents",
            r"C:\Users\Dan & Jane!\Documents",
            r"%CEBG_TEST_PROFILE%\Documents",
        ] {
            std::fs::write(
                &registry_line,
                format!("\r\nHKEY_TEST\r\n    Personal    REG_EXPAND_SZ    {value}\r\n"),
            )
            .unwrap();
            let output = Command::new("cmd.exe")
                .args(["/d", "/v:off", "/c"])
                .arg(&batch_path)
                .env("CEBG_TEST_PROFILE", r"C:\Users\Dan & Jane!")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                std::fs::read_to_string(&output_path).unwrap().trim_end(),
                value.replace("%CEBG_TEST_PROFILE%", r"C:\Users\Dan & Jane!")
            );
        }
    }
}

//! Narrow, audited compatibility fixes to a staged EET installer, never its archive/cache.

use std::io::Read;
use std::path::Path;

use serde::Serialize;

use crate::digest::sha256_bytes;
use crate::held_folder::{EntryError, EntryErrorKind, HeldFolder};
use crate::manifest::GameRoot;

#[cfg_attr(not(windows), allow(dead_code))]
const ARTIFACT_ID: &str = "eet-official-74e91d72bca5d073fa11c1d088b90d7ff0c7105d";
#[cfg_attr(not(windows), allow(dead_code))]
const ARCHIVE_SHA256: &str = "9834a53322b7fe9d8923bfde36c0e6bc8bef54d730d32ac29286b80d6f08287e";
// Full upstream EET/lib/macros.tph, LF bytes, at the exact artifact commit above.
#[cfg_attr(not(windows), allow(dead_code))]
const ORIGINAL_SHA256: &str = "f7c6fc721f05d7bb38149f8f935153f80f6f4b0040df5ea637a5d7b955287168";
#[cfg_attr(not(windows), allow(dead_code))]
const CORRECTED_SHA256: &str = "1bd191c84ff6c890a5df5d8b556349ec54a996812d0d5f50998f9b86d524ef89";

pub(crate) struct EetCompatibilityInput<'a> {
    pub mod_id: &'a str,
    pub artifact_id: &'a str,
    /// SHA-256 of the archive the staged EET files came from.
    pub artifact_sha256: &'a str,
    /// The official link served a different archive than the reviewed pin, so the staged
    /// files may be a newer EET release.
    #[cfg_attr(not(windows), allow(dead_code))]
    pub archive_drifted: bool,
    pub target: GameRoot,
    pub tp2: &'a str,
    pub components: &'a [u32],
    pub target_root: &'a Path,
    /// Verified host WeiDU binary bytes, used on Unix to refresh `EET/bin/unix/.../weidu`.
    #[cfg_attr(not(unix), allow(dead_code))]
    pub weidu_bytes: Option<&'a [u8]>,
}

/// Written into each affected invocation's evidence directory, including old frozen retries.
#[derive(Debug, Serialize)]
pub(crate) struct EetCompatibilityEvidence {
    fix_id: &'static str,
    artifact_id: String,
    archive_sha256: String,
    relative_path: &'static str,
    upstream_lf_sha256: &'static str,
    corrected_lf_sha256: &'static str,
    observed_before_sha256: String,
    observed_after_sha256: String,
    changed: bool,
    /// Present only for a changed download or newer EET release: the fix then matched the faulty
    /// text alone, because the rest of the file may differ from the reviewed release.
    #[serde(skip_serializing_if = "Option::is_none")]
    matched: Option<&'static str>,
}

/// Apply platform-specific compatibility fixes to staged EET core after materialization.
/// Neither the original download nor extracted cache is rewritten.
///
/// On Windows, fixes `EET/lib/macros.tph`'s `reg query` batch snippet for user Documents discovery
/// (for newer drifted EET releases, only the exact faulty batch text is replaced).
/// On Unix, replaces EET's pre-251 `EET/bin/unix/**/weidu` helpers with the host's verified
/// WeiDU 251 binary (`0o755`) and marks bundled `lua` helpers executable so `AT_NOW` works on
/// case-sensitive Linux filesystems.
pub(crate) fn prepare(
    input: EetCompatibilityInput<'_>,
) -> Result<Option<EetCompatibilityEvidence>, String> {
    if input.mod_id != "eet"
        || input.target != GameRoot::Bg2
        || !input.components.contains(&0)
        || !input
            .tp2
            .replace('\\', "/")
            .eq_ignore_ascii_case("eet/eet.tp2")
    {
        return Ok(None);
    }
    #[cfg(windows)]
    {
        prepare_windows(input)
    }
    #[cfg(not(windows))]
    {
        prepare_unix(input)
    }
}

#[cfg(windows)]
fn prepare_windows(
    input: EetCompatibilityInput<'_>,
) -> Result<Option<EetCompatibilityEvidence>, String> {
    let use_exact_reviewed_patch = !input.archive_drifted
        && input.artifact_id == ARTIFACT_ID
        && input.artifact_sha256.eq_ignore_ascii_case(ARCHIVE_SHA256);
    let matched = (!use_exact_reviewed_patch).then_some("faulty-text");
    let Some((before, after)) = (if use_exact_reviewed_patch {
        Some(patch_file(
            input.target_root,
            ORIGINAL_SHA256,
            CORRECTED_SHA256,
        )?)
    } else {
        patch_drifted_file(input.target_root)?
    }) else {
        return Ok(None);
    };
    Ok(Some(EetCompatibilityEvidence {
        fix_id: "eet-windows-documents-path-v1",
        artifact_id: input.artifact_id.to_owned(),
        archive_sha256: input.artifact_sha256.to_ascii_lowercase(),
        relative_path: "EET/lib/macros.tph",
        upstream_lf_sha256: ORIGINAL_SHA256,
        corrected_lf_sha256: CORRECTED_SHA256,
        changed: before != after,
        observed_before_sha256: before,
        observed_after_sha256: after,
        matched,
    }))
}

#[cfg_attr(not(unix), allow(dead_code))]
fn prepare_unix(
    input: EetCompatibilityInput<'_>,
) -> Result<Option<EetCompatibilityEvidence>, String> {
    if !input.target_root.is_absolute() {
        return Err("EET compatibility target must be absolute".into());
    }
    let unix_dir = match HeldFolder::open(input.target_root, ["EET", "bin", "unix"]) {
        Ok(folder) => folder,
        Err(error) if matches!(error.kind, EntryErrorKind::Missing) => return Ok(None),
        Err(error) => return Err(path_error(error)),
    };
    if unix_dir.contains("lua").map_err(|error| {
        format!(
            "Inspect EET compatibility path {}: {error}",
            unix_dir.path().join("lua").display()
        )
    })? {
        unix_dir.set_executable("lua").map_err(path_error)?;
    }
    let mut recorded: Option<(&'static str, String, String)> = None;
    if let Some(weidu_bytes) = input.weidu_bytes {
        if let Some((before, after)) = replace_unix_weidu(&unix_dir, "weidu", weidu_bytes)? {
            recorded = Some(("EET/bin/unix/weidu", before, after));
        }
    }
    match unix_dir.open_below(["x86_64"]) {
        Ok(arch_dir) => {
            if arch_dir.contains("lua").map_err(|error| {
                format!(
                    "Inspect EET compatibility path {}: {error}",
                    arch_dir.path().join("lua").display()
                )
            })? {
                arch_dir.set_executable("lua").map_err(path_error)?;
            }
            if let Some(weidu_bytes) = input.weidu_bytes {
                if let Some((before, after)) = replace_unix_weidu(&arch_dir, "weidu", weidu_bytes)?
                {
                    recorded = Some(("EET/bin/unix/x86_64/weidu", before, after));
                }
            }
        }
        Err(error) if matches!(error.kind, EntryErrorKind::Missing) => {}
        Err(error) => return Err(path_error(error)),
    }
    let Some((relative_path, before, after)) = recorded else {
        return Ok(None);
    };
    Ok(Some(EetCompatibilityEvidence {
        fix_id: "eet-unix-bin-weidu-v1",
        artifact_id: input.artifact_id.to_owned(),
        archive_sha256: input.artifact_sha256.to_ascii_lowercase(),
        relative_path,
        upstream_lf_sha256: "",
        corrected_lf_sha256: "",
        changed: before != after,
        observed_before_sha256: before,
        observed_after_sha256: after,
        matched: None,
    }))
}

#[cfg_attr(not(unix), allow(dead_code))]
fn replace_unix_weidu(
    folder: &HeldFolder,
    name: &str,
    weidu_bytes: &[u8],
) -> Result<Option<(String, String)>, String> {
    let path = folder.path().join(name);
    let before = match folder.open_file(name) {
        Ok((file, _)) => {
            let mut bytes = Vec::new();
            (&file).read_to_end(&mut bytes).map_err(|error| {
                format!("Read EET compatibility path {}: {error}", path.display())
            })?;
            bytes
        }
        Err(error) if matches!(error.kind, EntryErrorKind::Missing) => return Ok(None),
        Err(error) => return Err(path_error(error)),
    };
    if before != weidu_bytes {
        folder.replace_file(name, weidu_bytes).map_err(|error| {
            format!("Replace EET compatibility path {}: {error}", path.display())
        })?;
    }
    folder.set_executable(name).map_err(path_error)?;
    Ok(Some((sha256_bytes(&before), sha256_bytes(weidu_bytes))))
}

/// Replace the faulty batch text of a newer EET release, when it is there exactly once.
/// Returns the before/after hashes, or `None` when the file is missing, already has the
/// corrected text, or lacks the faulty text: then nothing is written.
#[cfg_attr(not(windows), allow(dead_code))]
fn patch_drifted_file(target_root: &Path) -> Result<Option<(String, String)>, String> {
    if !target_root.is_absolute() {
        return Err("EET compatibility target must be absolute".into());
    }
    let folder = match HeldFolder::open(target_root, ["EET", "lib"]) {
        Ok(folder) => folder,
        Err(error) if matches!(error.kind, EntryErrorKind::Missing) => return Ok(None),
        Err(error) => return Err(path_error(error)),
    };
    let path = folder.path().join(MACROS);
    let before = match folder.open_file(MACROS) {
        Ok((file, _)) => {
            let mut bytes = Vec::new();
            (&file).read_to_end(&mut bytes).map_err(|error| {
                format!("Read EET compatibility path {}: {error}", path.display())
            })?;
            bytes
        }
        Err(error) if matches!(error.kind, EntryErrorKind::Missing) => return Ok(None),
        Err(error) => return Err(path_error(error)),
    };
    let Ok(text) = std::str::from_utf8(&before) else {
        return Ok(None);
    };
    let normalized = text.replace("\r\n", "\n");
    let crlf = text.contains("\r\n");
    if (crlf && normalized.replace('\n', "\r\n") != text)
        || normalized.matches(ORIGINAL_BATCH).count() != 1
    {
        return Ok(None);
    }
    let corrected = normalized.replacen(ORIGINAL_BATCH, CORRECTED_BATCH, 1);
    let after = if crlf {
        corrected.replace('\n', "\r\n").into_bytes()
    } else {
        corrected.into_bytes()
    };
    folder
        .replace_file(MACROS, &after)
        .map_err(|error| format!("Replace EET compatibility path {}: {error}", path.display()))?;
    Ok(Some((sha256_bytes(&before), sha256_bytes(&after))))
}

// Upstream match text: EET/lib/macros.tph at 74e91d72bca5d073fa11c1d088b90d7ff0c7105d.
// GET_USER_DIRECTORY was contributed by Argent77 in upstream commit 38db44c2 (2024).
// See THIRD_PARTY_NOTICES.md and docs/audits/2026-09-07-eet-attribution.md for provenance
// and the upstream GPLv3 statement. CEBG's MIT license does not relicense this excerpt.
// CORRECTED_BATCH records CEBG's 2026-09-07 correction to the Windows path handling.
#[cfg_attr(not(windows), allow(dead_code))]
const ORIGINAL_BATCH: &str = r#"@echo off
for /f "tokens=3,*" %%a in ('reg query "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders" /v Personal') do set DOC_DIR=%%a
for /f "delims=" %%a in ('echo %DOC_DIR%') do echo %%a>"${temp_directory}\documents_path.txt""#;

#[cfg_attr(not(windows), allow(dead_code))]
const CORRECTED_BATCH: &str = r#"@echo off
setlocal DisableDelayedExpansion
for /f "tokens=2,*" %%a in ('reg query "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders" /v Personal') do set "DOC_DIR=%%b"
for /f "delims=" %%a in ('echo "%DOC_DIR%"') do >"${temp_directory}\documents_path.txt" echo %%~a
endlocal"#;

#[cfg_attr(not(windows), allow(dead_code))]
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

#[cfg_attr(not(windows), allow(dead_code))]
fn patch_file(
    target_root: &Path,
    original_sha256: &str,
    corrected_sha256: &str,
) -> Result<(String, String), String> {
    if !target_root.is_absolute() {
        return Err("EET compatibility target must be absolute".into());
    }
    // EET and lib are opened from the target root down without following a link and held until
    // the file is replaced, so neither a junction nor a folder swapped for one can redirect this
    // scoped write. Nothing above the target root is inspected: once the root is held, a link
    // above it can no longer redirect anything.
    let folder = HeldFolder::open(target_root, ["EET", "lib"]).map_err(path_error)?;
    patch_in_folder(&folder, original_sha256, corrected_sha256)
}

/// Rewrite `macros.tph` in the held `EET/lib` folder, replacing it in one step.
#[cfg_attr(not(windows), allow(dead_code))]
fn patch_in_folder(
    folder: &HeldFolder,
    original_sha256: &str,
    corrected_sha256: &str,
) -> Result<(String, String), String> {
    let path = folder.path().join(MACROS);
    // The file is closed again before it is replaced: Windows does not replace a file that is
    // open without delete sharing.
    let before = {
        let (file, _) = folder.open_file(MACROS).map_err(path_error)?;
        let mut bytes = Vec::new();
        (&file)
            .read_to_end(&mut bytes)
            .map_err(|error| format!("Read EET compatibility path {}: {error}", path.display()))?;
        bytes
    };
    let after = corrected_bytes(&before, original_sha256, corrected_sha256)?;
    if before != after {
        folder.replace_file(MACROS, &after).map_err(|error| {
            format!("Replace EET compatibility path {}: {error}", path.display())
        })?;
    }
    Ok((sha256_bytes(&before), sha256_bytes(&after)))
}

#[cfg_attr(not(windows), allow(dead_code))]
const MACROS: &str = "macros.tph";

fn path_error(error: EntryError) -> String {
    let path = error.path.display();
    match error.kind {
        EntryErrorKind::Missing => format!("EET compatibility path is missing: {path}"),
        EntryErrorKind::NotDirect => {
            format!("EET compatibility path is not a direct file/directory: {path}")
        }
        EntryErrorKind::Io(error) => format!("Inspect EET compatibility path {path}: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::held_folder::tests::entries;
    #[cfg(windows)]
    use crate::held_folder::tests::junction;

    fn fixture() -> (String, String) {
        let original = format!("unrelated prefix\n{ORIGINAL_BATCH}\nunrelated suffix\n");
        let corrected = original.replace(ORIGINAL_BATCH, CORRECTED_BATCH);
        (original, corrected)
    }

    /// The fixture's original and corrected hashes.
    fn fixture_hashes() -> (String, String) {
        let (original, corrected) = fixture();
        (
            sha256_bytes(original.as_bytes()),
            sha256_bytes(corrected.as_bytes()),
        )
    }

    /// A staged `game` folder below `parent` whose macros.tph is the fixture's original.
    #[cfg_attr(not(windows), allow(dead_code))]
    fn staged_game(parent: &Path) -> std::path::PathBuf {
        let game = parent.join("game");
        std::fs::create_dir_all(game.join("EET").join("lib")).unwrap();
        std::fs::write(game.join("EET").join("lib").join(MACROS), fixture().0).unwrap();
        game
    }

    fn macros_text(game: &Path) -> String {
        std::fs::read_to_string(game.join("EET").join("lib").join(MACROS)).unwrap()
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
        // Replaced in place: the corrected file alone, with no temporary file left behind.
        assert_eq!(macros_text(&target), corrected);
        assert_eq!(entries(&target.join("EET/lib")), [MACROS]);
    }

    #[test]
    fn eet_compat_publishes_a_regular_file_and_leaves_no_temporary_behind() {
        let (original, corrected) = fixture();
        let temp = tempfile::tempdir().unwrap();
        let lib = temp.path().join("EET/lib");
        std::fs::create_dir_all(&lib).unwrap();
        std::fs::write(lib.join("macros.tph"), &original).unwrap();

        patch_file(
            temp.path(),
            &sha256_bytes(original.as_bytes()),
            &sha256_bytes(corrected.as_bytes()),
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(lib.join("macros.tph")).unwrap(),
            corrected
        );
        let names = std::fs::read_dir(&lib)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        assert_eq!(names, ["macros.tph"]);
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_TEMPORARY;

            let attributes = std::fs::metadata(lib.join("macros.tph"))
                .unwrap()
                .file_attributes();
            assert_eq!(attributes & FILE_ATTRIBUTE_TEMPORARY, 0);
        }
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
                archive_drifted: false,
                target,
                tp2,
                components,
                target_root: temp.path(),
                weidu_bytes: None,
            })
            .unwrap()
            .is_none());
        }
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[cfg(windows)]
    #[test]
    fn eet_compat_patches_other_artifact_id_when_faulty_batch_present() {
        let temp = tempfile::tempdir().unwrap();
        let macros = temp.path().join("EET/lib/macros.tph");
        std::fs::create_dir_all(macros.parent().unwrap()).unwrap();
        let original = format!("// custom header\n{ORIGINAL_BATCH}// custom footer\n");
        std::fs::write(&macros, original.as_bytes()).unwrap();

        let evidence = prepare(EetCompatibilityInput {
            mod_id: "eet",
            artifact_id: "eet-v14-custom",
            artifact_sha256: "cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34",
            archive_drifted: false,
            target: GameRoot::Bg2,
            tp2: "EET/EET.tp2",
            components: &[0],
            target_root: temp.path(),
            weidu_bytes: None,
        })
        .unwrap()
        .expect("custom EET artifact with faulty batch should be patched");

        assert_eq!(evidence.artifact_id, "eet-v14-custom");
        assert!(evidence.changed);
        assert_eq!(evidence.matched, Some("faulty-text"));
        assert_eq!(
            std::fs::read_to_string(&macros).unwrap(),
            format!("// custom header\n{CORRECTED_BATCH}// custom footer\n")
        );
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    fn drifted_input(target_root: &Path) -> EetCompatibilityInput<'_> {
        EetCompatibilityInput {
            mod_id: "eet",
            artifact_id: ARTIFACT_ID,
            artifact_sha256: "ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12",
            archive_drifted: true,
            target: GameRoot::Bg2,
            tp2: "EET/EET.tp2",
            components: &[0],
            target_root,
            weidu_bytes: None,
        }
    }

    #[test]
    fn eet_compat_unix_replaces_bundled_weidu_and_marks_helpers_executable() {
        let temp = tempfile::tempdir().unwrap();
        let target = std::fs::canonicalize(temp.path()).unwrap();
        let unix_dir = target.join("EET/bin/unix");
        let arch_dir = unix_dir.join("x86_64");
        std::fs::create_dir_all(&arch_dir).unwrap();
        std::fs::write(unix_dir.join("lua"), b"old lua").unwrap();
        std::fs::write(arch_dir.join("lua"), b"old lua 64").unwrap();
        let weidu_dest = arch_dir.join("weidu");
        std::fs::write(&weidu_dest, b"pre-251 weidu").unwrap();
        let verified_bytes = b"\x7fELF-verified-weidu-251";

        let evidence = prepare_unix(EetCompatibilityInput {
            mod_id: "eet",
            artifact_id: ARTIFACT_ID,
            artifact_sha256: ARCHIVE_SHA256,
            archive_drifted: false,
            target: GameRoot::Bg2,
            tp2: "EET/EET.tp2",
            components: &[0],
            target_root: &target,
            weidu_bytes: Some(verified_bytes),
        })
        .unwrap()
        .expect("unix bin compatibility should record replacement");

        assert!(evidence.changed);
        assert_eq!(evidence.relative_path, "EET/bin/unix/x86_64/weidu");
        assert_eq!(std::fs::read(&weidu_dest).unwrap(), verified_bytes);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for p in [
                unix_dir.join("lua"),
                arch_dir.join("lua"),
                weidu_dest.clone(),
            ] {
                let mode = std::fs::metadata(p).unwrap().permissions().mode();
                assert_eq!(mode & 0o111, 0o111);
            }
        }
        // Idempotent retry leaves bytes unchanged.
        let retry = prepare_unix(EetCompatibilityInput {
            mod_id: "eet",
            artifact_id: ARTIFACT_ID,
            artifact_sha256: ARCHIVE_SHA256,
            archive_drifted: false,
            target: GameRoot::Bg2,
            tp2: "EET/EET.tp2",
            components: &[0],
            target_root: &target,
            weidu_bytes: Some(verified_bytes),
        })
        .unwrap()
        .unwrap();
        assert!(!retry.changed);
    }

    #[cfg(windows)]
    #[test]
    fn eet_compat_corrects_only_the_faulty_text_of_a_newer_release() {
        let temp = tempfile::tempdir().unwrap();
        let target = std::fs::canonicalize(temp.path()).unwrap();
        std::fs::create_dir_all(target.join("EET/lib")).unwrap();
        let path = target.join("EET/lib/macros.tph");
        // A newer release: the rest of the file changed, the faulty batch did not.
        let newer = format!(
            "newer prefix\r\n{}\r\nnewer suffix\r\n",
            ORIGINAL_BATCH.replace('\n', "\r\n")
        );
        std::fs::write(&path, &newer).unwrap();
        let evidence = prepare(drifted_input(&target)).unwrap().unwrap();
        assert!(evidence.changed);
        assert_eq!(evidence.matched, Some("faulty-text"));
        assert_eq!(
            evidence.archive_sha256,
            "ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12"
        );
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            newer.replace(
                &ORIGINAL_BATCH.replace('\n', "\r\n"),
                &CORRECTED_BATCH.replace('\n', "\r\n")
            )
        );
        // Run again, as a retry does: nothing left to correct, nothing written.
        assert!(prepare(drifted_input(&target)).unwrap().is_none());
    }

    #[cfg(windows)]
    #[test]
    fn eet_compat_leaves_a_newer_release_without_the_faulty_text_alone() {
        let temp = tempfile::tempdir().unwrap();
        let target = std::fs::canonicalize(temp.path()).unwrap();
        // No EET folder at all: nothing is created.
        assert!(prepare(drifted_input(&target)).unwrap().is_none());
        assert_eq!(std::fs::read_dir(&target).unwrap().count(), 0);
        std::fs::create_dir_all(target.join("EET/lib")).unwrap();
        let path = target.join("EET/lib/macros.tph");
        std::fs::write(&path, "upstream fixed this differently").unwrap();
        assert!(prepare(drifted_input(&target)).unwrap().is_none());
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "upstream fixed this differently"
        );
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

    #[test]
    fn eet_compat_requires_the_staged_folders_and_creates_none() {
        let temp = tempfile::tempdir().unwrap();
        let (old, new) = fixture_hashes();

        let error = patch_file(temp.path(), &old, &new).unwrap_err();
        assert!(error.contains("is missing"), "{error}");
        std::fs::create_dir_all(temp.path().join("EET/lib")).unwrap();
        let error = patch_file(temp.path(), &old, &new).unwrap_err();
        assert!(error.contains("is missing"), "{error}");
        std::fs::create_dir(temp.path().join("EET/lib").join(MACROS)).unwrap();
        let error = patch_file(temp.path(), &old, &new).unwrap_err();
        assert!(error.contains("not a direct file/directory"), "{error}");

        assert_eq!(entries(&temp.path().join("EET/lib")), [MACROS]);
        assert_eq!(
            entries(&temp.path().join("EET/lib").join(MACROS)),
            Vec::<String>::new()
        );
    }

    /// The target root, EET, lib and macros.tph itself are junctions in turn. None of them is
    /// followed, and the file they point to is never touched.
    #[cfg(windows)]
    #[test]
    fn eet_compat_never_writes_through_a_junction_at_any_level() {
        let temp = tempfile::tempdir().unwrap();
        let (old, new) = fixture_hashes();
        let outside = staged_game(&temp.path().join("outside"));
        let outside_lib = outside.join("EET").join("lib");

        let linked_root = temp.path().join("linked-root");
        junction(&linked_root, &outside);
        let linked_eet = temp.path().join("linked-eet");
        std::fs::create_dir(&linked_eet).unwrap();
        junction(&linked_eet.join("EET"), &outside.join("EET"));
        let linked_lib = temp.path().join("linked-lib");
        std::fs::create_dir_all(linked_lib.join("EET")).unwrap();
        junction(&linked_lib.join("EET").join("lib"), &outside_lib);
        let linked_file = temp.path().join("linked-file");
        std::fs::create_dir_all(linked_file.join("EET").join("lib")).unwrap();
        junction(
            &linked_file.join("EET").join("lib").join(MACROS),
            &outside_lib,
        );

        for (target, refused) in [
            (&linked_root, linked_root.clone()),
            (&linked_eet, linked_eet.join("EET")),
            (&linked_lib, linked_lib.join("EET").join("lib")),
            (
                &linked_file,
                linked_file.join("EET").join("lib").join(MACROS),
            ),
        ] {
            let error = patch_file(target, &old, &new).unwrap_err();
            assert!(
                error.contains(&format!(
                    "not a direct file/directory: {}",
                    refused.display()
                )),
                "{error}"
            );
        }
        assert_eq!(macros_text(&outside), fixture().0);
        assert_eq!(entries(&outside_lib), [MACROS]);
    }

    /// Nothing above the target root is inspected. A link there, such as a cloud-files folder or
    /// a volume mounted in a folder, is no reason to refuse, and the real file is corrected.
    #[cfg(windows)]
    #[test]
    fn eet_compat_patches_below_a_link_above_the_target_root() {
        let temp = tempfile::tempdir().unwrap();
        let (old, new) = fixture_hashes();
        let real = temp.path().join("real");
        let game = staged_game(&real);
        let link = temp.path().join("link");
        junction(&link, &real);

        assert_eq!(
            patch_file(&link.join("game"), &old, &new).unwrap(),
            (old, new)
        );

        assert_eq!(macros_text(&game), fixture().1);
        assert_eq!(entries(&game.join("EET").join("lib")), [MACROS]);
    }

    /// Re-pointing a link above the target root after the folders were opened redirects every
    /// path through it. The file must still be read and replaced in the folder that was checked.
    #[cfg(windows)]
    #[test]
    fn eet_compat_replaces_the_held_file_after_a_link_above_is_repointed() {
        let temp = tempfile::tempdir().unwrap();
        let (old, new) = fixture_hashes();
        let checked = staged_game(&temp.path().join("checked"));
        // Unrecognized bytes: reading through the redirected path would refuse the fix.
        let other = staged_game(&temp.path().join("other"));
        std::fs::write(other.join("EET").join("lib").join(MACROS), "user changes").unwrap();
        let link = temp.path().join("link");
        junction(&link, &temp.path().join("checked"));

        let folder = HeldFolder::open(&link.join("game"), ["EET", "lib"]).unwrap();
        std::fs::remove_dir(&link).unwrap();
        junction(&link, &temp.path().join("other"));
        assert_eq!(patch_in_folder(&folder, &old, &new).unwrap(), (old, new));
        drop(folder);

        assert_eq!(macros_text(&checked), fixture().1);
        assert_eq!(entries(&checked.join("EET").join("lib")), [MACROS]);
        assert_eq!(macros_text(&other), "user changes");
        assert_eq!(entries(&other.join("EET").join("lib")), [MACROS]);
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
                archive_drifted: false,
                target: GameRoot::Bg2,
                tp2: "EET/EET.tp2",
                components: &[0, 100],
                target_root: temp.path(),
                weidu_bytes: None,
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

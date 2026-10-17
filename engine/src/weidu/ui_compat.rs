//! A reviewed setup-name correction for the frozen September 29 Infinity UI recipe.
//! This never enables explicit-TP2 execution or rewrites a frozen plan.

use std::ffi::OsString;
use std::path::Path;

use serde::Serialize;

use crate::digest::sha256_reader;
use crate::held_folder::{EntryError, EntryErrorKind, HeldFolder};
use crate::manifest::{Artifact, GameRoot, InvocationMode, ModFile};
use crate::resolve::PlannedRun;

pub(crate) const LEGACY_FAILURE: &str = "explicit-relative-TP2 invocation is unavailable until the signed recipe schema records separately reviewed compatibility evidence; use setup-name mode";
const ARCHIVE_SHA256: &str = "4bb0b81414b4198b866c3e7f2473479c7d1ee1f1a1bacd15c0e98913d5b83055";
const TP2_SHA256: &str = "3e5f311f32353c1c579a23c19630c9e9b6923d4652f3dc67a673411fbadc516e";
const TOOL_SHA256: &str = "ad70f5897a6d0ba4b0d226f845a9b14cf345f56cc9697ca8d05cac9fe4932c1a";

/// Match the shipped mistake, not arbitrary explicit-mode recipes or other UI releases.
pub(crate) fn reviewed(
    run: &PlannedRun,
    mod_file: &ModFile,
    artifact: &Artifact,
    tool: &Artifact,
) -> bool {
    run.run_id == "game-interface-infinity-bg2"
        && run.mod_id == "infinity-ui"
        && mod_file.id == run.mod_id
        && run.target == GameRoot::Bg2
        && run.phase == crate::manifest::Phase::Main
        && run.components == [0]
        && run.args.is_empty()
        && run.prompt_scripts.is_empty()
        && mod_file.invocation_mode == InvocationMode::ExplicitTp2
        && mod_file.tp2 == "infinity_ui/infinity_ui.tp2"
        && mod_file.language == 0
        && run.artifact_id == "infinity-ui-1.28"
        && mod_file.artifact_id == run.artifact_id
        && artifact.id == run.artifact_id
        && artifact.source.sha256 == ARCHIVE_SHA256
        && run.weidu_artifact_id == "weidu-249-amd64"
        && mod_file.weidu_artifact_id == run.weidu_artifact_id
        && tool.id == run.weidu_artifact_id
        && tool.source.sha256 == "b156910cbec69359fc2e42f6739aa959d49047d6fd3dc172f6bed88ffad8f927"
        && tool
            .tool
            .as_ref()
            .is_some_and(|spec| spec.sha256 == TOOL_SHA256 && spec.expected_length == 1_364_992)
}

#[derive(Debug, Serialize)]
pub(crate) struct Evidence {
    fix_id: &'static str,
    archive_sha256: &'static str,
    tp2_sha256: &'static str,
    weidu_sha256: &'static str,
    original_mode: InvocationMode,
    effective_mode: InvocationMode,
}

/// Verify the exact staged TP2 and reject alternate files that could shadow setup discovery.
/// Archive verification/materialization and the normal target/process guards still apply.
pub(crate) fn prepare(root: &Path) -> Result<Evidence, String> {
    check_discovery(root, TP2_SHA256)?;
    Ok(Evidence {
        fix_id: "infinity-ui-1.28-setup-name-v1",
        archive_sha256: ARCHIVE_SHA256,
        tp2_sha256: TP2_SHA256,
        weidu_sha256: TOOL_SHA256,
        original_mode: InvocationMode::ExplicitTp2,
        effective_mode: InvocationMode::SetupName,
    })
}

fn check_discovery(root: &Path, expected_sha256: &str) -> Result<(), String> {
    let root = HeldFolder::open(root, []).map_err(discovery_error)?;
    check_held_discovery(root, expected_sha256)
}

/// The staged root and `infinity_ui` are opened from the target root down without following a
/// link and stay held while they are inspected, so the TP2 that is hashed and the names that are
/// looked up are those of the checked folders, never of a tree that a link points to. Nothing
/// above the target root is inspected. WeiDU resolves the same names by path again when it
/// starts; this check cannot hold them for it.
fn check_held_discovery(root: HeldFolder, expected_sha256: &str) -> Result<(), String> {
    // WeiDU's setup-name discovery checks the root and the same-named subdirectory, with and
    // without the setup- prefix.
    reject_shadows(&root, &["setup-infinity_ui.tp2", "infinity_ui.tp2"])?;
    let folder = root.descend("infinity_ui").map_err(discovery_error)?;
    let (tp2, _) = folder
        .open_file("infinity_ui.tp2")
        .map_err(discovery_error)?;
    let digest = sha256_reader(&tp2).map_err(|error| {
        format!(
            "Infinity UI launch correction could not read {}: {error}",
            folder.path().join("infinity_ui.tp2").display()
        )
    })?;
    if digest != expected_sha256 {
        return Err(
            "Infinity UI launch correction does not recognize the staged TP2; files left unchanged"
                .into(),
        );
    }
    reject_shadows(&folder, &["setup-infinity_ui.tp2"])
}

/// An entry shadows a name when the file system opens it under that name, which covers every
/// spelling it treats as equal, or when the two differ only in ASCII case: a case-sensitive
/// folder tells such names apart, but WeiDU may look the name up in another case.
fn reject_shadows(folder: &HeldFolder, names: &[&str]) -> Result<(), String> {
    let inspect_error = |error: std::io::Error| {
        format!(
            "Infinity UI launch correction could not inspect {}: {error}",
            folder.path().display()
        )
    };
    let listed = folder.names().map_err(inspect_error)?;
    for name in names {
        if folder.contains(name).map_err(inspect_error)? || listed_as(&listed, name) {
            return Err(
                "Infinity UI launch correction found an ambiguous setup-name TP2; files left unchanged"
                    .into(),
            );
        }
    }
    Ok(())
}

fn listed_as(listed: &[OsString], name: &str) -> bool {
    listed
        .iter()
        .any(|entry| entry.to_string_lossy().eq_ignore_ascii_case(name))
}

fn discovery_error(error: EntryError) -> String {
    let path = error.path.display();
    match error.kind {
        EntryErrorKind::Missing => format!("Infinity UI launch correction could not find {path}"),
        EntryErrorKind::NotDirect => {
            format!("Infinity UI launch correction requires direct files and folders: {path}")
        }
        EntryErrorKind::Io(error) => {
            format!("Infinity UI launch correction could not inspect {path}: {error}")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::digest::sha256_bytes;
    #[cfg(windows)]
    use crate::held_folder::tests::junction;

    #[test]
    fn only_the_exact_reviewed_frozen_ui_and_tool_match() {
        let manifest =
            crate::Manifest::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../manifest"))
                .unwrap();
        let mut mod_file = manifest.mods["infinity-ui"].clone();
        mod_file.invocation_mode = InvocationMode::ExplicitTp2;
        let artifact = &manifest.artifacts["infinity-ui-1.28"];
        let tool = &manifest.artifacts["weidu-249-amd64"];
        let run = PlannedRun {
            run_id: "game-interface-infinity-bg2".into(),
            mod_id: mod_file.id.clone(),
            target: GameRoot::Bg2,
            phase: crate::manifest::Phase::Main,
            components: vec![0],
            args: vec![],
            postconditions: vec![],
            artifact_id: artifact.id.clone(),
            weidu_artifact_id: tool.id.clone(),
            prompt_scripts: vec![],
        };
        assert!(reviewed(&run, &mod_file, artifact, tool));
        let mut changed = mod_file.clone();
        changed.invocation_mode = InvocationMode::SetupName;
        assert!(!reviewed(&run, &changed, artifact, tool));
        changed = mod_file.clone();
        changed.language = 1;
        assert!(!reviewed(&run, &changed, artifact, tool));
        changed = mod_file.clone();
        changed.tp2 = "other/infinity_ui.tp2".into();
        assert!(!reviewed(&run, &changed, artifact, tool));
        let mut changed = artifact.clone();
        changed.source.sha256 = "00".repeat(32);
        assert!(!reviewed(&run, &mod_file, &changed, tool));
        let mut changed = tool.clone();
        changed.tool.as_mut().unwrap().sha256 = "00".repeat(32);
        assert!(!reviewed(&run, &mod_file, artifact, &changed));
        let mut changed = run.clone();
        changed.components.push(1);
        assert!(!reviewed(&changed, &mod_file, artifact, tool));
        changed = run.clone();
        changed.target = GameRoot::Bg1;
        assert!(!reviewed(&changed, &mod_file, artifact, tool));
        changed = run;
        changed.mod_id = "other-ui".into();
        assert!(!reviewed(&changed, &mod_file, artifact, tool));
    }

    #[test]
    fn discovery_rejects_shadowing_or_changed_tp2_without_writing() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::create_dir(root.join("infinity_ui")).unwrap();
        let path = root.join("infinity_ui/infinity_ui.tp2");
        fs::write(&path, b"reviewed fixture").unwrap();
        let digest = sha256_bytes(b"reviewed fixture");
        check_discovery(root, &digest).unwrap();
        for alias in [
            "infinity_ui.tp2",
            "INFINITY_UI.TP2",
            "Setup-INFINITY_UI.tp2",
            "infinity_ui/setup-infinity_ui.tp2",
            "infinity_ui/SETUP-INFINITY_UI.TP2",
        ] {
            let alias = root.join(alias);
            fs::write(&alias, b"shadow").unwrap();
            assert!(check_discovery(root, &digest)
                .unwrap_err()
                .contains("ambiguous"));
            fs::remove_file(alias).unwrap();
        }
        fs::write(&path, b"changed").unwrap();
        assert!(check_discovery(root, &digest)
            .unwrap_err()
            .contains("does not recognize"));
        assert_eq!(fs::read(path).unwrap(), b"changed");
    }

    /// A case-sensitive folder keeps names that differ only in case apart, so opening the name
    /// does not find them there; the listing does, because WeiDU may look the name up in
    /// another case.
    #[test]
    fn a_listed_name_in_another_ascii_case_counts_as_a_shadow() {
        let listed = [
            "INFINITY_UI.TP2",
            "Setup-Infinity_UI.tp2",
            "infinity_ui.tp2.bak",
        ]
        .map(OsString::from);

        assert!(listed_as(&listed, "infinity_ui.tp2"));
        assert!(listed_as(&listed, "setup-infinity_ui.tp2"));
        assert!(!listed_as(&listed[2..], "infinity_ui.tp2"));
        assert!(!listed_as(&listed[2..], "setup-infinity_ui.tp2"));
    }

    /// Inside a case-sensitive folder, opening a name does not find an entry that differs only in
    /// case, yet WeiDU may look the name up in that case. The listing still refuses it.
    #[cfg(windows)]
    #[test]
    fn discovery_refuses_a_shadow_in_another_case_in_a_case_sensitive_folder() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("game");
        fs::create_dir(&root).unwrap();
        let output = std::process::Command::new("fsutil")
            .args(["file", "setCaseSensitiveInfo"])
            .arg(&root)
            .arg("enable")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "fsutil could not make {} case-sensitive: {}",
            root.display(),
            String::from_utf8_lossy(&output.stdout)
        );
        // Created inside the case-sensitive root, infinity_ui is case-sensitive as well.
        fs::create_dir(root.join("infinity_ui")).unwrap();
        fs::write(
            root.join("infinity_ui/infinity_ui.tp2"),
            b"reviewed fixture",
        )
        .unwrap();
        let digest = sha256_bytes(b"reviewed fixture");
        check_discovery(&root, &digest).unwrap();

        for (folder, shadow, name) in [
            ("", "INFINITY_UI.TP2", "infinity_ui.tp2"),
            ("", "SETUP-INFINITY_UI.TP2", "setup-infinity_ui.tp2"),
            (
                "infinity_ui",
                "SETUP-INFINITY_UI.TP2",
                "setup-infinity_ui.tp2",
            ),
        ] {
            let folder_path = root.join(folder);
            fs::write(folder_path.join(shadow), b"shadow").unwrap();
            let held =
                HeldFolder::open(&root, [folder].into_iter().filter(|name| !name.is_empty()))
                    .unwrap();
            assert!(!held.contains(name).unwrap(), "{shadow} is told apart");
            drop(held);
            assert!(
                check_discovery(&root, &digest)
                    .unwrap_err()
                    .contains("ambiguous"),
                "{shadow}"
            );
            fs::remove_file(folder_path.join(shadow)).unwrap();
        }
        check_discovery(&root, &digest).unwrap();
    }

    /// A staged root below `parent` with the reviewed fixture TP2; returns the root.
    fn staged_ui(parent: &Path) -> std::path::PathBuf {
        let root = parent.join("game");
        fs::create_dir_all(root.join("infinity_ui")).unwrap();
        fs::write(
            root.join("infinity_ui/infinity_ui.tp2"),
            b"reviewed fixture",
        )
        .unwrap();
        root
    }

    #[test]
    fn discovery_requires_the_folder_and_a_regular_tp2() {
        let temp = tempfile::tempdir().unwrap();
        let digest = sha256_bytes(b"reviewed fixture");

        let error = check_discovery(temp.path(), &digest).unwrap_err();
        assert!(error.contains("could not find"), "{error}");
        fs::create_dir_all(temp.path().join("infinity_ui/infinity_ui.tp2")).unwrap();
        let error = check_discovery(temp.path(), &digest).unwrap_err();
        assert!(
            error.contains("requires direct files and folders"),
            "{error}"
        );
    }

    /// The staged root, infinity_ui and the TP2 itself are junctions in turn, and so is a name
    /// that would shadow the TP2. None of them is followed.
    #[cfg(windows)]
    #[test]
    fn discovery_refuses_a_junction_at_every_level() {
        let temp = tempfile::tempdir().unwrap();
        let digest = sha256_bytes(b"reviewed fixture");
        let outside = staged_ui(&temp.path().join("outside"));

        let linked_root = temp.path().join("linked-root");
        junction(&linked_root, &outside);
        let linked_folder = temp.path().join("linked-folder");
        fs::create_dir(&linked_folder).unwrap();
        junction(
            &linked_folder.join("infinity_ui"),
            &outside.join("infinity_ui"),
        );
        let linked_tp2 = temp.path().join("linked-tp2");
        fs::create_dir_all(linked_tp2.join("infinity_ui")).unwrap();
        junction(
            &linked_tp2.join("infinity_ui").join("infinity_ui.tp2"),
            &outside.join("infinity_ui"),
        );
        for (root, refused) in [
            (&linked_root, linked_root.clone()),
            (&linked_folder, linked_folder.join("infinity_ui")),
            (
                &linked_tp2,
                linked_tp2.join("infinity_ui").join("infinity_ui.tp2"),
            ),
        ] {
            let error = check_discovery(root, &digest).unwrap_err();
            assert!(
                error.contains(&format!(
                    "requires direct files and folders: {}",
                    refused.display()
                )),
                "{error}"
            );
        }

        let shadowed = staged_ui(&temp.path().join("shadowed"));
        junction(&shadowed.join("SETUP-infinity_ui.tp2"), &outside);
        assert!(check_discovery(&shadowed, &digest)
            .unwrap_err()
            .contains("ambiguous"));
    }

    /// Nothing above the staged root is inspected, so a link there, such as a cloud-files folder
    /// or a volume mounted in a folder, is no reason to refuse.
    #[cfg(windows)]
    #[test]
    fn discovery_accepts_a_link_above_the_staged_root() {
        let temp = tempfile::tempdir().unwrap();
        staged_ui(&temp.path().join("real"));
        let link = temp.path().join("link");
        junction(&link, &temp.path().join("real"));

        check_discovery(&link.join("game"), &sha256_bytes(b"reviewed fixture")).unwrap();
    }

    /// Re-pointing a link above the staged root after it was opened redirects every path through
    /// it. The TP2 that is hashed and the names that are looked up stay those of the held folders.
    #[cfg(windows)]
    #[test]
    fn discovery_inspects_the_held_folders_after_a_link_above_is_repointed() {
        let temp = tempfile::tempdir().unwrap();
        let digest = sha256_bytes(b"reviewed fixture");
        let clean = temp.path().join("clean");
        staged_ui(&clean);
        let ambiguous = temp.path().join("ambiguous");
        let ambiguous_root = staged_ui(&ambiguous);
        fs::write(ambiguous_root.join("infinity_ui.tp2"), b"shadow").unwrap();
        fs::write(
            ambiguous_root.join("infinity_ui/infinity_ui.tp2"),
            b"changed",
        )
        .unwrap();
        let link = temp.path().join("link");

        // Checked clean, then pointed at the ambiguous tree: the clean folders decide.
        junction(&link, &clean);
        let root = HeldFolder::open(&link.join("game"), []).unwrap();
        fs::remove_dir(&link).unwrap();
        junction(&link, &ambiguous);
        check_held_discovery(root, &digest).unwrap();

        // Checked ambiguous, then pointed at the clean tree: the ambiguous folders decide.
        let root = HeldFolder::open(&link.join("game"), []).unwrap();
        fs::remove_dir(&link).unwrap();
        junction(&link, &clean);
        assert!(check_held_discovery(root, &digest)
            .unwrap_err()
            .contains("ambiguous"));
    }
}

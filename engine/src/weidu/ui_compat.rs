//! A reviewed setup-name correction for the frozen September 29 Infinity UI recipe.
//! This never enables explicit-TP2 execution or rewrites a frozen plan.

use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::digest::sha256_bytes;
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
    let tp2 = root.join("infinity_ui/infinity_ui.tp2");
    for path in tp2.ancestors() {
        let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
        #[cfg(windows)]
        let reparse = {
            use std::os::windows::fs::MetadataExt;
            metadata.file_attributes() & 0x400 != 0
        };
        #[cfg(not(windows))]
        let reparse = false;
        if metadata.file_type().is_symlink()
            || reparse
            || (path == tp2 && !metadata.is_file())
            || (path != tp2 && !metadata.is_dir())
        {
            return Err("Infinity UI launch correction requires direct files and folders".into());
        }
    }
    if sha256_bytes(&fs::read(tp2).map_err(|error| error.to_string())?) != expected_sha256 {
        return Err(
            "Infinity UI launch correction does not recognize the staged TP2; files left unchanged"
                .into(),
        );
    }
    // WeiDU's setup-name discovery checks the root and the same-named subdirectory,
    // with and without the setup- prefix. Inspect names case-insensitively everywhere.
    for folder in [root.to_path_buf(), root.join("infinity_ui")] {
        for entry in fs::read_dir(&folder).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            if name == "setup-infinity_ui.tp2" || (folder == root && name == "infinity_ui.tp2") {
                return Err("Infinity UI launch correction found an ambiguous setup-name TP2; files left unchanged".into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
            "Setup-INFINITY_UI.tp2",
            "infinity_ui/setup-infinity_ui.tp2",
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
}

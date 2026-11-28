//! Host-platform WeiDU tool substitution and Unix executable permissions.
//!
//! Shipped recipes pin `weidu-249-amd64` to the reviewed Windows executable so recipe content
//! digests remain invariant across platforms. When running natively on Linux, the runtime adapts
//! that exact reviewed tool artifact to the verified upstream Linux ELF64 `WeiDU-Linux-251`
//! release (`v251.00`), which handles case-sensitive game trees directly without `tolower`.

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use crate::manifest::{
    AcquisitionPolicy, ArchiveKind, ArchiveLimitsSpec, ArchiveRootRule, ArchiveSpec, Artifact,
    PeMachine, Provenance, Source, SourceKind, ToolSpec,
};

/// Canonical reviewed tool artifact id used in shipped recipes.
pub(crate) const REVIEWED_WEIDU_ARTIFACT_ID: &str = "weidu-249-amd64";

/// Pinned SHA-256 of the reviewed `WeiDU-Windows-249-amd64.zip` source archive.
pub(crate) const WINDOWS_WEIDU_249_ARCHIVE_SHA256: &str =
    "b156910cbec69359fc2e42f6739aa959d49047d6fd3dc172f6bed88ffad8f927";

/// Pinned byte length of the reviewed `WeiDU-Windows-249-amd64.zip` source archive.
pub(crate) const WINDOWS_WEIDU_249_ARCHIVE_LENGTH: u64 = 2_154_755;

/// Pinned download URL of `WeiDU-Windows-249-amd64.zip`.
pub(crate) const WINDOWS_WEIDU_249_URL: &str =
    "https://github.com/WeiDUorg/weidu/releases/download/v249.00/WeiDU-Windows-249-amd64.zip";

/// Pinned SHA-256 of the extracted Windows `weidu.exe` (v249.00).
pub(crate) const WINDOWS_WEIDU_249_EXE_SHA256: &str =
    "ad70f5897a6d0ba4b0d226f845a9b14cf345f56cc9697ca8d05cac9fe4932c1a";

/// Pinned byte length of the extracted Windows `weidu.exe` (v249.00).
pub(crate) const WINDOWS_WEIDU_249_EXE_LENGTH: u64 = 1_364_992;

/// Reviewed upstream Linux `WeiDU-Linux-251.zip` release URL.
pub(crate) const LINUX_WEIDU_251_URL: &str =
    "https://github.com/WeiDUorg/weidu/releases/download/v251.00/WeiDU-Linux-251.zip";

/// Reviewed archive length for `WeiDU-Linux-251.zip`.
pub(crate) const LINUX_WEIDU_251_ARCHIVE_LENGTH: u64 = 2_723_764;

/// Reviewed SHA-256 of `WeiDU-Linux-251.zip`.
pub(crate) const LINUX_WEIDU_251_ARCHIVE_SHA256: &str =
    "3e1a34ec5d6e934b1d4a34541a1312d0170b120c6efe903a732415acde3bd3c0";

/// Relative executable path extracted from `WeiDU-Linux-251.zip` after wrapper stripping.
pub(crate) const LINUX_WEIDU_251_EXECUTABLE: &str = "weidu";

/// Reviewed byte length of the Linux x86_64 ELF binary `weidu` (`v251.00`).
pub(crate) const LINUX_WEIDU_251_BIN_LENGTH: u64 = 1_845_512;

/// Reviewed SHA-256 of the Linux x86_64 ELF binary `weidu` (`v251.00`).
pub(crate) const LINUX_WEIDU_251_BIN_SHA256: &str =
    "881c6b62e031480c305b11cbdd6acabf8403d5550661d17e505872ccc4c7415d";

/// Numeric version output printed by Linux `weidu` (`v251.00`) `--version`.
pub(crate) const LINUX_WEIDU_251_VERSION: &str = "25100";

/// Build the reviewed Linux WeiDU 251 [`Artifact`] while preserving `id`.
pub(crate) fn linux_weidu_251_artifact(id: &str) -> Artifact {
    Artifact {
        id: id.to_owned(),
        name: "WeiDU 251 for Linux x64".to_owned(),
        version: "251.00".to_owned(),
        source: Source {
            kind: SourceKind::GithubRelease,
            url: LINUX_WEIDU_251_URL.to_owned(),
            reference: "v251.00".to_owned(),
            expected_filename: Some("WeiDU-Linux-251.zip".to_owned()),
            expected_length: Some(LINUX_WEIDU_251_ARCHIVE_LENGTH),
            sha256: LINUX_WEIDU_251_ARCHIVE_SHA256.to_owned(),
            redirect_hosts: vec!["release-assets.githubusercontent.com".to_owned()],
        },
        archive: ArchiveSpec {
            kind: ArchiveKind::Zip,
            root_rule: ArchiveRootRule::SingleWrapper,
            publish_roots: vec![LINUX_WEIDU_251_EXECUTABLE.to_owned()],
            tp2_paths: Vec::new(),
            limits: ArchiveLimitsSpec {
                max_depth: 8,
                max_entries: 64,
                max_entry_uncompressed_bytes: 8_388_608,
                max_total_uncompressed_bytes: 16_777_216,
                max_compression_ratio: 100,
            },
        },
        acquisition: AcquisitionPolicy::FetchOnly,
        provenance: Provenance {
            homepage: "https://weidu.org/".to_owned(),
            license: "GPL-2.0".to_owned(),
            url: "https://github.com/WeiDUorg/weidu/releases/tag/v251.00".to_owned(),
            reviewed_on: "2026-11-14".to_owned(),
        },
        tool: Some(ToolSpec {
            executable: LINUX_WEIDU_251_EXECUTABLE.to_owned(),
            expected_length: LINUX_WEIDU_251_BIN_LENGTH,
            sha256: LINUX_WEIDU_251_BIN_SHA256.to_owned(),
            weidu_version: LINUX_WEIDU_251_VERSION.to_owned(),
            pe_machine: PeMachine::X86_64,
        }),
    }
}

/// Adapt one artifact for Linux execution when it matches the pinned reviewed Windows WeiDU tool.
/// Synthetic test fixtures or non-matching tools are returned untouched.
pub(crate) fn adapt_artifact_for_linux(artifact: &Artifact) -> Artifact {
    let matches_reviewed_windows_tool = artifact.id == REVIEWED_WEIDU_ARTIFACT_ID
        && artifact
            .source
            .sha256
            .eq_ignore_ascii_case(WINDOWS_WEIDU_249_ARCHIVE_SHA256)
        && artifact.tool.is_some();
    if matches_reviewed_windows_tool {
        linux_weidu_251_artifact(&artifact.id)
    } else {
        artifact.clone()
    }
}

/// Adapt one artifact for the active compile-time host OS.
pub(crate) fn adapt_artifact_for_host(artifact: &Artifact) -> Artifact {
    if cfg!(target_os = "linux") {
        adapt_artifact_for_linux(artifact)
    } else {
        artifact.clone()
    }
}

/// Adapt an entire artifact map for the active compile-time host OS.
pub(crate) fn adapt_artifacts_for_host(
    artifacts: &BTreeMap<String, Artifact>,
) -> BTreeMap<String, Artifact> {
    artifacts
        .iter()
        .map(|(id, artifact)| (id.clone(), adapt_artifact_for_host(artifact)))
        .collect()
}

/// Active reviewed binary SHA-256 expected by [`super::ui_compat`] and `patches`.
pub(crate) const fn active_weidu_exe_sha256() -> &'static str {
    if cfg!(target_os = "linux") {
        LINUX_WEIDU_251_BIN_SHA256
    } else {
        WINDOWS_WEIDU_249_EXE_SHA256
    }
}

/// Active reviewed archive SHA-256 for the pinned `weidu-249-amd64` host tool.
pub(crate) const fn active_weidu_archive_sha256() -> &'static str {
    if cfg!(target_os = "linux") {
        LINUX_WEIDU_251_ARCHIVE_SHA256
    } else {
        WINDOWS_WEIDU_249_ARCHIVE_SHA256
    }
}

/// Active reviewed binary byte length for the pinned `weidu-249-amd64` host tool.
pub(crate) const fn active_weidu_exe_length() -> u64 {
    if cfg!(target_os = "linux") {
        LINUX_WEIDU_251_BIN_LENGTH
    } else {
        WINDOWS_WEIDU_249_EXE_LENGTH
    }
}

/// Marks a materialized direct regular executable as `0o755` on Unix; no-op elsewhere.
pub(crate) fn ensure_executable(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::fs::{self, Permissions};
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::symlink_metadata(path)?;
        if crate::reparse::metadata_is_link(path, &metadata) || !metadata.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "cannot set executable mode on non-regular file {}",
                    path.display()
                ),
            ));
        }
        fs::set_permissions(path, Permissions::from_mode(0o755))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_shipped_weidu_artifact() -> Artifact {
        let manifest =
            crate::Manifest::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../manifest"))
                .expect("shipped manifest should load");
        manifest.artifacts[REVIEWED_WEIDU_ARTIFACT_ID].clone()
    }

    #[test]
    fn linux_adaptation_substitutes_exact_reviewed_windows_tool_and_keeps_id() {
        let original = load_shipped_weidu_artifact();
        let adapted = adapt_artifact_for_linux(&original);
        assert_eq!(adapted.id, REVIEWED_WEIDU_ARTIFACT_ID);
        assert_eq!(adapted.source.sha256, LINUX_WEIDU_251_ARCHIVE_SHA256);
        assert_eq!(
            adapted.source.expected_length,
            Some(LINUX_WEIDU_251_ARCHIVE_LENGTH)
        );
        let tool = adapted.tool.as_ref().expect("linux tool spec");
        assert_eq!(tool.executable, LINUX_WEIDU_251_EXECUTABLE);
        assert_eq!(tool.sha256, LINUX_WEIDU_251_BIN_SHA256);
        assert_eq!(tool.expected_length, LINUX_WEIDU_251_BIN_LENGTH);
        assert_eq!(tool.weidu_version, LINUX_WEIDU_251_VERSION);
        assert_eq!(tool.pe_machine, PeMachine::X86_64);
    }

    #[test]
    fn linux_adaptation_leaves_synthetic_or_mod_artifacts_untouched() {
        let mut synthetic = load_shipped_weidu_artifact();
        synthetic.source.sha256 = "00".repeat(32);
        assert_eq!(adapt_artifact_for_linux(&synthetic), synthetic);

        let mut other_id = load_shipped_weidu_artifact();
        other_id.id = "custom-weidu".to_owned();
        assert_eq!(adapt_artifact_for_linux(&other_id), other_id);
    }
}

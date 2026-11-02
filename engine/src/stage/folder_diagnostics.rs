//! Operation context for staging failures. This never changes filesystem policy.

use std::io;
use std::path::Path;

use crate::error::{EngineError, Result};

pub(super) fn context(operation: &'static str, path: &Path, source: EngineError) -> EngineError {
    EngineError::FolderOperation {
        operation,
        path: path.to_path_buf(),
        source: Box::new(source),
    }
}

pub(super) fn io_failure(operation: &'static str, path: &Path, source: io::Error) -> EngineError {
    context(
        operation,
        path,
        EngineError::Io {
            path: path.to_path_buf(),
            source,
        },
    )
}

pub(super) fn with_cleanup_failure(original: EngineError, cleanup: Result<()>) -> EngineError {
    match cleanup {
        Ok(()) => original,
        Err(cleanup) => EngineError::CleanupFailed {
            original: Box::new(original),
            cleanup: Box::new(cleanup),
        },
    }
}

/// Keep the HRESULT while exposing the originating Win32 error kind when present.
#[cfg(windows)]
pub(super) fn windows_hresult_error(operation: &'static str, hresult: i32) -> io::Error {
    #[derive(Debug, thiserror::Error)]
    #[error("{operation} failed with HRESULT 0x{hresult:08X}")]
    struct HresultFailure {
        operation: &'static str,
        hresult: u32,
        #[source]
        win32: Option<io::Error>,
    }

    let hresult = hresult as u32;
    let win32 = (hresult & 0xFFFF_0000 == 0x8007_0000)
        .then(|| io::Error::from_raw_os_error((hresult & 0xFFFF) as i32));
    let kind = win32.as_ref().map_or(io::ErrorKind::Other, io::Error::kind);
    io::Error::new(
        kind,
        HresultFailure {
            operation,
            hresult,
            win32,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_context_keeps_the_os_code_and_error_kind() {
        #[cfg(windows)]
        let denied_code = 5;
        #[cfg(not(windows))]
        let denied_code = 13;
        let path = Path::new("fixture/save/.chriz-managed-install.json");
        let error = io_failure(
            "write save-folder ownership marker",
            path,
            io::Error::from_raw_os_error(denied_code),
        );
        assert_eq!(
            error.folder_operation(),
            Some(("write save-folder ownership marker", path))
        );
        let EngineError::Io { source, .. } = error.primary_cause() else {
            panic!("original I/O cause was lost: {error}");
        };
        assert_eq!(source.raw_os_error(), Some(denied_code));
        assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(error.folder_failure_code(), Some("folder_access_denied"));
    }

    #[test]
    fn cleanup_failure_cannot_mask_an_ownership_rejection() {
        let path = Path::new("fixture/save");
        let original = context(
            "revalidate save folder",
            path,
            EngineError::SaveRootCollision {
                path: path.to_path_buf(),
                reason: "different owner".to_owned(),
            },
        );
        let cleanup = io_failure(
            "clean up incomplete save folder",
            path,
            io::Error::new(io::ErrorKind::PermissionDenied, "cleanup denied"),
        );
        let error = with_cleanup_failure(original, Err(cleanup));
        assert!(matches!(
            error.primary_cause(),
            EngineError::SaveRootCollision { .. }
        ));
        assert_eq!(
            error.folder_operation(),
            Some(("revalidate save folder", path))
        );
        let message = error.to_string();
        assert!(message.contains("different owner"));
        assert!(message.contains("cleanup also failed"));
        assert!(message.contains("cleanup denied"));
        assert_eq!(error.folder_failure_code(), Some("folder_conflict"));
    }

    #[test]
    fn successful_cleanup_leaves_original_error_unchanged() {
        let original = io_failure(
            "create save-folder ownership marker",
            Path::new("fixture/save/marker"),
            io::Error::new(io::ErrorKind::NotFound, "folder disappeared"),
        );
        let before = original.to_string();
        let error = with_cleanup_failure(original, Ok(()));
        assert_eq!(error.to_string(), before);
        let EngineError::Io { source, .. } = error.primary_cause() else {
            panic!("original I/O cause was lost");
        };
        assert_eq!(source.kind(), io::ErrorKind::NotFound);
        assert_eq!(error.folder_failure_code(), Some("folder_unavailable"));
    }

    #[test]
    fn folder_codes_do_not_reclassify_unrelated_io_or_unknown_failures_as_safe() {
        let path = Path::new("fixture/path");
        let unrelated = EngineError::Io {
            path: path.to_path_buf(),
            source: io::Error::new(io::ErrorKind::PermissionDenied, "unrelated operation"),
        };
        assert_eq!(unrelated.folder_failure_code(), None);
        let unknown = io_failure(
            "create save folder",
            path,
            io::Error::other("unclassified filesystem failure"),
        );
        assert_eq!(unknown.folder_failure_code(), Some("folder_failed"));
    }

    #[cfg(windows)]
    #[test]
    fn known_folder_hresult_keeps_the_original_code_and_classifies_win32_causes() {
        use std::error::Error;

        let denied = windows_hresult_error("SHGetKnownFolderPath", 0x8007_0005_u32 as i32);
        assert_eq!(denied.kind(), io::ErrorKind::PermissionDenied);
        assert!(denied.to_string().contains("0x80070005"));
        let raw_source = denied
            .get_ref()
            .and_then(Error::source)
            .and_then(|source| source.downcast_ref::<io::Error>())
            .unwrap();
        assert_eq!(raw_source.raw_os_error(), Some(5));

        let missing = windows_hresult_error("SHGetKnownFolderPath", 0x8007_0003_u32 as i32);
        assert_eq!(missing.kind(), io::ErrorKind::NotFound);
        assert!(missing.to_string().contains("0x80070003"));

        let unknown = windows_hresult_error("CoInitializeEx", 0x8000_4005_u32 as i32);
        assert_eq!(unknown.kind(), io::ErrorKind::Other);
        assert!(unknown.to_string().contains("0x80004005"));
    }

    #[test]
    fn vanished_child_keeps_marker_creation_failure_even_when_cleanup_also_fails() {
        use super::super::{
            cleanup_failed_save_reservation, write_save_owner, SaveIdentity, SaveOwner,
            SAVE_OWNER_SCHEMA,
        };

        let temp = tempfile::tempdir().unwrap();
        let save_root = temp.path().join("vanished-child");
        std::fs::create_dir(&save_root).unwrap();
        let identity = SaveIdentity {
            engine_name: "vanished-child".to_owned(),
            save_root: save_root.clone(),
            marker: SaveOwner {
                schema: SAVE_OWNER_SCHEMA,
                install_id: "fixture-install".to_owned(),
                managed_root: temp.path().to_string_lossy().into_owned(),
                engine_name: "vanished-child".to_owned(),
            },
        };
        std::fs::remove_dir(&save_root).unwrap();
        let original = write_save_owner(&identity).unwrap_err();
        let error = with_cleanup_failure(original, cleanup_failed_save_reservation(&identity));
        assert!(matches!(error, EngineError::CleanupFailed { .. }));
        assert_eq!(
            error.folder_operation(),
            Some((
                "create save-folder ownership marker",
                identity.owner_marker().as_path()
            ))
        );
        let EngineError::Io { path, source } = error.primary_cause() else {
            panic!("primary marker failure was lost: {error}");
        };
        assert_eq!(path, &identity.owner_marker());
        assert_eq!(source.kind(), io::ErrorKind::NotFound);
        let message = error.to_string();
        assert!(message.contains("create save-folder ownership marker"));
        assert!(message.contains("cleanup also failed"));
        assert!(!save_root.exists());
    }
}

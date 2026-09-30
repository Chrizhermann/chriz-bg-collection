//! Optional Radar launch: exact managed files, normal Windows consent, nonfatal feedback.

use std::path::Path;

use serde::Serialize;

use super::BridgeSystem;

/// A successful game launch and the independently requested overlay outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchInstallResponse {
    pub install_id: String,
    pub radar: RadarLaunchResponse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RadarLaunchState {
    NotRequested,
    Started,
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarLaunchResponse {
    pub state: RadarLaunchState,
    pub detail: Option<String>,
    pub technical_detail: Option<String>,
}

impl RadarLaunchResponse {
    pub(super) fn not_requested() -> Self {
        Self {
            state: RadarLaunchState::NotRequested,
            detail: None,
            technical_detail: None,
        }
    }

    fn failed(detail: &str, technical_detail: impl Into<String>) -> Self {
        Self {
            state: RadarLaunchState::Failed,
            detail: Some(detail.to_owned()),
            technical_detail: Some(technical_detail.into()),
        }
    }
}

/// OS launch acceptance, not a claim that Radar finished startup or attached to the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadarProcessOutcome {
    Started,
    Cancelled,
}

pub(super) fn launch_checked(
    system: &dyn BridgeSystem,
    managed_root: &Path,
    game_root: &Path,
) -> RadarLaunchResponse {
    use bg_engine::radar::RadarState;

    let overlay = match bg_engine::radar::status(managed_root, game_root, None) {
        Ok(overlay) => overlay,
        Err(error) => {
            return RadarLaunchResponse::failed(
                "CEBG could not verify the overlay. Check it in Updates.",
                error.to_string(),
            );
        }
    };
    match overlay.state {
        RadarState::NotInstalled => {
            return RadarLaunchResponse::failed(
                "Install BG Radar Overlay from Updates to use it.",
                "No managed BG Radar Overlay installation was found.",
            );
        }
        RadarState::Modified => {
            return RadarLaunchResponse::failed(
                "Its files are missing or changed; check it in Updates.",
                "The managed BG Radar Overlay receipt or owned files did not pass integrity checks.",
            );
        }
        RadarState::UpdateAvailable => {
            // A local-only status check cannot normally produce this state.
            return RadarLaunchResponse::failed(
                "CEBG could not verify the overlay. Check it in Updates.",
                "Unexpected UpdateAvailable state during a local-only Radar launch check.",
            );
        }
        RadarState::UpToDate => {}
    }
    let Some(executable) = overlay.executable else {
        return RadarLaunchResponse::failed(
            "The overlay executable is unavailable.",
            "The verified Radar status did not contain an executable path.",
        );
    };
    let Some(directory) = executable.parent() else {
        return RadarLaunchResponse::failed(
            "The overlay folder is unavailable.",
            format!("Radar executable has no parent: {}", executable.display()),
        );
    };
    match system.launch_radar(&executable, directory) {
        Ok(RadarProcessOutcome::Started) => RadarLaunchResponse {
            state: RadarLaunchState::Started,
            detail: None,
            technical_detail: None,
        },
        Ok(RadarProcessOutcome::Cancelled) => RadarLaunchResponse {
            state: RadarLaunchState::Cancelled,
            detail: Some("Windows permission was cancelled.".to_owned()),
            technical_detail: Some(
                "Windows cancelled the Radar launch request (ERROR_CANCELLED).".to_owned(),
            ),
        },
        Err(error) => RadarLaunchResponse::failed("Windows could not start the overlay.", error),
    }
}

#[cfg(windows)]
pub(super) fn launch_with_consent(
    executable: &Path,
    working_directory: &Path,
) -> Result<RadarProcessOutcome, String> {
    let executable = executable.to_path_buf();
    let working_directory = working_directory.to_path_buf();
    // Shell extensions may require STA. Keep this separate from Tauri's worker COM mode.
    std::thread::Builder::new()
        .name("cebg-radar-launch".to_owned())
        .spawn(move || {
            use windows::core::{w, PCWSTR};
            use windows::Win32::System::Com::{
                CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
            };
            use windows::Win32::UI::Shell::{
                ShellExecuteExW, SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW,
            };
            use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

            let executable = shell_path(&executable)?;
            let working_directory = shell_path(&working_directory)?;
            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) }
                .ok()
                .map_err(|error| format!("initialize Radar launch COM apartment: {error}"))?;
            struct ComApartment;
            impl Drop for ComApartment {
                fn drop(&mut self) {
                    unsafe { CoUninitialize() };
                }
            }
            let _apartment = ComApartment;
            let mut request = SHELLEXECUTEINFOW {
                cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
                // NO_UI suppresses duplicate error dialogs, never Windows security prompts.
                // NOASYNC completes the launch before this short-lived STA exits.
                fMask: SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI,
                // The executable's manifest decides whether consent is needed; no forced
                // elevation of future Radar versions, the game, or CEBG itself.
                lpVerb: w!("open"),
                lpFile: PCWSTR(executable.as_ptr()),
                lpDirectory: PCWSTR(working_directory.as_ptr()),
                nShow: SW_SHOWNORMAL.0,
                ..Default::default()
            };
            // No process handle is requested and no wait for the overlay's lifetime occurs.
            classify_shell_result(unsafe { ShellExecuteExW(&mut request) })
        })
        .map_err(|error| format!("start Radar launch worker: {error}"))?
        .join()
        .map_err(|_| "Radar launch worker panicked".to_owned())?
}

#[cfg(windows)]
fn shell_path(path: &Path) -> Result<Vec<u16>, String> {
    // The filesystem's canonical extended-path syntax is not universally supported by
    // Shell APIs. Remove only that prefix, and prove the ordinary spelling still points
    // to exactly the path that passed Radar's managed-file checks.
    let text = path
        .to_str()
        .ok_or("Radar launch path is not valid Unicode.")?;
    if text.contains('\0') || !path.is_absolute() {
        return Err("Radar launch path must be absolute and contain no null character.".to_owned());
    }
    let shell = super::display_windows_path_text(text);
    let expected = std::fs::canonicalize(path)
        .map_err(|error| format!("resolve verified Radar launch path: {error}"))?;
    let actual = std::fs::canonicalize(&shell)
        .map_err(|error| format!("resolve Radar Shell launch path: {error}"))?;
    if actual != expected {
        return Err("Radar Shell path does not resolve to the verified launch path.".to_owned());
    }
    Ok(shell.encode_utf16().chain(Some(0)).collect())
}

#[cfg(windows)]
fn classify_shell_result(result: windows::core::Result<()>) -> Result<RadarProcessOutcome, String> {
    use windows::Win32::Foundation::ERROR_CANCELLED;

    match result {
        Ok(()) => Ok(RadarProcessOutcome::Started),
        Err(error) if error.code() == windows::core::HRESULT::from_win32(ERROR_CANCELLED.0) => {
            Ok(RadarProcessOutcome::Cancelled)
        }
        Err(error) => Err(format!(
            "Windows ShellExecuteExW could not launch Radar: {error}"
        )),
    }
}

#[cfg(not(windows))]
pub(super) fn launch_with_consent(
    _executable: &Path,
    _working_directory: &Path,
) -> Result<RadarProcessOutcome, String> {
    Err("BG Radar Overlay launch is supported only on Windows.".to_owned())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use windows::core::{Error, HRESULT};
    use windows::Win32::Foundation::{ERROR_ACCESS_DENIED, ERROR_CANCELLED};

    #[test]
    fn shell_result_distinguishes_consent_cancellation_from_launch_failure() {
        assert_eq!(
            classify_shell_result(Ok(())),
            Ok(RadarProcessOutcome::Started)
        );
        assert_eq!(
            classify_shell_result(Err(Error::from_hresult(HRESULT::from_win32(
                ERROR_CANCELLED.0
            )))),
            Ok(RadarProcessOutcome::Cancelled),
        );
        let error = classify_shell_result(Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_ACCESS_DENIED.0,
        ))))
        .unwrap_err();
        assert!(error.contains("ShellExecuteExW"));
    }

    #[test]
    fn shell_paths_preserve_unicode_and_verified_identity_without_verbatim_prefix() {
        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("Radar 테스트 with spaces");
        std::fs::create_dir(&directory).unwrap();
        let executable = directory.join("BG Radar Overlay.exe");
        std::fs::write(&executable, b"inert test bytes").unwrap();
        for path in [directory, executable] {
            let canonical = std::fs::canonicalize(path).unwrap();
            assert!(canonical.to_str().unwrap().starts_with(r"\\?\"));
            let wide = shell_path(&canonical).unwrap();
            assert_eq!(wide.last(), Some(&0));
            let shell = String::from_utf16(&wide[..wide.len() - 1]).unwrap();
            assert!(!shell.starts_with(r"\\?\"));
            assert!(shell.contains("Radar 테스트 with spaces"));
            assert_eq!(std::fs::canonicalize(shell).unwrap(), canonical);
        }
        assert!(shell_path(Path::new("relative.exe")).is_err());
    }
}

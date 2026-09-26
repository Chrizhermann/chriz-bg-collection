//! Opt-in local traces of the actual app process's installation lookup.
//!
//! No game writes, remote uploads, environment dumps, or changed acceptance gates.
//! A diagnostic build writes a create-new, size-bounded JSONL file in Windows Temp.
//! Normal builds leave this instrumentation disabled.

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use bg_engine::digest::sha256_bytes;
use bg_engine::registry::{
    inspect_campaign_availability, inspect_install_availability, ManagedCampaignCard,
    ManagedInstallCard,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::bridge::ManagedInstallationResponse;
use crate::error::CommandError;
use crate::updates::UpdateCenterResponse;

const MAX_TRACE_BYTES: usize = 4 * 1024 * 1024;
static SINK: OnceLock<Option<Mutex<TraceSink>>> = OnceLock::new();

struct TraceSink {
    file: File,
    written: usize,
}

impl TraceSink {
    fn create(path: &Path) -> io::Result<Self> {
        // Never follow/overwrite a pre-existing file or symlink at the chosen path.
        Ok(Self {
            file: OpenOptions::new().write(true).create_new(true).open(path)?,
            written: 0,
        })
    }

    fn append(&mut self, event: &Value) -> io::Result<()> {
        let mut bytes = serde_json::to_vec(event)?;
        bytes.push(b'\n');
        if self.written.saturating_add(bytes.len()) > MAX_TRACE_BYTES {
            return Ok(());
        }
        self.file.write_all(&bytes)?;
        self.written += bytes.len();
        self.file.flush()
    }
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn write_event(kind: &str, data: Value) {
    if !cfg!(feature = "installation-lookup-diagnostics") {
        return;
    }
    let sink = SINK.get_or_init(|| {
        let directory = std::env::temp_dir();
        if !directory.is_absolute() {
            return None;
        }
        let path = directory.join(format!(
            "CEBG-installation-lookup-{}-{}.jsonl",
            std::process::id(),
            now_millis()
        ));
        let mut sink = TraceSink::create(&path).ok()?;
        let executable = std::env::current_exe().ok();
        let executable_sha256 = executable
            .as_ref()
            .and_then(|path| fs::read(path).ok())
            .map(|bytes| sha256_bytes(&bytes));
        let header = json!({
            "event": "process",
            "trace_schema": 1,
            "time_millis": now_millis(),
            "pid": std::process::id(),
            "app_version": env!("CARGO_PKG_VERSION"),
            "executable": executable,
            "executable_sha256": executable_sha256,
            "working_directory": std::env::current_dir().ok(),
            // Deliberate allowlist: never serialize the rest of the environment.
            "local_app_data": std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
            "trace_path": path,
        });
        sink.append(&header).ok()?;
        Some(Mutex::new(sink))
    });
    if let Some(sink) = sink {
        if let Ok(mut sink) = sink.lock() {
            // Tracing failure must never prevent a lookup or weaken a rejection.
            let _ = sink.append(&json!({
                "event": kind,
                "time_millis": now_millis(),
                "thread": format!("{:?}", std::thread::current().id()),
                "data": data,
            }));
        }
    }
}

fn io_failure(error: &io::Error) -> Value {
    json!({ "message": error.to_string(), "os_error": error.raw_os_error(),
        "kind": format!("{:?}", error.kind()) })
}

fn path_probe(path: &Path) -> Value {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            #[cfg(windows)]
            let attributes = {
                use std::os::windows::fs::MetadataExt;
                Some(metadata.file_attributes())
            };
            #[cfg(not(windows))]
            let attributes: Option<u32> = None;
            json!({
                "path": path.to_string_lossy(),
                "canonical_path": fs::canonicalize(path).ok().map(|path| path.to_string_lossy().into_owned()),
                "is_directory": metadata.is_dir(),
                "is_file": metadata.is_file(),
                "is_symlink": metadata.file_type().is_symlink(),
                "length": metadata.len(),
                "windows_attributes": attributes,
            })
        }
        Err(error) => json!({ "path": path.to_string_lossy(), "error": io_failure(&error) }),
    }
}

fn directory_probe(path: &Path) -> Value {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries
            .take(4096)
            .map(|entry| match entry {
                Ok(entry) => path_probe(&entry.path()),
                Err(error) => json!({ "error": io_failure(&error) }),
            })
            .collect::<Vec<_>>(),
        Err(error) => return json!({ "path": path.to_string_lossy(), "error": io_failure(&error) }),
    };
    json!({ "directory": path_probe(path), "entries": entries })
}

/// Keeps the exact native result separate from later, read-only diagnostic probes.
pub(crate) struct LookupTrace {
    app_data: Option<PathBuf>,
    data: Option<Value>,
}

impl LookupTrace {
    pub(crate) fn new(app_data: Option<&Path>, recipe: &Path, resources: Option<&Path>) -> Self {
        Self::create(
            cfg!(feature = "installation-lookup-diagnostics"),
            app_data,
            recipe,
            resources,
        )
    }

    fn create(
        enabled: bool,
        app_data: Option<&Path>,
        recipe: &Path,
        resources: Option<&Path>,
    ) -> Self {
        Self {
            app_data: app_data.map(Path::to_path_buf),
            data: enabled.then(|| json!({
                "started_millis": now_millis(),
                "app_data": app_data,
                "recipe": recipe,
                "resources": resources,
                "completed_directory_before": app_data.map(|root| directory_probe(&root.join("managed-installs"))),
                "campaign_directory_before": app_data.map(|root| directory_probe(&root.join("managed-campaigns"))),
                "note": "Original availability and result are authoritative for this lookup. Recheck probes run afterwards and may show changes between reads.",
            })),
        }
    }

    pub(crate) fn completed(&mut self, cards: &[ManagedInstallCard]) {
        let Some(data) = self.data.as_mut() else {
            return;
        };
        data["completed"] = json!(cards.iter().map(|card| {
            let receipt = card.record.managed_root.join(".chriz/install-receipt.json");
            json!({
                "record": card.record,
                "original_availability": card.availability,
                "recheck_error": inspect_install_availability(&card.record).err().map(|error| error.to_string()),
                "root_probe": path_probe(&card.record.managed_root),
                "receipt_probe": path_probe(&receipt),
                "receipt_sha256_now": fs::read(&receipt).ok().map(|bytes| sha256_bytes(&bytes)),
                "launcher_probe": path_probe(&card.record.launch_path),
            })
        }).collect::<Vec<_>>());
    }

    pub(crate) fn campaigns(
        &mut self,
        cards: &[ManagedCampaignCard],
        completed_ids: &BTreeSet<String>,
    ) {
        let Some(data) = self.data.as_mut() else {
            return;
        };
        data["campaigns"] = json!(cards.iter().map(|card| {
            let recheck = match inspect_campaign_availability(&card.record) {
                Ok(seal) => json!({ "fresh_copy_seal": seal.map(|seal| json!({
                    "step_id": seal.step_id, "attempt": seal.attempt, "detail": seal.detail,
                })) }),
                Err(error) => json!({ "error": error.to_string() }),
            };
            json!({
                "record": card.record,
                "original_availability": card.availability,
                "suppressed_by_completed_record": completed_ids.contains(&card.record.install_id),
                "recheck": recheck,
                "root_probe": path_probe(&card.record.managed_root),
                "completed_record_probe": self.app_data.as_ref().map(|root| path_probe(&root.join("managed-installs").join(format!("{}.json", card.record.install_id)))),
            })
        }).collect::<Vec<_>>());
    }

    pub(crate) fn finish(
        mut self,
        result: &Result<Vec<ManagedInstallationResponse>, CommandError>,
    ) {
        let Some(mut data) = self.data.take() else {
            return;
        };
        data["result"] = match result {
            Ok(copies) => json!({ "installations": copies.iter().map(|copy| json!({
                "id": copy.id, "name": copy.name, "path": copy.path,
                "status": copy.status, "available": copy.available,
                "resumable": copy.resumable, "recipe_version": copy.recipe_version,
            })).collect::<Vec<_>>() }),
            Err(error) => json!({ "error": error }),
        };
        write_event("installation_lookup", data);
    }
}

/// Capture the actual response at the IPC boundary, after update classification.
pub(crate) fn updates_returned(summary: &UpdateCenterResponse) {
    if cfg!(feature = "installation-lookup-diagnostics") {
        write_event(
            "updates_returned",
            json!({
                "checked_at": summary.checked_at,
                "managed_copies": summary.managed_copies,
            }),
        );
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClientUpdateTrace {
    checked_at: Option<String>,
    copies: Vec<ClientCopyTrace>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ClientCopyTrace {
    install_id: String,
    name: String,
    path: String,
    state: String,
}

pub(crate) fn client_received(report: ClientUpdateTrace) -> Result<(), CommandError> {
    if !cfg!(feature = "installation-lookup-diagnostics") {
        return Ok(());
    }
    if report.copies.len() > 4096
        || report
            .checked_at
            .as_ref()
            .is_some_and(|time| time.len() > 128)
        || report.copies.iter().any(|copy| {
            copy.install_id.len() > 128
                || copy.name.len() > 4096
                || copy.path.len() > 32768
                || copy.state.len() > 64
        })
    {
        return Err(CommandError::new(
            "diagnostic_trace_invalid",
            "Diagnostic report too large.",
            "No game files were changed.",
            "Client lookup report exceeds field bounds.",
        ));
    }
    write_event("updates_received_by_frontend", json!(report));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_trace_disabled_has_no_file_probes_or_created_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let absent = tmp.path().join("absent");
        let trace = LookupTrace::create(false, Some(&absent), &absent, None);
        assert!(trace.data.is_none());
        assert!(!absent.exists());
        assert_eq!(fs::read_dir(tmp.path()).unwrap().count(), 0);
    }

    #[test]
    fn lookup_trace_reports_missing_directory_without_creating_it() {
        let tmp = tempfile::tempdir().unwrap();
        let absent = tmp.path().join("absent");
        let trace = LookupTrace::create(true, Some(&absent), &absent, None);
        let data = trace.data.unwrap();
        assert_eq!(
            data["completed_directory_before"]["error"]["kind"],
            "NotFound"
        );
        assert!(!absent.exists());
    }

    #[test]
    fn lookup_trace_never_overwrites_existing_files_and_has_a_size_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("trace.jsonl");
        let mut sink = TraceSink::create(&path).unwrap();
        sink.append(&json!({"event": "first"})).unwrap();
        let original = fs::read(&path).unwrap();
        assert!(TraceSink::create(&path).is_err());
        sink.append(&json!({"oversized": "x".repeat(MAX_TRACE_BYTES)}))
            .unwrap();
        assert_eq!(fs::read(&path).unwrap(), original);
    }

    #[test]
    fn lookup_trace_keeps_io_error_codes_and_raw_directory_paths() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join("managed-installs")).unwrap();
        let path = tmp
            .path()
            .join("managed-installs")
            .join("install-example.json");
        fs::write(&path, b"{}").unwrap();
        let trace = LookupTrace::create(true, Some(tmp.path()), tmp.path(), None);
        let data = trace.data.unwrap();
        assert_eq!(
            data["completed_directory_before"]["entries"][0]["path"],
            json!(path)
        );
        assert_eq!(
            data["campaign_directory_before"]["error"]["kind"],
            "NotFound"
        );
        assert!(data["campaign_directory_before"]["error"]["os_error"].is_number());
        assert_eq!(fs::read(path).unwrap(), b"{}");
    }

    #[test]
    fn lookup_trace_preserves_original_decision_and_completed_precedence() {
        use bg_engine::registry::{CampaignAvailability, InstallAvailability};
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("not-present");
        let completed = ManagedInstallCard {
            record: serde_json::from_value(json!({
                "schema_version": 1, "install_id": "install-example", "display_name": "Repaired game",
                "managed_root": root, "recipe_version": "old (repaired)", "recipe_sha256": "00".repeat(32),
                "engine_name": "profile", "managed_save_root": tmp.path().join("profile"),
                "launch_path": root.join("game/InfinityLoader.exe"),
                "receipt_sha256": "00".repeat(32), "completed_at_millis": 123,
            })).unwrap(),
            availability: InstallAvailability::Available,
        };
        let campaign = ManagedCampaignCard {
            record: serde_json::from_value(json!({
                "schema_version": 1, "install_id": "install-example", "managed_root": root,
                "recipe_sha256": "00".repeat(32),
            }))
            .unwrap(),
            availability: CampaignAvailability::Stale,
        };
        let mut trace = LookupTrace::create(true, Some(tmp.path()), tmp.path(), None);
        trace.completed(&[completed]);
        trace.campaigns(&[campaign], &BTreeSet::from(["install-example".to_owned()]));
        let data = trace.data.unwrap();
        assert_eq!(data["completed"][0]["original_availability"], "available");
        assert!(data["completed"][0]["recheck_error"].is_string());
        assert_eq!(data["campaigns"][0]["suppressed_by_completed_record"], true);
        assert_eq!(data["campaigns"][0]["original_availability"], "stale");
        assert!(!root.exists());
    }

    #[test]
    fn lookup_trace_client_report_is_bounded() {
        let report = ClientUpdateTrace {
            checked_at: Some("x".repeat(129)),
            copies: vec![],
        };
        let result = client_received(report);
        if cfg!(feature = "installation-lookup-diagnostics") {
            assert_eq!(result.unwrap_err().code, "diagnostic_trace_invalid");
        } else {
            assert!(result.is_ok());
        }
    }
}

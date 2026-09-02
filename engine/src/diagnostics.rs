//! Privacy-bounded diagnostics bundles assembled from explicit managed-state paths.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;
use thiserror::Error;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

const STATE_DIRECTORY: &str = ".chriz";
const TEMP_SUFFIX: &str = "diagnostics-tmp";
const STEP_EVIDENCE_FILES: [&str; 5] = [
    "weidu.debug.log",
    "process-output.log",
    "before.log",
    "after.log",
    "invocation.json",
];
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Inputs for one create-once sanitized diagnostics export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticsRequest {
    /// Existing managed installation root.
    pub managed_root: PathBuf,
    /// Attempt whose receipt and logs should be included.
    pub attempt_id: String,
    /// New ZIP path; existing output is never replaced.
    pub output_path: PathBuf,
    /// Additional private path prefixes replaced by `<redacted-home>` in text entries.
    pub redact_roots: Vec<PathBuf>,
}

/// Published diagnostics bundle and its deterministic entry inventory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticsBundle {
    /// Canonical create-once ZIP path.
    pub path: PathBuf,
    /// Sorted archive entry names.
    pub entries: Vec<String>,
}

/// Failure to select, sanitize, or publish diagnostics.
#[derive(Debug, Error)]
pub enum DiagnosticsError {
    /// A request or managed path violates the diagnostics boundary.
    #[error("unsafe diagnostics path {path}: {message}")]
    UnsafePath {
        /// Rejected path.
        path: PathBuf,
        /// Validation detail.
        message: String,
    },
    /// A required receipt, ledger, or frozen-recipe file is absent.
    #[error("required diagnostics evidence is missing at {path}")]
    MissingEvidence {
        /// Required allowlisted path.
        path: PathBuf,
    },
    /// The create-once output path already exists.
    #[error("diagnostics output already exists at {path}")]
    OutputExists {
        /// Existing destination.
        path: PathBuf,
    },
    /// Filesystem access failed.
    #[error("{path}: {source}")]
    Io {
        /// Path being inspected or written.
        path: PathBuf,
        /// Underlying error.
        #[source]
        source: std::io::Error,
    },
    /// ZIP encoding failed without publishing a partial final.
    #[error("could not build diagnostics ZIP at {path}: {message}")]
    Zip {
        /// Temporary or final ZIP path.
        path: PathBuf,
        /// ZIP library detail.
        message: String,
    },
    /// Attempt receipt JSON could not be read as an evidence link.
    #[error("could not parse attempt receipt at {path}: {message}")]
    ReceiptJson {
        /// Attempt receipt path.
        path: PathBuf,
        /// JSON parsing detail.
        message: String,
    },
}

#[derive(Clone, Debug)]
struct SelectedEntry {
    source: PathBuf,
    archive_name: String,
    binary: bool,
}

#[derive(Deserialize)]
struct ReceiptEvidenceLink {
    #[serde(default)]
    evidence_attempt_id: Option<String>,
}

/// Export receipt, ledger, frozen recipe, and attempt logs through a fixed allowlist.
///
/// Archives, game content, credentials, key material, and arbitrary state files are never
/// discovered as candidates. Text candidates are redacted before entering the bundle.
pub fn export_diagnostics(
    request: &DiagnosticsRequest,
) -> Result<DiagnosticsBundle, DiagnosticsError> {
    validate_identifier(&request.attempt_id, &request.managed_root)?;
    let managed_root = canonical_direct_directory(&request.managed_root)?;
    let state_root = managed_root.join(STATE_DIRECTORY);
    validate_direct_directory(&state_root)?;

    let output_path = resolve_new_output(&request.output_path, &managed_root)?;
    let selected = select_entries(&state_root, &request.attempt_id)?;
    let archive_names = selected
        .iter()
        .map(|entry| entry.archive_name.clone())
        .collect::<Vec<_>>();
    let redactions = collect_redactions(&request.redact_roots);
    let temporary = temporary_sibling(&output_path)?;

    let result =
        write_bundle(&temporary, &selected, &redactions).and_then(|()| {
            match fs::hard_link(&temporary, &output_path) {
                Ok(()) => Ok(()),
                Err(_source) if output_path.exists() => Err(DiagnosticsError::OutputExists {
                    path: output_path.clone(),
                }),
                Err(source) => Err(DiagnosticsError::Io {
                    path: output_path.clone(),
                    source,
                }),
            }
        });
    let cleanup = fs::remove_file(&temporary);
    if let Err(error) = result {
        let _ = cleanup;
        return Err(error);
    }
    cleanup.map_err(|source| DiagnosticsError::Io {
        path: temporary,
        source,
    })?;

    Ok(DiagnosticsBundle {
        path: output_path,
        entries: archive_names,
    })
}

fn select_entries(
    state_root: &Path,
    attempt_id: &str,
) -> Result<Vec<SelectedEntry>, DiagnosticsError> {
    let mut selected = BTreeMap::<String, SelectedEntry>::new();
    let attempt_root = state_root.join("attempts").join(attempt_id);
    validate_direct_directory(&attempt_root)?;
    let receipt_path = attempt_root.join("receipt.json");
    add_required(
        &mut selected,
        receipt_path.clone(),
        "receipt/attempt-receipt.json",
        false,
    )?;
    let evidence_attempt_id = receipt_evidence_attempt_id(&receipt_path, attempt_id)?;
    validate_identifier(&evidence_attempt_id, &receipt_path)?;
    add_optional(
        &mut selected,
        state_root.join("install-receipt.json"),
        "receipt/install-receipt.json",
        false,
    )?;
    add_required(
        &mut selected,
        state_root.join("recipe/payload.zip"),
        "recipe/payload.zip",
        true,
    )?;
    add_required(
        &mut selected,
        state_root.join("recipe/envelope.json"),
        "recipe/envelope.json",
        false,
    )?;

    let ledger_root = state_root.join("ledger");
    validate_direct_directory(&ledger_root)?;
    let mut ledger_count = 0_usize;
    for entry in fs::read_dir(&ledger_root).map_err(|source| DiagnosticsError::Io {
        path: ledger_root.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| DiagnosticsError::Io {
            path: ledger_root.clone(),
            source,
        })?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if is_ledger_record_name(name) {
            add_required(
                &mut selected,
                entry.path(),
                &format!("ledger/{name}"),
                false,
            )?;
            ledger_count += 1;
        }
    }
    if ledger_count == 0 {
        return Err(DiagnosticsError::MissingEvidence { path: ledger_root });
    }

    let evidence_attempt_root = state_root.join("attempts").join(evidence_attempt_id);
    validate_direct_directory(&evidence_attempt_root)?;
    let steps_root = evidence_attempt_root.join("steps");
    if steps_root.exists() {
        validate_direct_directory(&steps_root)?;
        add_step_evidence(&mut selected, &steps_root)?;
    }

    Ok(selected.into_values().collect())
}

fn receipt_evidence_attempt_id(
    receipt_path: &Path,
    fallback: &str,
) -> Result<String, DiagnosticsError> {
    let bytes = fs::read(receipt_path).map_err(|source| DiagnosticsError::Io {
        path: receipt_path.to_path_buf(),
        source,
    })?;
    let link: ReceiptEvidenceLink =
        serde_json::from_slice(&bytes).map_err(|source| DiagnosticsError::ReceiptJson {
            path: receipt_path.to_path_buf(),
            message: source.to_string(),
        })?;
    Ok(link
        .evidence_attempt_id
        .unwrap_or_else(|| fallback.to_owned()))
}

fn add_required(
    entries: &mut BTreeMap<String, SelectedEntry>,
    source: PathBuf,
    archive_name: &str,
    binary: bool,
) -> Result<(), DiagnosticsError> {
    validate_direct_file(&source).map_err(|error| match error {
        DiagnosticsError::Io {
            ref source,
            path: _,
        } if source.kind() == std::io::ErrorKind::NotFound => DiagnosticsError::MissingEvidence {
            path: source_path(&error),
        },
        other => other,
    })?;
    entries.insert(
        archive_name.to_owned(),
        SelectedEntry {
            source,
            archive_name: archive_name.to_owned(),
            binary,
        },
    );
    Ok(())
}

fn source_path(error: &DiagnosticsError) -> PathBuf {
    match error {
        DiagnosticsError::Io { path, .. }
        | DiagnosticsError::UnsafePath { path, .. }
        | DiagnosticsError::MissingEvidence { path }
        | DiagnosticsError::OutputExists { path }
        | DiagnosticsError::Zip { path, .. }
        | DiagnosticsError::ReceiptJson { path, .. } => path.clone(),
    }
}

fn add_step_evidence(
    entries: &mut BTreeMap<String, SelectedEntry>,
    steps_root: &Path,
) -> Result<(), DiagnosticsError> {
    for step_entry in read_directory(steps_root)? {
        let step_path = step_entry.path();
        let metadata = direct_metadata(&step_path)?;
        if !metadata.is_dir() {
            continue;
        }
        let step_name = step_entry.file_name();
        let Some(step_name) = step_name.to_str() else {
            continue;
        };
        if !is_task13_step_directory(step_name) {
            continue;
        }

        for attempt_entry in read_directory(&step_path)? {
            let attempt_path = attempt_entry.path();
            let metadata = direct_metadata(&attempt_path)?;
            if !metadata.is_dir() {
                continue;
            }
            let attempt_name = attempt_entry.file_name();
            let Some(attempt_name) = attempt_name.to_str() else {
                continue;
            };
            if !is_task13_attempt_directory(attempt_name) {
                continue;
            }

            for evidence_entry in read_directory(&attempt_path)? {
                let evidence_path = evidence_entry.path();
                let metadata = direct_metadata(&evidence_path)?;
                if !metadata.is_file() {
                    continue;
                }
                let evidence_name = evidence_entry.file_name();
                let Some(evidence_name) = evidence_name.to_str() else {
                    continue;
                };
                if !STEP_EVIDENCE_FILES.contains(&evidence_name) {
                    continue;
                }
                let relative = Path::new(step_name).join(attempt_name).join(evidence_name);
                let portable = portable_relative(&relative)?;
                add_required(
                    entries,
                    evidence_path,
                    &format!("logs/steps/{portable}"),
                    false,
                )?;
            }
        }
    }
    Ok(())
}

fn read_directory(path: &Path) -> Result<Vec<fs::DirEntry>, DiagnosticsError> {
    fs::read_dir(path)
        .map_err(|source| DiagnosticsError::Io {
            path: path.to_path_buf(),
            source,
        })?
        .map(|entry| {
            entry.map_err(|source| DiagnosticsError::Io {
                path: path.to_path_buf(),
                source,
            })
        })
        .collect()
}

fn direct_metadata(path: &Path) -> Result<fs::Metadata, DiagnosticsError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| DiagnosticsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() {
        return Err(unsafe_path(path, "links are forbidden in diagnostics logs"));
    }
    Ok(metadata)
}

fn is_task13_step_directory(name: &str) -> bool {
    name.len() == 21
        && name.as_bytes()[..4]
            .iter()
            .all(|byte| byte.is_ascii_digit())
        && name.as_bytes()[4] == b'-'
        && name.as_bytes()[5..]
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
}

fn is_task13_attempt_directory(name: &str) -> bool {
    name.strip_prefix("attempt-").is_some_and(|attempt| {
        attempt.len() == 4 && attempt.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn add_optional(
    entries: &mut BTreeMap<String, SelectedEntry>,
    source: PathBuf,
    archive_name: &str,
    binary: bool,
) -> Result<(), DiagnosticsError> {
    match fs::symlink_metadata(&source) {
        Ok(_) => add_required(entries, source, archive_name, binary),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source_error) => Err(DiagnosticsError::Io {
            path: source,
            source: source_error,
        }),
    }
}

fn write_bundle(
    temporary: &Path,
    selected: &[SelectedEntry],
    redactions: &[String],
) -> Result<(), DiagnosticsError> {
    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temporary)
        .map_err(|source| DiagnosticsError::Io {
            path: temporary.to_path_buf(),
            source,
        })?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for entry in selected {
        writer
            .start_file(&entry.archive_name, options)
            .map_err(|source| DiagnosticsError::Zip {
                path: temporary.to_path_buf(),
                message: source.to_string(),
            })?;
        let bytes = fs::read(&entry.source).map_err(|source| DiagnosticsError::Io {
            path: entry.source.clone(),
            source,
        })?;
        if entry.binary {
            writer
                .write_all(&bytes)
                .map_err(|source| DiagnosticsError::Io {
                    path: temporary.to_path_buf(),
                    source,
                })?;
        } else {
            let sanitized = sanitize_text(&String::from_utf8_lossy(&bytes), redactions);
            writer
                .write_all(sanitized.as_bytes())
                .map_err(|source| DiagnosticsError::Io {
                    path: temporary.to_path_buf(),
                    source,
                })?;
        }
    }
    let file = writer.finish().map_err(|source| DiagnosticsError::Zip {
        path: temporary.to_path_buf(),
        message: source.to_string(),
    })?;
    file.sync_all().map_err(|source| DiagnosticsError::Io {
        path: temporary.to_path_buf(),
        source,
    })
}

fn sanitize_text(text: &str, redactions: &[String]) -> String {
    let mut redacted = text.to_owned();
    for root in redactions {
        redacted = replace_ascii_case_insensitive(&redacted, root, "<redacted-home>");
    }

    let mut output = String::new();
    let mut private_key = false;
    for line in redacted.split_inclusive('\n') {
        let lowercase = line.to_ascii_lowercase();
        if lowercase.contains("-----begin ") && lowercase.contains("private key-----") {
            private_key = true;
            output.push_str("<redacted-private-key>\n");
            continue;
        }
        if private_key {
            if lowercase.contains("-----end ") && lowercase.contains("private key-----") {
                private_key = false;
            }
            continue;
        }
        if contains_sensitive_marker(line) {
            output.push_str("<redacted-sensitive-line>\n");
        } else {
            output.push_str(line);
        }
    }
    output
}

fn contains_sensitive_marker(line: &str) -> bool {
    let lowercase = line.to_ascii_lowercase();
    let has_marker = [
        "authorization:",
        "bearer ",
        "password",
        "passwd",
        "api_key",
        "api-key",
        "client_secret",
        "clientsecret",
        "private_key",
        "secret-token",
        "access_token",
        "refresh_token",
        "ghp_",
        "gho_",
        "github_pat_",
        "cookie:",
    ]
    .iter()
    .any(|marker| lowercase.contains(marker));
    has_marker || contains_aws_access_key(line) || contains_query_secret(&lowercase)
}

fn contains_aws_access_key(line: &str) -> bool {
    line.as_bytes().windows(20).any(|candidate| {
        matches!(&candidate[..4], b"AKIA" | b"ASIA")
            && candidate[4..]
                .iter()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    })
}

fn contains_query_secret(lowercase: &str) -> bool {
    let Some((_, query)) = lowercase.split_once('?') else {
        return false;
    };
    query.split('&').any(|parameter| {
        let key = parameter.split_once('=').map_or(parameter, |(key, _)| key);
        matches!(
            key,
            "token"
                | "access_token"
                | "refresh_token"
                | "api_key"
                | "api-key"
                | "apikey"
                | "client_secret"
                | "client-secret"
                | "clientsecret"
                | "password"
                | "passwd"
                | "signature"
                | "sig"
                | "x-amz-signature"
                | "cookie"
        )
    })
}

fn replace_ascii_case_insensitive(input: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() {
        return input.to_owned();
    }
    let lowercase = input.to_ascii_lowercase();
    let needle_lower = needle.to_ascii_lowercase();
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0_usize;
    while let Some(relative) = lowercase[cursor..].find(&needle_lower) {
        let start = cursor + relative;
        output.push_str(&input[cursor..start]);
        output.push_str(replacement);
        cursor = start + needle.len();
    }
    output.push_str(&input[cursor..]);
    output
}

fn collect_redactions(authored: &[PathBuf]) -> Vec<String> {
    let roots = authored
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let mut values = roots;
    for variable in ["USERPROFILE", "HOME"] {
        if let Some(value) = std::env::var_os(variable) {
            let value = value.to_string_lossy().into_owned();
            if !value.is_empty() {
                values.push(value);
            }
        }
    }
    values = values
        .into_iter()
        .flat_map(|root| {
            let slash = root.replace('\\', "/");
            let backslash = root.replace('/', "\\");
            [root, slash, backslash]
                .into_iter()
                .flat_map(|variant| {
                    let json_escaped = variant.replace('\\', "\\\\");
                    [variant, json_escaped]
                })
                .collect::<Vec<_>>()
        })
        .filter(|value| !value.is_empty())
        .collect();
    values.sort_by_key(|value| std::cmp::Reverse(value.len()));
    values.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    values
}

fn is_ledger_record_name(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".json") else {
        return false;
    };
    stem.len() == 10 && stem.bytes().all(|byte| byte.is_ascii_digit())
}

fn portable_relative(path: &Path) -> Result<String, DiagnosticsError> {
    let mut parts = Vec::new();
    for component in path.components() {
        let Component::Normal(value) = component else {
            return Err(unsafe_path(path, "log path is not a strict relative path"));
        };
        let value = value
            .to_str()
            .ok_or_else(|| unsafe_path(path, "log path is not valid Unicode"))?;
        if value.is_empty() || value == "." || value == ".." {
            return Err(unsafe_path(path, "log path contains an unsafe component"));
        }
        parts.push(value);
    }
    Ok(parts.join("/"))
}

fn resolve_new_output(requested: &Path, managed_root: &Path) -> Result<PathBuf, DiagnosticsError> {
    if !requested.is_absolute() {
        return Err(unsafe_path(requested, "output path must be absolute"));
    }
    let parent = requested
        .parent()
        .ok_or_else(|| unsafe_path(requested, "output has no parent"))?;
    let parent = canonical_direct_directory(parent)?;
    let name = requested
        .file_name()
        .ok_or_else(|| unsafe_path(requested, "output has no filename"))?;
    let output = parent.join(name);
    if output.starts_with(managed_root) {
        return Err(unsafe_path(
            &output,
            "diagnostics output must be outside the managed installation",
        ));
    }
    match fs::symlink_metadata(&output) {
        Ok(_) => Err(DiagnosticsError::OutputExists { path: output }),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(output),
        Err(source) => Err(DiagnosticsError::Io {
            path: output,
            source,
        }),
    }
}

fn temporary_sibling(path: &Path) -> Result<PathBuf, DiagnosticsError> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| unsafe_path(path, "output filename is not valid Unicode"))?;
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Ok(path.with_file_name(format!(
        ".{name}.{}.{}.{}",
        std::process::id(),
        sequence,
        TEMP_SUFFIX
    )))
}

fn validate_identifier(value: &str, path: &Path) -> Result<(), DiagnosticsError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(unsafe_path(path, "attempt id is not path-safe"));
    }
    Ok(())
}

fn canonical_direct_directory(path: &Path) -> Result<PathBuf, DiagnosticsError> {
    validate_direct_directory(path)?;
    fs::canonicalize(path).map_err(|source| DiagnosticsError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn validate_direct_directory(path: &Path) -> Result<(), DiagnosticsError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| DiagnosticsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(unsafe_path(path, "expected a direct non-symlink directory"));
    }
    Ok(())
}

fn validate_direct_file(path: &Path) -> Result<(), DiagnosticsError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| DiagnosticsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(unsafe_path(path, "expected a direct non-symlink file"));
    }
    Ok(())
}

fn unsafe_path(path: &Path, message: &str) -> DiagnosticsError {
    DiagnosticsError::UnsafePath {
        path: path.to_path_buf(),
        message: message.to_owned(),
    }
}

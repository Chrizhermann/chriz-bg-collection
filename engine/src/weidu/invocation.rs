//! Deterministic, target-confined WeiDU command construction.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::manifest::{GameRoot, InvocationMode, RunArg};

const DEBUG_LOG_NAME: &str = "weidu.debug.log";

/// One prompt matcher and the exact bytes sent after it matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPrompt {
    /// Byte sequence that must appear in WeiDU output before answering.
    pub expected_output: Vec<u8>,
    /// Exact response bytes, including any authored line ending.
    pub answer: Vec<u8>,
}

/// Canonical staged roots available to typed invocation arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedRoots {
    /// Staged BG:EE plus SoD root.
    pub bg1: PathBuf,
    /// Staged BG2:EE / EET root.
    pub bg2: PathBuf,
}

impl StagedRoots {
    fn get(&self, root: GameRoot) -> &Path {
        match root {
            GameRoot::Bg1 => &self.bg1,
            GameRoot::Bg2 => &self.bg2,
        }
    }
}

/// A WeiDU executable whose bytes matched the recipe-pinned SHA-256.
#[derive(Debug, Clone)]
pub struct VerifiedWeidu {
    source_path: PathBuf,
    sha256: String,
    bytes: Vec<u8>,
}

impl VerifiedWeidu {
    /// Reads a regular executable and accepts it only when its digest matches `expected_sha256`.
    pub fn verify(path: &Path, expected_sha256: &str) -> Result<Self, InvocationError> {
        if expected_sha256.len() != 64
            || !expected_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(InvocationError::InvalidToolDigest {
                digest: expected_sha256.to_owned(),
            });
        }

        let metadata = fs::symlink_metadata(path).map_err(|source| InvocationError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(InvocationError::UnsafeToolPath {
                path: path.to_path_buf(),
            });
        }

        let source_path = path.canonicalize().map_err(|source| InvocationError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let bytes = fs::read(&source_path).map_err(|source| InvocationError::Io {
            path: source_path.clone(),
            source,
        })?;
        let actual = hex::encode(Sha256::digest(&bytes));
        let expected = expected_sha256.to_ascii_lowercase();
        if actual != expected {
            return Err(InvocationError::ToolHashMismatch {
                path: source_path,
                expected,
                actual,
            });
        }

        Ok(Self {
            source_path,
            sha256: expected,
            bytes,
        })
    }

    /// Canonical source path whose bytes were verified.
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    /// Lowercase SHA-256 identity of the verified bytes.
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

/// Narrow, already-resolved input needed to construct one install invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationInput {
    /// Staged game root selected for this run.
    pub target: GameRoot,
    /// Recipe-authored, relative TP2 path.
    pub tp2: String,
    /// How WeiDU locates the TP2.
    pub mode: InvocationMode,
    /// Compatibility evidence gate for non-default explicit TP2 invocation.
    pub explicit_tp2_tested: bool,
    /// Installer-authored WeiDU language number.
    pub language: u32,
    /// Exact ordered component suffix still needing installation.
    pub remaining_components: Vec<u32>,
    /// Typed extra arguments resolved at invocation time.
    pub run_args: Vec<RunArg>,
    /// Ordered prompt script resolved from selected component inputs.
    pub prompts: Vec<ResolvedPrompt>,
}

/// A fully resolved process description. This type does not execute a process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    /// Target-local executable containing only the verified pinned WeiDU bytes.
    pub program: PathBuf,
    /// Canonical staged game root used as the child working directory.
    pub cwd: PathBuf,
    /// Exact deterministic process arguments.
    pub args: Vec<OsString>,
    /// Output-gated prompt answers for the future process supervisor.
    pub prompts: Vec<ResolvedPrompt>,
    /// Canonical engine-owned debug log path inside this invocation attempt.
    pub debug_path: PathBuf,
    /// SHA-256 over every consequential invocation identity field.
    pub identity_digest: String,
}

/// Failure to validate or construct a safe WeiDU invocation.
#[derive(Debug, Error)]
pub enum InvocationError {
    /// A filesystem operation failed at a named path.
    #[error("{path}: {source}")]
    Io {
        /// Path involved in the failed operation.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The pinned digest was not syntactically a SHA-256.
    #[error("invalid pinned WeiDU SHA-256 {digest:?}")]
    InvalidToolDigest {
        /// Rejected digest text.
        digest: String,
    },
    /// The tool source was a symlink or not a regular file.
    #[error("WeiDU tool source is not a direct regular file: {path}")]
    UnsafeToolPath {
        /// Rejected source path.
        path: PathBuf,
    },
    /// Actual tool bytes did not match the signed identity.
    #[error("WeiDU hash mismatch for {path}: expected {expected}, found {actual}")]
    ToolHashMismatch {
        /// Canonical checked tool path.
        path: PathBuf,
        /// Recipe-pinned hash.
        expected: String,
        /// Hash of the bytes read.
        actual: String,
    },
    /// The recipe TP2 path could escape or alias its staged root.
    #[error("unsafe recipe TP2 path {path:?}")]
    UnsafeTp2Path {
        /// Rejected authored path.
        path: String,
    },
    /// Two setup-name declarations map to one executable identity.
    #[error("TP2 paths {first:?} and {second:?} both map to setup executable {executable:?}")]
    AmbiguousSetupName {
        /// Colliding executable filename.
        executable: String,
        /// First declaration.
        first: String,
        /// Later declaration.
        second: String,
    },
    /// Explicit TP2 mode lacked signed compatibility evidence.
    #[error("explicit TP2 invocation is not marked tested for {path:?}")]
    UntestedExplicitTp2 {
        /// TP2 path requiring evidence.
        path: String,
    },
    /// There is no component after reconciliation, so no invocation should run.
    #[error("cannot invoke WeiDU with an empty remaining component list")]
    NoComponents,
    /// A remaining component id appeared more than once.
    #[error("remaining WeiDU component {component} is duplicated")]
    DuplicateComponent {
        /// Repeated component id.
        component: u32,
    },
    /// A typed extra argument attempted to alter protected invocation behavior.
    #[error("forbidden recipe run argument {argument:?}")]
    ForbiddenRunArgument {
        /// Rejected literal argument.
        argument: String,
    },
    /// The attempt directory was a symlink or not a directory.
    #[error("unsafe invocation attempt directory {path}")]
    UnsafeAttemptDirectory {
        /// Rejected attempt path.
        path: PathBuf,
    },
    /// A staged game root was missing or not a directory.
    #[error("staged game root is not a directory: {path}")]
    UnsafeGameRoot {
        /// Rejected staged root.
        path: PathBuf,
    },
    /// The generated log resolved outside its create-once attempt root.
    #[error("generated log path {path} escapes attempt directory {attempt}")]
    LogEscapesAttempt {
        /// Canonical attempt directory.
        attempt: PathBuf,
        /// Resolved log destination.
        path: PathBuf,
    },
    /// The generated log destination was not a direct regular file.
    #[error("unsafe generated log destination {path}")]
    UnsafeLogDestination {
        /// Rejected log path.
        path: PathBuf,
    },
    /// The target-local program destination was a symlink or non-file.
    #[error("unsafe target-local WeiDU destination {path}")]
    UnsafeProgramDestination {
        /// Rejected program path.
        path: PathBuf,
    },
}

/// Constructs one deterministic invocation after resolving and confining every path.
///
/// The attempt directory must already be a direct, create-once directory. The builder
/// creates its fixed log file if absent and materializes the verified WeiDU bytes under
/// the target-local executable name. It never executes the resulting process.
pub fn build(
    input: &InvocationInput,
    roots: &StagedRoots,
    tool: &VerifiedWeidu,
    attempt_dir: &Path,
) -> Result<Invocation, InvocationError> {
    let tp2 = normalize_tp2_path(&input.tp2)?;
    if input.mode == InvocationMode::ExplicitTp2 && !input.explicit_tp2_tested {
        return Err(InvocationError::UntestedExplicitTp2 {
            path: input.tp2.clone(),
        });
    }
    validate_components(&input.remaining_components)?;
    validate_run_arguments(&input.run_args)?;

    let cwd = canonical_game_root(roots.get(input.target))?;
    let canonical_roots = CanonicalRoots {
        bg1: canonical_game_root(&roots.bg1)?,
        bg2: canonical_game_root(&roots.bg2)?,
    };
    let debug_path = resolve_debug_path(attempt_dir)?;

    let executable_name = match input.mode {
        InvocationMode::SetupName => setup_executable_name_from_normalized(&tp2)?,
        InvocationMode::ExplicitTp2 => "WeiDU.exe".to_owned(),
    };
    let program = materialize_verified_tool(&cwd, &executable_name, tool)?;

    let mut args = Vec::new();
    if input.mode == InvocationMode::ExplicitTp2 {
        args.push(OsString::from(&tp2));
    }
    args.extend([
        OsString::from("--language"),
        OsString::from(input.language.to_string()),
        OsString::from("--use-lang"),
        OsString::from("en_US"),
        OsString::from("--force-install-list"),
    ]);
    args.extend(
        input
            .remaining_components
            .iter()
            .map(|component| OsString::from(component.to_string())),
    );
    args.extend([
        OsString::from("--no-exit-pause"),
        OsString::from("--skip-at-view"),
        OsString::from("--safe-exit"),
        OsString::from("--noautoupdate"),
        OsString::from("--log"),
        debug_path.as_os_str().to_owned(),
    ]);
    for argument in &input.run_args {
        match argument {
            RunArg::Literal(value) => args.push(OsString::from(value)),
            RunArg::StagedRoot(root) => {
                args.push(canonical_roots.get(*root).as_os_str().to_owned())
            }
        }
    }

    let prompts = input.prompts.clone();
    let identity_digest =
        invocation_digest(&program, &cwd, &args, &prompts, &debug_path, tool.sha256());

    Ok(Invocation {
        program,
        cwd,
        args,
        prompts,
        debug_path,
        identity_digest,
    })
}

/// Rejects setup-mode TP2 declarations that collapse to one setup executable name.
///
/// Callers pass only declarations using [`InvocationMode::SetupName`]. This deliberately
/// shares the exact path validation and normalization used by [`build`].
pub fn validate_setup_name_declarations<I, S>(paths: I) -> Result<(), InvocationError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut seen = BTreeMap::<String, String>::new();
    for path in paths {
        let path = path.as_ref();
        let normalized = normalize_tp2_path(path)?;
        let executable = setup_executable_name_from_normalized(&normalized)?;
        let key = executable.to_ascii_lowercase();
        if let Some(first) = seen.insert(key, path.to_owned()) {
            return Err(InvocationError::AmbiguousSetupName {
                executable,
                first,
                second: path.to_owned(),
            });
        }
    }
    Ok(())
}

/// Derives the one canonical setup executable name for a safe recipe TP2 path.
pub fn setup_executable_name(tp2: &str) -> Result<String, InvocationError> {
    let normalized = normalize_tp2_path(tp2)?;
    setup_executable_name_from_normalized(&normalized)
}

struct CanonicalRoots {
    bg1: PathBuf,
    bg2: PathBuf,
}

impl CanonicalRoots {
    fn get(&self, root: GameRoot) -> &Path {
        match root {
            GameRoot::Bg1 => &self.bg1,
            GameRoot::Bg2 => &self.bg2,
        }
    }
}

fn normalize_tp2_path(value: &str) -> Result<String, InvocationError> {
    let unsafe_path = || InvocationError::UnsafeTp2Path {
        path: value.to_owned(),
    };
    if value.is_empty()
        || value.starts_with(['/', '\\'])
        || value.contains(':')
        || value
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_control())
    {
        return Err(unsafe_path());
    }

    let parts = value.split(['/', '\\']).collect::<Vec<_>>();
    if parts.iter().any(|part| {
        part.is_empty()
            || matches!(*part, "." | "..")
            || part.ends_with([' ', '.'])
            || part.contains(['<', '>', '"', '|', '?', '*'])
    }) {
        return Err(unsafe_path());
    }

    let file = parts.last().copied().ok_or_else(unsafe_path)?;
    if file.len() <= 4 || !file[file.len() - 4..].eq_ignore_ascii_case(".tp2") {
        return Err(unsafe_path());
    }
    let stem = &file[..file.len() - 4];
    let normalized_stem = strip_setup_prefix(stem);
    if normalized_stem.is_empty() {
        return Err(unsafe_path());
    }

    Ok(parts.join("/"))
}

fn setup_executable_name_from_normalized(tp2: &str) -> Result<String, InvocationError> {
    let file = tp2
        .rsplit('/')
        .next()
        .ok_or_else(|| InvocationError::UnsafeTp2Path {
            path: tp2.to_owned(),
        })?;
    let stem = &file[..file.len() - 4];
    let stem = strip_setup_prefix(stem).to_ascii_lowercase();
    Ok(format!("Setup-{stem}.exe"))
}

fn strip_setup_prefix(stem: &str) -> &str {
    stem.get(..6)
        .filter(|prefix| prefix.eq_ignore_ascii_case("setup-"))
        .map_or(stem, |_| &stem[6..])
}

fn validate_components(components: &[u32]) -> Result<(), InvocationError> {
    if components.is_empty() {
        return Err(InvocationError::NoComponents);
    }
    let mut seen = BTreeSet::new();
    for component in components {
        if !seen.insert(*component) {
            return Err(InvocationError::DuplicateComponent {
                component: *component,
            });
        }
    }
    Ok(())
}

fn validate_run_arguments(arguments: &[RunArg]) -> Result<(), InvocationError> {
    const PROTECTED: &[&str] = &[
        "--quick-log",
        "--yes",
        "--reinstall",
        "--uninstall",
        "--uninstall-list",
        "--force-uninstall-list",
        "--language",
        "--use-lang",
        "--force-install-list",
        "--no-exit-pause",
        "--skip-at-view",
        "--safe-exit",
        "--noautoupdate",
        "--log",
    ];

    for argument in arguments {
        let RunArg::Literal(argument) = argument else {
            continue;
        };
        let lowercase = argument.to_ascii_lowercase();
        if argument.contains('\0')
            || PROTECTED.iter().any(|protected| {
                lowercase == *protected
                    || lowercase
                        .strip_prefix(protected)
                        .is_some_and(|suffix| suffix.starts_with('='))
            })
        {
            return Err(InvocationError::ForbiddenRunArgument {
                argument: argument.clone(),
            });
        }
    }
    Ok(())
}

fn canonical_game_root(path: &Path) -> Result<PathBuf, InvocationError> {
    let metadata = fs::metadata(path).map_err(|source| InvocationError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.is_dir() {
        return Err(InvocationError::UnsafeGameRoot {
            path: path.to_path_buf(),
        });
    }
    path.canonicalize().map_err(|source| InvocationError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn resolve_debug_path(attempt_dir: &Path) -> Result<PathBuf, InvocationError> {
    let metadata = fs::symlink_metadata(attempt_dir).map_err(|source| InvocationError::Io {
        path: attempt_dir.to_path_buf(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(InvocationError::UnsafeAttemptDirectory {
            path: attempt_dir.to_path_buf(),
        });
    }
    let attempt = attempt_dir
        .canonicalize()
        .map_err(|source| InvocationError::Io {
            path: attempt_dir.to_path_buf(),
            source,
        })?;
    let candidate = attempt.join(DEBUG_LOG_NAME);

    match fs::symlink_metadata(&candidate) {
        Ok(candidate_metadata) => {
            let resolved = candidate
                .canonicalize()
                .map_err(|source| InvocationError::Io {
                    path: candidate.clone(),
                    source,
                })?;
            if !resolved.starts_with(&attempt) || resolved == attempt {
                return Err(InvocationError::LogEscapesAttempt {
                    attempt,
                    path: resolved,
                });
            }
            if candidate_metadata.file_type().is_symlink() || !candidate_metadata.is_file() {
                return Err(InvocationError::UnsafeLogDestination { path: candidate });
            }
            Ok(resolved)
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&candidate)
                .map_err(|source| InvocationError::Io {
                    path: candidate.clone(),
                    source,
                })?;
            file.sync_all().map_err(|source| InvocationError::Io {
                path: candidate.clone(),
                source,
            })?;
            drop(file);
            let resolved = candidate
                .canonicalize()
                .map_err(|source| InvocationError::Io {
                    path: candidate.clone(),
                    source,
                })?;
            if !resolved.starts_with(&attempt) || resolved == attempt {
                return Err(InvocationError::LogEscapesAttempt {
                    attempt,
                    path: resolved,
                });
            }
            Ok(resolved)
        }
        Err(source) => Err(InvocationError::Io {
            path: candidate,
            source,
        }),
    }
}

fn materialize_verified_tool(
    root: &Path,
    executable_name: &str,
    tool: &VerifiedWeidu,
) -> Result<PathBuf, InvocationError> {
    let destination = root.join(executable_name);
    match fs::symlink_metadata(&destination) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(InvocationError::UnsafeProgramDestination { path: destination });
            }
            let existing = fs::read(&destination).map_err(|source| InvocationError::Io {
                path: destination.clone(),
                source,
            })?;
            if existing != tool.bytes {
                let mut file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&destination)
                    .map_err(|source| InvocationError::Io {
                        path: destination.clone(),
                        source,
                    })?;
                file.write_all(&tool.bytes)
                    .map_err(|source| InvocationError::Io {
                        path: destination.clone(),
                        source,
                    })?;
                file.sync_all().map_err(|source| InvocationError::Io {
                    path: destination.clone(),
                    source,
                })?;
            }
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination)
                .map_err(|source| InvocationError::Io {
                    path: destination.clone(),
                    source,
                })?;
            file.write_all(&tool.bytes)
                .map_err(|source| InvocationError::Io {
                    path: destination.clone(),
                    source,
                })?;
            file.sync_all().map_err(|source| InvocationError::Io {
                path: destination.clone(),
                source,
            })?;
        }
        Err(source) => {
            return Err(InvocationError::Io {
                path: destination,
                source,
            });
        }
    }

    let program = destination
        .canonicalize()
        .map_err(|source| InvocationError::Io {
            path: destination.clone(),
            source,
        })?;
    if !program.starts_with(root) {
        return Err(InvocationError::UnsafeProgramDestination { path: program });
    }
    let bytes = fs::read(&program).map_err(|source| InvocationError::Io {
        path: program.clone(),
        source,
    })?;
    let actual = hex::encode(Sha256::digest(bytes));
    if actual != tool.sha256 {
        return Err(InvocationError::ToolHashMismatch {
            path: program,
            expected: tool.sha256.clone(),
            actual,
        });
    }
    Ok(program)
}

fn invocation_digest(
    program: &Path,
    cwd: &Path,
    args: &[OsString],
    prompts: &[ResolvedPrompt],
    debug_path: &Path,
    tool_sha256: &str,
) -> String {
    let mut digest = Sha256::new();
    digest_field(&mut digest, b"format", b"chriz-weidu-invocation-v1");
    digest_field(&mut digest, b"tool-sha256", tool_sha256.as_bytes());
    digest_os(&mut digest, b"program", program.as_os_str());
    digest_os(&mut digest, b"cwd", cwd.as_os_str());
    digest_count(&mut digest, b"args", args.len());
    for argument in args {
        digest_os(&mut digest, b"arg", argument);
    }
    digest_count(&mut digest, b"prompts", prompts.len());
    for prompt in prompts {
        digest_field(&mut digest, b"expected-output", &prompt.expected_output);
        digest_field(&mut digest, b"answer", &prompt.answer);
    }
    digest_os(&mut digest, b"debug-path", debug_path.as_os_str());
    hex::encode(digest.finalize())
}

fn digest_count(digest: &mut Sha256, tag: &[u8], count: usize) {
    digest_field(digest, tag, &(count as u64).to_le_bytes());
}

fn digest_field(digest: &mut Sha256, tag: &[u8], value: &[u8]) {
    digest.update((tag.len() as u64).to_le_bytes());
    digest.update(tag);
    digest.update((value.len() as u64).to_le_bytes());
    digest.update(value);
}

#[cfg(windows)]
fn digest_os(digest: &mut Sha256, tag: &[u8], value: &OsStr) {
    use std::os::windows::ffi::OsStrExt;

    let bytes = value
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    digest_field(digest, tag, &bytes);
}

#[cfg(unix)]
fn digest_os(digest: &mut Sha256, tag: &[u8], value: &OsStr) {
    use std::os::unix::ffi::OsStrExt;

    digest_field(digest, tag, value.as_bytes());
}

#[cfg(not(any(windows, unix)))]
fn digest_os(digest: &mut Sha256, tag: &[u8], value: &OsStr) {
    digest_field(digest, tag, value.to_string_lossy().as_bytes());
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};
    use std::fs;
    use std::path::{Path, PathBuf};

    use sha2::{Digest, Sha256};
    use tempfile::TempDir;

    use super::{
        build, validate_setup_name_declarations, InvocationError, InvocationInput, ResolvedPrompt,
        StagedRoots, VerifiedWeidu,
    };
    use crate::manifest::{GameRoot, InvocationMode, RunArg};

    struct Fixture {
        _temp: TempDir,
        roots: StagedRoots,
        attempt: PathBuf,
        tool: VerifiedWeidu,
    }

    impl Fixture {
        fn new(tool_bytes: &[u8]) -> Self {
            let temp = TempDir::new().unwrap();
            let bg1 = temp.path().join("bg1");
            let bg2 = temp.path().join("bg2");
            let attempt = temp.path().join("attempt");
            let tool_path = temp.path().join("cache").join("weidu.exe");
            fs::create_dir_all(tool_path.parent().unwrap()).unwrap();
            fs::create_dir_all(&bg1).unwrap();
            fs::create_dir_all(&bg2).unwrap();
            fs::create_dir_all(&attempt).unwrap();
            fs::write(&tool_path, tool_bytes).unwrap();
            let expected = hex::encode(Sha256::digest(tool_bytes));
            let tool = VerifiedWeidu::verify(&tool_path, &expected).unwrap();

            Self {
                _temp: temp,
                roots: StagedRoots { bg1, bg2 },
                attempt,
                tool,
            }
        }
    }

    fn input(target: GameRoot) -> InvocationInput {
        InvocationInput {
            target,
            tp2: "testmod/testmod.tp2".to_owned(),
            mode: InvocationMode::SetupName,
            explicit_tp2_tested: false,
            language: 0,
            remaining_components: vec![0, 10],
            run_args: Vec::new(),
            prompts: vec![ResolvedPrompt {
                expected_output: b"Choose an option".to_vec(),
                answer: b"1\n".to_vec(),
            }],
        }
    }

    fn strings(values: &[OsString]) -> Vec<String> {
        values
            .iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect()
    }

    fn canonical(path: &Path) -> PathBuf {
        path.canonicalize().unwrap()
    }

    #[test]
    fn builds_exact_safe_setup_name_install_invocation() {
        let fixture = Fixture::new(b"pinned weidu A");

        let invocation = build(
            &input(GameRoot::Bg2),
            &fixture.roots,
            &fixture.tool,
            &fixture.attempt,
        )
        .unwrap();

        assert_eq!(
            invocation.program,
            canonical(&fixture.roots.bg2).join("Setup-testmod.exe")
        );
        assert_eq!(invocation.cwd, canonical(&fixture.roots.bg2));
        assert_eq!(
            strings(&invocation.args),
            [
                "--language",
                "0",
                "--use-lang",
                "en_US",
                "--force-install-list",
                "0",
                "10",
                "--no-exit-pause",
                "--skip-at-view",
                "--safe-exit",
                "--noautoupdate",
                "--log",
                invocation.debug_path.to_string_lossy().as_ref(),
            ]
        );
        for forbidden in [
            "--quick-log",
            "--yes",
            "--reinstall",
            "--force-uninstall-list",
            "--uninstall",
        ] {
            assert!(!strings(&invocation.args).iter().any(|arg| arg == forbidden));
        }
        assert_eq!(invocation.prompts, input(GameRoot::Bg2).prompts);
        assert_eq!(fs::read(&invocation.program).unwrap(), b"pinned weidu A");
        assert!(invocation.debug_path.is_absolute());
        assert!(invocation
            .debug_path
            .starts_with(canonical(&fixture.attempt)));
        assert_eq!(invocation.identity_digest.len(), 64);
    }

    #[test]
    fn selects_the_requested_bg1_or_bg2_working_directory() {
        let bg1_fixture = Fixture::new(b"pinned weidu");
        let bg1 = build(
            &input(GameRoot::Bg1),
            &bg1_fixture.roots,
            &bg1_fixture.tool,
            &bg1_fixture.attempt,
        )
        .unwrap();
        let bg2_fixture = Fixture::new(b"pinned weidu");
        let bg2 = build(
            &input(GameRoot::Bg2),
            &bg2_fixture.roots,
            &bg2_fixture.tool,
            &bg2_fixture.attempt,
        )
        .unwrap();

        assert_eq!(bg1.cwd, canonical(&bg1_fixture.roots.bg1));
        assert_eq!(bg2.cwd, canonical(&bg2_fixture.roots.bg2));
    }

    #[test]
    fn eet_initialization_ends_with_typed_bg1_args_list() {
        let fixture = Fixture::new(b"pinned weidu");
        let mut eet = input(GameRoot::Bg2);
        eet.tp2 = "eet/eet.tp2".to_owned();
        eet.run_args = vec![
            RunArg::Literal("--args-list".to_owned()),
            RunArg::Literal("p".to_owned()),
            RunArg::StagedRoot(GameRoot::Bg1),
        ];

        let invocation = build(&eet, &fixture.roots, &fixture.tool, &fixture.attempt).unwrap();
        let tail = &invocation.args[invocation.args.len() - 3..];

        assert_eq!(tail[0], OsStr::new("--args-list"));
        assert_eq!(tail[1], OsStr::new("p"));
        assert!(!tail.iter().any(|arg| arg == OsStr::new("s")));
        assert_eq!(tail[2], canonical(&fixture.roots.bg1).as_os_str());
    }

    #[test]
    fn every_consequential_input_changes_the_identity_digest() {
        let fixture = Fixture::new(b"tool A");
        let baseline_input = input(GameRoot::Bg2);
        let baseline = build(
            &baseline_input,
            &fixture.roots,
            &fixture.tool,
            &fixture.attempt,
        )
        .unwrap()
        .identity_digest;

        let mut changed_component = baseline_input.clone();
        changed_component.remaining_components[1] = 11;
        let mut changed_argument = baseline_input.clone();
        changed_argument
            .run_args
            .push(RunArg::Literal("x".to_owned()));
        let mut changed_root = baseline_input.clone();
        changed_root.target = GameRoot::Bg1;
        let other_attempt = fixture._temp.path().join("other-attempt");
        fs::create_dir(&other_attempt).unwrap();

        let other_tool_path = fixture._temp.path().join("cache").join("other-weidu.exe");
        fs::write(&other_tool_path, b"tool B").unwrap();
        let other_tool_hash = hex::encode(Sha256::digest(b"tool B"));
        let other_tool = VerifiedWeidu::verify(&other_tool_path, &other_tool_hash).unwrap();

        let variants = [
            (
                "component",
                build(
                    &changed_component,
                    &fixture.roots,
                    &fixture.tool,
                    &fixture.attempt,
                )
                .unwrap()
                .identity_digest,
            ),
            (
                "typed argument",
                build(
                    &changed_argument,
                    &fixture.roots,
                    &fixture.tool,
                    &fixture.attempt,
                )
                .unwrap()
                .identity_digest,
            ),
            (
                "tool hash",
                build(
                    &baseline_input,
                    &fixture.roots,
                    &other_tool,
                    &fixture.attempt,
                )
                .unwrap()
                .identity_digest,
            ),
            (
                "target root",
                build(
                    &changed_root,
                    &fixture.roots,
                    &fixture.tool,
                    &fixture.attempt,
                )
                .unwrap()
                .identity_digest,
            ),
            (
                "log destination",
                build(
                    &baseline_input,
                    &fixture.roots,
                    &fixture.tool,
                    &other_attempt,
                )
                .unwrap()
                .identity_digest,
            ),
        ];

        for (field, digest) in variants {
            assert_ne!(digest, baseline, "changing {field} must change identity");
        }
    }

    #[test]
    fn setup_prefix_is_removed_once_case_insensitively() {
        for tp2 in ["testmod.tp2", "setup-testmod.tp2", "SETUP-testmod.TP2"] {
            let fixture = Fixture::new(b"pinned weidu");
            let mut request = input(GameRoot::Bg2);
            request.tp2 = tp2.to_owned();
            let invocation =
                build(&request, &fixture.roots, &fixture.tool, &fixture.attempt).unwrap();
            assert_eq!(
                invocation.program.file_name().unwrap(),
                OsStr::new("Setup-testmod.exe")
            );
        }

        let fixture = Fixture::new(b"pinned weidu");
        let mut request = input(GameRoot::Bg2);
        request.tp2 = "setup-setup-testmod.tp2".to_owned();
        let invocation = build(&request, &fixture.roots, &fixture.tool, &fixture.attempt).unwrap();
        assert_eq!(
            invocation.program.file_name().unwrap(),
            OsStr::new("Setup-setup-testmod.exe")
        );
    }

    #[test]
    fn setup_name_aliases_are_rejected_as_ambiguous() {
        let error =
            validate_setup_name_declarations(["mods/testmod.tp2", "other/setup-testmod.tp2"])
                .unwrap_err();

        assert!(matches!(error, InvocationError::AmbiguousSetupName { .. }));
    }

    #[test]
    fn explicit_tp2_requires_tested_evidence_and_stays_relative() {
        let fixture = Fixture::new(b"pinned weidu");
        let mut request = input(GameRoot::Bg2);
        request.mode = InvocationMode::ExplicitTp2;

        let error = build(&request, &fixture.roots, &fixture.tool, &fixture.attempt).unwrap_err();
        assert!(matches!(error, InvocationError::UntestedExplicitTp2 { .. }));

        request.explicit_tp2_tested = true;
        let invocation = build(&request, &fixture.roots, &fixture.tool, &fixture.attempt).unwrap();
        assert_eq!(invocation.args[0], OsStr::new("testmod/testmod.tp2"));
        assert_eq!(
            invocation.program.file_name().unwrap(),
            OsStr::new("WeiDU.exe")
        );
    }

    #[test]
    fn unsafe_recipe_tp2_paths_are_rejected() {
        for path in [
            "C:/mods/test.tp2",
            "//server/share/test.tp2",
            r"\\server\share\test.tp2",
            r"\\?\C:\mods\test.tp2",
            r"\\.\C:\mods\test.tp2",
            "mods/../test.tp2",
            "mods/file.tp2:stream",
            "mods/nested:stream/test.tp2",
        ] {
            let fixture = Fixture::new(b"pinned weidu");
            let mut request = input(GameRoot::Bg2);
            request.tp2 = path.to_owned();
            let error =
                build(&request, &fixture.roots, &fixture.tool, &fixture.attempt).unwrap_err();
            assert!(
                matches!(error, InvocationError::UnsafeTp2Path { .. }),
                "unexpected result for {path:?}: {error:?}"
            );
        }
    }

    #[test]
    fn typed_arguments_cannot_reenable_destructive_or_nondeterministic_modes() {
        for forbidden in [
            "--quick-log",
            "--yes",
            "--reinstall",
            "--force-uninstall-list",
            "--uninstall",
            "--log",
        ] {
            let fixture = Fixture::new(b"pinned weidu");
            let mut request = input(GameRoot::Bg2);
            request.run_args = vec![RunArg::Literal(forbidden.to_owned())];

            let error =
                build(&request, &fixture.roots, &fixture.tool, &fixture.attempt).unwrap_err();

            assert!(
                matches!(error, InvocationError::ForbiddenRunArgument { .. }),
                "unexpected result for {forbidden}: {error:?}"
            );
        }
    }

    #[test]
    fn an_empty_remaining_component_list_is_rejected() {
        let fixture = Fixture::new(b"pinned weidu");
        let mut request = input(GameRoot::Bg2);
        request.remaining_components.clear();

        let error = build(&request, &fixture.roots, &fixture.tool, &fixture.attempt).unwrap_err();

        assert!(matches!(error, InvocationError::NoComponents));
    }

    #[test]
    fn log_symlink_escape_is_rejected() {
        let fixture = Fixture::new(b"pinned weidu");
        let outside = fixture._temp.path().join("outside.log");
        fs::write(&outside, b"outside").unwrap();
        let debug = fixture.attempt.join("weidu.debug.log");

        #[cfg(windows)]
        match std::os::windows::fs::symlink_file(&outside, &debug) {
            Ok(()) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::Unsupported
                ) =>
            {
                return;
            }
            Err(error) => panic!("failed to create test symlink: {error}"),
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &debug).unwrap();

        let error = build(
            &input(GameRoot::Bg2),
            &fixture.roots,
            &fixture.tool,
            &fixture.attempt,
        )
        .unwrap_err();

        assert!(matches!(error, InvocationError::LogEscapesAttempt { .. }));
        assert_eq!(fs::read(outside).unwrap(), b"outside");
    }

    #[test]
    fn an_archive_setup_executable_is_replaced_with_the_pinned_tool() {
        let fixture = Fixture::new(b"trusted pinned weidu");
        let archive_setup = fixture.roots.bg2.join("Setup-testmod.exe");
        fs::write(&archive_setup, b"untrusted archive executable").unwrap();

        let invocation = build(
            &input(GameRoot::Bg2),
            &fixture.roots,
            &fixture.tool,
            &fixture.attempt,
        )
        .unwrap();

        assert_eq!(
            fs::read(invocation.program).unwrap(),
            b"trusted pinned weidu"
        );
    }

    #[test]
    fn verified_tool_rejects_a_hash_mismatch() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("weidu.exe");
        fs::write(&path, b"wrong bytes").unwrap();

        let error = VerifiedWeidu::verify(&path, &"0".repeat(64)).unwrap_err();

        assert!(matches!(error, InvocationError::ToolHashMismatch { .. }));
    }
}

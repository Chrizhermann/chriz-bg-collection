use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use bg_engine::acquire::{
    download_for_inspection, extract_archive, ArchiveFormat, ArchiveLimits, ArchiveMode,
    ArchiveRequirements, ArtifactCache, CacheDisposition, DownloadRequest,
};
use bg_engine::events::ChannelSink;
use bg_engine::manifest::{AcquisitionPolicy, ArchiveKind, ArchiveRootRule, Artifact, PeMachine};
use bg_engine::weidu::log::parse_active_entries;
use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use zip::ZipArchive;

#[derive(Debug, Parser)]
#[command(name = "chriz-bg-author")]
#[command(about = "Read-only recipe authoring helpers")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Capture active WeiDU.log entries as stable TSV evidence on stdout.
    CaptureLog {
        /// Human-readable provenance label repeated on every emitted row.
        #[arg(long)]
        source_label: String,
        /// Existing WeiDU.log to read. This file is never modified.
        #[arg(long)]
        log: PathBuf,
    },
    /// Inspect or verify one immutable artifact without editing recipe files.
    Artifact(ArtifactArgs),
}

#[derive(Debug, Args)]
struct ArtifactArgs {
    #[command(subcommand)]
    command: ArtifactCommand,
}

#[derive(Debug, Subcommand)]
enum ArtifactCommand {
    /// Download an untrusted candidate and report bounded authoring evidence.
    Inspect {
        /// Candidate artifact URL. HTTPS is required except for loopback test servers.
        url: String,
        /// Cache parent; inspection always writes below its authoring-quarantine child.
        #[arg(long)]
        cache_root: Option<PathBuf>,
        /// Expected logical TP2 path to match directly or below one wrapper.
        #[arg(long = "expected-tp2")]
        expected_tp2s: Vec<String>,
    },
    /// Reacquire and prove one complete artifact contract without editing it.
    Verify {
        /// Standalone artifact TOML to verify.
        artifact: PathBuf,
        /// Cache parent; verified bytes use its production cache namespace.
        #[arg(long)]
        cache_root: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::CaptureLog { source_label, log } => {
            let input = fs::read_to_string(log)?;
            let entries = parse_active_entries(&input)?;

            // Build the complete output before touching stdout. A malformed active row therefore
            // cannot leave a plausible-looking partial capture behind.
            let mut output = String::from(
                "position\tsource_label\tsource_line\ttp2\tlanguage\tcomponent\tcomponent_name\n",
            );
            for (position, entry) in entries.iter().enumerate() {
                writeln!(
                    output,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    position + 1,
                    escape_tsv(&source_label),
                    entry.line_number,
                    escape_tsv(&entry.tp2),
                    entry.language,
                    entry.component,
                    escape_tsv(entry.annotation.as_deref().unwrap_or("")),
                )?;
            }
            print!("{output}");
        }
        Command::Artifact(ArtifactArgs {
            command:
                ArtifactCommand::Inspect {
                    url,
                    cache_root,
                    expected_tp2s,
                },
        }) => {
            let cache_root = cache_root.unwrap_or_else(default_authoring_cache_parent);
            let download = download_for_inspection(&url, &cache_root)?;
            let report = inspect_archive(
                &download.path,
                download.final_url,
                download.length,
                download.sha256,
                &expected_tp2s,
            )?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::Artifact(ArtifactArgs {
            command:
                ArtifactCommand::Verify {
                    artifact,
                    cache_root,
                },
        }) => {
            let cache_root = cache_root.unwrap_or_else(default_authoring_cache_parent);
            let report = verify_artifact(&artifact, &cache_root)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }
    Ok(())
}

fn default_authoring_cache_parent() -> PathBuf {
    std::env::temp_dir().join("chriz-bg-author")
}

#[derive(Debug, Serialize)]
struct InspectionReport {
    final_url: String,
    length: u64,
    sha256: String,
    quarantined_path: PathBuf,
    archive: ArchiveEvidence,
    evidence_scope: &'static str,
    component_menu_evidence: Vec<ComponentEvidence>,
    version_evidence: Vec<VersionEvidence>,
}

#[derive(Debug, Serialize)]
struct ArchiveEvidence {
    kind: &'static str,
    entry_count: usize,
    entries: Vec<ArchiveEntryEvidence>,
    top_level_entries: Vec<String>,
    wrapper_candidates: Vec<String>,
    tp2_paths: Vec<String>,
    expected_tp2_matches: Vec<ExpectedTp2Match>,
}

#[derive(Debug, Serialize)]
struct ArchiveEntryEvidence {
    path: String,
    length: u64,
    compressed_length: u64,
    is_directory: bool,
}

#[derive(Debug, Serialize)]
struct ExpectedTp2Match {
    expected: String,
    matches: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ComponentEvidence {
    tp2_path: String,
    line: usize,
    component: Option<u32>,
    title: String,
}

#[derive(Debug, Serialize)]
struct VersionEvidence {
    tp2_path: String,
    line: usize,
    value: String,
}

#[derive(Debug, Serialize)]
struct VerificationReport {
    verified: bool,
    artifact_id: String,
    final_url: String,
    length: u64,
    sha256: String,
    cache_disposition: &'static str,
    archive: VerifiedArchiveEvidence,
    pe_machine: Option<PeMachine>,
}

#[derive(Debug, Serialize)]
struct VerifiedArchiveEvidence {
    wrapper_directory: Option<String>,
    publish_roots: Vec<String>,
    tp2_paths: Vec<String>,
}

fn verify_artifact(
    artifact_path: &Path,
    cache_root: &Path,
) -> Result<VerificationReport, Box<dyn std::error::Error>> {
    let artifact_text = fs::read_to_string(artifact_path)?;
    let artifact: Artifact = toml::from_str(&artifact_text)?;
    verify_standalone_contract(&artifact, artifact_path)?;
    let expected_length = artifact
        .source
        .expected_length
        .ok_or("artifact has no expected length")?;
    let production_cache = ArtifactCache::open(cache_root.join("production"))?;
    let request = DownloadRequest {
        request_id: artifact.id.clone(),
        url: artifact.source.url.clone(),
        expected_length,
        expected_sha256: artifact.source.sha256.clone(),
        redirect_hosts: artifact.source.redirect_hosts.clone(),
        max_attempts: 3,
    };
    let (sink, _events) = ChannelSink::unbounded();
    let acquired = production_cache.acquire(&request, &sink)?;

    let requirements = ArchiveRequirements {
        artifact_sha256: artifact.source.sha256.clone(),
        format: match artifact.archive.kind {
            ArchiveKind::Zip => ArchiveFormat::Zip,
            ArchiveKind::Iemod => ArchiveFormat::Iemod,
        },
        expected_roots: artifact.archive.publish_roots.clone(),
        expected_tp2_paths: artifact.archive.tp2_paths.clone(),
        limits: ArchiveLimits {
            max_depth: artifact.archive.limits.max_depth,
            max_entries: artifact.archive.limits.max_entries,
            max_entry_uncompressed_bytes: artifact.archive.limits.max_entry_uncompressed_bytes,
            max_total_uncompressed_bytes: artifact.archive.limits.max_total_uncompressed_bytes,
            max_compression_ratio: artifact.archive.limits.max_compression_ratio,
        },
        mode: ArchiveMode::Public,
    };
    let extracted = extract_archive(
        &acquired.archive_path,
        &cache_root.join("production-extracted"),
        &requirements,
    )?;
    match (
        artifact.archive.root_rule,
        extracted.wrapper_directory.is_some(),
    ) {
        (ArchiveRootRule::Direct, true) => {
            return Err("archive uses a wrapper but the contract requires direct layout".into())
        }
        (ArchiveRootRule::SingleWrapper, false) => {
            return Err("archive is direct but the contract requires one wrapper".into())
        }
        _ => {}
    }

    let pe_machine = if let Some(tool) = &artifact.tool {
        let actual = read_pe_machine(&extracted.root.join(&tool.executable))?;
        if actual != tool.pe_machine {
            return Err(format!(
                "PE machine drift for {:?}: expected {:?}, got {:?}",
                tool.executable, tool.pe_machine, actual
            )
            .into());
        }
        Some(actual)
    } else {
        None
    };

    Ok(VerificationReport {
        verified: true,
        artifact_id: artifact.id,
        final_url: acquired.metadata.final_url,
        length: acquired.metadata.length,
        sha256: acquired.metadata.sha256,
        cache_disposition: match acquired.disposition {
            CacheDisposition::Hit => "hit",
            CacheDisposition::Downloaded => "downloaded",
        },
        archive: VerifiedArchiveEvidence {
            wrapper_directory: extracted.wrapper_directory,
            publish_roots: extracted.expected_roots,
            tp2_paths: extracted.expected_tp2_paths,
        },
        pe_machine,
    })
}

fn verify_standalone_contract(
    artifact: &Artifact,
    artifact_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let stem = artifact_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or("artifact TOML has no Unicode file stem")?;
    if !stem.eq_ignore_ascii_case(&artifact.id) {
        return Err(format!(
            "artifact id {:?} does not match file stem {stem:?}",
            artifact.id
        )
        .into());
    }
    if !matches!(
        artifact.acquisition,
        AcquisitionPolicy::FetchOnly | AcquisitionPolicy::BundlePermitted
    ) {
        return Err("artifact verify supports only fetchable artifact policies".into());
    }
    if artifact.version.trim().is_empty() || artifact.source.reference.trim().is_empty() {
        return Err("artifact version and exact source reference are required".into());
    }
    let Some(filename) = artifact.source.expected_filename.as_deref() else {
        return Err("artifact expected filename is required for verification".into());
    };
    let filename_matches_kind = match artifact.archive.kind {
        ArchiveKind::Zip => filename.to_ascii_lowercase().ends_with(".zip"),
        ArchiveKind::Iemod => filename.to_ascii_lowercase().ends_with(".iemod"),
    };
    if !filename_matches_kind {
        return Err("artifact expected filename disagrees with its archive kind".into());
    }
    if artifact
        .source
        .expected_length
        .is_none_or(|length| length == 0)
    {
        return Err("artifact expected length must be greater than zero".into());
    }
    if artifact.source.sha256.len() != 64
        || !artifact
            .source
            .sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || artifact.source.sha256.bytes().all(|byte| byte == b'0')
    {
        return Err("artifact SHA-256 must be exact and nonzero".into());
    }
    let url = url::Url::parse(&artifact.source.url)?;
    let loopback = match url.host() {
        Some(url::Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(url::Host::Ipv4(address)) => address.is_loopback(),
        Some(url::Host::Ipv6(address)) => address.is_loopback(),
        None => false,
    };
    if url.scheme() != "https" && !(url.scheme() == "http" && loopback) {
        return Err("artifact source URL is not HTTPS".into());
    }
    let url_filename = url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .unwrap_or_default();
    let url_names_archive = [".zip", ".iemod"]
        .iter()
        .any(|extension| url_filename.to_ascii_lowercase().ends_with(extension));
    if url_names_archive && !url_filename.eq_ignore_ascii_case(filename) {
        return Err(format!(
            "artifact expected filename {filename:?} disagrees with source URL filename {url_filename:?}"
        )
        .into());
    }
    if artifact.archive.publish_roots.is_empty()
        || (artifact.tool.is_none() && artifact.archive.tp2_paths.is_empty())
    {
        return Err("artifact archive publication contract is incomplete".into());
    }
    Ok(())
}

fn read_pe_machine(path: &Path) -> Result<PeMachine, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    if bytes.len() < 0x40 || &bytes[..2] != b"MZ" {
        return Err(format!("tool {:?} is not a PE executable", path).into());
    }
    let pe_offset = u32::from_le_bytes(bytes[0x3c..0x40].try_into()?) as usize;
    let machine_offset = pe_offset
        .checked_add(4)
        .ok_or("PE header offset overflow")?;
    if machine_offset + 2 > bytes.len()
        || bytes.get(pe_offset..machine_offset) != Some(b"PE\0\0".as_slice())
    {
        return Err(format!("tool {:?} has an invalid PE header", path).into());
    }
    let machine = u16::from_le_bytes(bytes[machine_offset..machine_offset + 2].try_into()?);
    match machine {
        0x014c => Ok(PeMachine::X86),
        0x8664 => Ok(PeMachine::X86_64),
        0xaa64 => Ok(PeMachine::Arm64),
        _ => Err(format!("tool {:?} has unsupported PE machine 0x{machine:04x}", path).into()),
    }
}

fn inspect_archive(
    path: &Path,
    final_url: String,
    length: u64,
    sha256: String,
    expected_tp2s: &[String],
) -> Result<InspectionReport, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    if archive.len() > 100_000 {
        return Err(format!(
            "archive has {} entries; inspection limit is 100000",
            archive.len()
        )
        .into());
    }

    let mut entries = Vec::with_capacity(archive.len());
    let mut top_levels = std::collections::BTreeSet::new();
    let mut tp2_indices = Vec::new();
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        let entry_path = entry.name().replace('\\', "/");
        if let Some(top) = entry_path
            .split('/')
            .next()
            .filter(|value| !value.is_empty())
        {
            top_levels.insert(top.to_owned());
        }
        if !entry.is_dir() && entry_path.to_ascii_lowercase().ends_with(".tp2") {
            tp2_indices.push((index, entry_path.clone()));
        }
        entries.push(ArchiveEntryEvidence {
            path: entry_path,
            length: entry.size(),
            compressed_length: entry.compressed_size(),
            is_directory: entry.is_dir(),
        });
    }
    entries.sort_by(|left, right| {
        left.path
            .to_ascii_lowercase()
            .cmp(&right.path.to_ascii_lowercase())
            .then_with(|| left.path.cmp(&right.path))
    });
    tp2_indices.sort_by(|left, right| {
        left.1
            .to_ascii_lowercase()
            .cmp(&right.1.to_ascii_lowercase())
    });
    let tp2_paths = tp2_indices
        .iter()
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();
    let wrapper_candidates = common_wrapper(&entries).into_iter().collect();
    let expected_tp2_matches = expected_tp2s
        .iter()
        .map(|expected| ExpectedTp2Match {
            expected: expected.clone(),
            matches: tp2_paths
                .iter()
                .filter(|path| logical_tp2_matches(path, expected))
                .cloned()
                .collect(),
        })
        .collect();

    let mut component_menu_evidence = Vec::new();
    let mut version_evidence = Vec::new();
    for (index, tp2_path) in tp2_indices {
        let mut entry = archive.by_index(index)?;
        if entry.size() > 4 * 1024 * 1024 {
            continue;
        }
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut bytes)?;
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        inspect_tp2_text(
            &tp2_path,
            &text,
            &mut component_menu_evidence,
            &mut version_evidence,
        );
    }

    let kind = if final_url
        .split(['?', '#'])
        .next()
        .is_some_and(|url| url.to_ascii_lowercase().ends_with(".iemod"))
    {
        "iemod"
    } else {
        "zip"
    };
    Ok(InspectionReport {
        final_url,
        length,
        sha256,
        quarantined_path: path.to_path_buf(),
        archive: ArchiveEvidence {
            kind,
            entry_count: entries.len(),
            entries,
            top_level_entries: top_levels.into_iter().collect(),
            wrapper_candidates,
            tp2_paths,
            expected_tp2_matches,
        },
        evidence_scope: "Static TP2 text evidence only; INCLUDEs, macros, translations, and conditional menus are not evaluated.",
        component_menu_evidence,
        version_evidence,
    })
}

fn common_wrapper(entries: &[ArchiveEntryEvidence]) -> Option<String> {
    let mut wrapper: Option<&str> = None;
    for entry in entries.iter().filter(|entry| !entry.is_directory) {
        let (first, _) = entry.path.split_once('/')?;
        match wrapper {
            None => wrapper = Some(first),
            Some(existing) if existing.eq_ignore_ascii_case(first) => {}
            Some(_) => return None,
        }
    }
    wrapper.map(str::to_owned)
}

fn logical_tp2_matches(archive_path: &str, expected: &str) -> bool {
    archive_path.eq_ignore_ascii_case(expected)
        || archive_path
            .split_once('/')
            .is_some_and(|(_, remainder)| remainder.eq_ignore_ascii_case(expected))
}

fn inspect_tp2_text(
    tp2_path: &str,
    text: &str,
    components: &mut Vec<ComponentEvidence>,
    versions: &mut Vec<VersionEvidence>,
) {
    let mut pending_component = None;
    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        if let Some(value) = directive_value(line, "VERSION") {
            versions.push(VersionEvidence {
                tp2_path: tp2_path.to_owned(),
                line: line_number,
                value,
            });
        }
        if let Some(title) = directive_value(line, "BEGIN") {
            components.push(ComponentEvidence {
                tp2_path: tp2_path.to_owned(),
                line: line_number,
                component: None,
                title,
            });
            pending_component = Some(components.len() - 1);
        }
        if let Some(value) = directive_value(line, "DESIGNATED") {
            if let (Some(component), Some(component_index)) = (
                value
                    .split_ascii_whitespace()
                    .next()
                    .and_then(|value| value.parse().ok()),
                pending_component,
            ) {
                components[component_index].component = Some(component);
            }
        }
    }
}

fn directive_value(line: &str, directive: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return None;
    }
    let prefix = trimmed.get(..directive.len())?;
    if !prefix.eq_ignore_ascii_case(directive) {
        return None;
    }
    let rest = trimmed.get(directive.len()..)?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let rest = rest.trim_start();
    if rest.is_empty() {
        return None;
    }
    if let Some(rest) = rest.strip_prefix('~') {
        return rest.split_once('~').map(|(value, _)| value.to_owned());
    }
    if let Some(rest) = rest.strip_prefix('"') {
        return rest.split_once('"').map(|(value, _)| value.to_owned());
    }
    Some(
        rest.split_ascii_whitespace()
            .next()
            .unwrap_or_default()
            .to_owned(),
    )
}

fn escape_tsv(value: &str) -> String {
    value
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

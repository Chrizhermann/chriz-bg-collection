//! Acceptance of an already completed, explicitly supervised top-tail repair.
//!
//! This does not run WeiDU, reopen the original ledger, or offer automatic recovery.
//! The operator supplies immutable process evidence; we reconcile it against the
//! original plan before finalizing the isolated save identity and publishing a
//! separate completion receipt.

use super::*;
use crate::recovery_receipt::{
    self, ArtifactReplacement, EvidenceFile, RecoveryKind, RecoveryReceipt,
    RECOVERY_RECEIPT_SCHEMA_VERSION,
};
use crate::registry::{ManagedInstallRecord, REGISTRY_SCHEMA_VERSION};
use crate::weidu::recovery::{
    plan_partial_tail_recovery, verify_rollback, ExpectedInstall, RecoveryAction,
};

/// Bounded operator input for a completed one-mod replacement and remaining tail.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupervisedRecoveryRequest {
    pub managed_root: PathBuf,
    pub recovery_id: String,
    pub replacement: ArtifactReplacement,
    /// Independently obtained replacement archive, read-only during acceptance.
    pub replacement_archive: PathBuf,
    /// Original failed invocation's immutable before/after/output snapshots.
    pub before_log: PathBuf,
    pub partial_log: PathBuf,
    pub partial_output: PathBuf,
    /// Relative evidence stems: uninstall, replacement install, then each remaining run.
    /// Each stem has .intent.json, .process.json, .after.log, .stdout.log,
    /// .stderr.log and .debug.log sidecars from the supervised worker.
    pub operation_stems: Vec<PathBuf>,
    /// Explicit hashes covering all supplied evidence, backup and protected history.
    pub evidence: Vec<EvidenceFile>,
}

struct Inspected {
    root: PathBuf,
    base: InstallReceipt,
    frozen: FrozenCliRecipe,
    logs: Vec<FinalLogReceipt>,
    launch_path: PathBuf,
}

fn error(message: impl std::fmt::Display) -> CliError {
    CliError::new("recovery_unavailable", message.to_string())
}

/// Read-only acceptance preflight: hashes, rollback chain, original plan and tool identity.
pub fn inspect_supervised_recovery(
    request: &SupervisedRecoveryRequest,
) -> Result<Vec<FinalLogReceipt>, CliError> {
    Ok(inspect(request)?.logs)
}

fn evidence_bytes(request: &SupervisedRecoveryRequest, path: &Path) -> Result<Vec<u8>, CliError> {
    if !request.evidence.iter().any(|item| item.path == path) {
        return Err(error(format!(
            "unhashed recovery evidence: {}",
            path.display()
        )));
    }
    fs::read(request.managed_root.join(path)).map_err(error)
}

fn evidence_text(request: &SupervisedRecoveryRequest, path: &Path) -> Result<String, CliError> {
    Ok(String::from_utf8_lossy(&evidence_bytes(request, path)?).into_owned())
}

fn sidecar(stem: &Path, suffix: &str) -> PathBuf {
    let mut name = stem.as_os_str().to_os_string();
    name.push(format!(".{suffix}"));
    PathBuf::from(name)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationIntent {
    program: PathBuf,
    tool_sha256: String,
    arguments: Vec<String>,
    before_log_sha256: String,
    at: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationResult {
    exit_code: i32,
    after_log_sha256: String,
    at: String,
}

fn inspect(request: &SupervisedRecoveryRequest) -> Result<Inspected, CliError> {
    validate_cli_identifier(&request.recovery_id, "recovery id").map_err(error)?;
    let root = canonical_direct_directory(&request.managed_root, "recovery target")?;
    recovery_receipt::verify_evidence(&root, &request.evidence).map_err(error)?;
    let replay = SessionStore::open(&root)
        .and_then(|store| store.replay())
        .map_err(error)?;
    let created = replay.created();
    let frozen: FrozenCliRecipe = serde_json::from_slice(&created.recipe_payload).map_err(error)?;
    validate_frozen_cli_recipe(created, &frozen)?;
    // Reporting validates the original failure against the immutable ledger. It never
    // upgrades that outcome, even when a separate recovery completion already exists.
    let base = report_managed_install(&root)?.receipt;
    let index = frozen
        .plan
        .runs
        .iter()
        .position(|run| run.run_id == request.replacement.run_id)
        .ok_or_else(|| error("replacement run is not in the frozen plan"))?;
    let tail = &frozen.plan.runs[index..];
    let failed = &tail[0];
    if failed.target != GameRoot::Bg2
        || tail.iter().any(|run| {
            run.target != GameRoot::Bg2 || !run.args.is_empty() || !run.prompt_scripts.is_empty()
        })
    {
        return Err(error(
            "supervised acceptance currently supports only a prompt-free BG2 tail",
        ));
    }
    if request.operation_stems.len() != tail.len() + 1 {
        return Err(error(
            "operation evidence must cover rollback and the entire remaining tail",
        ));
    }
    let old = created
        .artifact_identities
        .iter()
        .find(|item| item.id == failed.artifact_id)
        .ok_or_else(|| error("original artifact identity is missing"))?;
    if old != &request.replacement.original || request.replacement.replacement.id != old.id {
        return Err(error(
            "replacement does not name the frozen original artifact",
        ));
    }
    let archive = fs::read(&request.replacement_archive).map_err(error)?;
    if archive.len() as u64 != request.replacement.replacement.length
        || sha256_bytes(&archive) != request.replacement.replacement.sha256
    {
        return Err(error(
            "replacement archive differs from its recorded release identity",
        ));
    }
    let expected_step = format!("install:{}", failed.run_id);
    let seal = replay
        .fresh_copy_required()
        .ok_or_else(|| error("original terminal seal is absent"))?;
    if seal.step_id != expected_step {
        return Err(error(
            "replacement is not the run named by the original terminal failure",
        ));
    }
    let attempt_root = request
        .before_log
        .parent()
        .ok_or_else(|| error("missing attempt path"))?;
    if !attempt_root.starts_with(
        Path::new(".chriz/attempts")
            .join(&created.attempt_id)
            .join("steps"),
    ) || request.partial_log != attempt_root.join("after.log")
        || request.partial_output != attempt_root.join("stdout.log")
    {
        return Err(error(
            "failed snapshots are not from the original campaign attempt",
        ));
    }
    let prepared: PreparedInvocationEvidence = serde_json::from_slice(&evidence_bytes(
        request,
        &attempt_root.join("invocation.json"),
    )?)
    .map_err(error)?;
    if prepared.run_id != failed.run_id
        || prepared.components != failed.components
        || prepared.attempt != seal.attempt
    {
        return Err(error("failed invocation differs from the sealed run"));
    }
    // Ambiguous attempts deliberately have no successful RunAttemptReceipt. Anchor
    // their before-log to the preceding completed invocation instead of inventing one.
    let previous_receipt = base.runs[..index]
        .iter()
        .rev()
        .filter(|run| run.target == GameRoot::Bg2)
        .find_map(|run| run.attempts.last())
        .ok_or_else(|| error("retained prefix evidence is absent"))?;
    let before = evidence_bytes(request, &request.before_log)?;
    let partial = evidence_bytes(request, &request.partial_log)?;
    let partial_output = evidence_bytes(request, &request.partial_output)?;
    if sha256_bytes(&before) != previous_receipt.log_diff.after_sha256 {
        return Err(error(
            "repair prefix differs from the preceding completed invocation",
        ));
    }
    let mod_file = &frozen.mods[&failed.mod_id];
    let plan = plan_partial_tail_recovery(
        &String::from_utf8_lossy(&before),
        &String::from_utf8_lossy(&partial),
        &String::from_utf8_lossy(&partial_output),
        &ExpectedInstall {
            tp2: mod_file.tp2.clone(),
            language: mod_file.language,
            components: failed.components.clone(),
        },
    )
    .map_err(error)?;
    let RecoveryAction::RollbackAndReinstall {
        installed_to_uninstall,
        ..
    } = &plan.action
    else {
        return Err(error(
            "evidence does not describe a bounded rollback-and-reinstall repair",
        ));
    };
    let mut previous = partial;
    for (operation, stem) in request.operation_stems.iter().enumerate() {
        let run = if operation == 0 {
            failed
        } else {
            &tail[operation - 1]
        };
        let mod_file = &frozen.mods[&run.mod_id];
        let tool = created
            .tool_identities
            .iter()
            .find(|tool| tool.id == run.weidu_artifact_id)
            .ok_or_else(|| error("frozen WeiDU identity is missing"))?;
        let intent: OperationIntent =
            serde_json::from_slice(&evidence_bytes(request, &sidecar(stem, "intent.json"))?)
                .map_err(error)?;
        let process: OperationResult =
            serde_json::from_slice(&evidence_bytes(request, &sidecar(stem, "process.json"))?)
                .map_err(error)?;
        let after = evidence_bytes(request, &sidecar(stem, "after.log"))?;
        let output = evidence_text(request, &sidecar(stem, "stdout.log"))?;
        evidence_bytes(request, &sidecar(stem, "stderr.log"))?;
        evidence_bytes(request, &sidecar(stem, "debug.log"))?;
        if intent.at.is_empty()
            || process.at.is_empty()
            || intent.tool_sha256 != tool.sha256
            || intent.before_log_sha256 != sha256_bytes(&previous)
            || process.after_log_sha256 != sha256_bytes(&after)
            || process.exit_code != 0
        {
            return Err(error(format!(
                "incomplete or contradictory process evidence: {}",
                stem.display()
            )));
        }
        let setup = Path::new(&mod_file.tp2)
            .file_stem()
            .ok_or_else(|| error("frozen TP2 has no setup filename"))?;
        let program = root.join("game").join(setup).with_extension("exe");
        if !direct_regular_file(&program)
            || fs::canonicalize(&intent.program).map_err(error)?
                != fs::canonicalize(&program).map_err(error)?
            || sha256_bytes(&fs::read(&program).map_err(error)?) != tool.sha256
        {
            return Err(error("operation used a different setup executable"));
        }
        verify_arguments(
            &intent.arguments,
            mod_file.language,
            if operation == 0 {
                installed_to_uninstall
            } else {
                &run.components
            },
            operation == 0,
            &root.join(sidecar(stem, "debug.log")),
        )?;
        if operation == 0 {
            verify_rollback(&plan, &String::from_utf8_lossy(&after)).map_err(error)?;
        } else {
            let result = reconcile_weidu(
                &String::from_utf8_lossy(&previous),
                &String::from_utf8_lossy(&after),
                &output,
                &ExpectedRun {
                    tp2: mod_file.tp2.clone(),
                    language: mod_file.language,
                    components: run.components.clone(),
                    exit_code: process.exit_code,
                },
            );
            if result != Reconciliation::ProvenDone {
                return Err(error(format!(
                    "{} is not a proven complete install: {result:?}",
                    run.run_id
                )));
            }
            let prior = parse_active_entries(&String::from_utf8_lossy(&previous)).map_err(error)?;
            let next = parse_active_entries(&String::from_utf8_lossy(&after)).map_err(error)?;
            if !prior.iter().zip(&next).all(|(left, right)| {
                same_log_entry(left, right) && left.annotation == right.annotation
            }) {
                return Err(error(
                    "recovery changed an earlier installed version or annotation",
                ));
            }
            crate::postcondition::verify(&root.join("game"), &run.postconditions).map_err(error)?;
        }
        previous = after;
    }
    if fs::read(root.join("game/WeiDU.log")).map_err(error)? != previous {
        return Err(error(
            "current game log differs from the completed recovery chain",
        ));
    }
    let logs = final_logs(&root, &frozen)?;
    let launch_path = verified_launch_path(&frozen.plan, &root.join("game")).map_err(error)?;
    Ok(Inspected {
        root,
        base,
        frozen,
        logs,
        launch_path,
    })
}

fn verify_arguments(
    args: &[String],
    language: u32,
    components: &[u32],
    uninstall: bool,
    debug: &Path,
) -> Result<(), CliError> {
    let mut expected = vec![
        "--language".to_owned(),
        language.to_string(),
        "--use-lang".to_owned(),
        "en_US".to_owned(),
        if uninstall {
            "--force-uninstall-list"
        } else {
            "--force-install-list"
        }
        .to_owned(),
    ];
    expected.extend(components.iter().map(u32::to_string));
    if !uninstall {
        expected.push("--safe-exit".to_owned());
    }
    expected.extend(
        [
            "--no-exit-pause",
            "--skip-at-view",
            "--noautoupdate",
            "--log",
        ]
        .map(str::to_owned),
    );
    if args.len() != expected.len() + 1
        || args[..expected.len()] != expected
        || fs::canonicalize(args.last().unwrap().trim_matches('"')).map_err(error)? != debug
    {
        return Err(error(
            "supervised invocation differs from the frozen component command",
        ));
    }
    Ok(())
}

fn final_logs(root: &Path, frozen: &FrozenCliRecipe) -> Result<Vec<FinalLogReceipt>, CliError> {
    [GameRoot::Bg1, GameRoot::Bg2]
        .into_iter()
        .map(|target| {
            let folder = if target == GameRoot::Bg1 {
                "bg1"
            } else {
                "game"
            };
            let path = root.join(folder).join("WeiDU.log");
            if !direct_regular_file(&path) {
                return Err(error("final mod list is not a direct file"));
            }
            let bytes = fs::read(path).map_err(error)?;
            let actual = parse_active_entries(&String::from_utf8_lossy(&bytes)).map_err(error)?;
            let expected =
                expected_log_components(&frozen.plan, &frozen.mods, target).map_err(error)?;
            let components: Vec<_> = actual.iter().map(log_component_receipt).collect();
            if components != expected {
                return Err(error(
                    "final mod list differs from the exact frozen selection/order",
                ));
            }
            Ok(FinalLogReceipt {
                target,
                sha256: sha256_bytes(&bytes),
                components,
            })
        })
        .collect()
}

/// Finish only save isolation and metadata publication for a verified supervised repair.
/// No install command, download, source staging or ledger mutation occurs here.
pub fn accept_supervised_recovery(
    request: &SupervisedRecoveryRequest,
) -> Result<RecoveryReceipt, CliError> {
    let app_data = application_data_root()?;
    let _lock =
        crate::lock::TargetLock::try_acquire(app_data.join(LOCKS_DIRECTORY), &request.managed_root)
            .map_err(error)?;
    let checked = inspect(request)?;
    let root = &checked.root;
    let install_receipt = root.join(".chriz/install-receipt.json");
    if install_receipt.exists() {
        let existing: RecoveryReceipt =
            serde_json::from_slice(&fs::read(&install_receipt).map_err(error)?).map_err(error)?;
        if existing.recovery_id != request.recovery_id
            || existing.replacements != vec![request.replacement.clone()]
            || existing.evidence != request.evidence
        {
            return Err(error(
                "a different completed recovery already owns this installation",
            ));
        }
        recovery_receipt::read_completed_state(root).map_err(error)?;
        publish_registry(&app_data, &checked.frozen, &existing, &install_receipt)?;
        return Ok(existing);
    }
    // Check process/TLK ownership without installing anything. Then reuse the already
    // reserved save identity instead of allowing the game to share EET's default folder.
    recheck_target_before_mutation(&root.join("game"), "en_US").map_err(error)?;
    let layout = ManagedLayout::prepare(root, &checked.frozen.bg1.root, &checked.frozen.bg2.root)
        .map_err(error)?;
    let identity = reserve_save_identity(
        &CliDocumentsLocator,
        &layout,
        checked.frozen.effective_display_name(),
        &checked.base.install_id,
    )
    .map_err(error)?;
    let bg1_engine_name = read_engine_name(layout.bg1_root()).map_err(error)?;
    if bg1_engine_name != identity.engine_name {
        return Err(error("BG1 save identity differs from its reservation"));
    }
    finalize_game_identity(&layout, &identity).map_err(error)?;
    let bg2_engine_name = read_engine_name(layout.game_root()).map_err(error)?;
    if bg2_engine_name != identity.engine_name {
        return Err(error("final EET save identity was not retained"));
    }
    let base_path = root
        .join(".chriz/attempts")
        .join(&checked.base.attempt_id)
        .join("receipt.json");
    let receipt = RecoveryReceipt {
        schema_version: RECOVERY_RECEIPT_SCHEMA_VERSION, kind: RecoveryKind::SupervisedRecovery,
        recovery_id: request.recovery_id.clone(), install_id: checked.base.install_id.clone(),
        managed_root: root.clone(), base_attempt_id: checked.base.attempt_id.clone(),
        base_receipt_sha256: sha256_bytes(&fs::read(base_path).map_err(error)?),
        base_recipe_version: checked.base.versions.recipe.clone(),
        base_recipe_payload_sha256: checked.base.recipe_payload_sha256.clone(),
        base_plan_sha256: checked.base.plan_sha256.clone(), replacements: vec![request.replacement.clone()],
        evidence: request.evidence.clone(), completed_at_millis: SystemTime::now().duration_since(UNIX_EPOCH).map_err(error)?.as_millis() as u64,
        final_state: FinalReceiptState { logs: checked.logs, bg1_engine_name, bg2_engine_name,
            managed_save_root: identity.save_root, launch_path: checked.launch_path,
            verification_summary: "Exact original selection/order verified after supervised tail repair; earlier mod versions retained. Gameplay smoke test remains separate.".to_owned() },
    };
    let path = recovery_receipt::publish(root, &receipt).map_err(error)?;
    publish_registry(&app_data, &checked.frozen, &receipt, &path)?;
    Ok(receipt)
}

fn publish_registry(
    app_data: &Path,
    frozen: &FrozenCliRecipe,
    receipt: &RecoveryReceipt,
    path: &Path,
) -> Result<(), CliError> {
    ManagedInstallRegistry::open_or_create(app_data)
        .and_then(|registry| {
            registry.publish(&ManagedInstallRecord {
                schema_version: REGISTRY_SCHEMA_VERSION,
                install_id: receipt.install_id.clone(),
                display_name: frozen.effective_display_name().to_owned(),
                managed_root: receipt.managed_root.clone(),
                recipe_version: receipt.effective_version(),
                recipe_sha256: receipt.base_recipe_payload_sha256.clone(),
                engine_name: receipt.final_state.bg2_engine_name.clone(),
                managed_save_root: receipt.final_state.managed_save_root.clone(),
                launch_path: receipt.final_state.launch_path.clone(),
                receipt_sha256: sha256_bytes(&fs::read(path).map_err(|source| {
                    crate::registry::RegistryError::Io {
                        path: path.to_owned(),
                        source,
                    }
                })?),
                completed_at_millis: receipt.completed_at_millis,
            })
        })
        .map_err(error)?;
    Ok(())
}

pub(super) fn report_recovery(
    root: &Path,
    base: &InstallReceipt,
    frozen: &FrozenCliRecipe,
) -> Result<Option<RecoveryReceipt>, CliError> {
    let path = root.join(".chriz/install-receipt.json");
    if !path.try_exists().map_err(error)? {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(error)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(error)?;
    if value.get("kind").is_none() {
        return Ok(None);
    }
    let receipt: RecoveryReceipt = serde_json::from_slice(&bytes).map_err(error)?;
    receipt.validate(base).map_err(error)?;
    recovery_receipt::read_completed_state(root).map_err(error)?;
    for log in &receipt.final_state.logs {
        if log.components
            != expected_log_components(&frozen.plan, &frozen.mods, log.target).map_err(error)?
        {
            return Err(error(
                "recovered receipt changes the original selected component plan",
            ));
        }
    }
    Ok(Some(receipt))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_keeps_full_stem_and_does_not_replace_an_extension() {
        assert_eq!(
            sidecar(Path::new(".chriz/recoveries/repair.1/install"), "after.log"),
            PathBuf::from(".chriz/recoveries/repair.1/install.after.log")
        );
    }

    #[test]
    fn supervised_command_rejects_silent_skip_and_changed_component_order() {
        let temp = tempfile::tempdir().unwrap();
        let debug = temp.path().join("debug.log");
        fs::write(&debug, "debug").unwrap();
        let debug = fs::canonicalize(debug).unwrap();
        let mut args: Vec<_> = [
            "--language",
            "0",
            "--use-lang",
            "en_US",
            "--force-install-list",
            "10",
            "20",
            "--safe-exit",
            "--no-exit-pause",
            "--skip-at-view",
            "--noautoupdate",
            "--log",
        ]
        .map(str::to_owned)
        .to_vec();
        args.push(debug.display().to_string());
        assert!(verify_arguments(&args, 0, &[10, 20], false, &debug).is_ok());
        assert!(verify_arguments(&args, 0, &[20, 10], false, &debug).is_err());
        args.insert(7, "--skip-at-now".to_owned());
        assert!(verify_arguments(&args, 0, &[10, 20], false, &debug).is_err());
    }
}

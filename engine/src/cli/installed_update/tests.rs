use super::*;

use crate::installed_updates::{ComparisonState, InstalledRunState};
use crate::registry::REGISTRY_SCHEMA_VERSION;

const OLD_BUFFBOT: &str = "1.8.3-alpha";
const NEW_BUFFBOT: &str = "1.8.4-alpha";
const OLD_BUFFBOT_HASH: &str = "fe6e0b4cd393a608d8c3df75f26993106f3550e954030a7b66dbfd28956820a7";
const NEW_BUFFBOT_HASH: &str = "1081abc59511263570603560059ef361f45dd3288933f7faa26166ce0e805ced";

fn buffbot_log(version: &str) -> String {
    format!("~buffbot/setup-buffbot.tp2~ #0 #1 // Loader helper: v{version}\n~buffbot/setup-buffbot.tp2~ #0 #0 // BuffBot: v{version}\n")
}

/// A real selected BuffBot identity in an otherwise synthetic recipe. No third-party
/// source bytes, game assets, downloads or installer execution are used here.
fn select_buffbot(manifest: &mut Manifest) {
    use crate::manifest::{ComponentRef, Decision, Feature, Phase, Readiness};
    let mut artifact = manifest.artifacts["eefixpack"].clone();
    artifact.id = "buffbot".into();
    artifact.name = "BuffBot".into();
    artifact.version = OLD_BUFFBOT.into();
    artifact.source.url = "https://example.invalid/buffbot-old.zip".into();
    artifact.source.expected_filename = Some("buffbot-old.zip".into());
    artifact.source.sha256 = OLD_BUFFBOT_HASH.into();
    artifact.source.expected_length = Some(4_240_743);
    artifact.archive.publish_roots = vec!["buffbot".into()];
    artifact.archive.tp2_paths = vec!["buffbot/setup-buffbot.tp2".into()];
    manifest.artifacts.insert(artifact.id.clone(), artifact);
    let mut installer = manifest.mods["eefixpack"].clone();
    installer.id = "buffbot".into();
    installer.artifact_id = "buffbot".into();
    installer.name = "BuffBot".into();
    installer.tp2 = "buffbot/setup-buffbot.tp2".into();
    let component = installer.components[0].clone();
    installer.components = [1, 0]
        .into_iter()
        .map(|id| crate::manifest::Component {
            id,
            ..component.clone()
        })
        .collect();
    manifest.mods.insert(installer.id.clone(), installer);
    let mut run = manifest.collection.runs[1].clone();
    run.run_id = "buffbot-bg2".into();
    run.mod_id = "buffbot".into();
    run.phase = Phase::PostEetEnd;
    run.components = vec![1, 0];
    // Declare the EET anchors so the recipe remains structurally valid. They are
    // intentionally unselected in this focused BuffBot-only comparison fixture.
    let mut initialize = manifest.collection.runs[1].clone();
    initialize.run_id = "fixture-eet-initialize".into();
    initialize.phase = Phase::EetInitialization;
    let mut finalize = initialize.clone();
    finalize.run_id = "fixture-eet-finalize".into();
    finalize.phase = Phase::EetFinalization;
    finalize.components = vec![2];
    manifest.collection.runs = vec![initialize, finalize, run];
    manifest.collection.features = vec![Feature {
        id: "buffbot".into(),
        title: "BuffBot".into(),
        description: "Selected BuffBot fixture".into(),
        category: "Utilities".into(),
        source_label: None,
        group_label: None,
        choice_group: None,
        decision: Decision::Default,
        readiness: Readiness::Ready,
        unavailable_reason: None,
        parent: None,
        components: [1, 0]
            .into_iter()
            .map(|component| ComponentRef {
                run_id: "buffbot-bg2".into(),
                component,
            })
            .collect(),
        requires: vec![],
        requires_any: vec![],
        conflicts: vec![],
        inputs: vec![],
    }];
}

struct Fixture {
    temporary: tempfile::TempDir,
    record: ManagedInstallRecord,
    receipt: InstallReceipt,
    registry_path: PathBuf,
    target_recipe: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        Self::with_buffbot(false)
    }

    fn with_buffbot(selected: bool) -> Self {
        let temporary = tempfile::tempdir().unwrap();
        let base = fs::canonicalize(temporary.path()).unwrap();
        let root = base.join("managed");
        let cache = base.join("cache");
        for path in [
            root.clone(),
            root.join("bg1"),
            root.join("game"),
            cache.clone(),
            base.join("source-bg1"),
            base.join("source-bg2"),
            base.join("saves"),
        ] {
            fs::create_dir_all(path).unwrap();
        }
        let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let mut manifest = Manifest::load(&fixture_root.join("manifest")).unwrap();
        // Keep the fixture's valid declared runs. An empty feature selection
        // yields no executed runs without making the recipe itself invalid.
        manifest.collection.features.clear();
        if selected {
            select_buffbot(&mut manifest);
        }
        let evaluation = evaluate(&manifest, &Selection::defaults("windows")).unwrap();
        let frozen = FrozenCliRecipe {
            manual_checkpoints: Vec::new(),
            schema: CLI_FROZEN_RECIPE_SCHEMA,
            release_version: Some("0.1.0".into()),
            application_version: Some("0.1.0".into()),
            display_name: Some("Fixture installation".into()),
            collection: manifest.collection.clone(),
            artifacts: manifest.artifacts.clone(),
            mods: manifest.mods.clone(),
            presets: manifest.presets.clone(),
            profiles: GameProfiles::load(fixture_root.join("games/profiles")).unwrap(),
            plan: evaluation.plan,
            normalized_selection: evaluation.normalized_selection,
            preset: "recommended".into(),
            bg1: Some(FrozenSource {
                role: GameRole::BgeeSod,
                storefront: Storefront::Steam,
                root: base.join("source-bg1"),
                build: Some("2.7.3.0".into()),
                fingerprint: "11".repeat(32),
            }),
            bg2: FrozenSource {
                role: GameRole::Bg2ee,
                storefront: Storefront::Steam,
                root: base.join("source-bg2"),
                build: Some("2.7.3.0".into()),
                fingerprint: "22".repeat(32),
            },
            recipe_id: None,
        };
        let payload = serialize_frozen_recipe(&frozen).unwrap();
        let envelope = b"fixture-only-envelope".to_vec();
        let created = CampaignCreated {
            install_id: "comparison-fixture".into(),
            attempt_id: "fixture-attempt".into(),
            managed_root: root.clone(),
            cache_root: cache,
            recipe_payload_sha256: sha256_bytes(&payload),
            recipe_payload: payload,
            recipe_envelope_sha256: sha256_bytes(&envelope),
            recipe_envelope: envelope,
            selection_sha256: selection_digest(&frozen.normalized_selection).unwrap(),
            normalized_selection: frozen.normalized_selection.clone(),
            plan_sha256: plan_digest(&frozen.plan).unwrap(),
            source_games: SourceGameFingerprints {
                bg1: frozen.bg1.as_ref().map(|bg1| bg1.fingerprint.clone()),
                bg2: frozen.bg2.fingerprint.clone(),
                standalone_role: None,
            },
            artifact_identities: frozen_payload_identities(&frozen).unwrap(),
            tool_identities: frozen_tool_identities(&frozen).unwrap(),
            staged_bg1: Some(root.join("bg1")),
            staged_bg2: root.join("game"),
        };
        let store =
            SessionStore::create(&root, SessionEvent::Created(Box::new(created.clone()))).unwrap();
        store
            .append(SessionEvent::StepStarted {
                step_id: "verify:final".into(),
                attempt: 1,
            })
            .unwrap();
        let completion = store
            .append(SessionEvent::StepCompleted {
                step_id: "verify:final".into(),
                attempt: 1,
            })
            .unwrap();
        let replay = store.replay().unwrap();
        let bg2_log = if selected {
            buffbot_log(OLD_BUFFBOT)
        } else {
            String::new()
        };
        fs::write(created.staged_bg1.as_ref().unwrap().join("WeiDU.log"), b"").unwrap();
        fs::write(created.staged_bg2.join("WeiDU.log"), &bg2_log).unwrap();
        let bg2_components: Vec<LogComponentReceipt> = if selected {
            [1, 0]
                .into_iter()
                .map(|component| LogComponentReceipt {
                    tp2: "buffbot/setup-buffbot.tp2".into(),
                    language: 0,
                    component,
                })
                .collect()
        } else {
            vec![]
        };
        let final_state = FinalReceiptState {
            logs: [GameRoot::Bg1, GameRoot::Bg2]
                .into_iter()
                .map(|target| FinalLogReceipt {
                    target,
                    sha256: sha256_bytes(if target == GameRoot::Bg2 {
                        bg2_log.as_bytes()
                    } else {
                        b""
                    }),
                    components: if target == GameRoot::Bg2 {
                        bg2_components.clone()
                    } else {
                        vec![]
                    },
                })
                .collect(),
            bg1_engine_name: Some("Fixture staging".into()),
            bg2_engine_name: "Fixture installation".into(),
            managed_save_root: base.join("saves"),
            launch_path: root.join("game/InfinityLoader.exe"),
            verification_summary: "Synthetic receipt fixture; no installer process was executed."
                .into(),
        };
        fs::write(&final_state.launch_path, b"fixture-only; not an executable").unwrap();
        let receipt = InstallReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION,
            install_id: created.install_id.clone(),
            attempt_id: created.attempt_id.clone(),
            evidence_attempt_id: created.attempt_id.clone(),
            managed_root: root.clone(),
            staged_bg1: created.staged_bg1.clone(),
            staged_bg2: created.staged_bg2.clone(),
            started_at_millis: replay.records[0].recorded_at,
            completed_at_millis: completion.recorded_at,
            outcome: ReceiptOutcome::Succeeded,
            versions: ReceiptVersions {
                application: "0.1.0".into(),
                engine: env!("CARGO_PKG_VERSION").into(),
                manifest_schema: frozen.collection.schema,
                recipe: "0.1.0".into(),
            },
            source_games: frozen
                .sources()
                .into_iter()
                .map(FrozenSource::receipt)
                .collect(),
            recipe_payload_sha256: created.recipe_payload_sha256.clone(),
            recipe_envelope_sha256: created.recipe_envelope_sha256.clone(),
            selection_sha256: created.selection_sha256.clone(),
            normalized_selection: frozen.normalized_selection,
            plan_sha256: created.plan_sha256.clone(),
            plan: frozen.plan.clone(),
            artifacts: created
                .artifact_identities
                .iter()
                .map(|identity| ArtifactReceipt {
                    id: identity.id.clone(),
                    version: identity.version.clone(),
                    original_url: "https://example.invalid/synthetic".into(),
                    final_url: "https://example.invalid/synthetic".into(),
                    length: identity.length,
                    sha256: identity.sha256.clone(),
                    cache_outcome: ArtifactCacheOutcome::Hit,
                    served_difference: None,
                })
                .collect(),
            weidu_tools: created
                .tool_identities
                .iter()
                .map(|identity| WeiDuToolReceipt {
                    id: identity.id.clone(),
                    version: identity.version.clone(),
                    length: identity.length,
                    sha256: identity.sha256.clone(),
                    reviewed: None,
                })
                .collect(),
            runs: frozen
                .plan
                .runs
                .iter()
                .map(|run| RunReceipt {
                    run_id: run.run_id.clone(),
                    target: run.target,
                    components: run.components.clone(),
                    attempts: vec![RunAttemptReceipt {
                        attempt: 1,
                        components: run.components.clone(),
                        prompts: vec![],
                        timing: RunTiming {
                            started_at_millis: completion.recorded_at,
                            completed_at_millis: completion.recorded_at,
                        },
                        exit_code: 0,
                        warnings: vec![],
                        invocation_sha256: "33".repeat(32),
                        stdout_sha256: sha256_bytes(b""),
                        stderr_sha256: sha256_bytes(b""),
                        debug_sha256: sha256_bytes(b""),
                        log_diff: LogDiffReceipt {
                            before_sha256: sha256_bytes(b""),
                            after_sha256: sha256_bytes(bg2_log.as_bytes()),
                            added: bg2_components.clone(),
                            removed: vec![],
                        },
                    }],
                })
                .collect(),
            final_state: Some(final_state.clone()),
            left_out: Vec::new(),
            installed_plan: None,
        };
        let receipt_bytes = serde_json::to_vec_pretty(&receipt).unwrap();
        fs::write(root.join(".chriz/install-receipt.json"), &receipt_bytes).unwrap();
        fs::write(
            root.join(".chriz/attempts/fixture-attempt/receipt.json"),
            &receipt_bytes,
        )
        .unwrap();
        let record = ManagedInstallRecord {
            schema_version: REGISTRY_SCHEMA_VERSION,
            install_id: created.install_id,
            display_name: "Fixture installation".into(),
            managed_root: root,
            recipe_version: "0.1.0".into(),
            recipe_sha256: created.recipe_payload_sha256,
            engine_name: final_state.bg2_engine_name,
            managed_save_root: final_state.managed_save_root,
            launch_path: final_state.launch_path,
            receipt_sha256: sha256_bytes(&receipt_bytes),
            completed_at_millis: completion.recorded_at,
            recipe_id: None,
        };
        let registry = ManagedInstallRegistry::open_or_create(&base.join("app-data")).unwrap();
        let registry_path = registry.publish(&record).unwrap();
        let record = registry.list().unwrap().remove(0).record;
        let target_recipe = base.join("target-recipe");
        copy_fixture(&fixture_root.join("manifest"), &target_recipe);
        fs::write(
            target_recipe.join("collection.toml"),
            toml::to_string(&manifest.collection).unwrap(),
        )
        .unwrap();
        if selected {
            let mut target_artifact = manifest.artifacts["buffbot"].clone();
            target_artifact.version = NEW_BUFFBOT.into();
            target_artifact.source.sha256 = NEW_BUFFBOT_HASH.into();
            target_artifact.source.expected_length = Some(4_241_150);
            fs::write(
                target_recipe.join("artifacts/buffbot.toml"),
                toml::to_string(&target_artifact).unwrap(),
            )
            .unwrap();
            fs::write(
                target_recipe.join("mods/buffbot.toml"),
                toml::to_string(&manifest.mods["buffbot"]).unwrap(),
            )
            .unwrap();
        }
        fs::write(
            target_recipe.join("release.json"),
            br#"{"version":"0.2.0"}"#,
        )
        .unwrap();
        Self {
            temporary,
            record,
            receipt,
            registry_path,
            target_recipe,
        }
    }

    /// Alter only a disposable fixture and rebind its registry hash so downstream
    /// validation must reject the content rather than stopping at the hash check.
    fn rewrite_receipt(&mut self) {
        let bytes = serde_json::to_vec_pretty(&self.receipt).unwrap();
        fs::write(
            self.record.managed_root.join(".chriz/install-receipt.json"),
            &bytes,
        )
        .unwrap();
        self.record.receipt_sha256 = sha256_bytes(&bytes);
        fs::write(
            &self.registry_path,
            serde_json::to_vec_pretty(&self.record).unwrap(),
        )
        .unwrap();
    }

    fn compare_without_writes(&self) -> Result<InstalledUpdatePlan, CliError> {
        let before = snapshot(self.temporary.path());
        let result = inspect_installed_update(&self.record, &self.target_recipe);
        assert_eq!(
            snapshot(self.temporary.path()),
            before,
            "comparison changed fixture files or directories"
        );
        result
    }

    /// Write the durable journal format around selected BuffBot rows, with real
    /// before/after file hashes, as a boundary fixture rather than an upgrade test.
    fn record_synthetic_buffbot_upgrade(&self, applied: bool) {
        use crate::patches::files;
        let root = &self.record.managed_root;
        let game = root.join("game");
        fs::create_dir_all(game.join("lang/en_us")).unwrap();
        fs::create_dir_all(game.join("override")).unwrap();
        fs::write(
            game.join("lang/en_us/dialog.tlk"),
            b"synthetic unchanged strings",
        )
        .unwrap();
        fs::write(game.join("override/bfbotscn.lua"), b"synthetic old BuffBot").unwrap();
        let before = files::inventory(&game, false).unwrap();
        let hashes = before
            .iter()
            .filter(|(_, stamp)| !stamp.directory)
            .map(|(name, _)| (name.clone(), files::hash(&game.join(name)).unwrap()))
            .collect::<BTreeMap<_, _>>();
        let backups = [
            "weidu.log",
            "lang/en_us/dialog.tlk",
            "override/bfbotscn.lua",
        ]
        .into_iter()
        .map(|name| (name.to_owned(), hashes[name].clone()))
        .collect::<BTreeMap<_, _>>();
        let directory = root.join(".chriz/mod-upgrades/v1/00000000000000000001");
        fs::create_dir_all(&directory).unwrap();
        for name in backups.keys() {
            let destination = directory.join("affected").join(name);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(game.join(name), destination).unwrap();
        }
        let prepared = serde_json::json!({
            "schema": 1, "upgrade_id": "buffbot-1.8.3-to-1.8.4",
            "plan": { "install_id": self.record.install_id, "managed_root": root,
                "base_receipt_sha256": self.record.receipt_sha256, "recipe_sha256": self.record.recipe_sha256,
                "selection_sha256": self.receipt.selection_sha256, "previous": null, "hashes": hashes },
            "before": before, "backups": backups,
        });
        let prepared_bytes = serde_json::to_vec_pretty(&prepared).unwrap();
        fs::write(directory.join("prepared.json"), &prepared_bytes).unwrap();
        if !applied {
            return;
        }
        fs::write(game.join("WeiDU.log"), buffbot_log(NEW_BUFFBOT)).unwrap();
        fs::write(game.join("override/bfbotscn.lua"), b"synthetic new BuffBot").unwrap();
        let after = files::inventory(&game, false).unwrap();
        let post_hashes = after
            .iter()
            .filter(|(name, stamp)| !stamp.directory && backups.contains_key(*name))
            .map(|(name, _)| (name.clone(), files::hash(&game.join(name)).unwrap()))
            .collect::<BTreeMap<_, _>>();
        let record = serde_json::json!({
            "prepared_sha256": sha256_bytes(&prepared_bytes),
            "upgrade": { "run_id": "buffbot-bg2", "version": NEW_BUFFBOT, "artifact_sha256": NEW_BUFFBOT_HASH,
                "before_log_sha256": hashes["weidu.log"], "log_sha256": post_hashes["weidu.log"] },
            "after": after, "post_hashes": post_hashes,
        });
        fs::write(
            directory.join("applied.json"),
            serde_json::to_vec_pretty(&record).unwrap(),
        )
        .unwrap();
    }
}

fn copy_fixture(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_fixture(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(base: &Path, directory: &Path, files: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let relative = path.strip_prefix(base).unwrap().to_path_buf();
            if entry.file_type().unwrap().is_dir() {
                files.insert(relative, None);
                visit(base, &path, files);
            } else {
                files.insert(relative, Some(fs::read(path).unwrap()));
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn ordinary_completed_installation_compares_without_writing_any_fixture_file() {
    let fixture = Fixture::new();
    let result = fixture.compare_without_writes().unwrap();
    assert_eq!(result.state, ComparisonState::ComparisonReady);
    assert_eq!(result.install_id, fixture.record.install_id);
    assert_eq!(result.base_recipe_version, "0.1.0");
    assert_eq!(result.target_recipe_version, "0.2.0");
    assert!(result.choices_preserved);
    assert!(result.issues.is_empty());
    assert!(result.runs.is_empty());
}

#[test]
fn unknown_receipt_schema_is_rejected_even_with_a_matching_registry_hash() {
    let mut fixture = Fixture::new();
    fixture.receipt.schema_version = RECEIPT_SCHEMA_VERSION + 1;
    fixture.rewrite_receipt();
    let error = fixture.compare_without_writes().unwrap_err();
    assert_eq!(error.code(), "installed_update_unavailable");
    assert!(error.to_string().contains("unsupported schema"));
}

#[test]
fn changed_receipt_bytes_are_rejected_before_comparison() {
    let fixture = Fixture::new();
    let path = fixture
        .record
        .managed_root
        .join(".chriz/install-receipt.json");
    let mut bytes = fs::read(&path).unwrap();
    bytes.push(b'\n');
    fs::write(path, bytes).unwrap();
    let error = fixture.compare_without_writes().unwrap_err();
    assert_eq!(error.code(), "installed_update_unavailable");
    assert!(error.to_string().contains("record has changed"));
}

#[test]
fn manual_mod_log_drift_is_rejected_without_changing_the_installation() {
    let fixture = Fixture::new();
    fs::write(
        fixture.record.managed_root.join("game/WeiDU.log"),
        b"~MANUAL/SETUP-MANUAL.TP2~ #0 #99 // Manual component\n",
    )
    .unwrap();
    let error = fixture.compare_without_writes().unwrap_err();
    assert_eq!(error.code(), "installed_update_unavailable");
    assert!(error.to_string().contains("outside CEBG"));
}

#[test]
fn bg1_mod_log_drift_is_checked_too() {
    let fixture = Fixture::new();
    fs::write(
        fixture.record.managed_root.join("bg1/WeiDU.log"),
        b"~MANUAL/SETUP-MANUAL.TP2~ #0 #99 // Manual component\n",
    )
    .unwrap();
    let error = fixture.compare_without_writes().unwrap_err();
    assert!(error.to_string().contains("outside CEBG"));
}

#[test]
fn older_bundled_collection_does_not_look_like_an_upgrade() {
    let fixture = Fixture::new();
    fs::write(
        fixture.target_recipe.join("release.json"),
        br#"{"version":"0.0.9"}"#,
    )
    .unwrap();
    let error = fixture.compare_without_writes().unwrap_err();
    assert!(error.to_string().contains("newer collection"));
}

#[test]
fn unknown_patch_history_is_rejected_without_guessing_its_effect() {
    let fixture = Fixture::new();
    fs::create_dir_all(
        fixture
            .record
            .managed_root
            .join(".chriz/patches/future-format"),
    )
    .unwrap();
    let error = fixture.compare_without_writes().unwrap_err();
    assert_eq!(error.code(), "installed_update_unavailable");
    assert!(error.to_string().contains("cannot interpret"));
}

#[test]
fn failed_receipt_cannot_be_presented_as_a_completed_installation_update() {
    let mut fixture = Fixture::new();
    fixture.receipt.outcome = ReceiptOutcome::Failed {
        step_id: "verify:final".into(),
        detail: "Synthetic failure".into(),
    };
    fixture.receipt.final_state = None;
    fixture.rewrite_receipt();
    let error = fixture.compare_without_writes().unwrap_err();
    assert_eq!(error.code(), "installed_update_unavailable");
    assert!(error.to_string().contains("incomplete"));
}

#[test]
fn recorded_buffbot_update_compares_effective_version_without_rewriting_base_choices() {
    let fixture = Fixture::with_buffbot(true);
    let original_receipt = fs::read(
        fixture
            .record
            .managed_root
            .join(".chriz/install-receipt.json"),
    )
    .unwrap();
    let original_selection = fixture.receipt.normalized_selection.clone();
    let before = fixture.compare_without_writes().unwrap();
    assert_eq!(before.runs.len(), 1);
    assert_eq!(before.runs[0].run_id, "buffbot-bg2");
    assert_eq!(before.runs[0].installed_version, OLD_BUFFBOT);
    assert_eq!(before.runs[0].state, InstalledRunState::Unassessed);

    fixture.record_synthetic_buffbot_upgrade(true);
    let baseline = verify_installed_baseline(&fixture.record).unwrap();
    assert!(baseline.upgraded);
    assert_eq!(baseline.receipt.normalized_selection, original_selection);
    assert_eq!(baseline.receipt.versions.recipe, "0.1.0");
    assert_eq!(baseline.receipt.plan.runs[0].components, vec![1, 0]);
    assert_eq!(baseline.replacements["buffbot-bg2"].version, NEW_BUFFBOT);
    assert_eq!(
        baseline.replacements["buffbot-bg2"].sha256,
        NEW_BUFFBOT_HASH
    );
    let after = fixture.compare_without_writes().unwrap();
    assert_eq!(after.state, ComparisonState::ComparisonReady);
    assert!(after.choices_preserved);
    assert_eq!(after.base_recipe_version, "0.1.0");
    assert_eq!(after.runs[0].installed_version, NEW_BUFFBOT);
    assert_eq!(after.runs[0].state, InstalledRunState::Unchanged);
    assert!(after
        .detail
        .contains("base collection version is unchanged"));
    assert_eq!(
        fs::read(
            fixture
                .record
                .managed_root
                .join(".chriz/install-receipt.json")
        )
        .unwrap(),
        original_receipt
    );
}

/// The receipt of an installation whose BuffBot link served `served` instead of the pin.
fn record_changed_buffbot_download(fixture: &mut Fixture, served: &DownloadIdentity) {
    let artifact = fixture
        .receipt
        .artifacts
        .iter_mut()
        .find(|artifact| artifact.id == "buffbot")
        .unwrap();
    artifact.served_difference = Some(ServedDifference {
        reviewed: DownloadIdentity {
            filename: Some("buffbot-old.zip".into()),
            length: artifact.length,
            sha256: artifact.sha256.clone(),
        },
        served: served.clone(),
        unreviewed_hosts: Vec::new(),
    });
    artifact.length = served.length;
    artifact.sha256 = served.sha256.clone();
    fixture.rewrite_receipt();
}

#[test]
fn a_changed_download_is_compared_as_the_file_that_was_installed() {
    // The link served the file the new collection pins, so the installed mod is already it.
    let mut fixture = Fixture::with_buffbot(true);
    let served = DownloadIdentity {
        filename: Some("buffbot-new.zip".into()),
        length: 4_241_150,
        sha256: NEW_BUFFBOT_HASH.into(),
    };
    let reviewed = fixture.compare_without_writes().unwrap();
    assert_eq!(reviewed.runs[0].state, InstalledRunState::Unassessed);
    record_changed_buffbot_download(&mut fixture, &served);

    let baseline = verify_installed_baseline(&fixture.record).unwrap();
    assert!(!baseline.recovered, "a changed download is not a repair");
    let installed = &baseline.replacements["buffbot-bg2"];
    assert_eq!(installed.id, "buffbot");
    assert_eq!(installed.length, served.length);
    assert_eq!(installed.sha256, served.sha256);
    let comparison = fixture.compare_without_writes().unwrap();
    assert_eq!(comparison.state, ComparisonState::ComparisonReady);
    assert_eq!(comparison.runs[0].run_id, "buffbot-bg2");
    assert_eq!(comparison.runs[0].state, InstalledRunState::Unchanged);
}

#[test]
fn an_installation_that_left_out_a_mod_is_verified_but_offered_no_game_update() {
    let mut fixture = Fixture::with_buffbot(true);
    let root = fixture.record.managed_root.clone();
    let reason = "its official link now serves a different archive, which cannot be installed";
    let remaining = InstallPlan { runs: Vec::new() };
    let store = SessionStore::open(&root).unwrap();
    store
        .append(SessionEvent::StepStarted {
            step_id: "acquire:buffbot".into(),
            attempt: 1,
        })
        .unwrap();
    store
        .append(SessionEvent::ArtifactLeftOut {
            step_id: "acquire:buffbot".into(),
            attempt: 1,
            artifact_id: "buffbot".into(),
            detail: reason.into(),
            plan_sha256: plan_digest(&remaining).unwrap(),
            removed_runs: vec!["buffbot-bg2".into()],
            changed_runs: Vec::new(),
        })
        .unwrap();
    // BuffBot was never installed, so the finished mod list is empty.
    fs::write(root.join("game/WeiDU.log"), b"").unwrap();
    for log in &mut fixture.receipt.final_state.as_mut().unwrap().logs {
        log.sha256 = sha256_bytes(b"");
        log.components.clear();
    }
    fixture.receipt.runs.clear();
    fixture.receipt.left_out = vec![crate::receipt::LeftOutReceipt {
        artifact_id: "buffbot".into(),
        reason: reason.into(),
        runs: vec!["buffbot-bg2".into()],
        changed_runs: Vec::new(),
    }];
    fixture.receipt.installed_plan = Some(remaining);
    fixture.rewrite_receipt();

    // The records verify, so work that follows the installed mods, such as a hotpatch, can
    // rely on them.
    let baseline = verify_installed_baseline(&fixture.record).unwrap();
    assert!(baseline.receipt.effective_plan().runs.is_empty());
    assert_eq!(baseline.receipt.plan.runs.len(), 1, "the frozen plan stays");
    // A game update reasons about the frozen plan run by run, so it is not offered.
    let error = fixture.compare_without_writes().unwrap_err();
    assert_eq!(error.code(), "installed_update_unavailable");
    assert!(
        error.to_string().contains("left out of this installation"),
        "{error}"
    );
}

#[test]
fn interrupted_buffbot_upgrade_blocks_baseline_and_comparison_without_writes() {
    let fixture = Fixture::with_buffbot(true);
    fixture.record_synthetic_buffbot_upgrade(false);
    assert!(verify_installed_baseline(&fixture.record).is_err());
    let error = fixture.compare_without_writes().unwrap_err();
    assert!(error.to_string().contains("did not finish"));
}

#[test]
fn unknown_mod_update_history_cannot_be_ignored_by_comparison() {
    let fixture = Fixture::with_buffbot(true);
    fs::create_dir_all(
        fixture
            .record
            .managed_root
            .join(".chriz/mod-upgrades/future-format"),
    )
    .unwrap();
    assert!(verify_installed_baseline(&fixture.record).is_err());
    let error = fixture.compare_without_writes().unwrap_err();
    assert!(error.to_string().contains("Unsupported mod-update history"));
}

#[test]
fn manual_changes_after_a_recorded_upgrade_are_not_treated_as_current() {
    let fixture = Fixture::with_buffbot(true);
    fixture.record_synthetic_buffbot_upgrade(true);
    fs::write(
        fixture.record.managed_root.join("game/WeiDU.log"),
        format!("{}// manual change\n", buffbot_log(NEW_BUFFBOT)),
    )
    .unwrap();
    let error = fixture.compare_without_writes().unwrap_err();
    assert!(error
        .to_string()
        .contains("outside the recorded update history"));
}

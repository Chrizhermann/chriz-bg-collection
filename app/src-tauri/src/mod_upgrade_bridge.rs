// Whole-mod updates accept registered identities only, never arbitrary game paths.
fn mod_upgrade_inspection_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::new(
        "mod_upgrade_check_failed",
        "The BuffBot update check could not finish.",
        "This check did not apply an update or change game files. Retry it or export diagnostics; any earlier recovery warning still applies.",
        error.to_string(),
    )
}

fn mod_upgrade_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::new(
        "mod_upgrade_failed",
        "The BuffBot update could not be completed.",
        "Check this installation in Updates. If recovery is needed, restore its backup before playing.",
        error.to_string(),
    )
}

impl NativeBridge {
    pub fn inspect_mod_upgrade(
        &self,
        install_id: &str,
    ) -> Result<bg_engine::mod_upgrades::ModUpgradePreview, CommandError> {
        self.ensure_update_idle()?;
        let record = self.available_managed_install(install_id)?.record;
        let app_data = self
            .app_data
            .as_ref()
            .ok_or_else(|| mod_upgrade_inspection_error("Application data is unavailable."))?;
        let _target =
            bg_engine::lock::TargetLock::try_acquire(app_data.join("locks"), &record.managed_root)
                .map_err(|error| {
                    CommandError::new(
                "mod_upgrade_busy",
                "This installation is busy.",
                "Wait for its current operation to finish, then check installed mods again.",
                error.to_string(),
            )
                })?;
        bg_engine::mod_upgrades::inspect(&bg_engine::mod_upgrades::Context {
            record: &record,
            app_data,
            cache: &self.cache,
            recipe: &self.recipe,
        })
        .map_err(mod_upgrade_inspection_error)
    }

    pub fn apply_mod_upgrade(
        &self,
        install_id: &str,
        review_token: &str,
        full_backup: bool,
        backup_saves: bool,
    ) -> Result<bg_engine::mod_upgrades::ModUpgradePreview, CommandError> {
        self.apply_mod_upgrade_with_progress(
            install_id,
            review_token,
            full_backup,
            backup_saves,
            &|_| {},
        )
    }

    pub fn apply_mod_upgrade_with_progress(
        &self,
        install_id: &str,
        review_token: &str,
        full_backup: bool,
        backup_saves: bool,
        progress: bg_engine::operation_progress::ProgressSink<'_>,
    ) -> Result<bg_engine::mod_upgrades::ModUpgradePreview, CommandError> {
        let _guard = self.begin_app_update()?;
        let record = self.available_managed_install(install_id)?.record;
        let app_data = self
            .app_data
            .as_ref()
            .ok_or_else(|| mod_upgrade_error("Application data is unavailable."))?;
        // The engine acquires and holds the target lock through backup and mutation.
        bg_engine::mod_upgrades::apply_with_progress(
            &bg_engine::mod_upgrades::Context {
                record: &record,
                app_data,
                cache: &self.cache,
                recipe: &self.recipe,
            },
            review_token,
            full_backup,
            backup_saves,
            progress,
        )
        .map_err(mod_upgrade_error)
    }

    pub fn restore_mod_upgrade(
        &self,
        install_id: &str,
        undo: bool,
    ) -> Result<bg_engine::mod_upgrades::ModUpgradePreview, CommandError> {
        self.restore_mod_upgrade_with_progress(install_id, undo, &|_| {})
    }

    pub fn restore_mod_upgrade_with_progress(
        &self,
        install_id: &str,
        undo: bool,
        progress: bg_engine::operation_progress::ProgressSink<'_>,
    ) -> Result<bg_engine::mod_upgrades::ModUpgradePreview, CommandError> {
        let _guard = self.begin_app_update()?;
        let record = self.available_managed_install(install_id)?.record;
        let app_data = self
            .app_data
            .as_ref()
            .ok_or_else(|| mod_upgrade_error("Application data is unavailable."))?;
        bg_engine::mod_upgrades::restore_with_progress(
            &bg_engine::mod_upgrades::Context {
                record: &record,
                app_data,
                cache: &self.cache,
                recipe: &self.recipe,
            },
            undo,
            progress,
        )
        .map_err(mod_upgrade_error)
    }
}

#[cfg(test)]
mod mod_upgrade_bridge_tests {
    use super::*;

    fn bridge(root: &Path) -> NativeBridge {
        NativeBridge::with_configuration(
            root.join("recipe"),
            "recommended".to_owned(),
            root.join("cache"),
            Some(root.join("app-data")),
            Arc::new(SystemBridgeEngine),
            Arc::new(SystemBridgeSystem),
        )
    }

    #[test]
    fn inspection_errors_describe_a_read_only_check_and_preserve_the_cause() {
        let detail = "Cannot read the pinned BuffBot source.";
        let error = mod_upgrade_inspection_error(detail);
        assert_eq!(error.code, "mod_upgrade_check_failed");
        assert_eq!(error.message, "The BuffBot update check could not finish.");
        assert!(error
            .recovery_action
            .contains("did not apply an update or change game files"));
        assert!(error
            .recovery_action
            .contains("any earlier recovery warning still applies"));
        assert!(!error.recovery_action.contains("safe to play"));
        assert_eq!(error.technical_detail, detail);
    }

    #[test]
    fn inspect_failure_uses_check_wording_without_changing_game_or_recovery_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        let managed = root.join("managed");
        fs::create_dir_all(managed.join(".chriz")).unwrap();
        fs::create_dir_all(managed.join("game")).unwrap();
        // The registry accepts this exact receipt, but the engine cannot parse it.
        let receipt = managed.join(".chriz/install-receipt.json");
        let receipt_bytes = b"unreadable receipt";
        fs::write(&receipt, receipt_bytes).unwrap();
        let launch = managed.join("game/InfinityLoader.exe");
        let launch_bytes = b"fixture launcher";
        fs::write(&launch, launch_bytes).unwrap();
        let record = bg_engine::registry::ManagedInstallRecord {
            schema_version: bg_engine::registry::REGISTRY_SCHEMA_VERSION,
            install_id: "failed-check".to_owned(),
            display_name: "Read-only check fixture".to_owned(),
            managed_root: managed.clone(),
            recipe_version: "base-version".to_owned(),
            recipe_sha256: "11".repeat(32),
            engine_name: "Test".to_owned(),
            managed_save_root: root.join("profile"),
            launch_path: launch.clone(),
            receipt_sha256: sha256_bytes(receipt_bytes),
            completed_at_millis: 1,
            recipe_id: None,
        };
        ManagedInstallRegistry::open_or_create(&root.join("app-data"))
            .unwrap()
            .publish(&record)
            .unwrap();
        let bridge = bridge(&root);
        let expected_detail = bg_engine::mod_upgrades::inspect(&bg_engine::mod_upgrades::Context {
            record: &record,
            app_data: &root.join("app-data"),
            cache: &root.join("cache"),
            recipe: &root.join("recipe"),
        })
        .unwrap_err();
        let error = bridge.inspect_mod_upgrade(&record.install_id).unwrap_err();
        assert_eq!(error, mod_upgrade_inspection_error(&expected_detail));
        assert!(!managed.join(".chriz/mod-upgrades").exists());
        assert!(!root.join("app-data/mod-upgrade-backups").exists());
        assert!(!root.join("cache").exists());
        assert!(!bridge.file_update_active());

        // A previous unresolved update must not become "safe to play" just
        // because this separate, read-only check did not write game files.
        let prior = managed.join(".chriz/mod-upgrades/unknown-format");
        fs::create_dir_all(&prior).unwrap();
        fs::write(prior.join("evidence"), b"previous recovery evidence").unwrap();
        let error = bridge.inspect_mod_upgrade(&record.install_id).unwrap_err();
        assert_eq!(error, mod_upgrade_inspection_error(expected_detail));
        assert_eq!(fs::read(&receipt).unwrap(), receipt_bytes);
        assert_eq!(fs::read(&launch).unwrap(), launch_bytes);
        assert_eq!(
            fs::read(prior.join("evidence")).unwrap(),
            b"previous recovery evidence"
        );
        assert!(
            project_managed_install(ManagedInstallCard {
                availability: InstallAvailability::Available,
                record,
            })
            .unwrap()
            .patch_recovery_needed
        );
    }

    #[test]
    fn mutation_errors_keep_the_recovery_guidance_and_technical_cause() {
        let error = mod_upgrade_error("update or restoration failed");
        assert_eq!(error.code, "mod_upgrade_failed");
        assert_eq!(error.message, "The BuffBot update could not be completed.");
        assert!(error
            .recovery_action
            .contains("restore its backup before playing"));
        assert_eq!(error.technical_detail, "update or restoration failed");
    }

    #[test]
    fn mod_upgrade_commands_reject_unknown_ids_without_retaining_the_mutation_guard() {
        let temp = tempfile::tempdir().unwrap();
        let bridge = bridge(temp.path());
        assert_eq!(
            bridge.inspect_mod_upgrade("unknown").unwrap_err().code,
            "managed_install_unknown"
        );
        assert_eq!(
            bridge
                .apply_mod_upgrade("unknown", "untrusted-review", true, false)
                .unwrap_err()
                .code,
            "managed_install_unknown"
        );
        assert!(!bridge.file_update_active());
        for undo in [false, true] {
            assert_eq!(
                bridge
                    .restore_mod_upgrade("unknown", undo)
                    .unwrap_err()
                    .code,
                "managed_install_unknown"
            );
            assert!(!bridge.file_update_active());
        }
        assert!(!temp.path().join("cache").exists());
    }

    #[test]
    fn whole_mod_mutations_cannot_overlap_an_existing_update() {
        let temp = tempfile::tempdir().unwrap();
        let bridge = bridge(temp.path());
        let guard = bridge.begin_app_update().unwrap();
        assert_eq!(
            bridge
                .apply_mod_upgrade("unknown", "review", true, false)
                .unwrap_err()
                .code,
            "update_busy"
        );
        assert_eq!(
            bridge
                .restore_mod_upgrade("unknown", false)
                .unwrap_err()
                .code,
            "update_busy"
        );
        assert!(bridge.file_update_active());
        drop(guard);
        assert!(!bridge.file_update_active());
    }

    #[test]
    fn unknown_upgrade_history_is_visible_as_recovery_without_hiding_the_installation() {
        let temp = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(temp.path()).unwrap();
        fs::create_dir_all(root.join(".chriz/mod-upgrades/unknown-format")).unwrap();
        let response = project_managed_install(ManagedInstallCard {
            availability: InstallAvailability::Available,
            record: bg_engine::registry::ManagedInstallRecord {
                schema_version: bg_engine::registry::REGISTRY_SCHEMA_VERSION,
                install_id: "test-recovery".to_owned(),
                display_name: "Test game".to_owned(),
                managed_root: root.clone(),
                recipe_version: "base-version".to_owned(),
                recipe_sha256: "11".repeat(32),
                engine_name: "Test".to_owned(),
                managed_save_root: root.join("profile"),
                launch_path: root.join("game/InfinityLoader.exe"),
                receipt_sha256: "22".repeat(32),
                completed_at_millis: 1,
                recipe_id: None,
            },
        })
        .unwrap();
        assert!(
            response.available,
            "Updates must still be able to select this copy"
        );
        assert!(response.patch_recovery_needed);
        assert!(response.status.contains("open Updates"));
        assert_eq!(response.recipe_version.as_deref(), Some("base-version"));
    }
}

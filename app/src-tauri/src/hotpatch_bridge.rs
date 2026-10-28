// Included in bridge.rs: hotpatches accept registered installation IDs only, never game paths.
fn hotpatch_inspection_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::new(
        "hotpatch_check_failed",
        "The hotpatch check could not finish.",
        "This check did not change your game. Try it again, or export diagnostics.",
        error.to_string(),
    )
}

fn hotpatch_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::new(
        "hotpatch_failed",
        "The hotpatch could not be completed.",
        "Check this installation again in Updates. If restoring is offered, restore it before playing.",
        error.to_string(),
    )
}

impl NativeBridge {
    /// The catalog bundled with this app, or the repository's while developing.
    fn hotpatch_catalog(
        &self,
        error: fn(String) -> CommandError,
    ) -> Result<bg_engine::hotpatch::catalog::Catalog, CommandError> {
        let folder = self
            .flavor
            .hotpatches
            .ok_or_else(|| error("This app has no hotpatches.".into()))?;
        let root = self.resource_root.clone().unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
        });
        bg_engine::hotpatch::catalog::load(&root.join(folder)).map_err(error)
    }

    pub fn inspect_hotpatch(
        &self,
        install_id: &str,
    ) -> Result<bg_engine::hotpatch::HotpatchPreview, CommandError> {
        self.ensure_feature(self.flavor.hotpatches.is_some(), "Hotpatches")?;
        self.ensure_update_idle()?;
        let catalog = self.hotpatch_catalog(hotpatch_inspection_error)?;
        let record = self.available_managed_install(install_id)?.record;
        let app_data = self
            .app_data
            .as_ref()
            .ok_or_else(|| hotpatch_inspection_error("Application data is unavailable."))?;
        let locks = self
            .target_locks
            .as_ref()
            .ok_or_else(|| hotpatch_inspection_error("Application data is unavailable."))?;
        let _target = locks.try_acquire(&record.managed_root).map_err(|error| {
            CommandError::new(
                "hotpatch_busy",
                "This installation is busy.",
                "Wait for its current operation to finish, then check for hotpatches again.",
                error.to_string(),
            )
        })?;
        bg_engine::hotpatch::inspect(
            &bg_engine::hotpatch::Context {
                record: &record,
                app_data,
                locks,
                cache: &self.cache,
            },
            &catalog,
        )
        .map_err(hotpatch_inspection_error)
    }

    pub fn apply_hotpatch(
        &self,
        install_id: &str,
        review_token: &str,
        full_backup: bool,
        save_backup: bool,
    ) -> Result<bg_engine::hotpatch::HotpatchPreview, CommandError> {
        self.apply_hotpatch_with_progress(install_id, review_token, full_backup, save_backup, &|_| {})
    }

    pub fn apply_hotpatch_with_progress(
        &self,
        install_id: &str,
        review_token: &str,
        full_backup: bool,
        save_backup: bool,
        progress: bg_engine::operation_progress::ProgressSink<'_>,
    ) -> Result<bg_engine::hotpatch::HotpatchPreview, CommandError> {
        self.ensure_feature(self.flavor.hotpatches.is_some(), "Hotpatches")?;
        let _guard = self.begin_app_update()?;
        let catalog = self.hotpatch_catalog(hotpatch_error)?;
        let record = self.available_managed_install(install_id)?.record;
        let app_data = self
            .app_data
            .as_ref()
            .ok_or_else(|| hotpatch_error("Application data is unavailable."))?;
        let locks = self
            .target_locks
            .as_ref()
            .ok_or_else(|| hotpatch_error("Application data is unavailable."))?;
        // The engine acquires and holds the target lock through backup and mutation.
        bg_engine::hotpatch::apply_with_progress(
            &bg_engine::hotpatch::Context {
                record: &record,
                app_data,
                locks,
                cache: &self.cache,
            },
            &catalog,
            review_token,
            full_backup,
            save_backup,
            progress,
        )
        .map_err(hotpatch_error)
    }

    pub fn restore_hotpatch(
        &self,
        install_id: &str,
        undo: bool,
    ) -> Result<bg_engine::hotpatch::HotpatchPreview, CommandError> {
        self.restore_hotpatch_with_progress(install_id, undo, &|_| {})
    }

    /// Undo the latest applied hotpatch (`undo`), or finish restoring one that did not finish.
    pub fn restore_hotpatch_with_progress(
        &self,
        install_id: &str,
        undo: bool,
        progress: bg_engine::operation_progress::ProgressSink<'_>,
    ) -> Result<bg_engine::hotpatch::HotpatchPreview, CommandError> {
        self.ensure_feature(self.flavor.hotpatches.is_some(), "Hotpatches")?;
        let _guard = self.begin_app_update()?;
        let catalog = self.hotpatch_catalog(hotpatch_error)?;
        let record = self.available_managed_install(install_id)?.record;
        let app_data = self
            .app_data
            .as_ref()
            .ok_or_else(|| hotpatch_error("Application data is unavailable."))?;
        let locks = self
            .target_locks
            .as_ref()
            .ok_or_else(|| hotpatch_error("Application data is unavailable."))?;
        bg_engine::hotpatch::restore_with_progress(
            &bg_engine::hotpatch::Context {
                record: &record,
                app_data,
                locks,
                cache: &self.cache,
            },
            &catalog,
            undo,
            progress,
        )
        .map_err(hotpatch_error)
    }
}

#[cfg(test)]
mod hotpatch_bridge_tests {
    use super::*;

    #[test]
    fn only_the_iwd_app_offers_hotpatches_and_its_catalog_loads() {
        // Both flavors explicitly: CI also runs these tests in the IWD build.
        let bridge = || NativeBridge::new(PathBuf::from("unused"), "chris-recommended");
        let refused = bridge()
            .with_flavor(&crate::flavor::BG)
            .inspect_hotpatch("any")
            .unwrap_err();
        assert_eq!(refused.code, "feature_unavailable");
        let iwd = bridge().with_flavor(&crate::flavor::IWD);
        let catalog = iwd.hotpatch_catalog(hotpatch_error).unwrap();
        assert_eq!(catalog.game, bg_engine::manifest::CollectionGame::Iwdee);
    }
}

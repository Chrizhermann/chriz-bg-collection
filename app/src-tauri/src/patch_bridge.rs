// Included in bridge.rs so the command boundary can reuse its ID-based authority.
fn patch_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::new("install_patch_failed", "This game fix could not be completed.",
        "Check this installation again in Updates. If restoration is offered, restore its backup before playing.", error.to_string())
}

fn packaged_patch_catalog() -> Result<bg_engine::patches::catalog::Catalog, CommandError> {
    bg_engine::patches::catalog::verify(
        include_bytes!("../../../patches/catalog.json"),
        include_str!("../../../patches/catalog.json.sig"),
        include_str!("../../../patches/public-key.txt"),
        env!("CARGO_PKG_VERSION"),
    )
    .map_err(patch_error)
}

impl NativeBridge {
    pub fn inspect_install_patches(
        &self,
        id: &str,
    ) -> Result<bg_engine::patches::PatchPreview, CommandError> {
        let record = self.available_managed_install(id)?.record;
        let app_data = self
            .app_data
            .as_ref()
            .ok_or_else(|| patch_error("Application data unavailable"))?;
        let locks = self
            .target_locks
            .as_ref()
            .ok_or_else(|| patch_error("Application data unavailable"))?;
        let _target = locks
            .try_acquire(&record.managed_root)
            .map_err(patch_error)?;
        let ctx = bg_engine::patches::Context {
            record: &record,
            app_data,
            locks,
            cache: &self.cache,
        };
        bg_engine::patches::inspect(&ctx, &packaged_patch_catalog()?).map_err(patch_error)
    }
    pub fn apply_install_patch(
        &self,
        id: &str,
        token: &str,
        full: bool,
        saves: bool,
    ) -> Result<bg_engine::patches::PatchPreview, CommandError> {
        self.apply_install_patch_with_progress(id, token, full, saves, &|_| {})
    }

    pub fn apply_install_patch_with_progress(
        &self,
        id: &str,
        token: &str,
        full: bool,
        saves: bool,
        progress: bg_engine::operation_progress::ProgressSink<'_>,
    ) -> Result<bg_engine::patches::PatchPreview, CommandError> {
        let _guard = self.begin_app_update()?;
        let record = self.available_managed_install(id)?.record;
        let app_data = self
            .app_data
            .as_ref()
            .ok_or_else(|| patch_error("Application data unavailable"))?;
        let locks = self
            .target_locks
            .as_ref()
            .ok_or_else(|| patch_error("Application data unavailable"))?;
        let ctx = bg_engine::patches::Context {
            record: &record,
            app_data,
            locks,
            cache: &self.cache,
        };
        bg_engine::patches::apply_with_progress(
            &ctx,
            &packaged_patch_catalog()?,
            token,
            full,
            saves,
            progress,
        )
        .map_err(patch_error)
    }
    pub fn restore_install_patch(
        &self,
        id: &str,
        undo: bool,
    ) -> Result<bg_engine::patches::PatchPreview, CommandError> {
        self.restore_install_patch_with_progress(id, undo, &|_| {})
    }

    pub fn restore_install_patch_with_progress(
        &self,
        id: &str,
        undo: bool,
        progress: bg_engine::operation_progress::ProgressSink<'_>,
    ) -> Result<bg_engine::patches::PatchPreview, CommandError> {
        let _guard = self.begin_app_update()?;
        let record = self.available_managed_install(id)?.record;
        let app_data = self
            .app_data
            .as_ref()
            .ok_or_else(|| patch_error("Application data unavailable"))?;
        let locks = self
            .target_locks
            .as_ref()
            .ok_or_else(|| patch_error("Application data unavailable"))?;
        let ctx = bg_engine::patches::Context {
            record: &record,
            app_data,
            locks,
            cache: &self.cache,
        };
        bg_engine::patches::restore_with_progress(&ctx, &packaged_patch_catalog()?, undo, progress)
            .map_err(patch_error)
    }
    pub fn file_update_active(&self) -> bool {
        self.runtime_lock().app_update_active
    }
}

#[cfg(test)]
mod patch_catalog_tests {
    #[test]
    fn packaged_catalog_signature_and_domain_verify() {
        let catalog = super::packaged_patch_catalog().unwrap();
        assert_eq!(catalog.patch_id, bg_engine::patches::catalog::PATCH_ID);
        let ledger = include_str!(
            "../../../recipes/curated-full-current/releases/v0.1.0-alpha.14/ledger.toml"
        );
        for id in catalog.change_ids {
            assert!(ledger.contains(&format!("id = \"{id}\"")));
        }
    }
}

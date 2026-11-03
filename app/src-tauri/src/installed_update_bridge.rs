// ID-based read-only comparison; arbitrary game paths and latest defaults are not accepted.
impl NativeBridge {
    pub fn inspect_installed_update(
        &self,
        id: &str,
    ) -> Result<bg_engine::installed_updates::InstalledUpdatePlan, CommandError> {
        self.ensure_update_idle()?;
        let record = self.available_managed_install(id)?.record;
        let app_data = self
            .app_data
            .as_ref()
            .ok_or_else(|| patch_error("Application data unavailable"))?;
        let _target =
            bg_engine::lock::TargetLock::try_acquire(app_data.join("locks"), &record.managed_root)
                .map_err(|error| {
                    CommandError::new(
                        "installed_update_busy",
                        "This installation is busy.",
                        "Wait for its current operation to finish, then check again.",
                        error.to_string(),
                    )
                })?;
        bg_engine::cli::inspect_installed_update(&record, &self.recipe).map_err(|error|
            CommandError::new("installed_update_unavailable", "This installation could not be compared.", "Keep the installation unchanged and check the details. No game files were modified.", error.to_string()))
    }
}

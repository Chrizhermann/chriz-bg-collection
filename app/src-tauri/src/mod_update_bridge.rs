// Included in bridge.rs: whole-mod updates such as BuffBot, recorded in the hotpatch history so
// that Undo always takes back the latest change. Registered installation IDs only, never paths.

/// Where the catalog of mods this app updates lies, below the packaged resources.
const MOD_UPDATE_CATALOG: &str = "hotpatches/bg";

fn mod_update_inspection_error(title: Option<&str>, error: impl std::fmt::Display) -> CommandError {
    CommandError::new(
        "mod_update_check_failed",
        format!("The {} check could not finish.", title.unwrap_or("update")),
        "This check did not change your game. Try it again, or export diagnostics.",
        error.to_string(),
    )
}

fn mod_update_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::new(
        "mod_update_failed",
        "The update could not be completed.",
        "Check this installation again in Updates. If restoring is offered, restore it before playing.",
        error.to_string(),
    )
}

/// The catalog title of `mod_id`, or a refusal before anything is read or changed.
fn offered_mod_update(
    catalog: &bg_engine::hotpatch::catalog::Catalog,
    mod_id: &str,
) -> Result<String, CommandError> {
    catalog
        .update(mod_id)
        .map(|update| update.title.clone())
        .ok_or_else(|| {
            CommandError::new(
                "mod_update_unavailable",
                "This app offers no update for that mod.",
                "Return to Updates; nothing was changed.",
                format!("the update catalog lists no mod {mod_id:?}"),
            )
        })
}

impl NativeBridge {
    /// The catalog bundled with this app, or the repository's while developing.
    fn mod_update_catalog(
        &self,
        error: fn(String) -> CommandError,
    ) -> Result<bg_engine::hotpatch::catalog::Catalog, CommandError> {
        let root = self
            .resource_root
            .clone()
            .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
        bg_engine::hotpatch::catalog::load(&root.join(MOD_UPDATE_CATALOG)).map_err(error)
    }

    pub fn inspect_mod_update(
        &self,
        install_id: &str,
        mod_id: &str,
    ) -> Result<bg_engine::hotpatch::update::ModUpdatePreview, CommandError> {
        self.ensure_update_idle()?;
        let catalog = self.mod_update_catalog(|error| mod_update_inspection_error(None, error))?;
        let title = offered_mod_update(&catalog, mod_id)?;
        let record = self.available_managed_install(install_id)?.record;
        let app_data = self.app_data.as_ref().ok_or_else(|| {
            mod_update_inspection_error(Some(&title), "Application data is unavailable.")
        })?;
        let _target =
            bg_engine::lock::TargetLock::try_acquire(app_data.join("locks"), &record.managed_root)
                .map_err(|error| {
                    CommandError::new(
                        "mod_update_busy",
                        "This installation is busy.",
                        format!("Wait for its current operation to finish, then check for a {title} update again."),
                        error.to_string(),
                    )
                })?;
        bg_engine::hotpatch::update::inspect(
            &bg_engine::hotpatch::Context {
                record: &record,
                app_data,
                cache: &self.cache,
            },
            &catalog,
            mod_id,
        )
        .map_err(|error| mod_update_inspection_error(Some(&title), error))
    }

    pub fn apply_mod_update(
        &self,
        install_id: &str,
        mod_id: &str,
        review_token: &str,
        full_backup: bool,
        save_backup: bool,
    ) -> Result<bg_engine::hotpatch::update::ModUpdatePreview, CommandError> {
        self.apply_mod_update_with_progress(
            install_id,
            mod_id,
            review_token,
            full_backup,
            save_backup,
            &|_| {},
        )
    }

    pub fn apply_mod_update_with_progress(
        &self,
        install_id: &str,
        mod_id: &str,
        review_token: &str,
        full_backup: bool,
        save_backup: bool,
        progress: bg_engine::operation_progress::ProgressSink<'_>,
    ) -> Result<bg_engine::hotpatch::update::ModUpdatePreview, CommandError> {
        let _guard = self.begin_app_update()?;
        let catalog = self.mod_update_catalog(mod_update_error)?;
        offered_mod_update(&catalog, mod_id)?;
        let record = self.available_managed_install(install_id)?.record;
        let app_data = self
            .app_data
            .as_ref()
            .ok_or_else(|| mod_update_error("Application data is unavailable."))?;
        // The engine acquires and holds the target lock through backup and mutation.
        bg_engine::hotpatch::update::apply_with_progress(
            &bg_engine::hotpatch::Context {
                record: &record,
                app_data,
                cache: &self.cache,
            },
            &catalog,
            mod_id,
            review_token,
            full_backup,
            save_backup,
            progress,
        )
        .map_err(mod_update_error)
    }

    pub fn restore_mod_update(
        &self,
        install_id: &str,
        mod_id: &str,
        undo: bool,
    ) -> Result<bg_engine::hotpatch::update::ModUpdatePreview, CommandError> {
        self.restore_mod_update_with_progress(install_id, mod_id, undo, &|_| {})
    }

    /// Undo the latest change (`undo`), or finish restoring one that did not finish. Updates and
    /// hotpatches share one history, so this is the hotpatch restore; the result is the mod's
    /// fresh check.
    pub fn restore_mod_update_with_progress(
        &self,
        install_id: &str,
        mod_id: &str,
        undo: bool,
        progress: bg_engine::operation_progress::ProgressSink<'_>,
    ) -> Result<bg_engine::hotpatch::update::ModUpdatePreview, CommandError> {
        let _guard = self.begin_app_update()?;
        let catalog = self.mod_update_catalog(mod_update_error)?;
        let title = offered_mod_update(&catalog, mod_id)?;
        let record = self.available_managed_install(install_id)?.record;
        let app_data = self
            .app_data
            .as_ref()
            .ok_or_else(|| mod_update_error("Application data is unavailable."))?;
        let context = bg_engine::hotpatch::Context {
            record: &record,
            app_data,
            cache: &self.cache,
        };
        // Undo takes back the latest change of the shared history, so this card may undo only
        // when that change is an update of this mod. Reading the history needs no network.
        if undo
            && !bg_engine::hotpatch::update::undo_offered(&context, mod_id)
                .map_err(mod_update_error)?
        {
            return Err(mod_update_error(format!(
                "The latest change to this game is not a {title} update that can be undone."
            )));
        }
        // The engine acquires and releases the target lock itself.
        bg_engine::hotpatch::restore_with_progress(&context, &catalog, undo, progress)
            .map_err(mod_update_error)?;
        // The game files are back; the fresh check reads them under the target lock. Its failure
        // is not a read-only check failure: the game did change, so say so.
        let checked = |error: String| {
            mod_update_error(format!(
                "The game files were put back, but checking {title} afterwards failed: {error}"
            ))
        };
        let _target =
            bg_engine::lock::TargetLock::try_acquire(app_data.join("locks"), &record.managed_root)
                .map_err(|error| checked(error.to_string()))?;
        bg_engine::hotpatch::update::inspect(&context, &catalog, mod_id).map_err(checked)
    }
}

#[cfg(test)]
mod mod_update_bridge_tests {
    use super::*;

    #[test]
    fn the_app_offers_the_buffbot_update_and_refuses_unknown_mods() {
        let bridge = NativeBridge::new(PathBuf::from("unused"), "chris-recommended");
        let catalog = bridge.mod_update_catalog(mod_update_error).unwrap();
        assert_eq!(offered_mod_update(&catalog, "buffbot").unwrap(), "BuffBot");
        for refused in [
            bridge.inspect_mod_update("any", "not-a-mod").unwrap_err(),
            bridge
                .apply_mod_update("any", "not-a-mod", "token", true, false)
                .unwrap_err(),
            bridge
                .restore_mod_update("any", "not-a-mod", true)
                .unwrap_err(),
        ] {
            assert_eq!(refused.code, "mod_update_unavailable");
        }
    }
}

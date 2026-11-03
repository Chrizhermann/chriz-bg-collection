// Included by bridge.rs: names are AppData display metadata, not game mutations.

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallationNameResponse {
    pub name: String,
}

impl NativeBridge {
    /// Suggest an unused display name and a separate unused home/Games child.
    /// No game/home folder or installation record is created or changed here.
    /// Normal destination inspection and creation-time checks still run afterwards.
    pub fn suggest_installation_defaults(
        &self,
        home: &Path,
    ) -> Result<InstallationDefaultsResponse, CommandError> {
        if !home.is_absolute()
            || home
                .components()
                .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
        {
            return Err(unsafe_destination(
                home,
                "the home folder must be absolute and normalized",
            ));
        }
        let registry = self.registry()?;
        let name = registry
            .suggested_display_name()
            .map_err(installation_name_error)?;
        let completed = registry.list().map_err(installation_name_error)?;
        let campaigns = registry.list_campaigns().map_err(installation_name_error)?;
        let roots = completed
            .into_iter()
            .map(|card| card.record.managed_root)
            .chain(campaigns.into_iter().map(|card| card.record.managed_root))
            .map(|root| display_windows_path(&root).map(PathBuf::from))
            .collect::<Result<Vec<_>, _>>()?;
        unused_installation_defaults(home, &name, &roots, &|path| fs::symlink_metadata(path))
    }

    /// Read a currently unused default. The engine rechecks uniqueness at creation.
    pub fn suggest_installation_name(&self) -> Result<InstallationNameResponse, CommandError> {
        let name = self
            .registry()?
            .suggested_display_name()
            .map_err(installation_name_error)?;
        Ok(InstallationNameResponse { name })
    }

    /// Advisory validation for inline feedback; this neither reserves nor changes a name.
    pub fn validate_installation_name(
        &self,
        name: &str,
        install_id: Option<&str>,
    ) -> Result<InstallationNameResponse, CommandError> {
        let name = self
            .registry()?
            .validate_display_name(name, install_id)
            .map_err(installation_name_error)?;
        Ok(InstallationNameResponse { name })
    }

    /// Rename a registered completed or incomplete copy without moving its game/save files.
    pub fn rename_installation(
        &self,
        install_id: &str,
        name: &str,
    ) -> Result<InstallationNameResponse, CommandError> {
        let runtime = self.runtime_lock();
        if !runtime.active_runs.is_empty() || runtime.app_update_active || runtime.removal_active {
            return Err(CommandError::new(
                "installation_name_busy",
                "Wait for the current operation to finish before renaming.",
                "Rename this installation from My installs when it finishes.",
                "installation rename requested while an installation, update or removal is active",
            ));
        }
        let name = self
            .registry()?
            .rename_display_name(install_id, name)
            .map_err(installation_name_error)?;
        Ok(InstallationNameResponse { name })
    }
}

fn unused_installation_defaults(
    home: &Path,
    name: &str,
    registered_roots: &[PathBuf],
    metadata: &impl Fn(&Path) -> io::Result<fs::Metadata>,
) -> Result<InstallationDefaultsResponse, CommandError> {
    let games = home.join("Games");
    // symlink_metadata catches dangling links as well as ordinary junctions.
    // Check parents before considering a missing child to be an unused location.
    for ancestor in games.ancestors() {
        match metadata(ancestor) {
            Ok(info) if !info.is_dir() || metadata_is_reparse(&info) => {
                return Err(unsafe_destination(
                    ancestor,
                    "the default installation parent must be a direct ordinary directory",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(default_destination_error(ancestor, error)),
        }
    }
    let games = PathBuf::from(display_windows_path(&normalize_prospective_path(&games)?)?);
    if registered_roots
        .iter()
        .any(|root| path_is_within(&games, root))
    {
        return Err(unsafe_destination(
            &games,
            "the default Games folder is inside a registered installation; choose another location",
        ));
    }
    for suffix in 1..=10_000 {
        let folder_name = if suffix == 1 {
            name.to_owned()
        } else {
            format!("{name} ({suffix})")
        };
        let candidate = games.join(folder_name);
        if registered_roots
            .iter()
            .any(|root| paths_overlap(&candidate, root))
        {
            continue;
        }
        match metadata(&candidate) {
            // Any existing entry is occupied, including empty folders and dangling links.
            Ok(_) => continue,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(InstallationDefaultsResponse {
                    name: name.to_owned(),
                    path: display_windows_path(&candidate)?,
                });
            }
            Err(error) => return Err(default_destination_error(&candidate, error)),
        }
    }
    Err(CommandError::new(
        "destination_unavailable",
        "An unused default installation folder could not be found.",
        "Choose a new installation location manually.",
        format!(
            "{}: all 10000 candidate folder names are already occupied",
            games.display()
        ),
    ))
}

fn default_destination_error(path: &Path, error: io::Error) -> CommandError {
    path_error(
        "destination_unavailable",
        "The default installation location could not be checked.",
        path,
        error,
    )
}

fn installation_name_error(error: bg_engine::registry::RegistryError) -> CommandError {
    use bg_engine::registry::RegistryError;
    let detail = error.to_string();
    match error {
        RegistryError::InvalidName(_) => CommandError::new(
            "invalid_display_name",
            &detail,
            "Choose a different installation name.",
            &detail,
        ),
        RegistryError::DuplicateName { .. } => CommandError::new(
            "duplicate_display_name",
            &detail,
            "Use another name, such as a playthrough or character name.",
            &detail,
        ),
        _ => CommandError::new(
            "installation_name_failed",
            "The installation name could not be saved or checked.",
            "Refresh My installs and try again. Your game files have not changed.",
            &detail,
        ),
    }
}

#[cfg(test)]
mod installation_name_tests {
    use super::*;
    use bg_engine::registry::{ManagedInstallRecord, REGISTRY_SCHEMA_VERSION};
    use tempfile::TempDir;

    fn fixture() -> (TempDir, NativeBridge, PathBuf) {
        fixture_at("managed")
    }

    fn fixture_at(relative_managed: &str) -> (TempDir, NativeBridge, PathBuf) {
        let temp = TempDir::new().unwrap();
        let app_data = temp.path().join("app-data");
        // The installer registers canonical roots. TEMP can be an 8.3 short path (`RUNNER~1` on
        // CI runners), and a registered short path never matches a canonical candidate.
        let managed = temp.path().canonicalize().unwrap().join(relative_managed);
        fs::create_dir_all(managed.join(".chriz")).unwrap();
        fs::create_dir_all(managed.join("game")).unwrap();
        fs::write(managed.join(".chriz/install-receipt.json"), b"receipt").unwrap();
        fs::write(managed.join("game/InfinityLoader.exe"), b"launcher").unwrap();
        ManagedInstallRegistry::open_or_create(&app_data)
            .unwrap()
            .publish(&ManagedInstallRecord {
                schema_version: REGISTRY_SCHEMA_VERSION,
                install_id: "name-fixture".into(),
                display_name: "Chriz Easy BG".into(),
                managed_root: managed.clone(),
                recipe_version: "fixture".into(),
                recipe_sha256: "11".repeat(32),
                engine_name: "Isolated profile".into(),
                managed_save_root: temp.path().join("saves"),
                launch_path: managed.join("game/InfinityLoader.exe"),
                receipt_sha256: sha256_bytes(b"receipt"),
                completed_at_millis: 1,
            })
            .unwrap();
        let bridge = NativeBridge::with_configuration(
            temp.path().join("recipe"),
            "fixture".into(),
            temp.path().join("cache"),
            Some(app_data),
            Arc::new(SystemBridgeEngine),
            Arc::new(SystemBridgeSystem),
        );
        (temp, bridge, managed)
    }

    #[test]
    fn naming_commands_check_unique_names_and_rename_display_metadata_only() {
        let (_temp, bridge, managed) = fixture();
        assert_eq!(
            bridge.suggest_installation_name().unwrap().name,
            "Chriz Easy BG (2)"
        );
        assert_eq!(
            bridge
                .validate_installation_name(" chriz EASY bg ", None)
                .unwrap_err()
                .code,
            "duplicate_display_name"
        );
        assert_eq!(
            bridge
                .validate_installation_name("  My next run  ", None)
                .unwrap()
                .name,
            "My next run"
        );
        assert_eq!(
            bridge
                .rename_installation("name-fixture", "  My next run  ")
                .unwrap()
                .name,
            "My next run"
        );
        assert_eq!(
            bridge
                .registry()
                .unwrap()
                .display_name("name-fixture")
                .unwrap(),
            "My next run"
        );
        assert_eq!(
            fs::read(managed.join(".chriz/install-receipt.json")).unwrap(),
            b"receipt"
        );
        assert_eq!(
            bridge.registry().unwrap().list().unwrap()[0]
                .record
                .engine_name,
            "Isolated profile"
        );
        assert_eq!(
            bridge.suggest_installation_name().unwrap().name,
            "Chriz Easy BG"
        );
        assert_eq!(
            bridge.list_managed_installations().unwrap()[0].name,
            "My next run"
        );
    }

    #[test]
    fn naming_rejects_unknown_ids_invalid_names_and_active_operations() {
        let (_temp, bridge, _managed) = fixture();
        assert_eq!(
            bridge
                .rename_installation("unknown", "Safe name")
                .unwrap_err()
                .code,
            "installation_name_failed"
        );
        assert_eq!(
            bridge
                .rename_installation("name-fixture", "  ")
                .unwrap_err()
                .code,
            "invalid_display_name"
        );
        bridge.runtime_lock().removal_active = true;
        assert_eq!(
            bridge
                .rename_installation("name-fixture", "Safe name")
                .unwrap_err()
                .code,
            "installation_name_busy"
        );
        assert_eq!(
            bridge
                .registry()
                .unwrap()
                .display_name("name-fixture")
                .unwrap(),
            "Chriz Easy BG"
        );
    }

    fn snapshot(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
        let mut result = BTreeMap::new();
        for entry in fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            let info = fs::symlink_metadata(&path).unwrap();
            if info.is_dir() && !metadata_is_reparse(&info) {
                result.insert(path.clone(), None);
                result.extend(snapshot(&path));
            } else {
                result.insert(path.clone(), Some(fs::read(&path).unwrap()));
            }
        }
        result
    }

    #[test]
    fn renamed_installation_defaults_use_a_free_folder_without_changing_the_display_name() {
        let (temp, bridge, managed) = fixture_at("Games/Chriz Easy BG");
        bridge
            .rename_installation("name-fixture", "Stream")
            .unwrap();
        let home = temp.path().canonicalize().unwrap();
        let before = snapshot(temp.path());
        let defaults = bridge.suggest_installation_defaults(&home).unwrap();
        assert_eq!(defaults.name, "Chriz Easy BG");
        assert_eq!(
            defaults.path,
            display_windows_path(&home.join("Games").join("Chriz Easy BG (2)")).unwrap()
        );
        assert_eq!(snapshot(temp.path()), before);
        assert_eq!(
            fs::read(managed.join(".chriz/install-receipt.json")).unwrap(),
            b"receipt"
        );
    }

    #[test]
    fn defaults_skip_existing_unregistered_files_and_folders() {
        let (temp, bridge, _managed) = fixture();
        bridge
            .rename_installation("name-fixture", "Stream")
            .unwrap();
        let home = temp.path().canonicalize().unwrap();
        let games = home.join("Games");
        fs::create_dir_all(games.join("Chriz Easy BG")).unwrap();
        fs::write(games.join("Chriz Easy BG (2)"), b"user file").unwrap();
        let before = snapshot(temp.path());
        let defaults = bridge.suggest_installation_defaults(&home).unwrap();
        assert_eq!(defaults.name, "Chriz Easy BG");
        assert_eq!(
            defaults.path,
            display_windows_path(&games.join("Chriz Easy BG (3)")).unwrap()
        );
        assert_eq!(snapshot(temp.path()), before);
    }

    #[test]
    fn defaults_skip_missing_completed_and_incomplete_registered_roots() {
        let (temp, bridge, managed) = fixture_at("Games/Chriz Easy BG");
        bridge
            .rename_installation("name-fixture", "Stream")
            .unwrap();
        let home = temp.path().canonicalize().unwrap();
        fs::rename(&managed, home.join("moved-completed")).unwrap();
        let campaign_root = home.join("Games/Chriz Easy BG (2)");
        let pointer = bg_engine::registry::ManagedCampaignRecord {
            schema_version: REGISTRY_SCHEMA_VERSION,
            install_id: "missing-campaign".into(),
            managed_root: campaign_root,
            recipe_sha256: "22".repeat(32),
        };
        fs::write(
            temp.path()
                .join("app-data/managed-campaigns/missing-campaign.json"),
            serde_json::to_vec(&pointer).unwrap(),
        )
        .unwrap();
        let before = snapshot(temp.path());
        let defaults = bridge.suggest_installation_defaults(&home).unwrap();
        assert_eq!(
            defaults.path,
            display_windows_path(&home.join("Games").join("Chriz Easy BG (3)")).unwrap()
        );
        assert_eq!(snapshot(temp.path()), before);
    }

    #[test]
    fn default_selection_does_not_create_a_games_directory() {
        let (temp, bridge, _managed) = fixture();
        let before = snapshot(temp.path());
        let defaults = bridge.suggest_installation_defaults(temp.path()).unwrap();
        assert_eq!(defaults.name, "Chriz Easy BG (2)");
        assert!(!temp.path().join("Games").exists());
        assert_eq!(snapshot(temp.path()), before);
    }

    #[test]
    fn default_selection_reports_permission_and_unexpected_io_failures_without_skipping_them() {
        let temp = TempDir::new().unwrap();
        let home = temp.path().canonicalize().unwrap();
        let candidate =
            PathBuf::from(display_windows_path(&home.join("Games/Chriz Easy BG")).unwrap());
        for kind in [io::ErrorKind::PermissionDenied, io::ErrorKind::Other] {
            let error = unused_installation_defaults(&home, "Chriz Easy BG", &[], &|path| {
                if path == candidate {
                    Err(io::Error::new(kind, "fixture access failure"))
                } else {
                    fs::symlink_metadata(path)
                }
            })
            .unwrap_err();
            assert_eq!(error.code, "destination_unavailable");
            assert!(error.technical_detail.contains("Chriz Easy BG"));
            assert!(error.technical_detail.contains("fixture access failure"));
            assert!(!home.join("Games").exists());
        }
    }

    #[test]
    fn default_selection_does_not_use_a_dangling_symlink() {
        let (temp, bridge, _managed) = fixture();
        bridge
            .rename_installation("name-fixture", "Stream")
            .unwrap();
        let home = temp.path().canonicalize().unwrap();
        let games = home.join("Games");
        fs::create_dir(&games).unwrap();
        let link = games.join("Chriz Easy BG");
        #[cfg(windows)]
        let result = std::os::windows::fs::symlink_dir(home.join("missing-link-target"), &link);
        #[cfg(not(windows))]
        let result = std::os::unix::fs::symlink(home.join("missing-link-target"), &link);
        if let Err(error) = result {
            if matches!(
                error.kind(),
                io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
            ) || error.raw_os_error() == Some(1314)
            {
                eprintln!("symlink fixture unavailable: {error}");
                return;
            }
            panic!("cannot create fixture symlink: {error}");
        }
        let defaults = bridge.suggest_installation_defaults(&home).unwrap();
        assert_eq!(
            defaults.path,
            display_windows_path(&games.join("Chriz Easy BG (2)")).unwrap()
        );
        assert!(metadata_is_reparse(&fs::symlink_metadata(link).unwrap()));
        assert!(!games.join("Chriz Easy BG (2)").exists());
    }
}

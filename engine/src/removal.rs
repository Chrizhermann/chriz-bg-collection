//! Explicit, registry-bound deletion of a managed game copy. Saves and cache are not targets.

use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::digest::sha256_bytes;
use crate::lock::TargetLock;
use crate::preflight::{
    recheck_staging_target_before_mutation_with, PreflightHost, SystemPreflight,
};
use crate::registry::ManagedInstallRegistry;
use crate::session::SessionStore;
use crate::stage::DocumentsLocator;

/// A freshly validated removal description. The desktop keeps it server-side behind a token.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemovalPlan {
    pub install_id: String,
    pub display_name: String,
    pub managed_root: PathBuf,
    pub preserved_save_path: Option<PathBuf>,
    pub action: RemovalAction,
    registry_sha256: String,
    directory_identity: Option<u64>,
    protected_paths: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RemovalAction {
    Delete,
    Forget,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct RemovalError(String);

fn failure(error: impl std::fmt::Display) -> RemovalError {
    RemovalError(error.to_string())
}
fn io_at(path: &Path, error: io::Error) -> RemovalError {
    failure(format!("{}: {error}", path.display()))
}

/// Does not remove files or registry entries. Unknown ids never confer path authority.
pub fn preview(app_data: &Path, install_id: &str) -> Result<RemovalPlan, RemovalError> {
    let registry = ManagedInstallRegistry::open_or_create(app_data).map_err(failure)?;
    let installs = registry.list().map_err(failure)?;
    let campaigns = registry.list_campaigns().map_err(failure)?;
    let install = installs
        .iter()
        .find(|card| card.record.install_id == install_id)
        .map(|card| &card.record);
    let campaign = campaigns
        .iter()
        .find(|card| card.record.install_id == install_id)
        .map(|card| &card.record);
    let root = install
        .map(|record| &record.managed_root)
        .or_else(|| campaign.map(|record| &record.managed_root))
        .ok_or_else(|| failure("Unknown installation. Refresh My installs."))?;
    if campaign.is_some_and(|record| record.managed_root != *root) {
        return Err(failure("Conflicting registered roots."));
    }
    let snapshot = serde_json::to_vec(&(install, campaign)).map_err(failure)?;
    let mut plan = RemovalPlan {
        install_id: install_id.to_owned(),
        display_name: install
            .map(|record| record.display_name.clone())
            .unwrap_or_else(|| "Chriz Easy BG — incomplete installation".to_owned()),
        managed_root: root.clone(),
        preserved_save_path: install.map(|record| record.managed_save_root.clone()),
        action: RemovalAction::Forget,
        registry_sha256: sha256_bytes(&snapshot),
        directory_identity: None,
        protected_paths: vec![app_data.to_path_buf()],
    };
    reject_broad_root(root)?;
    direct_ancestors(root)?;
    for other in installs
        .iter()
        .map(|card| (&card.record.install_id, &card.record.managed_root))
        .chain(
            campaigns
                .iter()
                .map(|card| (&card.record.install_id, &card.record.managed_root)),
        )
    {
        if other.0 != install_id && overlap(root, other.1) {
            return Err(failure("Another installation overlaps this folder."));
        }
    }
    match fs::symlink_metadata(root) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(plan),
        Err(error) => return Err(io_at(root, error)),
        Ok(metadata) if !metadata.is_dir() || reparse(&metadata) => {
            return Err(failure("Installation root is not a direct folder."))
        }
        Ok(_) => {}
    }
    if fs::canonicalize(root).map_err(|e| io_at(root, e))? != *root {
        return Err(failure("Installation root identity changed."));
    }
    plan.action = RemovalAction::Delete;
    plan.directory_identity = Some(directory_identity(root)?);
    let journal_path = journal_path(app_data, install_id);
    if let Some(journal) = read_journal(&journal_path)? {
        if journal.install_id != plan.install_id
            || journal.managed_root != plan.managed_root
            || journal.registry_sha256 != plan.registry_sha256
            || journal.directory_identity != plan.directory_identity
            || journal.action != RemovalAction::Delete
        {
            return Err(failure(
                "Removal journal does not match this installation folder.",
            ));
        }
        plan = journal;
    } else {
        let replay = SessionStore::open(root)
            .and_then(|store| store.replay())
            .map_err(|e| failure(format!("Cannot prove ownership of this folder: {e}")))?;
        let created = replay.created();
        if created.install_id != install_id
            || created.managed_root != *root
            || install.is_some_and(|record| record.recipe_sha256 != created.recipe_payload_sha256)
            || campaign.is_some_and(|record| record.recipe_sha256 != created.recipe_payload_sha256)
        {
            return Err(failure("The folder belongs to a different installation."));
        }
        let payload: serde_json::Value =
            serde_json::from_slice(&created.recipe_payload).map_err(failure)?;
        for role in ["bg1", "bg2"] {
            let source = payload
                .get(role)
                .and_then(|source| source.get("root"))
                .and_then(|root| root.as_str())
                .ok_or_else(|| {
                    failure("Frozen source-game paths are unavailable; refusing deletion.")
                })?;
            plan.protected_paths.push(PathBuf::from(source));
        }
        plan.protected_paths.push(created.cache_root.clone());
        if let Some(name) = payload.get("display_name").and_then(|value| value.as_str()) {
            if install.is_none() {
                plan.display_name = name.to_owned();
            }
        }
        if plan.preserved_save_path.is_none() {
            if let Ok(documents) = crate::stage::SystemDocuments.documents_dir() {
                if prospective_path(&documents)?.starts_with(root) {
                    return Err(failure("Refusing the Documents folder or its parent."));
                }
                if let Ok(name) = crate::stage::read_engine_name(root.join("game"))
                    .or_else(|_| crate::stage::read_engine_name(root.join("bg1")))
                {
                    // Treat engine.lua text as untrusted: it must name one Documents child.
                    if Path::new(&name).components().count() != 1 || name.contains(['/', '\\']) {
                        return Err(failure("Invalid saved-game folder name."));
                    }
                    plan.preserved_save_path = Some(documents.join(name));
                }
            }
        }
    }
    if let Some(save) = &plan.preserved_save_path {
        plan.protected_paths.push(save.clone());
    }
    plan.protected_paths.sort();
    plan.protected_paths.dedup();
    for protected in &plan.protected_paths {
        let protected = prospective_path(protected)?;
        if overlap(root, &protected) {
            return Err(failure(format!(
                "Protected source, saves, cache or app-data overlaps {}",
                root.display()
            )));
        }
    }
    Ok(plan)
}

/// Revalidates the preview under the same cross-process lock used by installation.
pub fn execute(app_data: &Path, expected: &RemovalPlan) -> Result<(), RemovalError> {
    execute_with(app_data, expected, &SystemPreflight, &mut |_| Ok(()))
}

fn execute_with(
    app_data: &Path,
    expected: &RemovalPlan,
    host: &dyn PreflightHost,
    before_remove: &mut dyn FnMut(&Path) -> io::Result<()>,
) -> Result<(), RemovalError> {
    let _lock =
        TargetLock::try_acquire(app_data.join("locks"), &expected.managed_root).map_err(failure)?;
    let current = preview(app_data, &expected.install_id)?;
    if current != *expected {
        return Err(failure(
            "Installation changed since confirmation. Review removal again.",
        ));
    }
    if current.action == RemovalAction::Delete {
        recheck_staging_target_before_mutation_with(&current.managed_root, "en_US", host)
            .map_err(failure)?;
        validate_tree(&current.managed_root)?;
        // This small durable record is outside the deleted tree. It retains folder file identity,
        // source/cache exclusions and registry binding if deletion is interrupted or files lock.
        persist_journal(app_data, &current)?;
        remove_tree(&current.managed_root, before_remove)?;
    }
    ManagedInstallRegistry::open_or_create(app_data)
        .map_err(failure)?
        .forget_missing(&current.install_id, &current.managed_root)
        .map_err(failure)?;
    let journal = journal_path(app_data, &current.install_id);
    if journal.exists() {
        fs::remove_file(&journal).map_err(|e| io_at(&journal, e))?;
    }
    Ok(())
}

fn journal_path(app_data: &Path, id: &str) -> PathBuf {
    app_data
        .join("installation-removals")
        .join(format!("{id}.json"))
}

fn read_journal(path: &Path) -> Result<Option<RemovalPlan>, RemovalError> {
    direct_ancestors(path)?;
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_at(path, error)),
        Ok(metadata) if !metadata.is_file() || reparse(&metadata) => {
            Err(failure("Unsafe removal journal."))
        }
        Ok(_) => serde_json::from_slice(&fs::read(path).map_err(|e| io_at(path, e))?)
            .map(Some)
            .map_err(failure),
    }
}

fn persist_journal(app_data: &Path, plan: &RemovalPlan) -> Result<(), RemovalError> {
    let path = journal_path(app_data, &plan.install_id);
    if let Some(existing) = read_journal(&path)? {
        if existing != *plan {
            return Err(failure("Removal identity changed."));
        }
        return Ok(());
    }
    let parent = path.parent().unwrap();
    fs::create_dir_all(parent).map_err(|e| io_at(parent, e))?;
    direct_ancestors(parent)?;
    // Publish complete bytes atomically: a crash must never leave a half-written authority record.
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| io_at(parent, e))?;
    temp.write_all(&serde_json::to_vec(plan).map_err(failure)?)
        .map_err(|e| io_at(temp.path(), e))?;
    temp.as_file()
        .sync_all()
        .map_err(|e| io_at(temp.path(), e))?;
    temp.persist_noclobber(&path)
        .map_err(|e| io_at(&path, e.error))?;
    Ok(())
}

fn directory_identity(path: &Path) -> Result<u64, RemovalError> {
    let handle = same_file::Handle::from_path(path).map_err(|e| io_at(path, e))?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    handle.hash(&mut hasher);
    Ok(hasher.finish())
}

fn direct_ancestors(path: &Path) -> Result<(), RemovalError> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if reparse(&metadata) => {
                return Err(failure(format!(
                    "Refusing linked/reparse path {}",
                    ancestor.display()
                )))
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(io_at(ancestor, error)),
        }
    }
    Ok(())
}

fn reject_broad_root(root: &Path) -> Result<(), RemovalError> {
    if !root.is_absolute()
        || root.parent().is_none()
        || root
            .parent()
            .is_some_and(|parent| parent.parent().is_none())
        || root
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return Err(failure("Refusing broad or non-normalized removal target."));
    }
    for name in [
        "USERPROFILE",
        "HOME",
        "SystemRoot",
        "ProgramFiles",
        "ProgramFiles(x86)",
        "APPDATA",
        "LOCALAPPDATA",
    ] {
        if let Some(path) = std::env::var_os(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
        {
            let path = prospective_path(&path)?;
            if path.starts_with(root) {
                return Err(failure(
                    "Refusing home, system, or application-data folder.",
                ));
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir().and_then(fs::canonicalize) {
        if cwd.starts_with(root) {
            return Err(failure("Refusing current workspace or its parent."));
        }
    }
    #[cfg(windows)]
    {
        let text = root.to_string_lossy().replace(r"\\?\", "").to_lowercase();
        if text == r"c:\games" || crate::preflight::is_creator_protected_destination(root) {
            return Err(failure("Creator reference directories are read-only."));
        }
    }
    Ok(())
}

fn prospective_path(path: &Path) -> Result<PathBuf, RemovalError> {
    if !path.is_absolute() {
        return Err(failure("Protected path must be absolute."));
    }
    for ancestor in path.ancestors() {
        if ancestor.exists() {
            let base = fs::canonicalize(ancestor).map_err(|e| io_at(ancestor, e))?;
            return Ok(base.join(path.strip_prefix(ancestor).map_err(failure)?));
        }
    }
    Err(failure("Cannot resolve protected path."))
}

fn overlap(a: &Path, b: &Path) -> bool {
    #[cfg(windows)]
    {
        let a = a.to_string_lossy().to_lowercase();
        let b = b.to_string_lossy().to_lowercase();
        let a = Path::new(&a);
        let b = Path::new(&b);
        a.starts_with(b) || b.starts_with(a)
    }
    #[cfg(not(windows))]
    {
        a.starts_with(b) || b.starts_with(a)
    }
}

fn reparse(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0 || metadata.file_type().is_symlink()
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn validate_tree(root: &Path) -> Result<(), RemovalError> {
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(failure)?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(|e| io_at(entry.path(), e))?;
        if reparse(&metadata) || (!metadata.is_dir() && !metadata.is_file()) {
            return Err(failure(format!(
                "Refusing linked or special file {}",
                entry.path().display()
            )));
        }
    }
    Ok(())
}

fn remove_tree(
    path: &Path,
    before_remove: &mut dyn FnMut(&Path) -> io::Result<()>,
) -> Result<(), RemovalError> {
    direct_ancestors(path)?;
    let metadata = fs::symlink_metadata(path).map_err(|e| io_at(path, e))?;
    if reparse(&metadata) {
        return Err(failure(format!("Refusing linked path {}", path.display())));
    }
    if metadata.is_dir() {
        let mut entries = fs::read_dir(path)
            .map_err(|e| io_at(path, e))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<io::Result<Vec<_>>>()
            .map_err(|e| io_at(path, e))?;
        // Keep normal ownership evidence until game payload deletion has succeeded.
        entries.sort_by_key(|entry| {
            (
                entry.file_name().is_some_and(|name| name == ".chriz"),
                entry.clone(),
            )
        });
        for entry in entries {
            remove_tree(&entry, before_remove)?;
        }
        before_remove(path).map_err(|e| io_at(path, e))?;
        fs::remove_dir(path).map_err(|e| io_at(path, e))
    } else if metadata.is_file() {
        before_remove(path).map_err(|e| io_at(path, e))?;
        fs::remove_file(path).map_err(|e| io_at(path, e))
    } else {
        Err(failure("Refusing special file."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::{plan_digest, selection_digest};
    use crate::preflight::ExclusiveFileError;
    use crate::recipe_view::NormalizedSelection;
    use crate::resolve::InstallPlan;
    use crate::session::{CampaignCreated, SessionEvent, SourceGameFingerprints};
    use std::collections::BTreeMap;
    use std::ffi::OsString;

    #[derive(Default)]
    struct Host {
        running: Vec<PathBuf>,
    }
    impl PreflightHost for Host {
        fn available_space(&self, _: &Path) -> io::Result<u64> {
            Ok(u64::MAX)
        }
        fn volume_key(&self, _: &Path) -> io::Result<OsString> {
            Ok("fixture".into())
        }
        fn probe_directory_writable(&self, _: &Path) -> io::Result<()> {
            Ok(())
        }
        fn running_executable_paths(&self) -> io::Result<Vec<PathBuf>> {
            Ok(self.running.clone())
        }
        fn probe_exclusive_writable_files(&self, _: &[PathBuf]) -> Result<(), ExclusiveFileError> {
            Ok(())
        }
    }
    struct Fixture {
        _temp: tempfile::TempDir,
        app: PathBuf,
        root: PathBuf,
        cache: PathBuf,
        source: PathBuf,
    }
    fn fixture() -> Fixture {
        let temp = tempfile::tempdir().unwrap();
        let base = fs::canonicalize(temp.path()).unwrap();
        let root = base.join("managed");
        let cache = base.join("cache");
        let source = base.join("source");
        let app = base.join("app-data");
        for path in [&root, &cache, &source] {
            fs::create_dir(path).unwrap();
        }
        fs::write(source.join("original.txt"), "source game").unwrap();
        fs::write(cache.join("archive.zip"), "shared cache").unwrap();
        fs::create_dir(root.join("game")).unwrap();
        fs::write(root.join("game/game.exe"), "fixture").unwrap();
        let payload = serde_json::to_vec(&serde_json::json!({"display_name":"Removal fixture", "bg1":{"root":source},"bg2":{"root":source}})).unwrap();
        let selection = NormalizedSelection {
            platform: "windows".into(),
            features: BTreeMap::new(),
            inputs: BTreeMap::new(),
        };
        let created = CampaignCreated {
            install_id: "fixture-removal".into(),
            attempt_id: "attempt-fixture".into(),
            managed_root: root.clone(),
            cache_root: cache.clone(),
            recipe_payload_sha256: sha256_bytes(&payload),
            recipe_payload: payload,
            recipe_envelope: vec![],
            recipe_envelope_sha256: sha256_bytes(&[]),
            selection_sha256: selection_digest(&selection).unwrap(),
            normalized_selection: selection,
            plan_sha256: plan_digest(&InstallPlan { runs: vec![] }).unwrap(),
            source_games: SourceGameFingerprints {
                bg1: "11".repeat(32),
                bg2: "22".repeat(32),
            },
            artifact_identities: vec![],
            tool_identities: vec![],
            staged_bg1: root.join("bg1"),
            staged_bg2: root.join("game"),
        };
        SessionStore::create(&root, SessionEvent::Created(Box::new(created.clone()))).unwrap();
        ManagedInstallRegistry::open_or_create(&app)
            .unwrap()
            .publish_campaign(&created)
            .unwrap();
        Fixture {
            _temp: temp,
            app,
            root,
            cache,
            source,
        }
    }
    fn run(f: &Fixture, plan: &RemovalPlan) -> Result<(), RemovalError> {
        execute_with(&f.app, plan, &Host::default(), &mut |_| Ok(()))
    }
    #[test]
    fn preview_and_cancel_do_not_remove_anything() {
        let f = fixture();
        let plan = preview(&f.app, "fixture-removal").unwrap();
        assert_eq!(plan.action, RemovalAction::Delete);
        assert!(f.root.join("game/game.exe").exists());
        assert!(!journal_path(&f.app, "fixture-removal").exists());
    }
    #[test]
    fn deletes_only_registered_copy_and_preserves_source_cache_and_saves() {
        let f = fixture();
        let saves = f._temp.path().join("Documents/save");
        fs::create_dir_all(&saves).unwrap();
        fs::write(saves.join("saved.gam"), "save").unwrap();
        let registry = ManagedInstallRegistry::open_or_create(&f.app).unwrap();
        let campaign = registry.list_campaigns().unwrap().remove(0).record;
        fs::write(
            f.root.join(".chriz/install-receipt.json"),
            "completed receipt",
        )
        .unwrap();
        registry
            .publish(&crate::registry::ManagedInstallRecord {
                schema_version: crate::registry::REGISTRY_SCHEMA_VERSION,
                install_id: campaign.install_id,
                display_name: "Completed fixture".into(),
                managed_root: f.root.clone(),
                recipe_version: "1.0.0".into(),
                recipe_sha256: campaign.recipe_sha256,
                engine_name: "fixture".into(),
                managed_save_root: saves.canonicalize().unwrap(),
                launch_path: f.root.join("game/game.exe"),
                receipt_sha256: sha256_bytes(b"completed receipt"),
                completed_at_millis: 1,
            })
            .unwrap();
        assert_eq!(
            preview(&f.app, "fixture-removal")
                .unwrap()
                .preserved_save_path,
            Some(saves.canonicalize().unwrap())
        );
        run(&f, &preview(&f.app, "fixture-removal").unwrap()).unwrap();
        assert!(!f.root.exists());
        assert!(saves.join("saved.gam").exists());
        assert!(f.source.join("original.txt").exists());
        assert!(f.cache.join("archive.zip").exists());
        assert!(ManagedInstallRegistry::open_or_create(&f.app)
            .unwrap()
            .list_campaigns()
            .unwrap()
            .is_empty());
        assert!(registry.list().unwrap().is_empty());
    }
    #[test]
    fn unknown_and_forged_identity_are_refused() {
        let f = fixture();
        assert!(preview(&f.app, "../source").is_err());
        let mut plan = preview(&f.app, "fixture-removal").unwrap();
        plan.managed_root = f.source.clone();
        assert!(run(&f, &plan).is_err());
        assert!(f.source.join("original.txt").exists());
    }
    #[test]
    fn missing_root_can_only_forget_and_reappearing_root_invalidates_confirmation() {
        let f = fixture();
        let moved = f._temp.path().join("moved");
        fs::rename(&f.root, &moved).unwrap();
        let plan = preview(&f.app, "fixture-removal").unwrap();
        assert_eq!(plan.action, RemovalAction::Forget);
        fs::create_dir(&f.root).unwrap();
        assert!(run(&f, &plan).is_err());
        fs::remove_dir(&f.root).unwrap();
        run(&f, &plan).unwrap();
        assert!(moved.join("game/game.exe").exists());
    }
    #[test]
    fn replacement_root_is_not_owned() {
        let f = fixture();
        let plan = preview(&f.app, "fixture-removal").unwrap();
        fs::rename(&f.root, f._temp.path().join("moved")).unwrap();
        fs::create_dir(&f.root).unwrap();
        fs::write(f.root.join("personal.txt"), "not ours").unwrap();
        assert!(run(&f, &plan).is_err());
        assert!(f.root.join("personal.txt").exists());
    }
    #[test]
    fn active_target_lock_and_process_refuse_deletion() {
        let f = fixture();
        let plan = preview(&f.app, "fixture-removal").unwrap();
        let lock = TargetLock::try_acquire(f.app.join("locks"), &f.root).unwrap();
        assert!(run(&f, &plan).is_err());
        drop(lock);
        let host = Host {
            running: vec![f.root.join("game/game.exe")],
        };
        assert!(execute_with(&f.app, &plan, &host, &mut |_| Ok(())).is_err());
        assert!(f.root.join("game/game.exe").exists());
    }
    #[test]
    fn interrupted_final_metadata_cleanup_is_retryable_without_losing_registry_identity() {
        let f = fixture();
        let plan = preview(&f.app, "fixture-removal").unwrap();
        let error = execute_with(&f.app, &plan, &Host::default(), &mut |path| {
            if path == f.root {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "fixture locked folder",
                ))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert!(error.to_string().contains("fixture locked folder"));
        assert!(!f.root.join(".chriz").exists());
        assert!(f
            .app
            .join("managed-campaigns/fixture-removal.json")
            .exists());
        let retry = preview(&f.app, "fixture-removal").unwrap();
        run(&f, &retry).unwrap();
        assert!(!f.root.exists());
    }
    #[test]
    fn partial_deletion_journal_cannot_authorize_replacement_folder() {
        let f = fixture();
        let plan = preview(&f.app, "fixture-removal").unwrap();
        execute_with(&f.app, &plan, &Host::default(), &mut |_| {
            Err(io::Error::other("stop"))
        })
        .unwrap_err();
        fs::rename(&f.root, f._temp.path().join("moved")).unwrap();
        fs::create_dir(&f.root).unwrap();
        assert!(preview(&f.app, "fixture-removal").is_err());
    }
    #[test]
    fn broad_roots_and_source_overlap_are_rejected() {
        let f = fixture();
        assert!(reject_broad_root(f.root.ancestors().last().unwrap()).is_err());
        let mut plan = preview(&f.app, "fixture-removal").unwrap();
        plan.protected_paths.push(f.root.join("game"));
        persist_journal(&f.app, &plan).unwrap();
        assert!(preview(&f.app, "fixture-removal").is_err());
    }
    #[test]
    fn nested_reparse_point_is_refused_without_touching_its_target() {
        let f = fixture();
        #[cfg(windows)]
        {
            // A directory junction needs no symlink privilege. Both paths are disposable fixtures.
            let output = std::process::Command::new("cmd.exe")
                .args(["/c", "mklink", "/J"])
                .arg(f.root.join("linked-source"))
                .arg(&f.source)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(&f.source, f.root.join("linked-source")).unwrap();
        let plan = preview(&f.app, "fixture-removal").unwrap();
        assert!(run(&f, &plan).is_err());
        assert!(f.source.join("original.txt").exists());
    }
}

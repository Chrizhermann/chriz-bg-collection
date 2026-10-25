use std::env;
use std::fs;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

use bg_engine::lock::{
    acquire_target_for_update, CacheDigestLock, LockError, TargetLock, TargetLockNamespace,
};

const CHILD_ROOT: &str = "CHRIZ_TEST_LOCK_CHILD_ROOT";
const CHILD_REGISTRY: &str = "CHRIZ_TEST_LOCK_CHILD_REGISTRY";
const CHILD_READY: &str = "CHRIZ_TEST_LOCK_CHILD_READY";
const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn managed_root(temp: &tempfile::TempDir) -> std::path::PathBuf {
    let root = temp.path().join("managed");
    fs::create_dir_all(&root).unwrap();
    root
}

fn assert_contended(error: LockError) {
    assert!(matches!(error, LockError::Contended { .. }), "{error}");
}

#[test]
fn target_lock_is_an_os_lock_and_not_a_stale_sentinel() {
    let temp = tempfile::tempdir().unwrap();
    let root = managed_root(&temp);
    let registry = temp.path().join("lock-registry");

    let first = TargetLock::try_acquire(&registry, &root).unwrap();
    assert_contended(TargetLock::try_acquire(&registry, &root).unwrap_err());
    assert_contended(acquire_target_for_update(&registry, &root).unwrap_err());

    drop(first);
    assert!(TargetLock::try_acquire(&registry, &root).is_ok());
    let _update_guard = acquire_target_for_update(&registry, &root).unwrap();
}

#[test]
fn target_lock_canonicalizes_aliases_without_creating_the_target() {
    let temp = tempfile::tempdir().unwrap();
    let registry = temp.path().join("lock-registry");
    let target = temp.path().join("new-managed-target");
    let alias_parent = temp.path().join("alias-parent");
    fs::create_dir(&alias_parent).unwrap();
    let alias = alias_parent.join("..").join("new-managed-target");

    let first = TargetLock::try_acquire(&registry, &target).unwrap();
    assert!(
        !target.exists(),
        "locking must not claim or mutate the target"
    );
    assert_contended(TargetLock::try_acquire(&registry, &alias).unwrap_err());

    drop(first);
    assert!(TargetLock::try_acquire(&registry, &alias).is_ok());
}

#[test]
fn target_lock_identity_is_stable_when_a_missing_target_is_created() {
    let temp = tempfile::tempdir().unwrap();
    let registry = temp.path().join("registry");
    let target = temp.path().join("managed");
    std::fs::create_dir(&registry).unwrap();

    let first = TargetLock::try_acquire(&registry, &target).unwrap();
    std::fs::create_dir(&target).unwrap();

    assert_contended(TargetLock::try_acquire(&registry, &target).unwrap_err());
    drop(first);
    assert!(TargetLock::try_acquire(&registry, &target).is_ok());
}

#[test]
fn cache_locks_are_exclusive_per_valid_digest() {
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("cache");

    let first = CacheDigestLock::try_acquire(&cache, DIGEST_A).unwrap();
    assert_contended(CacheDigestLock::try_acquire(&cache, DIGEST_A).unwrap_err());

    let other_digest = CacheDigestLock::try_acquire(&cache, DIGEST_B).unwrap();
    drop(other_digest);
    drop(first);

    assert!(CacheDigestLock::try_acquire(&cache, DIGEST_A).is_ok());
    assert!(matches!(
        CacheDigestLock::try_acquire(&cache, "../escape").unwrap_err(),
        LockError::InvalidDigest { .. }
    ));
}

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn target_lock_is_released_when_holder_process_is_terminated() {
    let temp = tempfile::tempdir().unwrap();
    let root = managed_root(&temp);
    let registry = temp.path().join("lock-registry");
    let ready = temp.path().join("child-ready");

    let child = Command::new(env::current_exe().unwrap())
        .args(["--exact", "lock_holder_child", "--nocapture"])
        .env(CHILD_ROOT, &root)
        .env(CHILD_REGISTRY, &registry)
        .env(CHILD_READY, &ready)
        .spawn()
        .unwrap();
    let mut child = ChildGuard(child);

    let deadline = Instant::now() + Duration::from_secs(10);
    while !ready.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(ready.exists(), "child never reported that it held the lock");
    assert_contended(TargetLock::try_acquire(&registry, &root).unwrap_err());

    child.0.kill().unwrap();
    child.0.wait().unwrap();
    assert!(TargetLock::try_acquire(&registry, &root).is_ok());
}

#[test]
fn lock_holder_child() {
    let Some(root) = env::var_os(CHILD_ROOT) else {
        return;
    };
    let registry = env::var_os(CHILD_REGISTRY).expect("child registry path");
    let ready = env::var_os(CHILD_READY).expect("child ready path");
    let _lock = TargetLock::try_acquire(registry, root).expect("child target lock");
    fs::write(ready, b"ready").expect("write ready marker");
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

/// Every application locks a managed root in `<local>\Chriz BG Collection\locks`, the exact
/// folder older builds use, whatever folder it keeps its registry in: the namespace is chosen
/// from the local application data folder alone, never from a registry path.
#[test]
fn target_locks_live_in_one_namespace_whatever_the_registry_path() {
    let temp = tempfile::tempdir().unwrap();
    let root = managed_root(&temp);
    let local = temp.path().join("Local");
    let locks = TargetLockNamespace::in_local_application_data(&local).unwrap();
    let legacy = local.join("Chriz BG Collection").join("locks");
    assert_eq!(locks.root(), legacy);

    let held = locks.try_acquire(&root).unwrap();
    assert!(held.path().starts_with(legacy.join("targets")));
    // An older build locks the legacy folder directly; it is the same lock file.
    assert!(matches!(
        TargetLock::try_acquire(&legacy, &root),
        Err(LockError::Contended { kind: "target", .. })
    ));
    let same = TargetLockNamespace::in_local_application_data(&local).unwrap();
    assert!(matches!(
        same.try_acquire(&root),
        Err(LockError::Contended { kind: "target", .. })
    ));
    drop(held);
    TargetLock::try_acquire(&legacy, &root).unwrap();
}

/// A relative, parent-relative or root-level local folder would give different processes
/// different lock files, or put locks at a drive root, so it is refused instead of falling back.
#[test]
fn target_lock_namespace_refuses_relative_unnormalized_and_root_folders() {
    let temp = tempfile::tempdir().unwrap();
    let drive_root = temp.path().ancestors().last().unwrap().to_path_buf();
    for local in [
        std::path::PathBuf::from("IWD"),
        std::path::PathBuf::from(r"Local\Programs"),
        std::path::PathBuf::from(""),
        drive_root,
        temp.path().join("Local").join("..").join("Other"),
    ] {
        assert!(
            matches!(
                TargetLockNamespace::in_local_application_data(&local),
                Err(LockError::UnsafePath { .. })
            ),
            "{} was accepted",
            local.display()
        );
    }
}

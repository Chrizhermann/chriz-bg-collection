//! Renames that outlast another program's brief hold on a file.
//!
//! On Windows a replacing rename fails while another process has the source or the target open
//! without delete sharing, and a folder rename fails while any file inside it is open at all.
//! Real-time antivirus and indexing services open files like that for a moment, typically right
//! after the engine wrote or read them, so these errors pass on their own. The functions here
//! retry them a bounded number of times and otherwise behave like the rename they wrap. The
//! `persist` functions publish a `tempfile` temporary the same way.

use std::convert::Infallible;
use std::fs;
use std::io;
use std::path::Path;
use std::thread;
use std::time::Duration;

use tempfile::NamedTempFile;

/// How many attempts one operation gets and how long it waits between them.
#[derive(Clone, Copy, Debug)]
struct Backoff {
    /// Attempts in total, the first one included.
    attempts: u32,
    /// Wait after the first failed attempt; every further wait doubles it.
    first_wait: Duration,
    /// Upper bound for a single wait.
    longest_wait: Duration,
}

/// Waits of 10 ms doubling up to 1 s, about 5.3 s in all: long enough for a scanner to finish a
/// large file, short enough that a lasting denial such as a read-only target surfaces quickly.
const BACKOFF: Backoff = Backoff {
    attempts: 12,
    first_wait: Duration::from_millis(10),
    longest_wait: Duration::from_secs(1),
};

/// Why a checked rename stopped.
#[derive(Debug)]
pub(crate) enum CheckedRenameError<E> {
    /// The check before an attempt failed, so that attempt never ran.
    Check(E),
    /// The rename failed: at once for a lasting error, or with the last attempt's error.
    Rename(io::Error),
}

/// Rename `from` to `to` like [`fs::rename`], replacing a file at `to`, and retry while another
/// program briefly holds either path. The rename's own error is returned unchanged.
pub(crate) fn rename(from: &Path, to: &Path) -> io::Result<()> {
    retry_transient(|| fs::rename(from, to))
}

/// [`rename`], with `check` run right before every attempt, retries included; a failed check
/// stops at once and its error is returned.
pub(crate) fn rename_checked<E>(
    from: &Path,
    to: &Path,
    check: impl FnMut() -> Result<(), E>,
) -> Result<(), CheckedRenameError<E>> {
    attempt(BACKOFF, &mut pause, check, || fs::rename(from, to))
}

/// Retry another rename primitive, such as a write-through `MoveFileExW`, the same way.
pub(crate) fn retry_transient(operation: impl FnMut() -> io::Result<()>) -> io::Result<()> {
    match attempt(BACKOFF, &mut pause, || Ok::<(), Infallible>(()), operation) {
        Ok(()) => Ok(()),
        Err(CheckedRenameError::Rename(error)) => Err(error),
        Err(CheckedRenameError::Check(never)) => match never {},
    }
}

/// Publish `temporary` at `to`, replacing a file there, like [`NamedTempFile::persist`]. It is
/// renamed with [`rename`]: the `MoveFileExW` that `persist` uses fails even while the holder of
/// the target shares delete access. The temporary is removed again when publication fails.
pub(crate) fn persist(temporary: NamedTempFile, to: &Path) -> io::Result<()> {
    match persist_checked(temporary, to, || Ok::<(), Infallible>(())) {
        Ok(()) => Ok(()),
        Err(CheckedRenameError::Rename(error)) => Err(error),
        Err(CheckedRenameError::Check(never)) => match never {},
    }
}

/// [`persist`], with `check` run right before every attempt, retries included; a failed check
/// stops at once, removes the temporary and returns its error.
pub(crate) fn persist_checked<E>(
    temporary: NamedTempFile,
    to: &Path,
    check: impl FnMut() -> Result<(), E>,
) -> Result<(), CheckedRenameError<E>> {
    // `keep` clears the temporary attribute, as `persist` does. Our own handle is closed first,
    // so only another program's hold can get in the rename's way.
    let (file, temporary) = temporary
        .keep()
        .map_err(|error| CheckedRenameError::Rename(error.error))?;
    drop(file);
    let published = rename_checked(&temporary, to, check);
    if published.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    published
}

/// Publish `temporary` at `to` like [`NamedTempFile::persist_noclobber`], retried the same way.
/// Every attempt is itself a no-clobber publish, so an existing `to` is never replaced: it fails
/// the first attempt with its own error, which is not retried. A temporary that could not be
/// published is removed when it is dropped, as before.
pub(crate) fn persist_noclobber(temporary: NamedTempFile, to: &Path) -> io::Result<()> {
    let mut pending = Some(temporary);
    retry_transient(|| {
        let Some(temporary) = pending.take() else {
            return Err(io::Error::other("the temporary file was already published"));
        };
        match temporary.persist_noclobber(to) {
            Ok(_) => Ok(()),
            Err(error) => {
                pending = Some(error.file);
                Err(error.error)
            }
        }
    })
}

/// The wait between two attempts. Tests can step in at every wait, for example to let go of a
/// hold they set up, so that no outcome depends on timing.
fn pause(duration: Duration) {
    #[cfg(test)]
    test_hook::at_wait(duration);
    thread::sleep(duration);
}

#[cfg(test)]
pub(crate) mod test_hook {
    use std::cell::RefCell;
    use std::time::Duration;

    type Hook = Box<dyn FnMut(Duration)>;

    thread_local! {
        static AT_WAIT: RefCell<Option<Hook>> = const { RefCell::new(None) };
    }

    /// Run `body` with `hook` called at every wait of a retried operation on this thread, right
    /// before the wait itself. The hook is dropped when `body` returns or panics.
    pub(crate) fn with_wait_hook<R>(
        hook: impl FnMut(Duration) + 'static,
        body: impl FnOnce() -> R,
    ) -> R {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                let hook = AT_WAIT.with(|slot| slot.borrow_mut().take());
                drop(hook);
            }
        }

        AT_WAIT.with(|slot| *slot.borrow_mut() = Some(Box::new(hook)));
        let _reset = Reset;
        body()
    }

    pub(super) fn at_wait(duration: Duration) {
        AT_WAIT.with(|slot| {
            if let Some(hook) = slot.borrow_mut().as_mut() {
                hook(duration);
            }
        });
    }

    /// Opens `path` the way a scanner can: readable by others, but not renamable or deletable.
    #[cfg(windows)]
    pub(crate) fn hold_without_delete_sharing(path: &std::path::Path) -> std::fs::File {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE};

        std::fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .open(path)
            .unwrap()
    }

    /// Run `body` while `path` is held without delete sharing, and let go at the first wait of
    /// a retried operation in it. Returns `body`'s result and how many waits there were: at
    /// least one when the hold made an attempt fail. Later waits really sleep, so a scanner that
    /// briefly holds the fresh test files is outlasted as well.
    #[cfg(windows)]
    pub(crate) fn hold_until_first_wait<R>(
        path: &std::path::Path,
        body: impl FnOnce() -> R,
    ) -> (R, usize) {
        use std::cell::Cell;
        use std::rc::Rc;

        let mut holder = Some(hold_without_delete_sharing(path));
        let waits = Rc::new(Cell::new(0));
        let counted = Rc::clone(&waits);
        let result = with_wait_hook(
            move |_| {
                counted.set(counted.get() + 1);
                drop(holder.take());
            },
            body,
        );
        (result, waits.get())
    }
}

fn attempt<E>(
    backoff: Backoff,
    wait: &mut dyn FnMut(Duration),
    mut check: impl FnMut() -> Result<(), E>,
    mut operation: impl FnMut() -> io::Result<()>,
) -> Result<(), CheckedRenameError<E>> {
    let mut next_wait = backoff.first_wait;
    let mut attempt = 1;
    loop {
        check().map_err(CheckedRenameError::Check)?;
        match operation() {
            Ok(()) => return Ok(()),
            Err(error) if attempt < backoff.attempts && is_transient(&error) => {
                wait(next_wait);
                next_wait = next_wait.saturating_mul(2).min(backoff.longest_wait);
                attempt += 1;
            }
            Err(error) => return Err(CheckedRenameError::Rename(error)),
        }
    }
}

/// Errors Windows reports while another program has a file open in a conflicting way. Access
/// denied also covers lasting causes, such as a read-only target or a folder in the way; those
/// fail with the same error once the attempts run out.
#[cfg(windows)]
fn is_transient(error: &io::Error) -> bool {
    use windows_sys::Win32::Foundation::{
        ERROR_ACCESS_DENIED, ERROR_LOCK_VIOLATION, ERROR_SHARING_VIOLATION,
    };

    error
        .raw_os_error()
        .and_then(|code| u32::try_from(code).ok())
        .is_some_and(|code| {
            matches!(
                code,
                ERROR_ACCESS_DENIED | ERROR_SHARING_VIOLATION | ERROR_LOCK_VIOLATION
            )
        })
}

#[cfg(not(windows))]
fn is_transient(_error: &io::Error) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    #[cfg(windows)]
    use windows_sys::Win32::Foundation::{
        ERROR_ACCESS_DENIED, ERROR_LOCK_VIOLATION, ERROR_SHARING_VIOLATION,
    };

    use super::*;
    #[cfg(windows)]
    use test_hook::{hold_until_first_wait, hold_without_delete_sharing};

    const SHORT: Backoff = Backoff {
        attempts: 4,
        first_wait: Duration::from_millis(10),
        longest_wait: Duration::from_millis(25),
    };

    fn no_check() -> Result<(), Infallible> {
        Ok(())
    }

    #[cfg(windows)]
    fn source_and_target(temp: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
        let source = temp.path().join(".data.txt.new.tmp");
        let target = temp.path().join("data.txt");
        fs::write(&source, b"new").unwrap();
        fs::write(&target, b"old").unwrap();
        (source, target)
    }

    #[cfg(windows)]
    fn is_transient_code(code: &Option<i32>) -> bool {
        [
            ERROR_ACCESS_DENIED,
            ERROR_SHARING_VIOLATION,
            ERROR_LOCK_VIOLATION,
        ]
        .map(|transient| i32::try_from(transient).ok())
        .contains(code)
    }

    /// Renames while one of the two files is held and lets go at the first wait. Later waits
    /// really sleep, so a scanner that briefly holds the fresh test files is outlasted as well.
    /// Returns the error code of every failed attempt.
    #[cfg(windows)]
    fn rename_once_the_holder_lets_go(hold_target: bool) -> Vec<Option<i32>> {
        let temp = TempDir::new().unwrap();
        let (source, target) = source_and_target(&temp);
        let mut holder = Some(hold_without_delete_sharing(if hold_target {
            &target
        } else {
            &source
        }));
        let mut waits = Vec::new();
        let mut errors = Vec::new();

        attempt(
            BACKOFF,
            &mut |wait| {
                waits.push(wait);
                holder = None;
                thread::sleep(wait);
            },
            no_check,
            || fs::rename(&source, &target).inspect_err(|error| errors.push(error.raw_os_error())),
        )
        .unwrap();

        assert_eq!(fs::read(&target).unwrap(), b"new");
        assert!(!source.exists());
        assert_eq!(waits.len(), errors.len());
        assert_eq!(waits.first(), Some(&BACKOFF.first_wait));
        assert!(errors.iter().all(is_transient_code), "{errors:?}");
        errors
    }

    #[cfg(windows)]
    #[test]
    fn a_held_target_is_replaced_once_the_holder_lets_go_within_the_retries() {
        let errors = rename_once_the_holder_lets_go(true);

        // Access denied, as in #34, unless something else held the source at that moment.
        assert!(!errors.is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn a_held_source_is_moved_once_the_holder_lets_go_within_the_retries() {
        let errors = rename_once_the_holder_lets_go(false);

        // The source is opened first, so holding it always shows as a sharing violation.
        assert_eq!(
            errors.first(),
            Some(&i32::try_from(ERROR_SHARING_VIOLATION).ok())
        );
    }

    #[cfg(windows)]
    #[test]
    fn a_target_held_past_the_last_attempt_fails_with_the_rename_error() {
        let temp = TempDir::new().unwrap();
        let (source, target) = source_and_target(&temp);
        let _holder = hold_without_delete_sharing(&target);
        let mut waits = Vec::new();
        let mut errors = Vec::new();

        let result = attempt(SHORT, &mut |wait| waits.push(wait), no_check, || {
            fs::rename(&source, &target).inspect_err(|error| errors.push(error.raw_os_error()))
        });

        // Access denied each time, as in #34; whatever the last attempt saw comes back unchanged.
        match result {
            Err(CheckedRenameError::Rename(error)) => {
                assert_eq!(Some(error.raw_os_error()), errors.last().copied());
            }
            other => panic!("expected the last rename error, got {other:?}"),
        }
        assert_eq!(errors.len(), 4);
        assert!(errors.iter().all(is_transient_code), "{errors:?}");
        assert_eq!(
            waits,
            [
                Duration::from_millis(10),
                Duration::from_millis(20),
                Duration::from_millis(25),
            ]
        );
        assert_eq!(fs::read(&target).unwrap(), b"old");
        assert_eq!(fs::read(&source).unwrap(), b"new");
    }

    #[test]
    fn a_lasting_error_is_returned_without_another_attempt() {
        let temp = TempDir::new().unwrap();
        let missing = temp.path().join("missing.tmp");
        let target = temp.path().join("data.txt");
        let mut waits = Vec::new();
        let mut attempts = 0;

        let result = attempt(SHORT, &mut |wait| waits.push(wait), no_check, || {
            attempts += 1;
            fs::rename(&missing, &target)
        });

        match result {
            Err(CheckedRenameError::Rename(error)) => {
                assert_eq!(error.kind(), io::ErrorKind::NotFound);
            }
            other => panic!("expected the rename error, got {other:?}"),
        }
        assert_eq!(attempts, 1);
        assert!(waits.is_empty());
        assert!(!target.exists());
    }

    #[cfg(windows)]
    #[test]
    fn the_check_runs_before_every_attempt_and_a_failed_check_stops_the_rename() {
        let temp = TempDir::new().unwrap();
        let (source, target) = source_and_target(&temp);
        let _holder = hold_without_delete_sharing(&target);
        let mut checks = 0;
        let mut attempts = 0;

        let result = attempt(
            SHORT,
            &mut |_| {},
            || {
                checks += 1;
                if checks == 3 {
                    Err("target changed")
                } else {
                    Ok(())
                }
            },
            || {
                attempts += 1;
                fs::rename(&source, &target)
            },
        );

        assert!(matches!(
            result,
            Err(CheckedRenameError::Check("target changed"))
        ));
        assert_eq!((checks, attempts), (3, 2));
        assert_eq!(fs::read(&target).unwrap(), b"old");
        assert_eq!(fs::read(&source).unwrap(), b"new");
    }

    #[cfg(windows)]
    #[test]
    fn only_conflicting_open_errors_count_as_transient() {
        use windows_sys::Win32::Foundation::{
            ERROR_ALREADY_EXISTS, ERROR_DIR_NOT_EMPTY, ERROR_FILE_NOT_FOUND,
        };

        let windows_error = |code: u32| io::Error::from_raw_os_error(i32::try_from(code).unwrap());
        for code in [
            ERROR_ACCESS_DENIED,
            ERROR_SHARING_VIOLATION,
            ERROR_LOCK_VIOLATION,
        ] {
            assert!(is_transient(&windows_error(code)), "{code}");
        }
        for code in [
            ERROR_FILE_NOT_FOUND,
            ERROR_ALREADY_EXISTS,
            ERROR_DIR_NOT_EMPTY,
        ] {
            assert!(!is_transient(&windows_error(code)), "{code}");
        }
        assert!(!is_transient(&io::Error::other("not an OS error")));
    }

    #[cfg(windows)]
    #[test]
    fn the_default_backoff_gives_up_after_twelve_attempts_and_about_five_seconds() {
        let mut waits = Vec::new();
        let mut attempts = 0;

        let result = attempt(BACKOFF, &mut |wait| waits.push(wait), no_check, || {
            attempts += 1;
            Err(io::Error::from_raw_os_error(
                i32::try_from(ERROR_SHARING_VIOLATION).unwrap(),
            ))
        });

        assert!(matches!(result, Err(CheckedRenameError::Rename(_))));
        assert_eq!(attempts, 12);
        assert_eq!(
            waits.iter().sum::<Duration>(),
            Duration::from_millis(10 + 20 + 40 + 80 + 160 + 320 + 640 + 4 * 1000)
        );
    }

    #[cfg(windows)]
    fn names(temp: &TempDir) -> Vec<std::ffi::OsString> {
        let mut names = fs::read_dir(temp.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    #[cfg(windows)]
    fn temporary_with(temp: &TempDir, content: &[u8]) -> NamedTempFile {
        use std::io::Write;

        let mut temporary = NamedTempFile::new_in(temp.path()).unwrap();
        temporary.write_all(content).unwrap();
        temporary
    }

    #[cfg(windows)]
    #[test]
    fn a_persisted_temporary_replaces_a_held_target_once_the_holder_lets_go() {
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_TEMPORARY;

        let temp = TempDir::new().unwrap();
        let target = temp.path().join("data.txt");
        fs::write(&target, b"old").unwrap();
        let temporary = temporary_with(&temp, b"new");

        let (result, waits) = hold_until_first_wait(&target, || persist(temporary, &target));

        result.unwrap();
        // Access denied while held, as in #34.
        assert!(waits >= 1);
        assert_eq!(fs::read(&target).unwrap(), b"new");
        assert_eq!(names(&temp), ["data.txt"]);
        let attributes = fs::metadata(&target).unwrap().file_attributes();
        assert_eq!(attributes & FILE_ATTRIBUTE_TEMPORARY, 0);
    }

    #[cfg(windows)]
    #[test]
    fn a_failed_check_removes_the_temporary_and_keeps_the_target() {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("data.txt");
        fs::write(&target, b"old").unwrap();
        let temporary = temporary_with(&temp, b"new");
        let mut checks = 0;

        let (result, waits) = hold_until_first_wait(&target, || {
            persist_checked(temporary, &target, || {
                checks += 1;
                if checks == 1 {
                    Ok(())
                } else {
                    Err("target changed")
                }
            })
        });

        assert!(matches!(
            result,
            Err(CheckedRenameError::Check("target changed"))
        ));
        assert_eq!((checks, waits), (2, 1));
        assert_eq!(fs::read(&target).unwrap(), b"old");
        assert_eq!(names(&temp), ["data.txt"]);
    }

    #[cfg(windows)]
    #[test]
    fn a_no_clobber_publish_outlasts_a_held_source() {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("record.json");
        let temporary = temporary_with(&temp, b"record");
        let source = temporary.path().to_path_buf();

        let (result, waits) =
            hold_until_first_wait(&source, || persist_noclobber(temporary, &target));

        result.unwrap();
        // The held source fails the move with a sharing violation until it is let go.
        assert!(waits >= 1);
        assert_eq!(fs::read(&target).unwrap(), b"record");
        assert_eq!(names(&temp), ["record.json"]);
    }

    #[cfg(windows)]
    #[test]
    fn a_no_clobber_publish_never_replaces_an_existing_target_even_while_it_is_held() {
        let temp = TempDir::new().unwrap();
        let target = temp.path().join("record.json");
        fs::write(&target, b"earlier record").unwrap();
        let temporary = temporary_with(&temp, b"record");

        let (result, waits) =
            hold_until_first_wait(&target, || persist_noclobber(temporary, &target));

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        // An existing target is a lasting error: no retry, and the temporary is gone.
        assert_eq!(waits, 0);
        assert_eq!(fs::read(&target).unwrap(), b"earlier record");
        assert_eq!(names(&temp), ["record.json"]);
    }
}

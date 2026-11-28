use std::io;
use std::process::{Child, Command};
#[cfg(unix)]
use std::sync::atomic::{AtomicI32, Ordering};

/// Configures process-group isolation on `command` before spawning.
pub(super) fn configure_command(command: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        // Place WeiDU into its own process group (pgid = pid) so cancellation or drop can
        // terminate the entire subtree without signalling CEBG itself.
        command.process_group(0);
    }
    #[cfg(not(unix))]
    {
        let _ = command;
    }
}

#[cfg(windows)]
pub(super) struct ProcessGroup {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl ProcessGroup {
    pub(super) fn new() -> io::Result<Self> {
        use std::ffi::c_void;
        use std::mem::{size_of, zeroed};
        use std::ptr;
        use windows_sys::Win32::System::JobObjects::{
            JobObjectExtendedLimitInformation, SetInformationJobObject,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        };

        #[link(name = "kernel32")]
        extern "system" {
            fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> *mut c_void;
        }

        // SAFETY: null attributes/name request a private unnamed Job Object; the returned handle
        // is checked and owned by `ProcessGroup` until Drop closes it.
        let handle = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }

        // SAFETY: this Windows POD is valid when zero-initialized and the API receives its exact
        // pointer and byte size for `JobObjectExtendedLimitInformation`.
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: `handle` is live and `limits` points to a correctly sized value for this class.
        let configured = unsafe {
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if configured == 0 {
            let error = io::Error::last_os_error();
            // SAFETY: `handle` is the live handle returned above and is closed exactly once here.
            unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
            return Err(error);
        }
        Ok(Self { handle })
    }

    pub(super) fn attach(&self, child: &Child) -> io::Result<()> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;

        // SAFETY: both handles are live for the duration of the call. Ownership is unchanged.
        if unsafe { AssignProcessToJobObject(self.handle, child.as_raw_handle()) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub(super) fn poll_exit(
        &self,
        child: &mut Child,
    ) -> io::Result<Option<std::process::ExitStatus>> {
        child.try_wait()
    }

    pub(super) fn terminate(&self, _child: &mut Child) -> io::Result<()> {
        use windows_sys::Win32::System::JobObjects::TerminateJobObject;

        // SAFETY: the Job Object handle is live and the supervising caller has selected a
        // terminal cancellation, timeout, or resource-limit outcome.
        if unsafe { TerminateJobObject(self.handle, 1) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

#[cfg(windows)]
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        // SAFETY: this type owns the live handle and Drop runs once. KILL_ON_JOB_CLOSE is the
        // crash-safety backstop required for descendants that outlive their parent.
        unsafe { windows_sys::Win32::Foundation::CloseHandle(self.handle) };
    }
}

#[cfg(unix)]
pub(super) struct ProcessGroup {
    pgid: AtomicI32,
}

#[cfg(unix)]
impl ProcessGroup {
    pub(super) fn new() -> io::Result<Self> {
        Ok(Self {
            pgid: AtomicI32::new(0),
        })
    }

    pub(super) fn attach(&self, child: &Child) -> io::Result<()> {
        let pid = i32::try_from(child.id())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "child pid exceeds i32"))?;
        if pid <= 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid child process group id",
            ));
        }
        self.pgid.store(pid, Ordering::SeqCst);
        Ok(())
    }

    pub(super) fn poll_exit(
        &self,
        child: &mut Child,
    ) -> io::Result<Option<std::process::ExitStatus>> {
        let pgid = self.pgid.load(Ordering::SeqCst);
        if pgid <= 0 {
            return child.try_wait();
        }
        // Check leader completion with WNOWAIT before reaping so the zombie still pins `pgid`
        // while we terminate any descendants that inherited stdout/stderr pipe handles.
        // SAFETY: zeroed `siginfo_t` is POD and required so `si_pid()` is 0 when WNOHANG returns
        // without a state change.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        loop {
            // SAFETY: `pgid` is an un-reaped child PID spawned in its own process group.
            let rc = unsafe {
                libc::waitid(
                    libc::P_PID,
                    pgid as libc::id_t,
                    &mut info,
                    libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
                )
            };
            if rc == -1 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error);
            }
            break;
        }
        // SAFETY: kernel initialized `si_pid` (non-zero iff the target exited).
        if unsafe { info.si_pid() } == 0 {
            return Ok(None);
        }
        let _ = kill_process_group(pgid);
        let status = child.wait()?;
        self.pgid.store(0, Ordering::SeqCst);
        Ok(Some(status))
    }

    pub(super) fn terminate(&self, child: &mut Child) -> io::Result<()> {
        let pgid = self.pgid.load(Ordering::SeqCst);
        let group_result = if pgid > 0 {
            kill_process_group(pgid)
        } else {
            Ok(())
        };
        let kill_result = match child.kill() {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::InvalidInput => Ok(()),
            Err(_) if pgid > 0 && group_result.is_ok() => Ok(()),
            Err(error) => Err(error),
        };
        if child.wait().is_ok() {
            self.pgid.store(0, Ordering::SeqCst);
        }
        group_result.and(kill_result)
    }
}

#[cfg(unix)]
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        let pgid = self.pgid.load(Ordering::SeqCst);
        if pgid > 0 {
            let _ = kill_process_group(pgid);
        }
    }
}

#[cfg(unix)]
fn kill_process_group(pgid: i32) -> io::Result<()> {
    // SAFETY: `pgid` is a positive child PID registered after `CommandExt::process_group(0)`
    // placed the child in its own process group.
    let rc = unsafe { libc::killpg(pgid as libc::pid_t, libc::SIGKILL) };
    if rc == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(error)
    }
}

#[cfg(not(any(windows, unix)))]
pub(super) struct ProcessGroup;

#[cfg(not(any(windows, unix)))]
impl ProcessGroup {
    pub(super) fn new() -> io::Result<Self> {
        Ok(Self)
    }

    pub(super) fn attach(&self, _child: &Child) -> io::Result<()> {
        Ok(())
    }

    pub(super) fn terminate(&self, child: &mut Child) -> io::Result<()> {
        child.kill()
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::io::Read;
    use std::process::{Command, Stdio};

    use super::*;

    #[test]
    fn terminate_kills_grandchild_and_closes_piped_stdout_promptly() {
        let group = ProcessGroup::new().unwrap();
        let mut command = Command::new("sh");
        command
            .args(["-c", "sleep 60 & wait"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .stdin(Stdio::null());
        configure_command(&mut command);
        let mut child = command.spawn().unwrap();
        group.attach(&child).unwrap();
        let mut stdout = child.stdout.take().unwrap();
        // Give the shell a moment to spawn the background grandchild.
        std::thread::sleep(std::time::Duration::from_millis(50));

        group.terminate(&mut child).unwrap();
        let _ = child.wait();
        drop(group);

        let mut drained = Vec::new();
        stdout.read_to_end(&mut drained).unwrap();
    }
}

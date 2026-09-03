#![cfg(all(windows, target_arch = "x86_64"))]

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use bg_engine::manifest::{PeMachine, ToolSpec};
use bg_engine::weidu::invocation::{verify_tool_contract, InvocationError};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const MOCK_CHILD: &str = env!("CARGO_BIN_EXE_mock-child");
const PROBE_TIMEOUT_UPPER_BOUND: Duration = Duration::from_secs(4);

fn environment_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct EnvironmentGuard {
    values: Vec<(&'static str, Option<OsString>)>,
}

impl EnvironmentGuard {
    fn set(values: &[(&'static str, &OsStr)]) -> Self {
        let mut previous = Vec::with_capacity(values.len());
        for (name, value) in values {
            previous.push((*name, env::var_os(name)));
            env::set_var(name, value);
        }
        Self { values: previous }
    }
}

impl Drop for EnvironmentGuard {
    fn drop(&mut self) {
        for (name, value) in self.values.drain(..).rev() {
            match value {
                Some(value) => env::set_var(name, value),
                None => env::remove_var(name),
            }
        }
    }
}

fn fixture() -> (TempDir, PathBuf, ToolSpec) {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("weidu.exe");
    fs::copy(MOCK_CHILD, &source).unwrap();
    let bytes = fs::read(&source).unwrap();
    let contract = ToolSpec {
        executable: "weidu.exe".to_owned(),
        expected_length: u64::try_from(bytes.len()).unwrap(),
        sha256: hex::encode(Sha256::digest(&bytes)),
        weidu_version: "24900".to_owned(),
        pe_machine: PeMachine::X86_64,
    };
    (temp, source, contract)
}

fn wait_for_file(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "version probe did not create marker before deadline"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn version_probe_executes_a_private_copy_of_the_verified_bytes() {
    let _environment = environment_lock();
    let (temp, source, contract) = fixture();
    let marker = temp.path().join("probe-started.marker");
    let delay = OsString::from("1500");
    let _variables = EnvironmentGuard::set(&[
        ("CHRIZ_TEST_MOCK_WEIDU_MARKER", marker.as_os_str()),
        ("CHRIZ_TEST_MOCK_WEIDU_DELAY_MS", delay.as_os_str()),
    ]);

    let verifier_source = source.clone();
    let verifier = thread::spawn(move || verify_tool_contract(&verifier_source, &contract));
    wait_for_file(&marker);
    let probe_path = PathBuf::from(fs::read_to_string(&marker).unwrap());

    assert_ne!(
        probe_path.canonicalize().unwrap(),
        source.canonicalize().unwrap()
    );
    assert!(
        fs::write(&probe_path, b"attempted probe substitution").is_err(),
        "the running private probe copy must remain locked against substitution"
    );

    fs::write(&source, b"substituted after verification")
        .expect("the cache source must not be the executable held by the running probe");
    let (_, evidence) = verifier.join().unwrap().unwrap();

    assert_eq!(evidence.weidu_version, "24900");
    assert_eq!(fs::read(source).unwrap(), b"substituted after verification");
    assert!(
        !probe_path.exists(),
        "engine-owned probe copy was not removed after verification"
    );
}

#[test]
fn version_probe_times_out_and_terminates_the_child() {
    let _environment = environment_lock();
    let (temp, source, contract) = fixture();
    let delay = OsString::from("6000");
    let descendant_marker = temp.path().join("descendant-survived.marker");
    let _variables = EnvironmentGuard::set(&[
        ("CHRIZ_TEST_MOCK_WEIDU_DELAY_MS", delay.as_os_str()),
        (
            "CHRIZ_TEST_MOCK_WEIDU_DESCENDANT_MARKER",
            descendant_marker.as_os_str(),
        ),
    ]);
    let started = Instant::now();

    let error = match verify_tool_contract(&source, &contract) {
        Err(error) => error,
        Ok(_) => panic!("unbounded version probe unexpectedly completed"),
    };

    assert!(
        started.elapsed() < PROBE_TIMEOUT_UPPER_BOUND,
        "probe was not terminated promptly: {error}"
    );
    assert!(matches!(
        error,
        InvocationError::ToolVersionProbeTimedOut { .. }
    ));
    thread::sleep(Duration::from_millis(1200));
    assert!(
        !descendant_marker.exists(),
        "timed-out probe left a descendant process alive"
    );
}

#[test]
fn version_probe_rejects_output_beyond_its_memory_bound() {
    let _environment = environment_lock();
    let (_temp, source, contract) = fixture();
    let output_bytes = OsString::from((1024 * 1024).to_string());
    let _variables = EnvironmentGuard::set(&[(
        "CHRIZ_TEST_MOCK_WEIDU_OUTPUT_BYTES",
        output_bytes.as_os_str(),
    )]);

    let error = match verify_tool_contract(&source, &contract) {
        Err(error) => error,
        Ok(_) => panic!("unbounded version probe output unexpectedly succeeded"),
    };

    assert!(matches!(
        error,
        InvocationError::ToolVersionProbeOutputLimit { limit: 65_536, .. }
    ));
}

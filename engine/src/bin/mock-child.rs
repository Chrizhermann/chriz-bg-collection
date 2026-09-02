//! Deterministic child-process fixture for the runner integration tests.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    if let Err(error) = dispatch(env::args_os().skip(1).collect()) {
        eprintln!("mock-child: {error}");
        std::process::exit(2);
    }
}

fn dispatch(args: Vec<OsString>) -> io::Result<()> {
    let mode = args
        .first()
        .and_then(|value| value.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing mode"))?;
    match mode {
        "fragmented-prompt" => fragmented_prompt(),
        "unmatched-prompt" => unmatched_prompt(),
        "quiet" => quiet(parse_millis(args.get(1))?),
        "expect-eof" => expect_eof(),
        "invalid-utf8" => invalid_utf8(),
        "large-streams" => large_streams(parse_usize(args.get(1))?),
        "spawn-child" => spawn_child(required_path(args.get(1))?),
        "delayed-marker" => delayed_marker(required_path(args.get(1))?, parse_millis(args.get(2))?),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unknown mode {mode:?}"),
        )),
    }
}

fn fragmented_prompt() -> io::Result<()> {
    let (answer_tx, answer_rx) = mpsc::channel();
    thread::spawn(move || {
        let mut answer = String::new();
        let result = io::stdin().lock().read_line(&mut answer).map(|_| answer);
        let _ = answer_tx.send(result);
    });

    match answer_rx.recv_timeout(Duration::from_millis(120)) {
        Ok(_) => {
            println!("answer-arrived-before-prompt");
            std::process::exit(9);
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "stdin reader disconnected",
            ));
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {}
    }

    let mut stdout = io::stdout().lock();
    stdout.write_all(b"Choose ")?;
    stdout.flush()?;
    thread::sleep(Duration::from_millis(30));
    stdout.write_all(b"option: ")?;
    stdout.flush()?;
    drop(stdout);

    let answer = answer_rx
        .recv_timeout(Duration::from_secs(3))
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "answer did not arrive"))??;
    println!("answer:{}", answer.trim_end());
    Ok(())
}

fn unmatched_prompt() -> io::Result<()> {
    print!("Unexpected prompt: ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    println!("unexpected-answer:{}", answer.trim_end());
    Ok(())
}

fn quiet(delay: Duration) -> io::Result<()> {
    thread::sleep(delay);
    println!("quiet-exit");
    Ok(())
}

fn expect_eof() -> io::Result<()> {
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes)?;
    if bytes.is_empty() {
        println!("stdin-closed");
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "stdin unexpectedly contained bytes",
        ))
    }
}

fn invalid_utf8() -> io::Result<()> {
    io::stdout().write_all(b"raw-\x80-bytes")?;
    io::stdout().flush()?;
    io::stderr().write_all(b"err-\xff-bytes")?;
    io::stderr().flush()
}

fn large_streams(bytes_per_stream: usize) -> io::Result<()> {
    let stdout = thread::spawn(move || write_repeated(io::stdout(), b'o', bytes_per_stream));
    let stderr = thread::spawn(move || write_repeated(io::stderr(), b'e', bytes_per_stream));
    stdout
        .join()
        .map_err(|_| io::Error::other("stdout writer panicked"))??;
    stderr
        .join()
        .map_err(|_| io::Error::other("stderr writer panicked"))??;
    Ok(())
}

fn write_repeated(mut output: impl Write, byte: u8, count: usize) -> io::Result<()> {
    let chunk = [byte; 8 * 1024];
    let mut remaining = count;
    while remaining > 0 {
        let write = remaining.min(chunk.len());
        output.write_all(&chunk[..write])?;
        remaining -= write;
    }
    output.flush()
}

fn spawn_child(marker: PathBuf) -> io::Result<()> {
    // Give the parent supervisor time to attach this process to its Job Object. Every child
    // created after attachment automatically joins the same process tree.
    thread::sleep(Duration::from_millis(150));
    Command::new(env::current_exe()?)
        .arg("delayed-marker")
        .arg(marker)
        .arg("650")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    println!("child-spawned");
    thread::sleep(Duration::from_secs(10));
    Ok(())
}

fn delayed_marker(marker: PathBuf, delay: Duration) -> io::Result<()> {
    thread::sleep(delay);
    fs::write(marker, b"descendant survived")
}

fn parse_millis(value: Option<&OsString>) -> io::Result<Duration> {
    parse_usize(value).map(|value| Duration::from_millis(value as u64))
}

fn parse_usize(value: Option<&OsString>) -> io::Result<usize> {
    value
        .and_then(|value| value.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing numeric argument"))?
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid numeric argument"))
}

fn required_path(value: Option<&OsString>) -> io::Result<PathBuf> {
    value
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing path argument"))
}

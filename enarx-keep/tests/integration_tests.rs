// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

const CRATE: &str = env!("CARGO_MANIFEST_DIR");
const OUT_DIR: &str = env!("OUT_DIR");
const TEST_DIR: &str = "test-bin";
const TIMEOUT_SECS: u64 = 5;

fn path(input: &str) -> PathBuf {
    Path::new(CRATE).join(OUT_DIR).join(TEST_DIR).join(input)
}

/// Returns true if running the binary exits with 0. Used in cases
/// where there is no output or output does not matter.
fn test_init() {
    let mut filtered_env: HashMap<String, String> = std::env::vars()
        .filter(|&(ref k, _)| {
            k == "TERM"
                || k == "TZ"
                || k == "LANG"
                || k == "PATH"
                || k == "RUSTUP_HOME"
                || k == "RUSTFLAGS"
        })
        .collect();

    filtered_env.insert("ENARX_INTEGRATION_TESTS".into(), "1".into());

    let status = Command::new("cargo")
        .current_dir(CRATE)
        .env_clear()
        .envs(filtered_env)
        .arg("+nightly")
        .arg("build")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("failed to prepare");

    if !status.success() {
        eprintln!("failed to prepare");
        std::process::exit(1);
    }
}

/// Returns true if running the binary exits with 0. Used in cases
/// where there is no output or output does not matter.
fn run_succeeds(bin: PathBuf) -> bool {
    let mut filtered_env: HashMap<String, String> = std::env::vars()
        .filter(|&(ref k, _)| {
            k == "TERM"
                || k == "TZ"
                || k == "LANG"
                || k == "PATH"
                || k == "RUSTUP_HOME"
                || k == "RUSTFLAGS"
        })
        .collect();

    filtered_env.insert("ENARX_INTEGRATION_TESTS".into(), "1".into());

    let mut cmd = Command::new("cargo")
        .current_dir(CRATE)
        .arg("+nightly")
        .env_clear()
        .envs(filtered_env)
        .arg("run")
        .arg("-q")
        .arg("exec")
        .arg(bin)
        .spawn()
        .expect("failed to run test bin");

    if let Some(status) = cmd.wait_timeout(Duration::from_secs(TIMEOUT_SECS)).unwrap() {
        return status.success();
    } else {
        cmd.kill().unwrap();
        panic!("error: test timeout");
    }
}

/// Returns a handle to a child process through which output (stdout, stderr) can
/// be accessed.
fn run_test(bin: PathBuf) -> std::process::Child {
    let mut filtered_env: HashMap<String, String> = std::env::vars()
        .filter(|&(ref k, _)| {
            k == "TERM"
                || k == "TZ"
                || k == "LANG"
                || k == "PATH"
                || k == "RUSTUP_HOME"
                || k == "RUSTFLAGS"
        })
        .collect();

    filtered_env.insert("ENARX_INTEGRATION_TESTS".into(), "1".into());

    let mut cmd = Command::new("cargo")
        .current_dir(CRATE)
        .env_clear()
        .envs(filtered_env)
        .arg("+nightly")
        .arg("run")
        .arg("-q")
        .arg("exec")
        .arg(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to run test bin");

    if let Ok(None) = cmd.wait_timeout(Duration::from_secs(TIMEOUT_SECS)) {
        cmd.kill().unwrap();
        panic!("error: test timeout");
    } else {
        cmd
    }
}

#[test]
fn exit_zero() {
    test_init();
    let code = path("exit_zero");
    assert!(run_succeeds(code));
}

#[test]
fn exit_one() {
    test_init();
    let code = path("exit_one");
    assert!(!run_succeeds(code));
}

#[test]
fn clock_gettime() {
    test_init();
    let code = path("clock_gettime");
    assert!(run_succeeds(code));
}

#[test]
fn write_stdout() {
    test_init();
    let mut buf = [0u8; 3];
    let code = path("write_stdout");
    let child = run_test(code);
    let mut child_stdout = child.stdout.unwrap();
    child_stdout
        .read(&mut buf)
        .expect("failed to read child stdout");

    assert_eq!("hi\n", String::from_utf8(buf.to_vec()).unwrap(),);
}

#[test]
fn write_stderr() {
    test_init();

    let mut buf = [0u8; 3];
    let code = path("write_stderr");
    let child = run_test(code);
    child
        .stderr
        .unwrap()
        .read(&mut buf)
        .expect("failed to read child stderr");
    assert_eq!("hi\n", String::from_utf8(buf.to_vec()).unwrap());
}

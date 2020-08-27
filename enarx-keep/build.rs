// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const CRATE: &str = env!("CARGO_MANIFEST_DIR");
const TEST_BIN: &str = "tests/test-bin";

fn rerun_src(path: impl AsRef<Path>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let path = entry?.path();

        if path.is_dir() {
            rerun_src(path)?;
        } else if path.is_file() {
            if let Some(ext) = path.extension() {
                if let Some(ext) = ext.to_str() {
                    if let Some(path) = path.to_str() {
                        match ext {
                            "rs" => println!("cargo:rerun-if-changed={}", path),
                            "s" => println!("cargo:rerun-if-changed={}", path),
                            "S" => println!("cargo:rerun-if-changed={}", path),
                            _ => (),
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn create_dir(path: &PathBuf) {
    match std::fs::create_dir(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => {
            eprintln!("Can't create {:#?} : {:#?}", path, e);
            std::process::exit(1);
        }
        Ok(_) => {}
    }
}

fn build_asm_tests(path: &Path) {
    let test_dir = Path::new(CRATE).join(TEST_BIN);
    let asm_out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("test-bin");

    create_dir(&asm_out_dir);

    for entry in path.read_dir().expect("failed to read test-bin dir") {
        let file = entry.expect("failed to read file in test-bin dir");
        let filename = file.file_name();
        let output = file.path().file_stem().unwrap().to_os_string();

        let mut cmd = cc::Build::new()
            .no_default_flags(true)
            .get_compiler()
            .to_command();

        cmd.current_dir(&asm_out_dir)
            .arg("-nostdlib")
            .arg("-static-pie")
            .arg("-fPIC")
            .arg("-o")
            .arg(output)
            .arg(test_dir.join(filename))
            .status()
            .expect("failed to compile binary");
    }
}

fn main() {
    let tests = Path::new(CRATE).join(TEST_BIN);
    build_asm_tests(&tests);

    println!("cargo:rerun-if-env-changed=OUT_DIR");
    println!("cargo:rerun-if-env-changed=PROFILE");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let out_dir_bin = out_dir.join("bin");

    create_dir(&out_dir_bin);

    let prof_str = std::env::var("PROFILE").unwrap();
    let profile: &[&str] = match prof_str.as_str() {
        "release" => &["--release"],
        _ => &[],
    };

    let target_name = "x86_64-unknown-linux-musl";

    let filtered_env: HashMap<String, String> = std::env::vars()
        .filter(|&(ref k, _)| {
            k == "TERM" || k == "TZ" || k == "LANG" || k == "PATH" || k == "RUSTUP_HOME"
        })
        .collect();

    for entry in std::fs::read_dir("shims").unwrap() {
        let shim_path = entry.unwrap().path();
        let shim_name = shim_path.clone();
        let shim_name = shim_name
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();

        let shim_out_dir = out_dir.join(&shim_path);

        let target_dir = shim_out_dir.clone().into_os_string().into_string().unwrap();

        let path: String = shim_path.into_os_string().into_string().unwrap();

        println!("cargo:rerun-if-changed={}/Cargo.toml", path);
        println!("cargo:rerun-if-changed={}/Cargo.lock", path);
        println!("cargo:rerun-if-changed={}/link.json", path);
        println!("cargo:rerun-if-changed={}/.cargo/config", path);
        rerun_src(&path).unwrap();

        let int_test = std::env::var("ENARX_INTEGRATION_TESTS").is_ok();

        let stdout: Stdio = if !int_test {
            OpenOptions::new()
                .write(true)
                .open("/dev/tty")
                .map(Stdio::from)
                .unwrap_or_else(|_| Stdio::inherit())
        } else {
            Stdio::null()
        };

        let stderr: Stdio = if !int_test {
            OpenOptions::new()
                .write(true)
                .open("/dev/tty")
                .map(Stdio::from)
                .unwrap_or_else(|_| Stdio::inherit())
        } else {
            Stdio::null()
        };

        let status = Command::new("cargo")
            .current_dir(&path)
            .env_clear()
            .envs(&filtered_env)
            .stdout(stdout)
            .stderr(stderr)
            .arg("+nightly")
            .arg("build")
            .args(profile)
            .arg("--target-dir")
            .arg(&target_dir)
            .arg("--target")
            .arg(target_name)
            .arg("--bin")
            .arg(&shim_name)
            .status()
            .expect("failed to build shim");

        if !status.success() {
            eprintln!("Failed to build shim {}", path);
            std::process::exit(1);
        }

        let out_bin = out_dir_bin.join(&shim_name);

        let shim_out_bin = shim_out_dir
            .join(&target_name)
            .join(&std::env::var("PROFILE").unwrap())
            .join(&shim_name);

        let status = Command::new("strip")
            .arg("--strip-unneeded")
            .arg("-o")
            .arg(&out_bin)
            .arg(&shim_out_bin)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        match status {
            Ok(status) if status.success() => {}
            _ => {
                eprintln!("Failed to run strip");
                std::fs::rename(&shim_out_bin, &out_bin).expect("move failed")
            }
        }
    }
}

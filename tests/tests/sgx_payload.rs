// SPDX-License-Identifier: Apache-2.0

use std::path::Path;
use std::process::Command;

/// This test runs the payload in the SGX keep using the SGX shim.
#[cfg_attr(not(has_sgx), ignore)]
#[test]
fn sgx_payload() {
    // Define directories
    let wksp_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_str()
        .unwrap();
    let sgx_keep_dir = wksp_root.to_owned() + "/enarx-keep-sgx/target/";
    let payload_dir = wksp_root.to_owned() + "/payload/target/x86_64-unknown-linux-musl/";
    let sgx_shim_dir = wksp_root.to_owned() + "/enarx-keep-sgx-shim/target/";

    // Find current SGX Keep binary
    let find_keep = String::from_utf8(
        Command::new("find")
            .current_dir(sgx_keep_dir.to_owned())
            .arg("-name")
            .arg("enarx-keep-sgx")
            .output()
            .expect("failed")
            .stdout,
    )
    .unwrap();

    // Find current payload
    let find_payload = String::from_utf8(
        Command::new("find")
            .current_dir(payload_dir.to_owned())
            .arg("-name")
            .arg("payload")
            .output()
            .expect("failed")
            .stdout,
    )
    .unwrap();

    // Find current SGX shim
    let find_shim = String::from_utf8(
        Command::new("find")
            .current_dir(sgx_shim_dir.to_owned())
            .arg("-name")
            .arg("enarx-keep-sgx-shim")
            .output()
            .expect("failed")
            .stdout,
    )
    .unwrap();

    // Trim off newline and current dir from results strings; join file name to path
    let sgx_keep = sgx_keep_dir.to_owned() + &find_keep[2..find_keep.len() - 1];
    let payload = payload_dir.to_owned() + &find_payload[2..find_payload.len() - 1];
    let sgx_shim = sgx_shim_dir.to_owned() + &find_shim[2..find_shim.len() - 1];

    let result = Command::new(sgx_keep)
        .current_dir(wksp_root.to_owned())
        .arg("--code")
        .arg(payload)
        .arg("--shim")
        .arg(sgx_shim)
        .output()
        .expect("failed");

    assert!(result.status.success());
}

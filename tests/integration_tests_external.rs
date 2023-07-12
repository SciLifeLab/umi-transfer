use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[path = "auxiliary.rs"]
mod auxiliary;

#[test]
fn external_fails_without_arguments() {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();

    cmd.arg("external");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--in <R1_IN>"))
        .stderr(predicate::str::contains("--in2 <R2_IN>"))
        .stderr(predicate::str::contains("--umi <RU_IN>"));
}

#[test]
fn external_with_minimal_arguments_plain() {
    let (mut cmd, temp_dir, test_files) = auxiliary::setup_integration_test();
    cmd.arg("external")
        .arg("--in")
        .arg(test_files.read1)
        .arg("--in2")
        .arg(test_files.read2)
        .arg("--umi")
        .arg(test_files.umi);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Transferring UMIs to records"))
        .stdout(predicate::str::contains("Processed 10 records"))
        .stdout(predicate::str::contains("umi-transfer finished after"));

    temp_dir.close().unwrap();
}

#[test]
fn external_with_minimal_arguments_compressed() {
    let (mut cmd, temp_dir, test_files) = auxiliary::setup_integration_test();
    cmd.arg("external")
        .arg("--in")
        .arg(test_files.read1_gz)
        .arg("--in2")
        .arg(test_files.read2_gz)
        .arg("--umi")
        .arg(test_files.umi_gz);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Transferring UMIs to records"))
        .stdout(predicate::str::contains("Processed 10 records"))
        .stdout(predicate::str::contains("umi-transfer finished after"));

    temp_dir.close().unwrap();
}

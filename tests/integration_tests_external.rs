use assert_cmd::prelude::*;
use assert_fs::prelude::*;
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

    temp_dir
        .child("read1_with_UMIs.fq")
        .assert(predicate::path::exists());

    temp_dir
        .child("read2_with_UMIs.fq")
        .assert(predicate::path::exists());

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
        .arg(test_files.umi_gz)
        .arg("--gzip");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Transferring UMIs to records"))
        .stdout(predicate::str::contains("Processed 10 records"))
        .stdout(predicate::str::contains("umi-transfer finished after"));

    temp_dir
        .child("read1_with_UMIs.fq.gz")
        .assert(predicate::path::exists());

    temp_dir
        .child("read2_with_UMIs.fq.gz")
        .assert(predicate::path::exists());

    temp_dir.close().unwrap();
}

#[test]
fn external_with_output_gz_suffix_no_compression() {
    let (mut cmd, temp_dir, test_files) = auxiliary::setup_integration_test();
    cmd.arg("external")
        .arg("--in")
        .arg(test_files.read1_gz)
        .arg("--in2")
        .arg(test_files.read2_gz)
        .arg("--umi")
        .arg(test_files.umi_gz)
        .arg("--out")
        .arg(test_files.new_output_read1_gz)
        .arg("--out2")
        .arg(test_files.new_output_read2_gz);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Transferring UMIs to records"))
        .stdout(predicate::str::contains("Processed 10 records"))
        .stdout(predicate::str::contains("umi-transfer finished after"));

    // Even though --out "read1_out.fq.gz" and --out2 "read2_out.fq.gz" are explicitly specified,
    // the .gz suffix will be automatically removed if no -z / --gzip was chosen.
    // This behavior was chosen because of FIFOs use.

    temp_dir
        .child("read1_out.fq")
        .assert(predicate::path::exists());

    temp_dir
        .child("read2_out.fq")
        .assert(predicate::path::exists());

    temp_dir
        .child("read1_out.fq.gz")
        .assert(predicate::path::missing());

    temp_dir
        .child("read2_out.fq.gz")
        .assert(predicate::path::missing());

    temp_dir.close().unwrap();
}

#[test]
fn external_fails_on_read_id_mismatch() {
    let (mut cmd, temp_dir, test_files) = auxiliary::setup_integration_test();
    cmd.arg("external")
        .arg("--in")
        .arg(test_files.read1_gz)
        .arg("--in2")
        .arg(test_files.read2_gz)
        .arg("--umi")
        .arg(test_files.umi_shuffle_gz);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to include the UMIs"))
        .stderr(predicate::str::contains(
            "IDs of UMI and read records mismatch",
        ))
        .stderr(predicate::str::contains(
            "Please provide sorted files as input",
        ));

    temp_dir.close().unwrap();
}

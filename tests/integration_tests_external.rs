use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command as StdCommand;

extern crate rexpect;

#[path = "auxiliary.rs"]
mod auxiliary;

#[test]
fn external_fails_without_arguments() {
    let mut cmd = cargo_bin_cmd!();

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
    let (mut cmd, temp_dir, test_files, _test_output) = auxiliary::setup_integration_test(false);
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
fn external_with_inline_position() {
    let (mut cmd, temp_dir, test_files, _test_output) = auxiliary::setup_integration_test(false);
    cmd.arg("external")
        .arg("--in")
        .arg(test_files.read1)
        .arg("--in2")
        .arg(test_files.read2)
        .arg("--umi")
        .arg(test_files.umi)
        .arg("--position")
        .arg("inline");

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
    let (mut cmd, temp_dir, test_files, _test_output) = auxiliary::setup_integration_test(false);
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
fn external_with_output_no_gz_suffix_compression() {
    let (mut cmd, temp_dir, test_files, _test_output) = auxiliary::setup_integration_test(false);
    cmd.arg("external")
        .arg("--in")
        .arg(test_files.read1_gz)
        .arg("--in2")
        .arg(test_files.read2_gz)
        .arg("--umi")
        .arg(test_files.umi_gz)
        .arg("--out")
        .arg(test_files.new_output_read1)
        .arg("--out2")
        .arg(test_files.new_output_read2)
        .arg("--gzip");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Transferring UMIs to records"))
        .stdout(predicate::str::contains("Processed 10 records"))
        .stdout(predicate::str::contains("umi-transfer finished after"));

    // Even though --out "read1_out.fq" and --out2 "read2_out.fq" are explicitly specified,
    // a .gz suffix will be automatically added when compressed output is generated.

    temp_dir
        .child("read1_out.fq")
        .assert(predicate::path::missing());

    temp_dir
        .child("read2_out.fq")
        .assert(predicate::path::missing());

    temp_dir
        .child("read1_out.fq.gz")
        .assert(predicate::path::exists());

    temp_dir
        .child("read2_out.fq.gz")
        .assert(predicate::path::exists());

    temp_dir.close().unwrap();
}

#[test]
fn external_with_output_gz_suffix_no_compression() {
    let (mut cmd, temp_dir, test_files, _test_output) = auxiliary::setup_integration_test(false);
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
fn external_fails_with_nonexisting_output_file() {
    let (mut cmd, temp_dir, test_files, _test_output) = auxiliary::setup_integration_test(false);
    cmd.arg("external")
        .arg("--in")
        .arg(test_files.read1_gz)
        .arg("--in2")
        .arg(test_files.read2_gz)
        .arg("--umi")
        .arg(test_files.umi_gz)
        .arg("--out")
        .arg(test_files.nonexisting_output)
        .arg("--out2")
        .arg(test_files.new_output_read2_gz);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to include the UMIs"))
        .stderr(predicate::str::contains("Caused by:"))
        .stderr(predicate::str::contains("Output file"))
        .stderr(predicate::str::contains("is missing or not writeable"));

    temp_dir
        .child("read2_out.fq")
        .assert(predicate::path::missing());

    temp_dir.close().unwrap();
}

#[test]
fn external_fails_with_existing_output_file_and_no_force() {
    let (_cmd, temp_dir, test_files, _test_output) = auxiliary::setup_integration_test(false);

    // create an existing output file
    temp_dir
        .child("read1_out.fq")
        .write_str("GCCATTAGCTGTACCATACTCAGGCACACAAAAATACTGATA")
        .unwrap();

    // This test comprises an interactive prompt, which is not supported by assert_cmd.
    // Therefore, we use rexpect to run the test in a session and must use
    // a different Command type: std::process::Command instead of assert_cmd::Command.

    let mut cmd = StdCommand::new(assert_cmd::cargo::cargo_bin!());
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

    // Evaluate that the prompt is shown, but do not overwrite the existing file.

    let mut p = rexpect::session::spawn_command(cmd, Some(10000)).unwrap();
    p.exp_string("read1_out.fq exists. Overwrite?").unwrap();
    p.send_line("n").unwrap();
    p.exp_string("read1_out.fq exists, but must not be overwritten.")
        .unwrap();

    temp_dir
        .child("read2_out.fq")
        .assert(predicate::path::missing());

    temp_dir.close().unwrap();
}

#[test]
fn external_succeeds_with_existing_output_file_and_force() {
    let (mut cmd, temp_dir, test_files, _test_output) = auxiliary::setup_integration_test(false);

    // create an existing output file
    temp_dir
        .child("read1_out.fq")
        .write_str("GCCATTAGCTGTACCATACTCAGGCACACAAAAATACTGATA")
        .unwrap();

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
        .arg(test_files.new_output_read2_gz)
        .arg("--force");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Transferring UMIs to records"))
        .stdout(predicate::str::contains("Processed 10 records"))
        .stdout(predicate::str::contains("umi-transfer finished after"));

    temp_dir
        .child("read2_out.fq")
        .assert(predicate::path::exists());

    temp_dir.close().unwrap();
}

#[test]
fn external_fails_on_read_id_mismatch() {
    let (mut cmd, temp_dir, test_files, _test_output) = auxiliary::setup_integration_test(false);
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

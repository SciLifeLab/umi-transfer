use assert_cmd::prelude::*;
use assert_fs::fixture::{NamedTempFile, TempDir};
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::path::PathBuf;
use std::process::Command;

#[derive()]
struct TestFiles {
    // Struct to hold the paths to test files.
    read1: PathBuf,
    read1_gz: PathBuf,
    read2: PathBuf,
    read2_gz: PathBuf,
    umi: PathBuf,
    umi_gz: PathBuf,
    umi_shuffle: PathBuf,
    umi_shuffle_gz: PathBuf,
    existing_output: NamedTempFile,
}

fn setup_integration_test() -> (Command, TempDir, TestFiles) {
    // Get the name of the binary (umi-transfer)
    let cmd = Command::cargo_bin(assert_cmd::crate_name!())
        .expect("Failed to pull binary name from Cargo.toml at compile time.");

    let temp_dir = assert_fs::TempDir::new().expect("Failed to create temporary directory");

    temp_dir
        .copy_from(
            std::env::current_dir()
                .expect("Failed to get directory")
                .join("./tests/seqdata"),
            &["*.fq", "*.gz"],
        )
        .expect("Failed to copy test data to temporary directory.");

    let test_files = TestFiles {
        read1: temp_dir.path().join("read1.fq"),
        read1_gz: temp_dir.path().join("read1.fq.gz"),
        read2: temp_dir.path().join("read2.fq"),
        read2_gz: temp_dir.path().join("read2.fq.gz"),
        umi: temp_dir.path().join("umi.fq"),
        umi_gz: temp_dir.path().join("umi.fq.gz"),
        umi_shuffle: temp_dir.path().join("umi_shuffle.fq"),
        umi_shuffle_gz: temp_dir.path().join("umi_shuffle.fq.gz"),
        existing_output: NamedTempFile::new("ACTG.fq").unwrap(),
    };

    return (cmd, temp_dir, test_files);
}

// Define the integration tests
#[test]
fn main_without_arguments_prints_help() {
    let mut cmd = Command::cargo_bin(assert_cmd::crate_name!()).unwrap();
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Usage:"))
        .stderr(predicate::str::contains("Commands:"))
        .stderr(predicate::str::contains("Options:"));
}

#[test]
fn external_with_minimal_arguments() {
    let (mut cmd, temp_dir, test_files) = setup_integration_test();
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

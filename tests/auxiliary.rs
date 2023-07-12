use assert_cmd::prelude::*;
use assert_fs::fixture::{NamedTempFile, TempDir};
use assert_fs::prelude::*;
use std::path::PathBuf;
use std::process::Command;

// since those are just needed for the tests, I didn't put it in src. Therefore, using this module is not detected and dead_code warnings issued.

#[derive()]
#[allow(dead_code)]
pub struct TestFiles {
    // Struct to hold the paths to test files.
    pub read1: PathBuf,
    pub read1_gz: PathBuf,
    pub read2: PathBuf,
    pub read2_gz: PathBuf,
    pub umi: PathBuf,
    pub umi_gz: PathBuf,
    pub umi_shuffle: PathBuf,
    pub umi_shuffle_gz: PathBuf,
    pub existing_output: NamedTempFile,
    pub new_output_read1: PathBuf,
    pub new_output_read1_gz: PathBuf,
    pub new_output_read2: PathBuf,
    pub new_output_read2_gz: PathBuf,
}

#[allow(dead_code)]
pub fn setup_integration_test() -> (Command, TempDir, TestFiles) {
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
        umi_shuffle: temp_dir.path().join("umi_shuffled.fq"),
        umi_shuffle_gz: temp_dir.path().join("umi_shuffled.fq.gz"),
        existing_output: NamedTempFile::new("ACTG.fq").unwrap(),
        new_output_read1: temp_dir.path().join("read1_out.fq"),
        new_output_read1_gz: temp_dir.path().join("read1_out.fq.gz"),
        new_output_read2: temp_dir.path().join("read2_out.fq"),
        new_output_read2_gz: temp_dir.path().join("read2_out.fq.gz"),
    };

    return (cmd, temp_dir, test_files);
}

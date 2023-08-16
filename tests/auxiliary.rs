use anyhow::{anyhow, Result};
use assert_cmd::Command;
use assert_fs::fixture::{NamedTempFile, TempDir};
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::path::PathBuf;
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
    pub nonexisting_output: PathBuf,
    pub new_output_read1: PathBuf,
    pub new_output_read1_gz: PathBuf,
    pub new_output_read2: PathBuf,
    pub new_output_read2_gz: PathBuf,
}

#[derive()]
#[allow(dead_code)]
pub struct TestOutput {
    // Struct to hold the paths to validated output files.
    pub correct_read1: PathBuf,
    pub correct_read2: PathBuf,
    pub corrected_read1: PathBuf,
    pub corrected_read2: PathBuf,
    pub delim_underscore_read1: PathBuf,
    pub delim_underscore_read2: PathBuf,
    pub umi_read2_switch_read1: PathBuf,
    pub umi_read2_switch_read2: PathBuf,
}

#[allow(dead_code)]
pub fn setup_integration_test(
    with_results: bool,
) -> (Command, TempDir, TestFiles, Option<TestOutput>) {
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

    if with_results {
        temp_dir
            .copy_from(
                std::env::current_dir()
                    .expect("Failed to get directory")
                    .join("./tests/results"),
                &["*.fq"],
            )
            .expect("Failed to copy result data to temporary directory.");
    };

    let test_files = TestFiles {
        read1: temp_dir.path().join("read1.fq"),
        read1_gz: temp_dir.path().join("read1.fq.gz"),
        read2: temp_dir.path().join("read2.fq"),
        read2_gz: temp_dir.path().join("read2.fq.gz"),
        umi: temp_dir.path().join("umi.fq"),
        umi_gz: temp_dir.path().join("umi.fq.gz"),
        umi_shuffle: temp_dir.path().join("umi_shuffled.fq"),
        umi_shuffle_gz: temp_dir.path().join("umi_shuffled.fq.gz"),
        nonexisting_output: NamedTempFile::new("ACTG.fq").unwrap().path().to_path_buf(), //goes out of scope too early
        new_output_read1: temp_dir.path().join("read1_out.fq"),
        new_output_read1_gz: temp_dir.path().join("read1_out.fq.gz"),
        new_output_read2: temp_dir.path().join("read2_out.fq"),
        new_output_read2_gz: temp_dir.path().join("read2_out.fq.gz"),
    };

    let test_output = if with_results {
        let temp = TestOutput {
            correct_read1: temp_dir.path().join("correct_read1.fq"),
            correct_read2: temp_dir.path().join("correct_read2.fq"),
            corrected_read1: temp_dir.path().join("corrected_read1.fq"),
            corrected_read2: temp_dir.path().join("corrected_read2.fq"),
            delim_underscore_read1: temp_dir.path().join("delim_underscore_read1.fq"),
            delim_underscore_read2: temp_dir.path().join("delim_underscore_read2.fq"),
            umi_read2_switch_read1: temp_dir.path().join("umi_read2_switch_read1.fq"),
            umi_read2_switch_read2: temp_dir.path().join("umi_read2_switch_read2.fq"),
        };
        Some(temp)
    } else {
        None
    };

    return (cmd, temp_dir, test_files, test_output);
}

// Function to compare two files, used to test if the program output matches the reference.
#[allow(dead_code)]
pub fn verify_file_contents(test_file: &PathBuf, reference_file: &PathBuf) -> Result<bool> {
    let test_file_content = std::fs::read_to_string(&test_file)
        .map_err(|err| anyhow!("Failed to read test file: {}", err))?;
    let reference_file_content = std::fs::read_to_string(&reference_file)
        .map_err(|err| anyhow!("Failed to read reference file: {}", err))?;

    let predicate_fn = predicate::str::diff(reference_file_content);

    if predicate_fn.eval(&test_file_content) {
        Ok(true)
    } else {
        Err(anyhow!(
            "{} and {} did not match!",
            reference_file.file_name().unwrap().to_string_lossy(),
            test_file.file_name().unwrap().to_string_lossy()
        ))
    }
}

use assert_fs::prelude::*;
use auxiliary::verify_file_contents;
use predicates::prelude::*;
use std::error::Error;

#[path = "auxiliary.rs"]
mod auxiliary;

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn external_produces_correct_output() -> TestResult {
    let (mut cmd, temp_dir, test_files, test_output) = auxiliary::setup_integration_test(true);
    cmd.arg("external")
        .arg("--in")
        .arg(test_files.read1)
        .arg("--in2")
        .arg(test_files.read2)
        .arg("--umi")
        .arg(test_files.umi);

    cmd.assert().success(); //further assertions have been tested in other tests

    temp_dir
        .child("read1_with_UMIs.fq")
        .assert(predicate::path::exists());

    temp_dir
        .child("read2_with_UMIs.fq")
        .assert(predicate::path::exists());

    let reference = test_output.unwrap();

    verify_file_contents(
        &temp_dir.child("read1_with_UMIs.fq").to_path_buf(),
        &reference.correct_read1,
    )?;

    verify_file_contents(
        &temp_dir.child("read2_with_UMIs.fq").to_path_buf(),
        &reference.correct_read2,
    )?;

    temp_dir.close().unwrap();
    Ok(())
}

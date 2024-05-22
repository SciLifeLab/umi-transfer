use assert_fs::prelude::*;
use auxiliary::{verify_file_binary,verify_file_contents};
use predicates::prelude::*;
use std::error::Error;

#[path = "auxiliary.rs"]
mod auxiliary;

type TestResult = Result<(), Box<dyn Error>>;

// First two tests to test that my tests will work.

#[test]
fn testing_file_verification_succeeds() -> TestResult {
    let (mut _cmd, temp_dir, test_files, _test_output) = auxiliary::setup_integration_test(false);

    // the same file should be identical
    verify_file_contents(&test_files.read1, &test_files.read1)?;

    temp_dir.close()?;
    Ok(())
}

#[test]
#[should_panic(expected = "read2.fq and read1.fq did not match!")]
fn testing_file_verification_fails() {
    let (mut _cmd, temp_dir, test_files, _test_output) = auxiliary::setup_integration_test(false);

    // the same file should be identical
    verify_file_contents(&test_files.read1, &test_files.read2).unwrap();

    temp_dir.close().unwrap();
}

// Yep, verify_file_contents() does its job. Ready to rumble!
// Do the same for binary files.

#[test]
fn testing_file_comparison_succeeds() -> TestResult {
    let (mut _cmd, temp_dir, test_files, _test_output) = auxiliary::setup_integration_test(false);

    // the same file should be identical
    verify_file_binary(&test_files.read1, &test_files.read1)?;

    temp_dir.close()?;
    Ok(())
}

#[test]
#[should_panic(expected = "read2.fq and read1.fq did not match!")]
fn testing_file_comparison_fails() {
    let (mut _cmd, temp_dir, test_files, _test_output) = auxiliary::setup_integration_test(false);

    // the same file should be identical
    verify_file_binary(&test_files.read1, &test_files.read2).unwrap();

    temp_dir.close().unwrap();
}

// Yep, verify_file_contents() does its job. Ready to rumble!



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

    temp_dir.close()?;
    Ok(())
}

#[test]
fn external_corrects_read_numbers_in_output() -> TestResult {
    let (mut cmd, temp_dir, test_files, test_output) = auxiliary::setup_integration_test(true);
    cmd.arg("external")
        .arg("--in")
        .arg(test_files.read1)
        .arg("--in2")
        .arg(test_files.read2)
        .arg("--umi")
        .arg(test_files.umi)
        .arg("--correct_numbers");

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
        &reference.corrected_read1,
    )?;

    verify_file_contents(
        &temp_dir.child("read2_with_UMIs.fq").to_path_buf(),
        &reference.corrected_read2,
    )?;

    temp_dir.close()?;
    Ok(())
}

#[test]
fn external_underscore_delimiter() -> TestResult {
    let (mut cmd, temp_dir, test_files, test_output) = auxiliary::setup_integration_test(true);
    cmd.arg("external")
        .arg("--in")
        .arg(test_files.read1)
        .arg("--in2")
        .arg(test_files.read2)
        .arg("--umi")
        .arg(test_files.umi)
        .arg("--delim")
        .arg("_");

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
        &reference.delim_underscore_read1,
    )?;

    verify_file_contents(
        &temp_dir.child("read2_with_UMIs.fq").to_path_buf(),
        &reference.delim_underscore_read2,
    )?;

    temp_dir.close()?;
    Ok(())
}

// Not really a serious test, but one can also integrate the read sequence into the UMI header if needed ;-)

#[test]
fn external_switch_umi_and_read2() -> TestResult {
    let (mut cmd, temp_dir, test_files, test_output) = auxiliary::setup_integration_test(true);
    cmd.arg("external")
        .arg("--in")
        .arg(test_files.read1)
        .arg("--in2")
        .arg(test_files.umi)
        .arg("--umi")
        .arg(test_files.read2);

    cmd.assert().success(); //further assertions have been tested in other tests

    temp_dir
        .child("read1_with_UMIs.fq")
        .assert(predicate::path::exists());

    temp_dir
        .child("umi_with_UMIs.fq")
        .assert(predicate::path::exists());

    let reference = test_output.unwrap();

    verify_file_contents(
        &temp_dir.child("read1_with_UMIs.fq").to_path_buf(),
        &reference.umi_read2_switch_read1,
    )?;

    verify_file_contents(
        &temp_dir.child("umi_with_UMIs.fq").to_path_buf(),
        &reference.umi_read2_switch_read2,
    )?;

    temp_dir.close()?;
    Ok(())
}

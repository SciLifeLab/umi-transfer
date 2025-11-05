use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[path = "auxiliary.rs"]
mod auxiliary;

#[test]
fn main_without_arguments_prints_help() {
    let mut cmd = cargo_bin_cmd!();
    // Clap prints help to stderr in this case, but to stdout with -h or --help.
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Usage:"))
        .stderr(predicate::str::contains("Commands:"))
        .stderr(predicate::str::contains("Options:"));
}

#[test]
fn main_help_prints_help() {
    let mut cmd = cargo_bin_cmd!();

    // Clap prints help to stdout with -h or --help.
    for help in &["-h", "--help"] {
        cmd.arg(help)
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"))
            .stdout(predicate::str::contains("Commands:"))
            .stdout(predicate::str::contains("Options:"));
    }
}

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[path = "auxiliary.rs"]
mod auxiliary;

#[test]
fn main_without_arguments_prints_help() {
    let mut cmd = Command::cargo_bin(assert_cmd::pkg_name!()).unwrap();
    // Clap prints help to stderr in this case, but to stdout with -h or --help.
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Usage:"))
        .stderr(predicate::str::contains("Commands:"))
        .stderr(predicate::str::contains("Options:"));
}

#[test]
fn main_help_prints_help() {
    let mut cmd = Command::cargo_bin(assert_cmd::pkg_name!()).unwrap();

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

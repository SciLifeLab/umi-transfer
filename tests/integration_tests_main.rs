use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[path = "auxiliary.rs"]
mod auxiliary;

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

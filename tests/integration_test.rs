//! Integration tests for hcscoder CLI
//!
//! These tests verify the end-to-end functionality of the hcscoder command-line interface.

use assert_cmd::Command;
use predicates::prelude::*;
use std::process::Command as StdCommand;

#[test]
fn test_cli_help_flag() {
    let mut cmd = Command::cargo_bin("hcscoder").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Commands:"));
}

#[test]
fn test_cli_version_flag() {
    let mut cmd = Command::cargo_bin("hcscoder").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("hcscoder"));
}

#[test]
fn test_cli_setup_help() {
    let mut cmd = Command::cargo_bin("hcscoder-setup").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("hcscoder-setup"));
}

#[test]
fn test_cli_invalid_command() {
    let mut cmd = Command::cargo_bin("hcscoder").unwrap();
    cmd.arg("nonexistent-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

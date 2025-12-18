use std::process::Command;
use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("gas").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Git Account Switcher"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("gas").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.3.0")); // main.rs の定義に合わせる
}

#[test]
fn test_cli_list_no_accounts_hint() {
    let mut cmd = Command::cargo_bin("gas").unwrap();
    cmd.arg("list")
        .assert()
        .success()
        .stderr(predicate::str::contains("--- Accounts ---"));
}

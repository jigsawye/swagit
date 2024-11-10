use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help() {
  Command::cargo_bin("sg")
    .unwrap()
    .arg("--help")
    .assert()
    .success()
    .stdout(predicate::str::contains(
      "A swag tool to use git with interactive cli",
    ));
}

#[test]
fn test_version() {
  Command::cargo_bin("sg")
    .unwrap()
    .arg("--version")
    .assert()
    .success()
    .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_invalid_flag() {
  Command::cargo_bin("sg")
    .unwrap()
    .arg("--invalid-flag")
    .assert()
    .failure();
}

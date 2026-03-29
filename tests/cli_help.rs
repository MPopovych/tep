use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn prints_help() {
    let mut cmd = Command::cargo_bin("tep").expect("binary should build");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("text entity pointers"));
}

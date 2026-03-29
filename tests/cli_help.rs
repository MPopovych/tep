use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn prints_help() {
    let mut cmd = Command::cargo_bin("tep").expect("binary should build");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("text entity pointers"))
        .stdout(predicate::str::contains("version"))
        .stdout(predicate::str::contains("entity"))
        .stdout(predicate::str::contains("anchor"))
        .stdout(predicate::str::contains("attach"))
        .stdout(predicate::str::contains("detach"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("e"))
        .stdout(predicate::str::contains("doctor").not())
        .stdout(predicate::str::contains("scan").not())
        .stdout(predicate::str::contains("resolve").not())
        .stdout(predicate::str::contains("graph").not())
        .stdout(predicate::str::contains("context").not())
        .stdout(predicate::str::contains("status").not())
        .stdout(predicate::str::contains("link").not());
}

#[test]
fn prints_version_subcommand() {
    let mut cmd = Command::cargo_bin("tep").expect("binary should build");
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

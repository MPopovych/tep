use assert_cmd::Command;
use assert_fs::assert::PathAssert;
use assert_fs::fixture::PathChild;
use predicates::prelude::*;

#[test]
fn init_creates_workspace_files() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");

    let mut cmd = Command::cargo_bin("tep").expect("binary should build");
    cmd.current_dir(temp.path());
    cmd.arg("init");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialized empty tep workspace"));

    temp.child(".tep").assert(predicates::path::is_dir());
    temp.child(".tep/tep.db")
        .assert(predicates::path::exists());
    temp.child(".tep_ignore")
        .assert(predicates::path::is_file());
}

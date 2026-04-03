use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_auto_runs_entity_and_anchor_sync() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("note.txt"),
        "#!#tep:(student){description=\"A learner\"}\n#!#tep:[student_anchor](student){description=\"Anchor desc\"}\n",
    )
    .expect("should write file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["auto", "./note.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("entity auto complete"))
        .stdout(predicate::str::contains("anchor sync complete"));

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "show", "student_anchor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("student"));
}

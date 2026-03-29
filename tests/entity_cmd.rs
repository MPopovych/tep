use assert_cmd::Command;
use assert_fs::assert::PathAssert;
use assert_fs::fixture::PathChild;
use predicates::prelude::*;

#[test]
fn entity_create_prints_created_entity() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args([
            "entity",
            "create",
            "student",
            "--ref",
            "./docs/student.md",
            "--description",
            "A learner enrolled in the system",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("created"))
        .stdout(predicate::str::contains("(student)"))
        .stdout(predicate::str::contains("description: A learner enrolled in the system"));

    temp.child(".tep/tep.db")
        .assert(predicates::path::exists());
}

#[test]
fn entity_command_fails_cleanly_outside_workspace() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no tep workspace found"))
        .stderr(predicate::str::contains("tep init"));
}

#[test]
fn entity_link_and_unlink_work() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["entity", "create", "Student"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["entity", "create", "Subject"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "entity", "link", "Student", "Subject", "--relation",
            "student has subjects assigned to him each semester",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("linked"))
        .stdout(predicate::str::contains("student has subjects assigned to him each semester"));

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "show", "Student"])
        .assert()
        .success()
        .stdout(predicate::str::contains("links:"))
        .stdout(predicate::str::contains("Subject"));

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "unlink", "Student", "Subject"])
        .assert()
        .success()
        .stdout(predicate::str::contains("unlinked"));
}

#[test]
fn entity_context_shows_ref_snippet_and_files() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "hello\n[#!#tep:](student)\n")
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
        .args([
            "entity",
            "create",
            "student",
            "--ref",
            "./docs/student.md",
            "--description",
            "A learner",
        ])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "auto", "./note.txt"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "context", "student"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ref:"))
        .stdout(predicate::str::contains("./docs/student.md"))
        .stdout(predicate::str::contains("description: A learner"))
        .stdout(predicate::str::contains("anchor "))
        .stdout(predicate::str::contains("snippet:"))
        .stdout(predicate::str::contains("[#!#1#tep:"))
        .stdout(predicate::str::contains("files:"))
        .stdout(predicate::str::contains("note.txt"));
}

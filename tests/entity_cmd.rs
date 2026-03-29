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
        .args(["entity", "create", "student", "--ref", "./docs/student.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("created"))
        .stdout(predicate::str::contains("(student)"));

    temp.child(".tep/tep.db")
        .assert(predicates::path::exists());
}

#[test]
fn entity_ensure_is_idempotent() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    let first = Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["e", "ensure", "student"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let second = Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "ensure", "student"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(first, second);
}

#[test]
fn entity_show_and_edit_work_by_name() {
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
        .args(["entity", "create", "student"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args([
            "entity",
            "edit",
            "student",
            "--name",
            "student.profile",
            "--ref",
            "./docs/profile.md",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("updated"))
        .stdout(predicate::str::contains("student.profile"));

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "show", "student.profile"])
        .assert()
        .success()
        .stdout(predicate::str::contains("student.profile"));
}

#[test]
fn entity_show_includes_related_anchors() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "[#!#tep:](student)")
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
        .args(["anchor", "auto", "./note.txt"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "show", "student"])
        .assert()
        .success()
        .stdout(predicate::str::contains("student"))
        .stdout(predicate::str::contains("./note.txt (1:"));
}

#[test]
fn entity_list_shows_entities() {
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
        .args(["entity", "create", "student"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "create", "student.permissions"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("student.permissions"))
        .stdout(predicate::str::contains("student"));
}

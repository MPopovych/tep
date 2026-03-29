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
fn entity_auto_creates_entity_fills_ref_and_rewrites_declaration() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "(#!#tep:Student)")
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
        .args(["entity", "auto", "./note.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("entity auto complete"))
        .stdout(predicate::str::contains("declarations_seen: 1"))
        .stdout(predicate::str::contains("refs_filled: 1"));

    let updated = std::fs::read_to_string(temp.path().join("note.txt"))
        .expect("should read file");
    assert!(updated.contains("(#!#1#tep:Student)"));

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "show", "Student"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(Student)"))
        .stdout(predicate::str::contains("./note.txt (1:"));
}

#[test]
fn entity_auto_does_not_overwrite_existing_ref() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "(#!#tep:Student)")
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
        .args(["entity", "create", "Student", "--ref", "./docs/student.md"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "auto", "./note.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("refs_filled: 0"));

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "show", "Student"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(Student)"));
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

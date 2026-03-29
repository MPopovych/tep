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
fn entity_show_includes_incoming_and_outgoing_links() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["entity", "create", "Student"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["entity", "create", "Subject"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["entity", "create", "Teacher"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "entity", "link", "Student", "Subject", "--relation",
            "student has subjects assigned to him each semester",
        ])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "entity", "link", "Teacher", "Student", "--relation",
            "teacher mentors student",
        ])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "show", "Student"])
        .assert()
        .success()
        .stdout(predicate::str::contains("outgoing links:"))
        .stdout(predicate::str::contains("incoming links:"))
        .stdout(predicate::str::contains("Subject"))
        .stdout(predicate::str::contains("Teacher"))
        .stdout(predicate::str::contains("student has subjects assigned to him each semester"))
        .stdout(predicate::str::contains("teacher mentors student"));
}

#[test]
fn entity_context_can_include_linked_entities() {
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
        .args([
            "entity",
            "create",
            "subject",
            "--ref",
            "./docs/subject.md",
            "--description",
            "A course",
        ])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args([
            "entity",
            "create",
            "teacher",
            "--ref",
            "./docs/teacher.md",
            "--description",
            "An instructor",
        ])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["anchor", "auto", "./note.txt"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "link", "student", "subject", "--relation", "student has subjects"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "link", "teacher", "student", "--relation", "teacher mentors student"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "context", "student", "--include-links"])
        .assert()
        .success()
        .stdout(predicate::str::contains("outgoing linked entities:"))
        .stdout(predicate::str::contains("incoming linked entities:"))
        .stdout(predicate::str::contains("subject"))
        .stdout(predicate::str::contains("teacher"))
        .stdout(predicate::str::contains("./docs/subject.md"))
        .stdout(predicate::str::contains("./docs/teacher.md"))
        .stdout(predicate::str::contains("student has subjects"))
        .stdout(predicate::str::contains("teacher mentors student"));
}

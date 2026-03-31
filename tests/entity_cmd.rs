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
            "Student",
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
fn entity_list_shows_one_line_intro_format() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "create", "Student", "--ref", "./docs/student.md", "--description", "A learner"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "create", "Teacher", "--description", "An instructor"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 student - \"A learner\" (./docs/student.md)"))
        .stdout(predicate::str::contains("2 teacher - \"An instructor\""));
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
        .args(["entity", "show", "student"])
        .assert()
        .success()
        .stdout(predicate::str::contains("outgoing links:"))
        .stdout(predicate::str::contains("incoming links:"))
        .stdout(predicate::str::contains("subject"))
        .stdout(predicate::str::contains("teacher"))
        .stdout(predicate::str::contains("student has subjects assigned to him each semester"))
        .stdout(predicate::str::contains("teacher mentors student"));
}

#[test]
fn entity_context_includes_all_link_directions_by_default() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "hello\n[#!#tep:](student)\n")
        .expect("should write file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    for args in [
        vec!["entity", "create", "student", "--ref", "./docs/student.md", "--description", "A learner"],
        vec!["entity", "create", "subject", "--ref", "./docs/subject.md", "--description", "A course"],
        vec!["entity", "create", "teacher", "--ref", "./docs/teacher.md", "--description", "An instructor"],
    ] {
        Command::cargo_bin("tep")
            .expect("binary should build")
            .current_dir(temp.path())
            .args(args)
            .assert()
            .success();
    }

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["anchor", "auto", "./note.txt"])
        .assert()
        .success();

    for args in [
        vec!["entity", "link", "student", "subject", "--relation", "student has subjects"],
        vec!["entity", "link", "teacher", "student", "--relation", "teacher mentors student"],
    ] {
        Command::cargo_bin("tep")
            .unwrap()
            .current_dir(temp.path())
            .args(args)
            .assert()
            .success();
    }

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "context", "student"])
        .assert()
        .success()
        .stdout(predicate::str::contains("linked entities:"))
        .stdout(predicate::str::contains("edge: (1->2)[1] student has subjects"))
        .stdout(predicate::str::contains("edge: (3->1)[1] teacher mentors student"))
        .stdout(predicate::str::contains("./docs/subject.md"))
        .stdout(predicate::str::contains("./docs/teacher.md"));
}

#[test]
fn entity_create_rejects_empty_name() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "create", "   "])
        .assert()
        .failure()
        .stderr(predicate::str::contains("entity name cannot be empty"));
}

#[test]
fn entity_create_rejects_invalid_name_characters() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "create", "basic-user"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("entity name may only contain lowercase letters, numbers, dots, and underscores"));
}

#[test]
fn entity_create_rejects_description_quotes() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "create", "student", "--description", "A \"learner\""])
        .assert()
        .failure()
        .stderr(predicate::str::contains("entity description cannot contain quotes"));
}

#[test]
fn entity_ensure_creates_entity_if_absent() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "ensure", "student", "--ref", "./docs/student.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(student)"));
}

#[test]
fn entity_ensure_is_idempotent() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "ensure", "student"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "ensure", "student"])
        .assert()
        .success();

    let out = Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(out).unwrap();
    assert_eq!(output.lines().filter(|l| l.contains("student")).count(), 1);
}

#[test]
fn entity_edit_success() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["entity", "create", "student"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "edit", "student", "--description", "A learner in the system"])
        .assert()
        .success()
        .stdout(predicate::str::contains("updated"))
        .stdout(predicate::str::contains("description: A learner in the system"));
}

#[test]
fn entity_unlink_removes_link() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["entity", "create", "student"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["entity", "create", "subject"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "link", "student", "subject", "--relation", "student attends subject"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "unlink", "student", "subject"])
        .assert()
        .success()
        .stdout(predicate::str::contains("unlinked"));

    // after unlink, student show should not mention subject at all
    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "show", "student"])
        .assert()
        .success()
        .stdout(predicate::str::contains("subject").not());
}

#[test]
fn entity_context_files_only_omits_snippets() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "hello world\n[#!#tep:](student)\n")
        .expect("should write file");

    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path())
        .args(["entity", "create", "student", "--ref", "./docs/student.md"])
        .assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path())
        .args(["anchor", "auto", "./note.txt"])
        .assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "context", "student", "--files-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("./note.txt"))
        .stdout(predicate::str::contains("hello world").not());
}

#[test]
fn entity_list_is_empty_on_fresh_workspace() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "list"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn entity_shorthand_alias_works() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["e", "create", "student"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(student)"));
}

#[test]
fn entity_create_rejects_duplicate_name() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["entity", "create", "student"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "create", "student"])
        .assert()
        .failure();
}

#[test]
fn entity_edit_requires_at_least_one_field() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["entity", "create", "student"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "edit", "student"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("entity edit requires at least one field to update"));
}

#[test]
fn entity_show_reports_missing_entity() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "show", "missing"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("entity not found: missing"));
}

#[test]
fn entity_link_rejects_empty_relation() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["entity", "create", "student"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["entity", "create", "subject"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "link", "student", "subject", "--relation", "   "])
        .assert()
        .failure()
        .stderr(predicate::str::contains("relation cannot be empty"));
}

#[test]
fn entity_link_rejects_self_link() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["entity", "create", "student"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["entity", "link", "student", "student", "--relation", "self link"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("entity cannot link to itself"));
}

#[test]
fn entity_context_can_expand_link_depth() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "hello\n[#!#tep:](student)\n")
        .expect("should write file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    for args in [
        vec!["entity", "create", "student", "--ref", "./docs/student.md", "--description", "A learner"],
        vec!["entity", "create", "subject", "--ref", "./docs/subject.md", "--description", "A course"],
        vec!["entity", "create", "semester", "--ref", "./docs/semester.md", "--description", "A term"],
        vec!["entity", "create", "teacher", "--ref", "./docs/teacher.md", "--description", "An instructor"],
        vec!["entity", "create", "department", "--ref", "./docs/department.md", "--description", "An org unit"],
    ] {
        Command::cargo_bin("tep")
            .expect("binary should build")
            .current_dir(temp.path())
            .args(args)
            .assert()
            .success();
    }

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["anchor", "auto", "./note.txt"])
        .assert()
        .success();

    for args in [
        vec!["entity", "link", "student", "subject", "--relation", "student has subjects"],
        vec!["entity", "link", "subject", "semester", "--relation", "subject is scheduled in semester"],
        vec!["entity", "link", "semester", "student", "--relation", "semester contains student records"],
        vec!["entity", "link", "teacher", "student", "--relation", "teacher mentors student"],
        vec!["entity", "link", "department", "teacher", "--relation", "department employs teacher"],
    ] {
        Command::cargo_bin("tep")
            .unwrap()
            .current_dir(temp.path())
            .args(args)
            .assert()
            .success();
    }

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "context", "student", "--link-depth", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("linked entities:"))
        .stdout(predicate::str::contains("subject"))
        .stdout(predicate::str::contains("semester"))
        .stdout(predicate::str::contains("teacher"))
        .stdout(predicate::str::contains("department"))
        .stdout(predicate::str::contains("edge: (1->2)[1] student has subjects"))
        .stdout(predicate::str::contains("edge: (3->1)[1] semester contains student records").or(predicate::str::contains("edge: (4->1)[1] teacher mentors student")))
        .stdout(predicate::str::contains("[2]"))
        .stdout(predicate::str::contains("./docs/department.md"));
}

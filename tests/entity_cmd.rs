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
fn entity_show_works_from_nested_directory() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let nested = temp.path().join("docs/nested");
    std::fs::create_dir_all(&nested).expect("nested dirs should be created");
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
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(&nested)
        .args(["entity", "show", "Student"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(Student)"));
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
fn entity_auto_ignores_line_with_tepignore() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let path = temp.path().join("examples.txt");
    std::fs::write(&path, "example (#!#tep:Student) #tepignore\n")
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
        .args(["entity", "auto", "./examples.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("declarations_seen: 0"))
        .stdout(predicate::str::contains("anchors_created: 0"));

    let updated = std::fs::read_to_string(&path).expect("should read file");
    assert_eq!(updated, "example (#!#tep:Student) #tepignore\n");
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
fn entity_auto_rescan_does_not_duplicate_backing_anchor() {
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
        .stdout(predicate::str::contains("anchors_created: 1"));

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "auto", "./note.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_created: 0"));

    let output = Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "show", "Student"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let rendered = String::from_utf8(output).expect("stdout should be utf8");
    let lines: Vec<&str> = rendered.lines().collect();
    let anchor_id_lines = lines
        .iter()
        .filter(|line| line.chars().all(|c| c.is_ascii_digit()))
        .count();
    assert_eq!(anchor_id_lines, 1);
}

#[test]
fn entity_auto_rescan_after_materialization_shift_stays_stable() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "header\n(#!#tep:Student)\n")
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
        .stdout(predicate::str::contains("anchors_created: 1"));

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "auto", "./note.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_created: 0"));
}

#[test]
fn entity_auto_same_entity_in_two_files_creates_two_anchors() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("one.txt"), "(#!#tep:Student)")
        .expect("should write file");
    std::fs::write(temp.path().join("two.txt"), "(#!#tep:Student)")
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
        .args(["entity", "auto", "./one.txt", "./two.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_created: 2"));

    let output = Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "show", "Student"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let rendered = String::from_utf8(output).expect("stdout should be utf8");
    assert!(rendered.contains("./one.txt"));
    assert!(rendered.contains("./two.txt"));
}

#[test]
fn entity_auto_two_entities_in_same_file_stay_distinct_on_rescan() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("note.txt"),
        "(#!#tep:Student)\n(#!#tep:Project)\n",
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
        .args(["entity", "auto", "./note.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_created: 2"));

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "auto", "./note.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_created: 0"));

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "show", "Student"])
        .assert()
        .success()
        .stdout(predicate::str::contains("./note.txt"));

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "show", "Project"])
        .assert()
        .success()
        .stdout(predicate::str::contains("./note.txt"));
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

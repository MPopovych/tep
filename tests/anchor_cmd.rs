use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn anchor_command_fails_cleanly_outside_workspace() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "#!#tep:[my_anchor](student)")
        .expect("should write file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "auto", "./note.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no tep workspace found"))
        .stderr(predicate::str::contains("tep init"));
}

#[test]
fn health_command_reports_workspace_anchor_issues() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let path = temp.path().join("note.txt");
    std::fs::write(&path, "hello #!#tep:[my_anchor](student)").expect("should write file");

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
        .args(["health"])
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace health report"))
        .stdout(predicate::str::contains("anchors_healthy: 1"));

    let updated = std::fs::read_to_string(&path).expect("should read file");
    std::fs::write(&path, updated.replace("hello ", "hello world ")).expect("should rewrite file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["health"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_moved: 1"));
}

#[test]
fn anchor_auto_registers_named_anchor() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("note.txt"),
        "hello #!#tep:[my_anchor](student)",
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
        .args(["anchor", "auto", "./note.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchor sync complete"))
        .stdout(predicate::str::contains("anchors_created: 1"))
        .stdout(predicate::str::contains("anchors_dropped: 0"))
        .stdout(predicate::str::contains("relations_synced: 1"));

    // File should not be rewritten — tag is already in final format
    let updated = std::fs::read_to_string(temp.path().join("note.txt")).expect("should read file");
    assert!(updated.contains("#!#tep:[my_anchor](student)"));
}

#[test]
fn anchor_auto_ignores_line_with_tepignore() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let path = temp.path().join("examples.txt");
    std::fs::write(&path, "example #!#tep:[my_anchor](student) #tepignore\n")
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
        .args(["anchor", "auto", "./examples.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_seen: 0"))
        .stdout(predicate::str::contains("anchors_created: 0"));

    let updated = std::fs::read_to_string(&path).expect("should read file");
    assert_eq!(updated, "example #!#tep:[my_anchor](student) #tepignore\n");
}

#[test]
fn anchor_auto_ignores_anchor_without_entity_refs() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let path = temp.path().join("note.txt");
    std::fs::write(&path, "#!#tep:[my_anchor]").expect("should write file");

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
        .success()
        .stdout(predicate::str::contains("anchors_created: 0"));

    // File should be unchanged
    let updated = std::fs::read_to_string(&path).expect("should read file");
    assert_eq!(updated, "#!#tep:[my_anchor]");
}

#[test]
fn anchor_show_returns_compact_format() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "#!#tep:[my_anchor](student)")
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
        .args(["anchor", "show", "my_anchor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("note.txt"))
        .stdout(predicate::str::contains("(student)"));
}

#[test]
fn anchor_show_works_from_nested_directory() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let nested = temp.path().join("src/deep");
    std::fs::create_dir_all(&nested).expect("nested dirs should be created");
    std::fs::write(temp.path().join("note.txt"), "#!#tep:[my_anchor](student)")
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
        .current_dir(&nested)
        .args(["anchor", "show", "my_anchor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(student)"));
}

#[test]
fn anchor_auto_handles_unicode_prefix_text() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("unicode.txt"),
        "żółw 🐢\n#!#tep:[my_anchor](student)",
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
        .args(["anchor", "auto", "./unicode.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_created: 1"));
}

#[test]
fn anchor_auto_ignores_malformed_anchor_like_text() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let path = temp.path().join("malformed.txt");
    // dash in name is invalid; unclosed entity ref is also invalid
    std::fs::write(
        &path,
        "#!#tep:[abc-def](student)\n#!#tep:[my_anchor](student\n",
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
        .args(["anchor", "auto", "./malformed.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_seen: 0"))
        .stdout(predicate::str::contains("anchors_created: 0"));

    let updated = std::fs::read_to_string(&path).expect("should read file");
    assert_eq!(
        updated,
        "#!#tep:[abc-def](student)\n#!#tep:[my_anchor](student\n"
    );
}

#[test]
fn anchor_auto_drops_removed_anchor() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let path = temp.path().join("drop.txt");
    std::fs::write(&path, "#!#tep:[my_anchor](student)").expect("should write file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "auto", "./drop.txt"])
        .assert()
        .success();

    std::fs::write(&path, "no anchors now\n").expect("should rewrite file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "auto", "./drop.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_dropped: 1"));
}

#[test]
fn anchor_auto_fails_for_duplicate_anchor_names_in_same_file() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("dup.txt"),
        "#!#tep:[my_anchor](student)\n#!#tep:[my_anchor](teacher)\n",
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
        .args(["anchor", "auto", "./dup.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "duplicate anchor name 'my_anchor'",
        ));
}

#[test]
fn anchor_show_reports_missing_anchor() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["anchor", "show", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("anchor not found: nonexistent"));
}

#[test]
fn anchor_auto_is_idempotent() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "#!#tep:[my_anchor](student)")
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
        .success()
        .stdout(predicate::str::contains("anchors_created: 1"));

    // second run should create nothing new
    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "auto", "./note.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_created: 0"));
}

#[test]
fn health_reports_missing_anchor_when_tag_removed_from_file() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let path = temp.path().join("note.txt");
    std::fs::write(&path, "#!#tep:[my_anchor](student)").expect("should write file");

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

    // overwrite without anchor tag
    std::fs::write(&path, "no anchors here\n").expect("should overwrite file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["health"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_missing: 1"));
}

#[test]
fn anchor_show_resolves_by_name() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("note.txt"),
        "#!#tep:[student_processor](student)",
    )
    .expect("should write file");

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();
    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["anchor", "auto", "./note.txt"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "show", "student_processor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("student_processor"))
        .stdout(predicate::str::contains("note.txt"));
}

#[test]
fn anchor_list_shows_all_anchors() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("a.txt"), "#!#tep:[proc_a](student)")
        .expect("should write file");
    std::fs::write(temp.path().join("b.txt"), "#!#tep:[proc_b](student)")
        .expect("should write file");

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();
    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["anchor", "auto", "./a.txt"])
        .assert()
        .success();
    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["anchor", "auto", "./b.txt"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("proc_a"))
        .stdout(predicate::str::contains("a.txt"))
        .stdout(predicate::str::contains("b.txt"));
}

#[test]
fn anchor_list_is_empty_on_fresh_workspace() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "list"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn anchor_command_fails_cleanly_outside_workspace() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "[#!#tep:](student)")
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
    std::fs::write(&path, "hello [#!#tep:](student)")
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
        .args(["health"])
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace health report"))
        .stdout(predicate::str::contains("anchors_healthy: 1"));

    let updated = std::fs::read_to_string(&path).expect("should read file");
    std::fs::write(&path, updated.replace("hello ", "hello world "))
        .expect("should rewrite file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["health"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_moved: 1"));
}

#[test]
fn attach_command_fails_cleanly_outside_workspace() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["attach", "student", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no tep workspace found"))
        .stderr(predicate::str::contains("tep init"));
}

#[test]
fn anchor_auto_materializes_incomplete_anchor() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "hello [#!#tep:](student)")
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

    let updated = std::fs::read_to_string(temp.path().join("note.txt"))
        .expect("should read file");
    assert!(updated.contains("[#!#1#tep:"));
}

#[test]
fn anchor_auto_ignores_line_with_tepignore() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let path = temp.path().join("examples.txt");
    std::fs::write(&path, "example [#!#tep:](student) #tepignore\n")
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
    assert_eq!(updated, "example [#!#tep:](student) #tepignore\n");
}

#[test]
fn anchor_show_returns_compact_format() {
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

    let updated = std::fs::read_to_string(temp.path().join("note.txt"))
        .expect("should read file");
    let id_start = updated.find("tep:").expect("materialized anchor should exist") + 4;
    let id_end = updated[id_start..].find(']').expect("anchor id should end") + id_start;
    let anchor_id = &updated[id_start..id_end];

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "show", anchor_id])
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

    let updated = std::fs::read_to_string(temp.path().join("note.txt"))
        .expect("should read file");
    let id_start = updated.find("tep:").expect("materialized anchor should exist") + 4;
    let id_end = updated[id_start..].find(']').expect("anchor id should end") + id_start;
    let anchor_id = &updated[id_start..id_end];

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(&nested)
        .args(["anchor", "show", anchor_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("(student)"));
}

#[test]
fn anchor_auto_handles_unicode_prefix_text() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("unicode.txt"), "żółw 🐢\n[#!#tep:](student)")
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

    let updated = std::fs::read_to_string(temp.path().join("unicode.txt"))
        .expect("should read file");
    assert!(updated.contains("[#!#1#tep:"));
}

#[test]
fn anchor_auto_ignores_malformed_anchor_like_text() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let path = temp.path().join("malformed.txt");
    // dash in name is invalid; unclosed entity ref is also invalid
    std::fs::write(&path, "[#!#1#tep:abc-def](student)\n[#!#tep:](student\n")
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
    assert_eq!(updated, "[#!#1#tep:abc-def](student)\n[#!#tep:](student\n");
}

#[test]
fn manual_attach_and_detach_work() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "[#!#tep:]")
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

    let updated = std::fs::read_to_string(temp.path().join("note.txt"))
        .expect("should read file");
    let id_start = updated.find("tep:").expect("materialized anchor should exist") + 4;
    let id_end = updated[id_start..].find(']').expect("anchor id should end") + id_start;
    let anchor_id = &updated[id_start..id_end];

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["attach", "student", anchor_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("attached"));

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["detach", "student", anchor_id])
        .assert()
        .success()
        .stdout(predicate::str::contains("detached"));
}

#[test]
fn anchor_auto_fails_cleanly_for_unknown_materialized_anchor() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("broken.txt"), "[#!#1#tep:999](student)")
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
        .args(["anchor", "auto", "./broken.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("materialized anchor 999 was found in a file but does not exist in the database"));
}

#[test]
fn anchor_auto_drops_removed_anchor() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let path = temp.path().join("drop.txt");
    std::fs::write(&path, "[#!#tep:]").expect("should write file");

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
fn anchor_auto_fails_for_duplicate_materialized_anchor_ids_in_same_file() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("dup.txt"), "[#!#1#tep:5]\n[#!#1#tep:5]\n")
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
        .stderr(predicate::str::contains("duplicate materialized anchor 5 found in the same file"));
}

#[test]
fn anchor_show_reports_missing_anchor() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["anchor", "show", "999"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("anchor not found: 999"));
}

#[test]
fn attach_reports_missing_anchor_after_workspace_init() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();

    Command::cargo_bin("tep")
        .unwrap()
        .current_dir(temp.path())
        .args(["attach", "student", "999"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("anchor not found: 999"));
}

#[test]
fn anchor_auto_is_idempotent() {
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
        .success()
        .stdout(predicate::str::contains("anchors_created: 1"));

    // second run on the already-materialized file should create nothing new
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
    std::fs::write(&path, "[#!#tep:](student)").expect("should write file");

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

    // overwrite the file without the anchor tag so the ID disappears
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
fn anchor_auto_materializes_named_anchor_from_file() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "[#!#1#tep:student_processor](student)")
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

    // idempotent on re-run
    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "auto", "./note.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_created: 0"));
}

#[test]
fn anchor_show_resolves_by_name() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "[#!#1#tep:student_processor](student)")
        .expect("should write file");

    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path())
        .args(["anchor", "auto", "./note.txt"]).assert().success();

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
fn anchor_edit_sets_name_and_rewrites_file() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "hello [#!#tep:](student)")
        .expect("should write file");

    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path())
        .args(["anchor", "auto", "./note.txt"]).assert().success();

    // find the assigned numeric id from the materialized file
    let materialized = std::fs::read_to_string(temp.path().join("note.txt")).unwrap();
    let id_start = materialized.find("tep:").unwrap() + 4;
    let id_end = materialized[id_start..].find(']').unwrap() + id_start;
    let anchor_id = &materialized[id_start..id_end];

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "edit", anchor_id, "--name", "student_processor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("student_processor"));

    let updated = std::fs::read_to_string(temp.path().join("note.txt")).unwrap();
    assert!(updated.contains("[#!#1#tep:student_processor]"), "file should contain named tag, got: {}", updated);
}

#[test]
fn anchor_edit_rejects_name_collision() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("a.txt"), "[#!#1#tep:proc_a](student)")
        .expect("should write file");
    std::fs::write(temp.path().join("b.txt"), "[#!#tep:](student)")
        .expect("should write file");

    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path())
        .args(["anchor", "auto", "./a.txt"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path())
        .args(["anchor", "auto", "./b.txt"]).assert().success();

    let materialized = std::fs::read_to_string(temp.path().join("b.txt")).unwrap();
    let id_start = materialized.find("tep:").unwrap() + 4;
    let id_end = materialized[id_start..].find(']').unwrap() + id_start;
    let anchor_id_b = &materialized[id_start..id_end];

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "edit", anchor_id_b, "--name", "proc_a"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("proc_a"));
}

#[test]
fn attach_resolves_anchor_by_name() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "[#!#1#tep:student_processor]")
        .expect("should write file");

    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path())
        .args(["anchor", "auto", "./note.txt"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path())
        .args(["entity", "create", "student"]).assert().success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["attach", "student", "student_processor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("attached"))
        .stdout(predicate::str::contains("student_processor"));
}

#[test]
fn anchor_list_shows_all_anchors() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("a.txt"), "[#!#1#tep:proc_a](student)")
        .expect("should write file");
    std::fs::write(temp.path().join("b.txt"), "[#!#tep:](student)")
        .expect("should write file");

    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path())
        .args(["anchor", "auto", "./a.txt"]).assert().success();
    Command::cargo_bin("tep").unwrap().current_dir(temp.path())
        .args(["anchor", "auto", "./b.txt"]).assert().success();

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
    Command::cargo_bin("tep").unwrap().current_dir(temp.path()).args(["init"]).assert().success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "list"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn anchor_auto_fails_for_cross_file_anchor_id_conflict() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("one.txt"), "[#!#tep:]").expect("should write file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "auto", "./one.txt"])
        .assert()
        .success();

    let one = std::fs::read_to_string(temp.path().join("one.txt")).expect("should read file");
    let id_start = one.find("tep:").unwrap() + 4;
    let id_end = one[id_start..].find(']').unwrap() + id_start;
    let anchor_id = &one[id_start..id_end];

    std::fs::write(
        temp.path().join("two.txt"),
        format!("[#!#1#tep:{}](student)", anchor_id),
    )
    .expect("should write conflicting file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "auto", "./two.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot also be updated from"));
}

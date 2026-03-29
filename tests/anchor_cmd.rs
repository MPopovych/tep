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
        .stdout(predicate::str::contains("./note.txt (1:"))
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
    std::fs::write(&path, "[#!#1#tep:abc](student)\n[#!#tep:](student\n")
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
    assert_eq!(updated, "[#!#1#tep:abc](student)\n[#!#tep:](student\n");
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

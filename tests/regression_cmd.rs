use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn health_is_consistent_from_nested_directory() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::create_dir_all(temp.path().join("src/deep")).expect("nested dirs should be created");
    std::fs::write(
        temp.path().join("note.txt"),
        "#!#tep:(student){description=\"A learner\"}\n#!#tep:[student_anchor](student)\n",
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
        .success();

    let root = Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["health"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let nested = Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path().join("src/deep"))
        .args(["health"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(
        String::from_utf8(root).unwrap(),
        String::from_utf8(nested).unwrap()
    );
}

#[test]
fn non_tty_output_does_not_emit_ansi_sequences() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("note.txt"),
        "#!#tep:(student){description=\"A learner\"}\n#!#tep:[student_anchor](student)\n",
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
        .success();

    let output = Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "show", "student"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let rendered = String::from_utf8(output).unwrap();
    assert!(!rendered.contains("\u{1b}["));
}

#[test]
fn auto_warns_and_skips_unreadable_files() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("note.txt"),
        "#!#tep:(student){description=\"A learner\"}\n",
    )
    .expect("should write file");
    std::fs::write(temp.path().join("binary.bin"), [0_u8, 159, 146, 150])
        .expect("should write binary file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["auto", "."])
        .assert()
        .success()
        .stdout(predicate::str::contains("warnings:"))
        .stdout(predicate::str::contains("skipping unreadable file"));
}

#[test]
fn reset_warns_and_continues_on_duplicate_anchor_names() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("good.md"),
        "#!#tep:(qa.flow){description=\"Flow\"}\n#!#tep:[qa.entry](qa.flow)\n",
    )
    .expect("should write good file");
    std::fs::write(
        temp.path().join("bad.md"),
        "#!#tep:[dup](qa.flow)\n#!#tep:[dup](qa.flow)\n",
    )
    .expect("should write bad file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["reset", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("warnings:"))
        .stdout(predicate::str::contains(
            "duplicate anchor name 'dup' found in the same file",
        ));

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("qa.flow"));
}

#[test]
fn health_check_exits_zero_when_clean() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("note.txt"),
        "#!#tep:(student){description=\"A learner\"}\n#!#tep:[student_anchor](student)\n",
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
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["health", "--check"])
        .assert()
        .success();
}

#[test]
fn health_check_exits_non_zero_when_issues_exist() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("bad.md"),
        "#!#tep:[dup](student)\n#!#tep:[dup](student)\n",
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
        .args(["health", "--check"])
        .assert()
        .failure();
}

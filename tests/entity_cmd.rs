use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn entity_command_fails_cleanly_outside_workspace() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "#!#tep:(student)")
        .expect("should write file");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "auto", "./note.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no tep workspace found"))
        .stderr(predicate::str::contains("tep init"));
}

#[test]
fn entity_auto_declares_entities_and_relations() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("note.txt"),
        "#!#tep:(student){ref=\"./docs/student.md\", description=\"A learner\"}\n#!#tep:(student)->(subject){description=\"has subject\"}\n#!#tep:(subject){description=\"A course\"}\n",
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
        .stdout(predicate::str::contains("declarations_seen: 2"))
        .stdout(predicate::str::contains("relations_seen: 1"))
        .stdout(predicate::str::contains("relations_synced: 1"));
}

#[test]
fn entity_show_includes_metadata_and_links() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("note.txt"),
        "#!#tep:(student){ref=\"./docs/student.md\", description=\"A learner\"}\n#!#tep:(student)->(subject){description=\"has subject\"}\n#!#tep:(subject){description=\"A course\"}\n#!#tep:[student_anchor](student)\n",
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
        .args(["entity", "show", "student"])
        .assert()
        .success()
        .stdout(predicate::str::contains("description: A learner"))
        .stdout(predicate::str::contains("./docs/student.md"))
        .stdout(predicate::str::contains("student_anchor"))
        .stdout(predicate::str::contains("has subject"));
}

#[test]
fn entity_context_files_only_omits_snippets() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("note.txt"),
        "hello\n#!#tep:(student){ref=\"./docs/student.md\"}\n#!#tep:[student_anchor](student)\nworld\n",
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
        .args(["entity", "context", "student", "--files-only"])
        .assert()
        .success()
        .stdout(predicate::str::contains("student_anchor"))
        .stdout(predicate::str::contains("./docs/student.md"))
        .stdout(predicate::str::contains("hello").not());
}

#[test]
fn entity_list_shows_one_line_intro_format() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(
        temp.path().join("note.txt"),
        "#!#tep:(student){ref=\"./docs/student.md\", description=\"A learner\"}\n#!#tep:(teacher){description=\"An instructor\"}\n",
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
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("student - \"A learner\" (./docs/student.md)"))
        .stdout(predicate::str::contains("teacher - \"An instructor\""));
}

#[test]
fn entity_shorthand_alias_works() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "#!#tep:(student)")
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
        .args(["e", "auto", "./note.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("entity auto complete"));
}

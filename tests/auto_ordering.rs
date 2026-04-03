use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn entity_auto_declares_entities_without_creating_anchors() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let source = temp.path().join("src");
    std::fs::create_dir_all(&source).expect("src dir should exist");

    std::fs::write(
        source.join("sample.rs"),
        "// #!#tep:(sample.context)\n// #!#tep:(sample.auto)\npub fn sample() {}\n",
    )
    .expect("sample file should be written");

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["init"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "auto", "src"])
        .assert()
        .success()
        .stdout(predicate::str::contains("declarations_seen: 2"))
        .stdout(predicate::str::contains("entities_ensured: 2"));

    // No anchors should have been created — entity auto only declares entities
    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["anchor", "list"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    // Entities should exist with their ref pointing to the declaring file
    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "show", "sample.context"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sample.context"))
        .stdout(predicate::str::contains("sample.rs"));
}

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn entity_auto_followed_by_health_does_not_drift_anchor_locations() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    let source = temp.path().join("src");
    std::fs::create_dir_all(&source).expect("src dir should exist");

    std::fs::write(
        source.join("sample.rs"),
        concat!(
            "// (#!#tep:sample.context)\n",
            "// [#!#tep:](sample.context,sample.auto)\n",
            "pub fn sample() {}\n",
        ),
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
        .args(["anchor", "auto", "./src/sample.rs"])
        .assert()
        .success();

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "auto", "src"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_created: 1"));

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["health"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchors_moved: 0"))
        .stdout(predicate::str::contains("anchors_missing: 0"));
}

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn prints_help() {
    let mut cmd = Command::cargo_bin("tep").expect("binary should build");
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("text entity pointers"))
        .stdout(predicate::str::contains("version"))
        .stdout(predicate::str::contains("Print the tep version"))
        .stdout(predicate::str::contains("health"))
        .stdout(predicate::str::contains(
            "Audit workspace health and graph integrity",
        ))
        .stdout(predicate::str::contains("entity"))
        .stdout(predicate::str::contains(
            "Work with entities, descriptions, refs, and directional links",
        ))
        .stdout(predicate::str::contains("anchor"))
        .stdout(predicate::str::contains(
            "Work with anchors, names, and anchor-entity attachments",
        ))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains(
            "Initialize a tep workspace in the current directory",
        ))
        .stdout(predicate::str::contains("e"))
        .stdout(predicate::str::contains("Shorthand for entity"))
        .stdout(predicate::str::contains("a"))
        .stdout(predicate::str::contains("Shorthand for anchor"));
}

#[test]
fn prints_anchor_help_with_descriptions() {
    let mut cmd = Command::cargo_bin("tep").expect("binary should build");
    cmd.args(["anchor", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("auto"))
        .stdout(predicate::str::contains("Sync anchors in files"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains(
            "Show one anchor and its related entities",
        ));
}

#[test]
fn prints_entity_help_with_descriptions() {
    let mut cmd = Command::cargo_bin("tep").expect("binary should build");
    cmd.args(["entity", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("Create a new entity"))
        .stdout(predicate::str::contains("ensure"))
        .stdout(predicate::str::contains("Ensure an entity exists"))
        .stdout(predicate::str::contains("auto"))
        .stdout(predicate::str::contains("Auto-declare entities from files"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains(
            "Show one entity and its related anchors and links",
        ))
        .stdout(predicate::str::contains("context"))
        .stdout(predicate::str::contains(
            "Show one entity with snippets, files, and linked entities",
        ))
        .stdout(predicate::str::contains("link"))
        .stdout(predicate::str::contains(
            "Create or update a directional entity link",
        ))
        .stdout(predicate::str::contains("unlink"))
        .stdout(predicate::str::contains("Remove a directional entity link"))
        .stdout(predicate::str::contains("edit"))
        .stdout(predicate::str::contains("Edit an existing entity"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("List entities"));
}

#[test]
fn prints_entity_context_help_with_flags() {
    let mut cmd = Command::cargo_bin("tep").expect("binary should build");
    cmd.args(["entity", "context", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Show one entity with snippets, files, and linked entities",
        ))
        .stdout(predicate::str::contains("--files-only"))
        .stdout(predicate::str::contains(
            "Show files and linked entities without anchor snippets",
        ))
        .stdout(predicate::str::contains("--link-depth"))
        .stdout(predicate::str::contains(
            "Traverse directional links up to this depth",
        ));
}

#[test]
fn prints_health_help_with_path_description() {
    let mut cmd = Command::cargo_bin("tep").expect("binary should build");
    cmd.args(["health", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Audit workspace health and graph integrity",
        ))
        .stdout(predicate::str::contains(
            "File or directory to audit relative to the workspace",
        ));
}

#[test]
fn anchor_shorthand_alias_works() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::write(temp.path().join("note.txt"), "[#!#tep:my_anchor](student)")
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
        .args(["a", "auto", "./note.txt"])
        .assert()
        .success()
        .stdout(predicate::str::contains("anchor sync complete"));
}

#[test]
fn prints_version_subcommand() {
    let mut cmd = Command::cargo_bin("tep").expect("binary should build");
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn prints_short_version_flag() {
    let mut cmd = Command::cargo_bin("tep").expect("binary should build");
    cmd.arg("-V");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn prints_long_version_flag() {
    let mut cmd = Command::cargo_bin("tep").expect("binary should build");
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

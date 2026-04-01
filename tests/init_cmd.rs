use assert_cmd::Command;
use assert_fs::assert::PathAssert;
use assert_fs::fixture::PathChild;
use predicates::prelude::*;

#[test]
fn init_creates_workspace_files() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");

    let mut cmd = Command::cargo_bin("tep").expect("binary should build");
    cmd.current_dir(temp.path());
    cmd.arg("init");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialized empty tep workspace"))
        .stdout(predicate::str::contains("Schema version: 3"));

    temp.child(".tep").assert(predicates::path::is_dir());
    temp.child(".tep/tep.db")
        .assert(predicates::path::exists());
    temp.child(".tepignore")
        .assert(predicates::path::is_file());
}

#[test]
fn legacy_workspace_db_is_migrated_on_access() {
    let temp = assert_fs::TempDir::new().expect("temp dir should be created");
    std::fs::create_dir_all(temp.path().join(".tep")).expect("tep dir should exist");

    let conn = rusqlite::Connection::open(temp.path().join(".tep/tep.db")).expect("db should open");
    conn.execute_batch(
        r#"
        CREATE TABLE entities (
            entity_id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            ref TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        INSERT INTO entities (name, ref, created_at, updated_at)
        VALUES ('student', './docs/student.md', '1', '1');
        "#,
    )
    .expect("legacy schema should be created");
    drop(conn);

    Command::cargo_bin("tep")
        .expect("binary should build")
        .current_dir(temp.path())
        .args(["entity", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("student"));

    let conn = rusqlite::Connection::open(temp.path().join(".tep/tep.db")).expect("db should reopen");
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .expect("schema version should be readable");
    assert_eq!(version, 3);
}

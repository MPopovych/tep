use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rusqlite::Connection;

pub const DEFAULT_TEP_DIR: &str = ".tep";
pub const DEFAULT_DB_FILE: &str = ".tep/tep.db";
pub const DEFAULT_IGNORE_FILE: &str = ".tep_ignore";

#[derive(Debug, Clone)]
pub struct WorkspacePaths {
    pub tep_dir: PathBuf,
    pub db_file: PathBuf,
    pub ignore_file: PathBuf,
}

pub fn open_in_memory() -> rusqlite::Result<Connection> {
    Connection::open_in_memory()
}

pub fn workspace_paths_for(root: &Path) -> WorkspacePaths {
    WorkspacePaths {
        tep_dir: root.join(DEFAULT_TEP_DIR),
        db_file: root.join(DEFAULT_DB_FILE),
        ignore_file: root.join(DEFAULT_IGNORE_FILE),
    }
}

pub fn find_workspace_root(start: &Path) -> Result<PathBuf> {
    let mut current = if start.is_dir() {
        start.to_path_buf()
    } else {
        start.parent().unwrap_or(start).to_path_buf()
    };

    loop {
        let candidate = current.join(DEFAULT_DB_FILE);
        if candidate.exists() {
            return Ok(current);
        }

        if !current.pop() {
            break;
        }
    }

    bail!(
        "no tep workspace found from {}\nrun `tep init` in the project root to create one",
        start.display()
    )
}

pub fn open_workspace_db() -> anyhow::Result<Connection> {
    let cwd = std::env::current_dir().context("failed to determine current directory")?;
    let root = find_workspace_root(&cwd)?;
    let paths = workspace_paths_for(&root);

    Connection::open(&paths.db_file)
        .with_context(|| format!("failed to open database at {}", paths.db_file.display()))
}

pub fn open_workspace_db_in(root: &Path) -> anyhow::Result<Connection> {
    let paths = workspace_paths_for(root);
    if let Some(parent) = paths.db_file.parent() {
        fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    }

    Connection::open(&paths.db_file)
        .with_context(|| format!("failed to open database at {}", paths.db_file.display()))
}

pub fn schema_sql() -> &'static str {
    r#"
    PRAGMA foreign_keys = ON;

    CREATE TABLE IF NOT EXISTS entities (
        entity_id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        ref TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS anchors (
        anchor_id INTEGER PRIMARY KEY,
        version INTEGER NOT NULL,
        file_path TEXT NOT NULL,
        line INTEGER,
        shift INTEGER,
        offset INTEGER,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS anchor_entities (
        anchor_id INTEGER NOT NULL,
        entity_id INTEGER NOT NULL,
        created_at TEXT NOT NULL,
        PRIMARY KEY (anchor_id, entity_id),
        FOREIGN KEY (anchor_id) REFERENCES anchors(anchor_id) ON DELETE CASCADE,
        FOREIGN KEY (entity_id) REFERENCES entities(entity_id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS links (
        from_entity_id INTEGER NOT NULL,
        to_entity_id INTEGER NOT NULL,
        relation_type TEXT NOT NULL,
        priority INTEGER NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        PRIMARY KEY (from_entity_id, relation_type, to_entity_id),
        FOREIGN KEY (from_entity_id) REFERENCES entities(entity_id) ON DELETE CASCADE,
        FOREIGN KEY (to_entity_id) REFERENCES entities(entity_id) ON DELETE CASCADE
    );

    CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(name);
    CREATE INDEX IF NOT EXISTS idx_anchors_file_path ON anchors(file_path);
    CREATE INDEX IF NOT EXISTS idx_anchor_entities_entity_id ON anchor_entities(entity_id);
    CREATE INDEX IF NOT EXISTS idx_links_from_entity ON links(from_entity_id);
    CREATE INDEX IF NOT EXISTS idx_links_to_entity ON links(to_entity_id);
    CREATE INDEX IF NOT EXISTS idx_links_priority ON links(priority);
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_applies_to_in_memory_db() {
        let conn = open_in_memory().expect("in-memory db should open");
        conn.execute_batch(schema_sql())
            .expect("schema should apply cleanly");
    }

    #[test]
    fn finds_nearest_workspace_root() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let root = temp.path();
        let nested = root.join("a/b/c");
        fs::create_dir_all(&nested).expect("nested dirs should be created");
        fs::create_dir_all(root.join(DEFAULT_TEP_DIR)).expect("tep dir should exist");
        fs::write(root.join(DEFAULT_DB_FILE), b"").expect("db marker should exist");

        let found = find_workspace_root(&nested).expect("workspace should be found");
        assert_eq!(found, root);
    }

    #[test]
    fn missing_workspace_error_is_actionable() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let err = find_workspace_root(temp.path()).expect_err("workspace should be missing");
        let rendered = err.to_string();
        assert!(rendered.contains("no tep workspace found"));
        assert!(rendered.contains("tep init"));
    }
}

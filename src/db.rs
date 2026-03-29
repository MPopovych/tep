use std::fs;
use std::path::Path;

use anyhow::Context;
use rusqlite::Connection;

pub const DEFAULT_TEP_DIR: &str = ".tep";
pub const DEFAULT_DB_FILE: &str = ".tep/tep.db";
pub const DEFAULT_IGNORE_FILE: &str = ".tep_ignore";

pub fn open_in_memory() -> rusqlite::Result<Connection> {
    Connection::open_in_memory()
}

pub fn open_workspace_db() -> anyhow::Result<Connection> {
    if let Some(parent) = Path::new(DEFAULT_DB_FILE).parent() {
        fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    }

    Connection::open(DEFAULT_DB_FILE)
        .with_context(|| format!("failed to open database at {DEFAULT_DB_FILE}"))
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
}

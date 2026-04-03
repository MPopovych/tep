use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OptionalExtension, params};

pub const DEFAULT_TEP_DIR: &str = ".tep";
pub const DEFAULT_DB_FILE: &str = ".tep/tep.db";
pub const DEFAULT_IGNORE_FILE: &str = ".tepignore";
pub const CURRENT_SCHEMA_VERSION: i64 = 4;

#[derive(Debug, Clone)]
pub struct WorkspacePaths {
    pub tep_dir: PathBuf,
    pub db_file: PathBuf,
    pub ignore_file: PathBuf,
}

#[cfg(test)]
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
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    Connection::open(&paths.db_file)
        .with_context(|| format!("failed to open database at {}", paths.db_file.display()))
}

pub fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(base_schema_sql())
        .context("failed to apply database schema")?;

    let current_version = schema_version(conn)?;
    let detected_version = detect_schema_version(conn)?;
    let mut version = current_version.max(detected_version);

    if version < 2 {
        migrate_to_v2(conn)?;
        version = 2;
    }
    if version < 3 {
        migrate_to_v3(conn)?;
        version = 3;
    }
    if version < 4 {
        migrate_to_v4(conn)?;
        version = 4;
    }

    set_schema_version(conn, version.max(CURRENT_SCHEMA_VERSION))?;
    Ok(())
}

#[cfg(test)]
pub fn schema_sql() -> &'static str {
    base_schema_sql()
}

fn base_schema_sql() -> &'static str {
    r#"
    PRAGMA foreign_keys = ON;

    CREATE TABLE IF NOT EXISTS entities (
        entity_id INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        ref TEXT,
        description TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS anchors (
        anchor_id INTEGER PRIMARY KEY,
        version INTEGER NOT NULL,
        name TEXT UNIQUE,
        file_path TEXT NOT NULL,
        line INTEGER,
        shift INTEGER,
        offset INTEGER,
        description TEXT,
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

    CREATE TABLE IF NOT EXISTS entity_links (
        from_entity_id INTEGER NOT NULL,
        to_entity_id INTEGER NOT NULL,
        relation TEXT NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        PRIMARY KEY (from_entity_id, to_entity_id),
        FOREIGN KEY (from_entity_id) REFERENCES entities(entity_id) ON DELETE CASCADE,
        FOREIGN KEY (to_entity_id) REFERENCES entities(entity_id) ON DELETE CASCADE
    );

    CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(name);
    CREATE INDEX IF NOT EXISTS idx_anchors_file_path ON anchors(file_path);
    CREATE INDEX IF NOT EXISTS idx_anchor_entities_entity_id ON anchor_entities(entity_id);
    CREATE INDEX IF NOT EXISTS idx_entity_links_to_entity ON entity_links(to_entity_id);
    "#
}

fn migrate_to_v2(conn: &Connection) -> Result<()> {
    if !column_exists(conn, "entities", "description")? {
        conn.execute("ALTER TABLE entities ADD COLUMN description TEXT", [])
            .context("failed to add description column to entities")?;
    }

    if !table_exists(conn, "entity_links")? {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS entity_links (
                from_entity_id INTEGER NOT NULL,
                to_entity_id INTEGER NOT NULL,
                relation TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (from_entity_id, to_entity_id),
                FOREIGN KEY (from_entity_id) REFERENCES entities(entity_id) ON DELETE CASCADE,
                FOREIGN KEY (to_entity_id) REFERENCES entities(entity_id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_entity_links_to_entity ON entity_links(to_entity_id);
            "#,
        )
        .context("failed to create entity_links table")?;
    }

    Ok(())
}

fn migrate_to_v3(conn: &Connection) -> Result<()> {
    if !column_exists(conn, "anchors", "name")? {
        conn.execute("ALTER TABLE anchors ADD COLUMN name TEXT", [])
            .context("failed to add name column to anchors")?;
    }
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_anchors_name_unique ON anchors(name) WHERE name IS NOT NULL",
        [],
    )
    .context("failed to create unique index on anchors(name)")?;
    Ok(())
}

fn migrate_to_v4(conn: &Connection) -> Result<()> {
    if !column_exists(conn, "anchors", "description")? {
        conn.execute("ALTER TABLE anchors ADD COLUMN description TEXT", [])
            .context("failed to add description column to anchors")?;
    }
    Ok(())
}

fn detect_schema_version(conn: &Connection) -> Result<i64> {
    if !table_exists(conn, "entities")? {
        return Ok(0);
    }
    if !column_exists(conn, "entities", "description")? {
        return Ok(1);
    }
    if !column_exists(conn, "anchors", "name")? {
        return Ok(2);
    }
    if !column_exists(conn, "anchors", "description")? {
        return Ok(3);
    }
    Ok(4)
}

fn table_exists(conn: &Connection, table: &str) -> Result<bool> {
    let exists = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1",
            params![table],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    Ok(exists)
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let pragma = format!("PRAGMA table_info({})", table);
    let mut stmt = conn.prepare(&pragma)?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn schema_version(conn: &Connection) -> Result<i64> {
    let version = conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))?;
    Ok(version)
}

fn set_schema_version(conn: &Connection, version: i64) -> Result<()> {
    conn.pragma_update(None, "user_version", version)
        .context("failed to update schema version")?;
    Ok(())
}

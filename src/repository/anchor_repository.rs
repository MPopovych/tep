use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OptionalExtension, params};

use crate::anchor::Anchor;
use crate::utils::path::normalize_to_workspace;
use crate::utils::time::now_utc;

pub struct AnchorRepository<'a> {
    pub(crate) conn: &'a Connection,
    workspace_root: PathBuf,
}

impl<'a> AnchorRepository<'a> {
    pub fn with_workspace_root(conn: &'a Connection, workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            conn,
            workspace_root: workspace_root.into(),
        }
    }

    #[allow(dead_code)]
    pub fn create(
        &self,
        version: i64,
        file_path: &str,
        line: Option<i64>,
        shift: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Anchor> {
        self.create_with_name(None, version, file_path, line, shift, offset, None)
    }

    pub fn update_location(
        &self,
        anchor_id: i64,
        file_path: &str,
        line: Option<i64>,
        shift: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Anchor> {
        let existing = self.find_by_id(anchor_id)?.with_context(|| {
            format!(
                "materialized anchor {} was found in a file but does not exist in the database",
                anchor_id
            )
        })?;

        let normalized = self.normalize_path(file_path);
        if self.normalize_path(&existing.file_path) != normalized {
            bail!(
                "materialized anchor {} is already associated with {} and cannot also be updated from {}",
                anchor_id,
                existing.file_path,
                file_path
            );
        }

        let now = now_utc();
        self.conn.execute(
            "UPDATE anchors SET file_path = ?1, line = ?2, shift = ?3, offset = ?4, updated_at = ?5 WHERE anchor_id = ?6",
            params![normalized, line, shift, offset, now, anchor_id],
        )
        .with_context(|| format!("failed to update anchor {}", anchor_id))?;

        self.find_by_id(anchor_id)?
            .context("updated anchor could not be reloaded")
    }

    pub fn update_description(&self, anchor_id: i64, description: Option<&str>) -> Result<Anchor> {
        let existing = self
            .find_by_id(anchor_id)?
            .with_context(|| format!("anchor {} not found for description update", anchor_id))?;
        let now = now_utc();
        self.conn.execute(
            "UPDATE anchors SET description = ?1, updated_at = ?2 WHERE anchor_id = ?3",
            params![description, now, existing.anchor_id],
        )?;
        self.find_by_id(existing.anchor_id)?
            .context("updated anchor could not be reloaded")
    }

    pub fn create_named(
        &self,
        name: &str,
        version: i64,
        file_path: &str,
        line: Option<i64>,
        shift: Option<i64>,
        offset: Option<i64>,
        description: Option<&str>,
    ) -> Result<Anchor> {
        self.create_with_name(Some(name), version, file_path, line, shift, offset, description)
    }

    fn create_with_name(
        &self,
        name: Option<&str>,
        version: i64,
        file_path: &str,
        line: Option<i64>,
        shift: Option<i64>,
        offset: Option<i64>,
        description: Option<&str>,
    ) -> Result<Anchor> {
        let now = now_utc();
        let normalized = self.normalize_path(file_path);
        self.conn.execute(
            "INSERT INTO anchors (version, name, file_path, line, shift, offset, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![version, name, normalized, line, shift, offset, description, now, now],
        )
        .with_context(|| match name {
            Some(n) => format!("failed to create anchor '{}' for file {}", n, file_path),
            None => format!("failed to create anchor for file {}", file_path),
        })?;

        let anchor_id = self.conn.last_insert_rowid();
        self.find_by_id(anchor_id)?
            .context("created anchor could not be reloaded")
    }

    pub fn find_by_id(&self, anchor_id: i64) -> Result<Option<Anchor>> {
        let mut stmt = self.conn.prepare(
            "SELECT anchor_id, version, name, file_path, line, shift, offset, description, created_at, updated_at FROM anchors WHERE anchor_id = ?1",
        )?;
        let anchor = stmt
            .query_row(params![anchor_id], map_anchor_row)
            .optional()?;
        Ok(anchor)
    }

    pub fn find_by_name(&self, name: &str) -> Result<Option<Anchor>> {
        let mut stmt = self.conn.prepare(
            "SELECT anchor_id, version, name, file_path, line, shift, offset, description, created_at, updated_at FROM anchors WHERE name = ?1",
        )?;
        let anchor = stmt.query_row(params![name], map_anchor_row).optional()?;
        Ok(anchor)
    }

    pub fn list_ids_for_file(&self, file_path: &str) -> Result<Vec<i64>> {
        let normalized = self.normalize_path(file_path);
        let mut stmt = self
            .conn
            .prepare("SELECT anchor_id FROM anchors WHERE file_path = ?1 ORDER BY anchor_id ASC")?;
        let rows = stmt.query_map(params![normalized], |row| row.get::<_, i64>(0))?;
        let ids = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(ids)
    }

    pub fn list_for_entity(&self, entity_id: i64) -> Result<Vec<Anchor>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.anchor_id, a.version, a.name, a.file_path, a.line, a.shift, a.offset, a.description, a.created_at, a.updated_at
             FROM anchor_entities ae
             JOIN anchors a ON a.anchor_id = ae.anchor_id
             WHERE ae.entity_id = ?1
             ORDER BY a.anchor_id ASC",
        )?;
        let rows = stmt.query_map(params![entity_id], map_anchor_row)?;
        let anchors = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(anchors)
    }

    pub fn list_all(&self) -> Result<Vec<Anchor>> {
        let mut stmt = self.conn.prepare(
            "SELECT anchor_id, version, name, file_path, line, shift, offset, description, created_at, updated_at
             FROM anchors
             ORDER BY anchor_id ASC",
        )?;
        let rows = stmt.query_map([], map_anchor_row)?;
        let anchors = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(anchors)
    }

    pub fn list_without_entities(&self) -> Result<Vec<Anchor>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.anchor_id, a.version, a.name, a.file_path, a.line, a.shift, a.offset, a.description, a.created_at, a.updated_at
             FROM anchors a
             LEFT JOIN anchor_entities ae ON ae.anchor_id = a.anchor_id
             WHERE ae.anchor_id IS NULL
             ORDER BY a.anchor_id ASC",
        )?;
        let rows = stmt.query_map([], map_anchor_row)?;
        let anchors = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(anchors)
    }

    pub fn find_latest_for_entity_in_file(
        &self,
        entity_id: i64,
        file_path: &str,
    ) -> Result<Option<Anchor>> {
        let normalized = self.normalize_path(file_path);
        let mut stmt = self.conn.prepare(
            "SELECT a.anchor_id, a.version, a.name, a.file_path, a.line, a.shift, a.offset, a.description, a.created_at, a.updated_at
             FROM anchor_entities ae
             JOIN anchors a ON a.anchor_id = ae.anchor_id
             WHERE ae.entity_id = ?1 AND a.file_path = ?2
             ORDER BY a.anchor_id DESC
             LIMIT 1",
        )?;
        let anchor = stmt
            .query_row(params![entity_id, normalized], map_anchor_row)
            .optional()?;
        Ok(anchor)
    }

    pub fn delete(&self, anchor_id: i64) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM anchors WHERE anchor_id = ?1",
                params![anchor_id],
            )
            .with_context(|| format!("failed to delete anchor {}", anchor_id))?;
        Ok(())
    }

    pub fn normalized_path_for(&self, input: &str) -> String {
        self.normalize_path(input)
    }

    fn normalize_path(&self, input: &str) -> String {
        normalize_to_workspace(Path::new(input), &self.workspace_root)
            .to_string_lossy()
            .to_string()
    }
}

fn map_anchor_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Anchor> {
    Ok(Anchor {
        anchor_id: row.get(0)?,
        version: row.get(1)?,
        name: row.get(2)?,
        file_path: row.get(3)?,
        line: row.get(4)?,
        shift: row.get(5)?,
        offset: row.get(6)?,
        description: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

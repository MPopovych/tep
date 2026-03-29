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

    pub fn create(
        &self,
        version: i64,
        file_path: &str,
        line: Option<i64>,
        shift: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Anchor> {
        let now = now_utc();
        let normalized = self.normalize_path(file_path);
        self.conn.execute(
            "INSERT INTO anchors (version, file_path, line, shift, offset, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![version, normalized, line, shift, offset, now, now],
        )
        .with_context(|| format!("failed to create anchor for file {}", file_path))?;

        let anchor_id = self.conn.last_insert_rowid();
        self.find_by_id(anchor_id)?
            .context("created anchor could not be reloaded")
    }

    pub fn update_location(
        &self,
        anchor_id: i64,
        file_path: &str,
        line: Option<i64>,
        shift: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Anchor> {
        let existing = self
            .find_by_id(anchor_id)?
            .with_context(|| format!("materialized anchor {} was found in a file but does not exist in the database", anchor_id))?;

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

    pub fn find_by_id(&self, anchor_id: i64) -> Result<Option<Anchor>> {
        let mut stmt = self.conn.prepare(
            "SELECT anchor_id, version, file_path, line, shift, offset, created_at, updated_at FROM anchors WHERE anchor_id = ?1",
        )?;
        let anchor = stmt.query_row(params![anchor_id], map_anchor_row).optional()?;
        Ok(anchor)
    }

    pub fn list_ids_for_file(&self, file_path: &str) -> Result<Vec<i64>> {
        let normalized = self.normalize_path(file_path);
        let mut stmt = self.conn.prepare(
            "SELECT anchor_id, file_path FROM anchors ORDER BY anchor_id ASC",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))?;
        let pairs = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(pairs
            .into_iter()
            .filter(|(_, stored_path)| self.normalize_path(stored_path) == normalized)
            .map(|(anchor_id, _)| anchor_id)
            .collect())
    }

    pub fn list_for_entity(&self, entity_id: i64) -> Result<Vec<Anchor>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.anchor_id, a.version, a.file_path, a.line, a.shift, a.offset, a.created_at, a.updated_at
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
            "SELECT anchor_id, version, file_path, line, shift, offset, created_at, updated_at
             FROM anchors
             ORDER BY anchor_id ASC",
        )?;
        let rows = stmt.query_map([], map_anchor_row)?;
        let anchors = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(anchors)
    }

    pub fn list_without_entities(&self) -> Result<Vec<Anchor>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.anchor_id, a.version, a.file_path, a.line, a.shift, a.offset, a.created_at, a.updated_at
             FROM anchors a
             LEFT JOIN anchor_entities ae ON ae.anchor_id = a.anchor_id
             WHERE ae.anchor_id IS NULL
             ORDER BY a.anchor_id ASC",
        )?;
        let rows = stmt.query_map([], map_anchor_row)?;
        let anchors = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(anchors)
    }

    pub fn find_latest_for_entity_in_file(&self, entity_id: i64, file_path: &str) -> Result<Option<Anchor>> {
        let normalized = self.normalize_path(file_path);
        let mut stmt = self.conn.prepare(
            "SELECT a.anchor_id, a.version, a.file_path, a.line, a.shift, a.offset, a.created_at, a.updated_at
             FROM anchor_entities ae
             JOIN anchors a ON a.anchor_id = ae.anchor_id
             WHERE ae.entity_id = ?1
             ORDER BY a.anchor_id DESC",
        )?;
        let rows = stmt.query_map(params![entity_id], map_anchor_row)?;
        let anchors = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(anchors
            .into_iter()
            .find(|anchor| self.normalize_path(&anchor.file_path) == normalized))
    }

    pub fn delete(&self, anchor_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM anchors WHERE anchor_id = ?1", params![anchor_id])
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
        file_path: row.get(2)?,
        line: row.get(3)?,
        shift: row.get(4)?,
        offset: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::entity::NewEntity;
    use crate::repository::anchor_entity_repository::AnchorEntityRepository;
    use crate::repository::entity_repository::EntityRepository;

    fn setup_repo() -> AnchorRepository<'static> {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql())
            .expect("schema should apply");
        AnchorRepository::with_workspace_root(conn, "/tmp/project")
    }

    #[test]
    fn create_persists_anchor() {
        let repo = setup_repo();
        let anchor = repo
            .create(1, "./file.txt", Some(2), Some(3), Some(12))
            .expect("create should succeed");

        assert!(anchor.anchor_id > 0);
        assert_eq!(anchor.version, 1);
        assert_eq!(anchor.file_path, "./file.txt");
        assert_eq!(anchor.line, Some(2));
        assert_eq!(anchor.shift, Some(3));
        assert_eq!(anchor.offset, Some(12));
    }

    #[test]
    fn update_location_updates_metadata() {
        let repo = setup_repo();
        let anchor = repo
            .create(1, "./file.txt", Some(2), Some(3), Some(12))
            .expect("create should succeed");

        let updated = repo
            .update_location(anchor.anchor_id, "file.txt", Some(5), Some(1), Some(30))
            .expect("update should succeed");

        assert_eq!(updated.file_path, "./file.txt");
        assert_eq!(updated.line, Some(5));
        assert_eq!(updated.shift, Some(1));
        assert_eq!(updated.offset, Some(30));
    }

    #[test]
    fn update_location_accepts_legacy_absolute_path_equivalent() {
        let repo = setup_repo();
        let anchor = repo.create(1, "/tmp/project/./file.txt", Some(1), Some(0), Some(0)).unwrap();
        let updated = repo.update_location(anchor.anchor_id, "./file.txt", Some(2), Some(0), Some(5)).unwrap();
        assert_eq!(updated.file_path, "./file.txt");
    }

    #[test]
    fn update_location_fails_for_unknown_anchor() {
        let repo = setup_repo();
        let result = repo.update_location(999, "./other.txt", Some(1), Some(0), Some(0));
        assert!(result.is_err());
    }

    #[test]
    fn update_location_fails_for_different_file() {
        let repo = setup_repo();
        let anchor = repo
            .create(1, "./file.txt", Some(2), Some(3), Some(12))
            .expect("create should succeed");

        let result = repo.update_location(anchor.anchor_id, "./other.txt", Some(1), Some(0), Some(0));
        assert!(result.is_err());
    }

    #[test]
    fn list_ids_for_file_and_delete_work() {
        let repo = setup_repo();
        let a = repo.create(1, "./file.txt", Some(1), Some(0), Some(0)).unwrap();
        let b = repo.create(1, "/tmp/project/./file.txt", Some(2), Some(0), Some(5)).unwrap();
        let ids = repo.list_ids_for_file("file.txt").unwrap();
        assert_eq!(ids, vec![a.anchor_id, b.anchor_id]);
        repo.delete(a.anchor_id).unwrap();
        let ids = repo.list_ids_for_file("./file.txt").unwrap();
        assert_eq!(ids, vec![b.anchor_id]);
    }

    #[test]
    fn list_without_entities_returns_orphan_anchor() {
        let repo = setup_repo();
        repo.create(1, "./docs/orphan.md", Some(1), Some(0), Some(0)).unwrap();
        let anchors = repo.list_without_entities().unwrap();
        assert_eq!(anchors.len(), 1);
        assert_eq!(anchors[0].file_path, "./docs/orphan.md");
    }

    #[test]
    fn list_for_entity_returns_related_anchors() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        let anchor_repo = AnchorRepository::with_workspace_root(conn, "/tmp/project");
        let entity_repo = EntityRepository::new(conn);
        let rel_repo = AnchorEntityRepository::new(conn);

        let anchor = anchor_repo.create(1, "./file.txt", Some(1), Some(0), Some(0)).unwrap();
        let entity = entity_repo.create(&NewEntity { name: "student".into(), r#ref: None, description: None }).unwrap();
        rel_repo.attach(anchor.anchor_id, entity.entity_id).unwrap();

        let anchors = anchor_repo.list_for_entity(entity.entity_id).unwrap();
        assert_eq!(anchors.len(), 1);
        assert_eq!(anchors[0].anchor_id, anchor.anchor_id);
    }

    #[test]
    fn find_latest_for_entity_in_file_returns_related_anchor() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        let anchor_repo = AnchorRepository::with_workspace_root(conn, "/tmp/project");
        let entity_repo = EntityRepository::new(conn);
        let rel_repo = AnchorEntityRepository::new(conn);

        let entity = entity_repo.create(&NewEntity { name: "student".into(), r#ref: None, description: None }).unwrap();
        let old_anchor = anchor_repo.create(1, "./file.txt", Some(1), Some(0), Some(0)).unwrap();
        let new_anchor = anchor_repo.create(1, "/tmp/project/./file.txt", Some(2), Some(0), Some(5)).unwrap();
        rel_repo.attach(old_anchor.anchor_id, entity.entity_id).unwrap();
        rel_repo.attach(new_anchor.anchor_id, entity.entity_id).unwrap();

        let found = anchor_repo
            .find_latest_for_entity_in_file(entity.entity_id, "file.txt")
            .unwrap()
            .unwrap();
        assert_eq!(found.anchor_id, new_anchor.anchor_id);
    }

    #[test]
    fn normalized_path_for_returns_workspace_relative_path() {
        let repo = setup_repo();
        assert_eq!(repo.normalized_path_for("/tmp/project/./docs/a.md"), "./docs/a.md");
    }
}

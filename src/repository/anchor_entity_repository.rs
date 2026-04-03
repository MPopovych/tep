use anyhow::{Context, Result};
use rusqlite::{Connection, params};

use crate::entity::Entity;

pub struct AnchorEntityRepository<'a> {
    conn: &'a Connection,
}

impl<'a> AnchorEntityRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn attach(&self, anchor_id: i64, entity_id: i64) -> Result<()> {
        let now = now_utc();
        self.conn.execute(
            "INSERT OR IGNORE INTO anchor_entities (anchor_id, entity_id, created_at) VALUES (?1, ?2, ?3)",
            params![anchor_id, entity_id, now],
        )
        .with_context(|| format!("failed to attach anchor {} to entity {}", anchor_id, entity_id))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn detach(&self, anchor_id: i64, entity_id: i64) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM anchor_entities WHERE anchor_id = ?1 AND entity_id = ?2",
                params![anchor_id, entity_id],
            )
            .with_context(|| {
                format!(
                    "failed to detach anchor {} from entity {}",
                    anchor_id, entity_id
                )
            })?;
        Ok(())
    }

    pub fn replace_for_anchor(&self, anchor_id: i64, entity_ids: &[i64]) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM anchor_entities WHERE anchor_id = ?1",
                params![anchor_id],
            )
            .with_context(|| {
                format!(
                    "failed to clear anchor-entity relations for anchor {}",
                    anchor_id
                )
            })?;

        for entity_id in entity_ids {
            self.attach(anchor_id, *entity_id)?;
        }

        Ok(())
    }

    pub fn list_entities_for_anchor(&self, anchor_id: i64) -> Result<Vec<Entity>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.entity_id, e.name, e.ref, e.description, e.created_at, e.updated_at
             FROM anchor_entities ae
             JOIN entities e ON e.entity_id = ae.entity_id
             WHERE ae.anchor_id = ?1
             ORDER BY e.entity_id ASC",
        )?;
        let rows = stmt.query_map(params![anchor_id], |row| {
            Ok(Entity {
                entity_id: row.get(0)?,
                name: row.get(1)?,
                r#ref: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        let entities = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(entities)
    }
}

fn now_utc() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_secs();

    secs.to_string()
}

// #tepignoreafter
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::entity::NewEntity;
    use crate::repository::anchor_repository::{AnchorRepository, NewAnchor};
    use crate::repository::entity_repository::EntityRepository;

    fn setup() -> (
        &'static Connection,
        AnchorEntityRepository<'static>,
        i64,
        i64,
    ) {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql())
            .expect("schema should apply");

        let anchor_repo = AnchorRepository::with_workspace_root(conn, "/tmp/project");
        let entity_repo = EntityRepository::new(conn);
        let relation_repo = AnchorEntityRepository::new(conn);

        let anchor = anchor_repo
            .create(&NewAnchor {
                name: None,
                version: 1,
                file_path: "./file.txt",
                line: Some(1),
                shift: Some(0),
                offset: Some(0),
                description: None,
            })
            .expect("anchor should be created");
        let entity = entity_repo
            .create(&NewEntity {
                name: "student".into(),
                r#ref: None,
                description: None,
            })
            .expect("entity should be created");

        (conn, relation_repo, anchor.anchor_id, entity.entity_id)
    }

    #[test]
    fn attach_and_list_relations() {
        let (_conn, repo, anchor_id, entity_id) = setup();
        repo.attach(anchor_id, entity_id)
            .expect("attach should succeed");
        let entities = repo
            .list_entities_for_anchor(anchor_id)
            .expect("list should succeed");
        assert_eq!(
            entities
                .iter()
                .map(|entity| entity.entity_id)
                .collect::<Vec<_>>(),
            vec![entity_id]
        );
    }

    #[test]
    fn list_entities_for_anchor_returns_full_rows() {
        let (_conn, repo, anchor_id, entity_id) = setup();
        repo.attach(anchor_id, entity_id)
            .expect("attach should succeed");
        let entities = repo
            .list_entities_for_anchor(anchor_id)
            .expect("list should succeed");
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].entity_id, entity_id);
        assert_eq!(entities[0].name, "student");
    }

    #[test]
    fn replace_for_anchor_replaces_existing_relations() {
        let (conn, repo, anchor_id, entity_id) = setup();
        repo.attach(anchor_id, entity_id)
            .expect("attach should succeed");

        let entity_repo = EntityRepository::new(conn);
        let other = entity_repo
            .create(&NewEntity {
                name: "basic_user".into(),
                r#ref: None,
                description: None,
            })
            .expect("entity should be created");

        repo.replace_for_anchor(anchor_id, &[other.entity_id])
            .expect("replace should succeed");

        let entities = repo
            .list_entities_for_anchor(anchor_id)
            .expect("list should succeed");
        assert_eq!(
            entities
                .iter()
                .map(|entity| entity.entity_id)
                .collect::<Vec<_>>(),
            vec![other.entity_id]
        );
    }

    #[test]
    fn detach_removes_relation() {
        let (_conn, repo, anchor_id, entity_id) = setup();
        repo.attach(anchor_id, entity_id)
            .expect("attach should succeed");
        repo.detach(anchor_id, entity_id)
            .expect("detach should succeed");
        let entities = repo
            .list_entities_for_anchor(anchor_id)
            .expect("list should succeed");
        assert!(entities.is_empty());
    }
}

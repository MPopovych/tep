use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};

use crate::entity::{Entity, EntityLookup, NewEntity, UpdateEntity, validate_name};

pub struct EntityRepository<'a> {
    conn: &'a Connection,
}

impl<'a> EntityRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, new_entity: &NewEntity) -> Result<Entity> {
        validate_name(&new_entity.name).map_err(anyhow::Error::msg)?;

        let now = now_utc();
        self.conn
            .execute(
                "INSERT INTO entities (name, ref, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![new_entity.name, new_entity.r#ref, now, now],
            )
            .with_context(|| format!("failed to create entity {}", new_entity.name))?;

        let entity_id = self.conn.last_insert_rowid();
        self.get_by_id(entity_id)?
            .context("created entity could not be reloaded")
    }

    pub fn ensure(&self, new_entity: &NewEntity) -> Result<Entity> {
        validate_name(&new_entity.name).map_err(anyhow::Error::msg)?;

        if let Some(existing) = self.get_by_name(&new_entity.name)? {
            return Ok(existing);
        }

        self.create(new_entity)
    }

    pub fn find(&self, lookup: &EntityLookup) -> Result<Option<Entity>> {
        match lookup {
            EntityLookup::Id(id) => self.get_by_id(*id),
            EntityLookup::Name(name) => self.get_by_name(name),
        }
    }

    pub fn update(&self, lookup: &EntityLookup, changes: &UpdateEntity) -> Result<Entity> {
        if let Some(name) = &changes.name {
            validate_name(name).map_err(anyhow::Error::msg)?;
        }

        let current = self
            .find(lookup)?
            .with_context(|| format!("entity not found for lookup {lookup:?}"))?;

        let new_name = changes.name.clone().unwrap_or_else(|| current.name.clone());
        let new_ref = changes.r#ref.clone().or(current.r#ref.clone());
        let now = now_utc();

        self.conn
            .execute(
                "UPDATE entities SET name = ?1, ref = ?2, updated_at = ?3 WHERE entity_id = ?4",
                params![new_name, new_ref, now, current.entity_id],
            )
            .with_context(|| format!("failed to update entity {}", current.entity_id))?;

        self.get_by_id(current.entity_id)?
            .context("updated entity could not be reloaded")
    }

    pub fn list(&self) -> Result<Vec<Entity>> {
        let mut stmt = self.conn.prepare(
            "SELECT entity_id, name, ref, created_at, updated_at FROM entities ORDER BY entity_id DESC",
        )?;

        let rows = stmt.query_map([], map_entity_row)?;
        let entities = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(entities)
    }

    fn get_by_id(&self, entity_id: i64) -> Result<Option<Entity>> {
        let mut stmt = self.conn.prepare(
            "SELECT entity_id, name, ref, created_at, updated_at FROM entities WHERE entity_id = ?1",
        )?;
        let entity = stmt.query_row(params![entity_id], map_entity_row).optional()?;
        Ok(entity)
    }

    fn get_by_name(&self, name: &str) -> Result<Option<Entity>> {
        let mut stmt = self.conn.prepare(
            "SELECT entity_id, name, ref, created_at, updated_at FROM entities WHERE name = ?1",
        )?;
        let entity = stmt.query_row(params![name], map_entity_row).optional()?;
        Ok(entity)
    }
}

fn map_entity_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entity> {
    Ok(Entity {
        entity_id: row.get(0)?,
        name: row.get(1)?,
        r#ref: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn now_utc() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_secs();

    secs.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_repo() -> EntityRepository<'static> {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql())
            .expect("schema should apply");
        EntityRepository::new(conn)
    }

    #[test]
    fn create_persists_entity() {
        let repo = setup_repo();
        let entity = repo
            .create(&NewEntity {
                name: "student".into(),
                r#ref: Some("./docs/student.md".into()),
            })
            .expect("create should succeed");

        assert!(entity.entity_id > 0);
        assert_eq!(entity.name, "student");
        assert_eq!(entity.r#ref.as_deref(), Some("./docs/student.md"));
    }

    #[test]
    fn ensure_returns_existing_entity() {
        let repo = setup_repo();
        let first = repo
            .ensure(&NewEntity {
                name: "student".into(),
                r#ref: None,
            })
            .expect("first ensure should succeed");
        let second = repo
            .ensure(&NewEntity {
                name: "student".into(),
                r#ref: Some("./ignored.md".into()),
            })
            .expect("second ensure should succeed");

        assert_eq!(first.entity_id, second.entity_id);
        assert_eq!(second.r#ref, None);
    }

    #[test]
    fn find_supports_lookup_by_id_and_name() {
        let repo = setup_repo();
        let created = repo
            .create(&NewEntity {
                name: "student.permissions".into(),
                r#ref: None,
            })
            .expect("create should succeed");

        let by_id = repo
            .find(&EntityLookup::Id(created.entity_id))
            .expect("lookup by id should succeed")
            .expect("entity should exist");
        let by_name = repo
            .find(&EntityLookup::Name("student.permissions".into()))
            .expect("lookup by name should succeed")
            .expect("entity should exist");

        assert_eq!(by_id, by_name);
    }

    #[test]
    fn update_changes_multiple_fields() {
        let repo = setup_repo();
        let created = repo
            .create(&NewEntity {
                name: "student".into(),
                r#ref: None,
            })
            .expect("create should succeed");

        let updated = repo
            .update(
                &EntityLookup::Id(created.entity_id),
                &UpdateEntity {
                    name: Some("student.profile".into()),
                    r#ref: Some("./docs/profile.md".into()),
                },
            )
            .expect("update should succeed");

        assert_eq!(updated.name, "student.profile");
        assert_eq!(updated.r#ref.as_deref(), Some("./docs/profile.md"));
    }

    #[test]
    fn list_returns_newest_first() {
        let repo = setup_repo();
        let _ = repo.create(&NewEntity { name: "a.x".into(), r#ref: None });
        let _ = repo.create(&NewEntity { name: "b.x".into(), r#ref: None });

        let list = repo.list().expect("list should succeed");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "b.x");
        assert_eq!(list[1].name, "a.x");
    }

    #[test]
    fn create_rejects_numeric_name() {
        let repo = setup_repo();
        let result = repo.create(&NewEntity {
            name: "123".into(),
            r#ref: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_name_fails() {
        let repo = setup_repo();
        repo.create(&NewEntity {
            name: "student".into(),
            r#ref: None,
        })
        .expect("first create should succeed");

        let result = repo.create(&NewEntity {
            name: "student".into(),
            r#ref: None,
        });

        assert!(result.is_err());
    }
}

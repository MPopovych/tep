use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OptionalExtension, params};

use crate::entity::{Entity, EntityLink, EntityLookup, NewEntity, UpdateEntity, validate_name};
use crate::utils::time::now_utc;

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
        self.conn.execute(
            "INSERT INTO entities (name, ref, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![new_entity.name, new_entity.r#ref, new_entity.description, now, now],
        )
        .with_context(|| format!("failed to create entity {}", new_entity.name))?;

        let entity_id = self.conn.last_insert_rowid();
        self.find(&EntityLookup::Id(entity_id))?
            .context("created entity could not be reloaded")
    }

    pub fn ensure(&self, new_entity: &NewEntity) -> Result<Entity> {
        validate_name(&new_entity.name).map_err(anyhow::Error::msg)?;

        if let Some(existing) = self.find(&EntityLookup::Name(new_entity.name.clone()))? {
            return Ok(existing);
        }

        self.create(new_entity)
    }

    pub fn find(&self, lookup: &EntityLookup) -> Result<Option<Entity>> {
        let (sql, params_vec): (&str, Vec<String>) = match lookup {
            EntityLookup::Id(id) => (
                "SELECT entity_id, name, ref, description, created_at, updated_at FROM entities WHERE entity_id = ?1",
                vec![id.to_string()],
            ),
            EntityLookup::Name(name) => (
                "SELECT entity_id, name, ref, description, created_at, updated_at FROM entities WHERE name = ?1",
                vec![name.clone()],
            ),
        };

        let mut stmt = self.conn.prepare(sql)?;
        let entity = stmt
            .query_row(params![params_vec[0]], map_entity_row)
            .optional()?;
        Ok(entity)
    }

    pub fn update(&self, lookup: &EntityLookup, update: &UpdateEntity) -> Result<Entity> {
        let existing = self
            .find(lookup)?
            .context("entity not found for update")?;

        let next_name = update.name.clone().unwrap_or(existing.name.clone());
        validate_name(&next_name).map_err(anyhow::Error::msg)?;

        let next_ref = update.r#ref.clone().or(existing.r#ref.clone());
        let next_description = update.description.clone().or(existing.description.clone());
        let now = now_utc();

        self.conn.execute(
            "UPDATE entities SET name = ?1, ref = ?2, description = ?3, updated_at = ?4 WHERE entity_id = ?5",
            params![next_name, next_ref, next_description, now, existing.entity_id],
        )?;

        self.find(&EntityLookup::Id(existing.entity_id))?
            .context("updated entity could not be reloaded")
    }

    pub fn list(&self) -> Result<Vec<Entity>> {
        let mut stmt = self.conn.prepare(
            "SELECT entity_id, name, ref, description, created_at, updated_at FROM entities ORDER BY entity_id ASC",
        )?;
        let rows = stmt.query_map([], map_entity_row)?;
        let entities = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(entities)
    }

    pub fn list_without_anchors(&self) -> Result<Vec<Entity>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.entity_id, e.name, e.ref, e.description, e.created_at, e.updated_at
             FROM entities e
             LEFT JOIN anchor_entities ae ON ae.entity_id = e.entity_id
             WHERE ae.entity_id IS NULL
             ORDER BY e.entity_id ASC",
        )?;
        let rows = stmt.query_map([], map_entity_row)?;
        let entities = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(entities)
    }

    pub fn link(&self, from: &EntityLookup, to: &EntityLookup, relation: &str) -> Result<EntityLink> {
        if relation.trim().is_empty() {
            bail!("relation cannot be empty");
        }
        let from_entity = self.find(from)?.context("source entity not found")?;
        let to_entity = self.find(to)?.context("target entity not found")?;
        let now = now_utc();
        self.conn.execute(
            "INSERT INTO entity_links (from_entity_id, to_entity_id, relation, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(from_entity_id, to_entity_id)
             DO UPDATE SET relation = excluded.relation, updated_at = excluded.updated_at",
            params![from_entity.entity_id, to_entity.entity_id, relation, now, now],
        )?;
        self.find_link(from_entity.entity_id, to_entity.entity_id)?
            .context("linked entity relation could not be reloaded")
    }

    pub fn unlink(&self, from: &EntityLookup, to: &EntityLookup) -> Result<()> {
        let from_entity = self.find(from)?.context("source entity not found")?;
        let to_entity = self.find(to)?.context("target entity not found")?;
        self.conn.execute(
            "DELETE FROM entity_links WHERE from_entity_id = ?1 AND to_entity_id = ?2",
            params![from_entity.entity_id, to_entity.entity_id],
        )?;
        Ok(())
    }

    pub fn list_outgoing_links(&self, entity_id: i64) -> Result<Vec<(EntityLink, Entity)>> {
        self.list_joined_links(
            "SELECT l.from_entity_id, l.to_entity_id, l.relation, l.created_at, l.updated_at,
                    e.entity_id, e.name, e.ref, e.description, e.created_at, e.updated_at
             FROM entity_links l
             JOIN entities e ON e.entity_id = l.to_entity_id
             WHERE l.from_entity_id = ?1
             ORDER BY e.entity_id ASC",
            entity_id,
        )
    }

    pub fn list_incoming_links(&self, entity_id: i64) -> Result<Vec<(EntityLink, Entity)>> {
        self.list_joined_links(
            "SELECT l.from_entity_id, l.to_entity_id, l.relation, l.created_at, l.updated_at,
                    e.entity_id, e.name, e.ref, e.description, e.created_at, e.updated_at
             FROM entity_links l
             JOIN entities e ON e.entity_id = l.from_entity_id
             WHERE l.to_entity_id = ?1
             ORDER BY e.entity_id ASC",
            entity_id,
        )
    }

    fn list_joined_links(&self, sql: &str, entity_id: i64) -> Result<Vec<(EntityLink, Entity)>> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![entity_id], |row| {
            let link = EntityLink {
                from_entity_id: row.get(0)?,
                to_entity_id: row.get(1)?,
                relation: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            };
            let entity = Entity {
                entity_id: row.get(5)?,
                name: row.get(6)?,
                r#ref: row.get(7)?,
                description: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            };
            Ok((link, entity))
        })?;
        let links = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(links)
    }

    fn find_link(&self, from_entity_id: i64, to_entity_id: i64) -> Result<Option<EntityLink>> {
        let mut stmt = self.conn.prepare(
            "SELECT from_entity_id, to_entity_id, relation, created_at, updated_at
             FROM entity_links WHERE from_entity_id = ?1 AND to_entity_id = ?2",
        )?;
        let link = stmt
            .query_row(params![from_entity_id, to_entity_id], |row| {
                Ok(EntityLink {
                    from_entity_id: row.get(0)?,
                    to_entity_id: row.get(1)?,
                    relation: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .optional()?;
        Ok(link)
    }
}

fn map_entity_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entity> {
    Ok(Entity {
        entity_id: row.get(0)?,
        name: row.get(1)?,
        r#ref: row.get(2)?,
        description: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
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
        let entity = repo.create(&NewEntity {
            name: "student".into(),
            r#ref: Some("./docs/student.md".into()),
            description: Some("A learner".into()),
        }).expect("create should succeed");
        assert!(entity.entity_id > 0);
        assert_eq!(entity.name, "student");
        assert_eq!(entity.r#ref.as_deref(), Some("./docs/student.md"));
        assert_eq!(entity.description.as_deref(), Some("A learner"));
    }

    #[test]
    fn ensure_returns_existing_entity() {
        let repo = setup_repo();
        let first = repo.ensure(&NewEntity {
            name: "student".into(),
            r#ref: None,
            description: None,
        }).unwrap();
        let second = repo.ensure(&NewEntity {
            name: "student".into(),
            r#ref: Some("./ignored.md".into()),
            description: Some("ignored".into()),
        }).unwrap();
        assert_eq!(first.entity_id, second.entity_id);
    }

    #[test]
    fn update_changes_fields() {
        let repo = setup_repo();
        let entity = repo.create(&NewEntity {
            name: "student".into(),
            r#ref: None,
            description: None,
        }).unwrap();
        let updated = repo.update(
            &EntityLookup::Id(entity.entity_id),
            &UpdateEntity {
                name: Some("student.profile".into()),
                r#ref: Some("./docs/profile.md".into()),
                description: Some("Profile entity".into()),
            },
        ).unwrap();
        assert_eq!(updated.name, "student.profile");
        assert_eq!(updated.r#ref.as_deref(), Some("./docs/profile.md"));
        assert_eq!(updated.description.as_deref(), Some("Profile entity"));
    }

    #[test]
    fn list_without_anchors_returns_orphan_entities() {
        let repo = setup_repo();
        repo.create(&NewEntity { name: "orphan.entity".into(), r#ref: None, description: None }).unwrap();
        let entities = repo.list_without_anchors().unwrap();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].name, "orphan.entity");
    }

    #[test]
    fn link_and_list_outgoing_links_work() {
        let repo = setup_repo();
        repo.create(&NewEntity { name: "Student".into(), r#ref: None, description: None }).unwrap();
        repo.create(&NewEntity { name: "Subject".into(), r#ref: None, description: None }).unwrap();
        let link = repo.link(
            &EntityLookup::Name("Student".into()),
            &EntityLookup::Name("Subject".into()),
            "student has subjects assigned to him each semester",
        ).unwrap();
        assert_eq!(link.relation, "student has subjects assigned to him each semester");
        let student = repo.find(&EntityLookup::Name("Student".into())).unwrap().unwrap();
        let outgoing = repo.list_outgoing_links(student.entity_id).unwrap();
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].1.name, "Subject");
    }

    #[test]
    fn list_incoming_links_returns_sources() {
        let repo = setup_repo();
        repo.create(&NewEntity { name: "Student".into(), r#ref: None, description: None }).unwrap();
        repo.create(&NewEntity { name: "Subject".into(), r#ref: None, description: None }).unwrap();
        repo.link(
            &EntityLookup::Name("Student".into()),
            &EntityLookup::Name("Subject".into()),
            "student has subjects assigned",
        ).unwrap();
        let subject = repo.find(&EntityLookup::Name("Subject".into())).unwrap().unwrap();
        let incoming = repo.list_incoming_links(subject.entity_id).unwrap();
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].1.name, "Student");
    }

    #[test]
    fn unlink_removes_directional_link() {
        let repo = setup_repo();
        repo.create(&NewEntity { name: "Student".into(), r#ref: None, description: None }).unwrap();
        repo.create(&NewEntity { name: "Subject".into(), r#ref: None, description: None }).unwrap();
        repo.link(
            &EntityLookup::Name("Student".into()),
            &EntityLookup::Name("Subject".into()),
            "student has subjects assigned",
        ).unwrap();
        repo.unlink(
            &EntityLookup::Name("Student".into()),
            &EntityLookup::Name("Subject".into()),
        ).unwrap();
        let student = repo.find(&EntityLookup::Name("Student".into())).unwrap().unwrap();
        let outgoing = repo.list_outgoing_links(student.entity_id).unwrap();
        assert!(outgoing.is_empty());
    }
}

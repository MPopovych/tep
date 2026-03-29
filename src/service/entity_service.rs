use anyhow::{Context, Result, bail};
use rusqlite::Connection;

use crate::anchor::Anchor;
use crate::entity::{Entity, NewEntity, UpdateEntity, parse_lookup};
use crate::repository::anchor_repository::AnchorRepository;
use crate::repository::entity_repository::EntityRepository;

pub struct EntityShowResult {
    pub entity: Entity,
    pub anchors: Vec<Anchor>,
}

pub struct EntityService<'a> {
    repo: EntityRepository<'a>,
    anchor_repo: AnchorRepository<'a>,
}

impl<'a> EntityService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            repo: EntityRepository::new(conn),
            anchor_repo: AnchorRepository::new(conn),
        }
    }

    pub fn create(&self, name: String, entity_ref: Option<String>) -> Result<Entity> {
        self.repo.create(&NewEntity {
            name,
            r#ref: entity_ref,
        })
    }

    pub fn ensure(&self, name: String, entity_ref: Option<String>) -> Result<Entity> {
        self.repo.ensure(&NewEntity {
            name,
            r#ref: entity_ref,
        })
    }

    pub fn show(&self, target: &str) -> Result<EntityShowResult> {
        let lookup = parse_lookup(target);
        let entity = self
            .repo
            .find(&lookup)?
            .with_context(|| format!("entity not found: {target}"))?;
        let anchors = self.anchor_repo.list_for_entity(entity.entity_id)?;
        Ok(EntityShowResult { entity, anchors })
    }

    pub fn edit(
        &self,
        target: &str,
        name: Option<String>,
        entity_ref: Option<String>,
    ) -> Result<Entity> {
        if name.is_none() && entity_ref.is_none() {
            bail!("entity edit requires at least one field to update");
        }

        self.repo.update(
            &parse_lookup(target),
            &UpdateEntity {
                name,
                r#ref: entity_ref,
            },
        )
    }

    pub fn list(&self) -> Result<Vec<Entity>> {
        self.repo.list()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::repository::anchor_entity_repository::AnchorEntityRepository;

    fn setup_service() -> EntityService<'static> {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql())
            .expect("schema should apply");
        EntityService::new(conn)
    }

    #[test]
    fn edit_requires_at_least_one_field() {
        let service = setup_service();
        let created = service
            .create("student".into(), None)
            .expect("create should succeed");

        let result = service.edit(&created.entity_id.to_string(), None, None);
        assert!(result.is_err());
    }

    #[test]
    fn show_supports_name_lookup() {
        let service = setup_service();
        service
            .create("student.permissions".into(), None)
            .expect("create should succeed");

        let result = service.show("student.permissions").expect("show should succeed");
        assert_eq!(result.entity.name, "student.permissions");
    }

    #[test]
    fn show_includes_related_anchors() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        let service = EntityService::new(conn);
        let anchor_repo = AnchorRepository::new(conn);
        let rel_repo = AnchorEntityRepository::new(conn);

        let entity = service.create("student".into(), None).unwrap();
        let anchor = anchor_repo.create(1, "./file.txt", Some(1), Some(0), Some(0)).unwrap();
        rel_repo.attach(anchor.anchor_id, entity.entity_id).unwrap();

        let result = service.show("student").unwrap();
        assert_eq!(result.anchors.len(), 1);
        assert_eq!(result.anchors[0].anchor_id, anchor.anchor_id);
    }
}

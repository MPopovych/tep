use anyhow::{Context, Result, bail};
use rusqlite::Connection;

use crate::entity::{Entity, NewEntity, UpdateEntity, parse_lookup};
use crate::repository::entity_repository::EntityRepository;

pub struct EntityService<'a> {
    repo: EntityRepository<'a>,
}

impl<'a> EntityService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            repo: EntityRepository::new(conn),
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

    pub fn read(&self, target: &str) -> Result<Entity> {
        let lookup = parse_lookup(target);
        self.repo
            .find(&lookup)?
            .with_context(|| format!("entity not found: {target}"))
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
    fn read_supports_name_lookup() {
        let service = setup_service();
        service
            .create("student.permissions".into(), None)
            .expect("create should succeed");

        let entity = service.read("student.permissions").expect("read should succeed");
        assert_eq!(entity.name, "student.permissions");
    }
}

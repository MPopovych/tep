// (#!#tep:entity.service)
// [#!#tep:entity.service](entity.service)
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::anchor::Anchor;
use crate::entity::{Entity, EntityLink, NewEntity, UpdateEntity, parse_lookup};
use crate::repository::anchor_repository::AnchorRepository;
use crate::repository::entity_repository::EntityRepository;
use crate::service::entity_context::extract_anchor_snippet;
use crate::service::entity_link_context::collect_link_context;
use crate::tep_tag::{ParsedEntityTag, ParsedRelationTag, parse_entity_tags, parse_relation_tags};
use crate::utils::workspace_scanner::{WorkspaceFile, collect_workspace_files};

pub struct EntityShowResult {
    pub entity: Entity,
    pub anchors: Vec<Anchor>,
    pub linked_entities: Vec<LinkedEntityContext>,
}

pub struct EntityContextAnchor {
    pub anchor: Anchor,
    pub snippet: Option<String>,
}

pub struct LinkedEntityContext {
    pub link: EntityLink,
    pub entity: Entity,
    pub depth: usize,
}

pub struct EntityContextResult {
    pub entity: Entity,
    pub anchors: Vec<EntityContextAnchor>,
    pub linked_entities: Vec<LinkedEntityContext>,
}

#[derive(Debug, Default)]
pub struct EntityAutoResult {
    pub files_processed: usize,
    pub declarations_seen: usize,
    pub relations_seen: usize,
    pub entities_ensured: usize,
    pub refs_filled: usize,
    pub descriptions_filled: usize,
    pub relations_synced: usize,
    pub warnings: Vec<String>,
}

pub struct EntityService<'a> {
    workspace_root: PathBuf,
    repo: EntityRepository<'a>,
    anchor_repo: AnchorRepository<'a>,
}

impl<'a> EntityService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::with_workspace_root(conn, workspace_root)
    }

    pub fn with_workspace_root(conn: &'a Connection, workspace_root: impl Into<PathBuf>) -> Self {
        let workspace_root = workspace_root.into();
        Self {
            workspace_root: workspace_root.clone(),
            repo: EntityRepository::new(conn),
            anchor_repo: AnchorRepository::with_workspace_root(conn, workspace_root),
        }
    }

    pub fn auto(&self, paths: &[String]) -> Result<EntityAutoResult> {
        let files = collect_workspace_files(&self.workspace_root, paths)?;
        let mut result = EntityAutoResult::default();
        for file in files {
            self.auto_file(&file, &mut result)?;
        }
        Ok(result)
    }

    pub fn show(&self, target: &str) -> Result<EntityShowResult> {
        let lookup = parse_lookup(target);
        let entity = self
            .repo
            .find(&lookup)?
            .with_context(|| format!("entity not found: {target}"))?;
        let anchors = self.anchor_repo.list_for_entity(entity.entity_id)?;
        let linked_entities = collect_link_context(&self.repo, entity.entity_id, 1)?;
        Ok(EntityShowResult {
            entity,
            anchors,
            linked_entities,
        })
    }

    pub fn context(&self, target: &str, link_depth: usize) -> Result<EntityContextResult> {
        let lookup = parse_lookup(target);
        let entity = self
            .repo
            .find(&lookup)?
            .with_context(|| format!("entity not found: {target}"))?;
        let anchors =
            self.build_anchor_context(&self.anchor_repo.list_for_entity(entity.entity_id)?);
        let linked_entities = collect_link_context(&self.repo, entity.entity_id, link_depth)?;
        Ok(EntityContextResult {
            entity,
            anchors,
            linked_entities,
        })
    }

    pub fn list(&self) -> Result<Vec<Entity>> {
        self.repo.list()
    }

    fn build_anchor_context(&self, anchors: &[Anchor]) -> Vec<EntityContextAnchor> {
        anchors
            .iter()
            .map(|anchor| EntityContextAnchor {
                snippet: extract_anchor_snippet(anchor).ok().flatten(),
                anchor: anchor.clone(),
            })
            .collect()
    }

    fn auto_file(&self, file: &WorkspaceFile, result: &mut EntityAutoResult) -> Result<()> {
        if !file.absolute_path.is_file() {
            return Ok(());
        }

        let content = fs::read_to_string(&file.absolute_path)
            .with_context(|| format!("failed to read {}", file.absolute_path.display()))?;
        let entity_tags = parse_entity_tags(&content);
        let relation_tags = parse_relation_tags(&content);
        if entity_tags.is_empty() && relation_tags.is_empty() {
            return Ok(());
        }

        result.declarations_seen += entity_tags.len();
        result.relations_seen += relation_tags.len();

        for tag in &entity_tags {
            self.sync_entity_tag(&file.display_path, tag, result)?;
        }
        for tag in &relation_tags {
            self.sync_relation_tag(tag, result)?;
        }
        result.files_processed += 1;

        Ok(())
    }

    fn sync_entity_tag(
        &self,
        file_path: &str,
        tag: &ParsedEntityTag,
        result: &mut EntityAutoResult,
    ) -> Result<()> {
        let mut entity = self.repo.ensure(&NewEntity {
            name: tag.name.clone(),
            r#ref: None,
            description: None,
        })?;
        result.entities_ensured += 1;

        entity = self.apply_ref_metadata(file_path, tag, entity, result)?;
        self.apply_description_metadata(tag, entity, result)?;
        self.collect_metadata_warnings(
            &tag.metadata.duplicate_keys,
            &tag.metadata.unknown_fields,
            &tag.name,
            "entity",
            result,
        );
        Ok(())
    }

    fn apply_ref_metadata(
        &self,
        file_path: &str,
        tag: &ParsedEntityTag,
        entity: Entity,
        result: &mut EntityAutoResult,
    ) -> Result<Entity> {
        if let Some(tag_ref) = tag.metadata.fields.get("ref") {
            let updated = self.repo.update(
                &parse_lookup(&entity.entity_id.to_string()),
                &UpdateEntity {
                    name: None,
                    r#ref: Some(tag_ref.clone()),
                    description: None,
                },
            )?;
            if entity.r#ref.as_deref() != Some(tag_ref.as_str()) {
                result.refs_filled += 1;
                if entity.r#ref.is_some() && entity.r#ref.as_deref() != Some(tag_ref.as_str()) {
                    result.warnings.push(format!(
                        "entity '{}' ref overwritten by later declaration",
                        tag.name
                    ));
                }
            }
            return Ok(updated);
        }

        if entity.r#ref.is_none() {
            let updated = self.repo.update(
                &parse_lookup(&entity.entity_id.to_string()),
                &UpdateEntity {
                    name: None,
                    r#ref: Some(file_path.to_string()),
                    description: None,
                },
            )?;
            result.refs_filled += 1;
            return Ok(updated);
        }

        Ok(entity)
    }

    fn apply_description_metadata(
        &self,
        tag: &ParsedEntityTag,
        entity: Entity,
        result: &mut EntityAutoResult,
    ) -> Result<Entity> {
        let Some(description) = tag.metadata.fields.get("description") else {
            return Ok(entity);
        };

        let updated = self.repo.update(
            &parse_lookup(&entity.entity_id.to_string()),
            &UpdateEntity {
                name: None,
                r#ref: None,
                description: Some(description.clone()),
            },
        )?;
        if entity.description.as_deref() != Some(description.as_str()) {
            result.descriptions_filled += 1;
            if entity.description.is_some()
                && entity.description.as_deref() != Some(description.as_str())
            {
                result.warnings.push(format!(
                    "entity '{}' description overwritten by later declaration",
                    tag.name
                ));
            }
        }
        Ok(updated)
    }

    fn sync_relation_tag(&self, tag: &ParsedRelationTag, result: &mut EntityAutoResult) -> Result<()> {
        self.repo.ensure(&NewEntity {
            name: tag.from.clone(),
            r#ref: None,
            description: None,
        })?;
        self.repo.ensure(&NewEntity {
            name: tag.to.clone(),
            r#ref: None,
            description: None,
        })?;
        result.entities_ensured += 2;

        let description = tag
            .metadata
            .fields
            .get("description")
            .cloned()
            .unwrap_or_else(|| "related to".to_string());
        let existing = self.repo.find_link_by_name(&tag.from, &tag.to)?;
        self.repo.link(&parse_lookup(&tag.from), &parse_lookup(&tag.to), &description)?;
        result.relations_synced += 1;
        if let Some(existing) = existing
            && existing.relation != description
        {
            result.warnings.push(format!(
                "relation '{} -> {}' description overwritten by later declaration",
                tag.from, tag.to
            ));
        }
        self.collect_metadata_warnings(
            &tag.metadata.duplicate_keys,
            &tag.metadata.unknown_fields,
            &format!("{} -> {}", tag.from, tag.to),
            "relation",
            result,
        );
        Ok(())
    }

    fn collect_metadata_warnings(
        &self,
        duplicate_keys: &[String],
        unknown_fields: &[String],
        target: &str,
        kind: &str,
        result: &mut EntityAutoResult,
    ) {
        for key in duplicate_keys {
            result.warnings.push(format!(
                "duplicate metadata key '{}' in {} {}",
                key, kind, target
            ));
        }
        for key in unknown_fields {
            result.warnings.push(format!(
                "unknown metadata field '{}' in {} {}",
                key, kind, target
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::repository::entity_repository::EntityRepository;

    fn setup_service() -> EntityService<'static> {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        db::ensure_schema(conn).expect("schema should apply");
        EntityService::with_workspace_root(conn, "/tmp/project")
    }

    #[test]
    fn build_anchor_context_collects_deduped_files() {
        let service = setup_service();
        let anchor = Anchor {
            anchor_id: 1,
            version: 1,
            name: None,
            file_path: "./docs/a.md".into(),
            line: Some(1),
            shift: Some(0),
            offset: Some(0),
            description: None,
            created_at: "1".into(),
            updated_at: "1".into(),
        };
        let anchors = service.build_anchor_context(&[anchor.clone(), anchor]);
        assert_eq!(anchors.len(), 2);
    }

    #[test]
    fn show_supports_name_lookup() {
        let service = setup_service();
        let repo = EntityRepository::new(service.anchor_repo.conn);
        repo.create(&NewEntity {
            name: "student.permissions".into(),
            r#ref: None,
            description: None,
        })
        .unwrap();

        let result = service.show("student.permissions").unwrap();
        assert_eq!(result.entity.name, "student.permissions");
    }

    #[test]
    fn show_includes_linked_entities() {
        let service = setup_service();
        let repo = EntityRepository::new(service.anchor_repo.conn);
        for name in ["student", "subject", "teacher"] {
            repo.create(&NewEntity {
                name: name.into(),
                r#ref: None,
                description: None,
            })
            .unwrap();
        }
        repo.link(&parse_lookup("student"), &parse_lookup("subject"), "student has subjects")
            .unwrap();
        repo.link(&parse_lookup("teacher"), &parse_lookup("student"), "teacher mentors student")
            .unwrap();

        let result = service.show("student").unwrap();
        assert_eq!(result.linked_entities.len(), 2);
    }

    #[test]
    fn context_traverses_link_depth_and_dedupes_entities() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        db::ensure_schema(conn).unwrap();
        let service = EntityService::with_workspace_root(conn, "/tmp/project");
        let entity_repo = EntityRepository::new(conn);

        for (name, desc) in [
            ("student", "A learner"),
            ("subject", "A course"),
            ("semester", "A term"),
            ("teacher", "An instructor"),
            ("department", "An org unit"),
        ] {
            entity_repo
                .create(&NewEntity {
                    name: name.into(),
                    r#ref: Some(format!("./docs/{name}.md")),
                    description: Some(desc.into()),
                })
                .unwrap();
        }

        entity_repo
            .link(&parse_lookup("student"), &parse_lookup("subject"), "student has subjects")
            .unwrap();
        entity_repo
            .link(
                &parse_lookup("subject"),
                &parse_lookup("semester"),
                "subject is scheduled in semester",
            )
            .unwrap();
        entity_repo
            .link(
                &parse_lookup("semester"),
                &parse_lookup("student"),
                "semester contains student records",
            )
            .unwrap();
        entity_repo
            .link(&parse_lookup("teacher"), &parse_lookup("student"), "teacher mentors student")
            .unwrap();
        entity_repo
            .link(
                &parse_lookup("department"),
                &parse_lookup("teacher"),
                "department employs teacher",
            )
            .unwrap();

        let result = service.context("student", 2).unwrap();
        assert_eq!(result.linked_entities.len(), 4);
    }

    #[test]
    fn auto_merges_entity_metadata_and_warns_on_overwrite() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join("a.txt"),
            "#!#tep:(student){ref=\"./docs/student.md\", description=\"A learner\"}\n#!#tep:(student){description=\"Learner v2\"}\n",
        )
        .unwrap();
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        db::ensure_schema(conn).unwrap();
        let service = EntityService::with_workspace_root(conn, temp.path());

        let result = service.auto(&["./a.txt".into()]).unwrap();
        let entity = EntityRepository::new(conn)
            .find(&parse_lookup("student"))
            .unwrap()
            .unwrap();

        assert_eq!(entity.r#ref.as_deref(), Some("./docs/student.md"));
        assert_eq!(entity.description.as_deref(), Some("Learner v2"));
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("description overwritten")));
    }

    #[test]
    fn auto_creates_relations_from_tags() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join("a.txt"),
            "#!#tep:(student)->(subject){description=\"has subject\"}\n",
        )
        .unwrap();
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        db::ensure_schema(conn).unwrap();
        let service = EntityService::with_workspace_root(conn, temp.path());

        let result = service.auto(&["./a.txt".into()]).unwrap();
        let repo = EntityRepository::new(conn);
        let link = repo.find_link_by_name("student", "subject").unwrap().unwrap();

        assert_eq!(result.relations_synced, 1);
        assert_eq!(link.relation, "has subject");
    }

    #[test]
    fn auto_warns_on_unknown_entity_metadata_fields() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join("a.txt"),
            "#!#tep:(student){foo=\"bar\"}\n",
        )
        .unwrap();
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        db::ensure_schema(conn).unwrap();
        let service = EntityService::with_workspace_root(conn, temp.path());

        let result = service.auto(&["./a.txt".into()]).unwrap();
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("unknown metadata field 'foo'")));
    }
}

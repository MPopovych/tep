// (#!#1#tep:service.entity.context)
// [#!#1#tep:48](service.entity.auto)
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use rusqlite::Connection;

use crate::anchor::Anchor;
use crate::entity::{Entity, EntityLink, NewEntity, ParsedEntityDeclaration, UpdateEntity, materialize_entity_declaration, parse_entity_declarations, parse_lookup};
use crate::filter::tep_ignore_filter::TepIgnoreFilter;
use crate::repository::anchor_entity_repository::AnchorEntityRepository;
use crate::repository::anchor_repository::AnchorRepository;
use crate::repository::entity_repository::EntityRepository;
use crate::service::entity_context::extract_anchor_snippet;
use crate::service::entity_link_context::collect_link_context;
use crate::utils::path::{display_path, resolve_from_workspace};

pub struct EntityShowResult {
    pub entity: Entity,
    pub anchors: Vec<Anchor>,
    pub outgoing_links: Vec<(EntityLink, Entity)>,
    pub incoming_links: Vec<(EntityLink, Entity)>,
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
    pub files: Vec<String>,
    pub linked_entities: Vec<LinkedEntityContext>,
}

pub struct EntityAutoResult {
    pub files_processed: usize,
    pub declarations_seen: usize,
    pub entities_ensured: usize,
    pub refs_filled: usize,
    pub anchors_created: usize,
    pub relations_synced: usize,
}

pub struct EntityLinkResult {
    pub from: Entity,
    pub to: Entity,
    pub relation: String,
}

#[derive(Debug, Clone)]
struct WorkspaceFile {
    absolute_path: PathBuf,
    display_path: String,
}

pub struct EntityService<'a> {
    workspace_root: PathBuf,
    repo: EntityRepository<'a>,
    anchor_repo: AnchorRepository<'a>,
    anchor_entity_repo: AnchorEntityRepository<'a>,
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
            anchor_entity_repo: AnchorEntityRepository::new(conn),
        }
    }

    pub fn create(&self, name: String, entity_ref: Option<String>, description: Option<String>) -> Result<Entity> {
        self.repo.create(&NewEntity {
            name,
            r#ref: entity_ref,
            description,
        })
    }

    pub fn ensure(&self, name: String, entity_ref: Option<String>) -> Result<Entity> {
        self.repo.ensure(&NewEntity {
            name,
            r#ref: entity_ref,
            description: None,
        })
    }

    pub fn auto(&self, paths: &[String]) -> Result<EntityAutoResult> {
        let files = self.collect_workspace_files(paths)?;

        let mut result = EntityAutoResult {
            files_processed: 0,
            declarations_seen: 0,
            entities_ensured: 0,
            refs_filled: 0,
            anchors_created: 0,
            relations_synced: 0,
        };

        for file in files {
            let changed = self.auto_file(&file, &mut result)?;
            if changed {
                result.files_processed += 1;
            }
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
        let outgoing_links = self.repo.list_outgoing_links(entity.entity_id)?;
        let incoming_links = self.repo.list_incoming_links(entity.entity_id)?;
        Ok(EntityShowResult { entity, anchors, outgoing_links, incoming_links })
    }

    pub fn context(&self, target: &str, link_depth: usize) -> Result<EntityContextResult> {
        let shown = self.show(target)?;
        let root_entity = shown.entity.clone();
        let (anchors, files) = self.build_anchor_context(&shown.anchors);
        let linked_entities = collect_link_context(&self.repo, root_entity.entity_id, link_depth)?;

        Ok(EntityContextResult {
            entity: root_entity,
            anchors,
            files,
            linked_entities,
        })
    }

    pub fn edit(
        &self,
        target: &str,
        name: Option<String>,
        entity_ref: Option<String>,
        description: Option<String>,
    ) -> Result<Entity> {
        if name.is_none() && entity_ref.is_none() && description.is_none() {
            bail!("entity edit requires at least one field to update");
        }

        self.repo.update(
            &parse_lookup(target),
            &UpdateEntity {
                name,
                r#ref: entity_ref,
                description,
            },
        )
    }

    pub fn link(&self, from: &str, to: &str, relation: &str) -> Result<EntityLinkResult> {
        let from_lookup = parse_lookup(from);
        let to_lookup = parse_lookup(to);
        let link = self.repo.link(&from_lookup, &to_lookup, relation)?;
        let from_entity = self.repo.find(&from_lookup)?.context("source entity not found")?;
        let to_entity = self.repo.find(&to_lookup)?.context("target entity not found")?;
        Ok(EntityLinkResult {
            from: from_entity,
            to: to_entity,
            relation: link.relation,
        })
    }

    pub fn unlink(&self, from: &str, to: &str) -> Result<(Entity, Entity)> {
        let from_lookup = parse_lookup(from);
        let to_lookup = parse_lookup(to);
        let from_entity = self.repo.find(&from_lookup)?.context("source entity not found")?;
        let to_entity = self.repo.find(&to_lookup)?.context("target entity not found")?;
        self.repo.unlink(&from_lookup, &to_lookup)?;
        Ok((from_entity, to_entity))
    }

    pub fn list(&self) -> Result<Vec<Entity>> {
        self.repo.list()
    }

    fn collect_workspace_files(&self, paths: &[String]) -> Result<Vec<WorkspaceFile>> {
        let filter = TepIgnoreFilter::for_workspace_root(&self.workspace_root);
        let files = filter.collect_paths(paths)?;
        Ok(files
            .into_iter()
            .map(|path| WorkspaceFile {
                absolute_path: resolve_from_workspace(&path, &self.workspace_root),
                display_path: display_path(&path),
            })
            .collect())
    }

    fn build_anchor_context(&self, anchors: &[Anchor]) -> (Vec<EntityContextAnchor>, Vec<String>) {
        let mut files = Vec::new();
        let mut context_anchors = Vec::new();

        for anchor in anchors {
            if !files.contains(&anchor.file_path) {
                files.push(anchor.file_path.clone());
            }
            let snippet = extract_anchor_snippet(anchor).ok().flatten();
            context_anchors.push(EntityContextAnchor {
                anchor: anchor.clone(),
                snippet,
            });
        }

        (context_anchors, files)
    }

    fn auto_file(&self, file: &WorkspaceFile, result: &mut EntityAutoResult) -> Result<bool> {
        if !file.absolute_path.is_file() {
            return Ok(false);
        }

        let original = fs::read_to_string(&file.absolute_path)
            .with_context(|| format!("failed to read {}", file.absolute_path.display()))?;
        let declarations = parse_entity_declarations(&original);
        if declarations.is_empty() {
            return Ok(false);
        }

        result.declarations_seen += declarations.len();

        let mut rewritten = String::with_capacity(original.len() + 64);
        let mut cursor = 0usize;

        for declaration in declarations {
            rewritten.push_str(&original[cursor..declaration.start_offset]);
            let replacement = self.sync_declaration(&file.display_path, &declaration, result)?;
            rewritten.push_str(&replacement);
            cursor = declaration.start_offset + declaration.raw.len();
        }

        rewritten.push_str(&original[cursor..]);

        if rewritten != original {
            fs::write(&file.absolute_path, rewritten)
                .with_context(|| format!("failed to write {}", file.absolute_path.display()))?;
        }

        Ok(true)
    }

    fn sync_declaration(
        &self,
        file_path: &str,
        declaration: &ParsedEntityDeclaration,
        result: &mut EntityAutoResult,
    ) -> Result<String> {
        let entity = self.ensure_entity_for_declaration(declaration, file_path, result)?;
        let anchor = self.ensure_anchor_for_declaration(file_path, declaration, entity.entity_id, result)?;
        self.anchor_entity_repo.attach(anchor.anchor_id, entity.entity_id)?;
        result.relations_synced += 1;

        Ok(materialize_entity_declaration(declaration, 1))
    }

    fn ensure_entity_for_declaration(
        &self,
        declaration: &ParsedEntityDeclaration,
        file_path: &str,
        result: &mut EntityAutoResult,
    ) -> Result<Entity> {
        let mut entity = self.repo.ensure(&NewEntity {
            name: declaration.name.clone(),
            r#ref: None,
            description: None,
        })?;
        result.entities_ensured += 1;

        if entity.r#ref.is_none() {
            entity = self.repo.update(
                &parse_lookup(&entity.entity_id.to_string()),
                &UpdateEntity {
                    name: None,
                    r#ref: Some(file_path.to_string()),
                    description: None,
                },
            )?;
            result.refs_filled += 1;
        }

        Ok(entity)
    }

    fn ensure_anchor_for_declaration(
        &self,
        file_path: &str,
        declaration: &ParsedEntityDeclaration,
        entity_id: i64,
        result: &mut EntityAutoResult,
    ) -> Result<Anchor> {
        let anchor = if declaration.version.is_some() {
            if let Some(existing) = self.anchor_repo.find_latest_for_entity_in_file(entity_id, file_path)? {
                self.anchor_repo.update_location(
                    existing.anchor_id,
                    file_path,
                    Some(declaration.line),
                    Some(declaration.shift),
                    Some(declaration.start_offset as i64),
                )?
            } else {
                self.create_declaration_anchor(file_path, declaration, result)?
            }
        } else {
            self.create_declaration_anchor(file_path, declaration, result)?
        };

        Ok(anchor)
    }

    fn create_declaration_anchor(
        &self,
        file_path: &str,
        declaration: &ParsedEntityDeclaration,
        result: &mut EntityAutoResult,
    ) -> Result<Anchor> {
        let created = self.anchor_repo.create(
            declaration.version.unwrap_or(1),
            file_path,
            Some(declaration.line),
            Some(declaration.shift),
            Some(declaration.start_offset as i64),
        )?;
        result.anchors_created += 1;
        Ok(created)
    }
}

// #tepignoreafter
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::repository::entity_repository::EntityRepository;

    fn setup_service() -> EntityService<'static> {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql())
            .expect("schema should apply");
        EntityService::with_workspace_root(conn, "/tmp/project")
    }

    #[test]
    fn build_anchor_context_collects_deduped_files() {
        let service = setup_service();
        let anchor = Anchor {
            anchor_id: 1,
            version: 1,
            file_path: "./docs/a.md".into(),
            line: Some(1),
            shift: Some(0),
            offset: Some(0),
            created_at: "1".into(),
            updated_at: "1".into(),
        };
        let (anchors, files) = service.build_anchor_context(&[anchor.clone(), anchor]);
        assert_eq!(anchors.len(), 2);
        assert_eq!(files, vec!["./docs/a.md".to_string()]);
    }

    #[test]
    fn edit_requires_at_least_one_field() {
        let service = setup_service();
        let created = service
            .create("student".into(), None, None)
            .expect("create should succeed");

        let result = service.edit(&created.entity_id.to_string(), None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn show_supports_name_lookup() {
        let service = setup_service();
        service
            .create("student.permissions".into(), None, None)
            .expect("create should succeed");

        let result = service.show("student.permissions").expect("show should succeed");
        assert_eq!(result.entity.name, "student.permissions");
    }

    #[test]
    fn show_includes_incoming_and_outgoing_links() {
        let service = setup_service();
        service.create("Student".into(), None, None).unwrap();
        service.create("Subject".into(), None, None).unwrap();
        service.create("Teacher".into(), None, None).unwrap();
        service.link("Student", "Subject", "student has subjects").unwrap();
        service.link("Teacher", "Student", "teacher mentors student").unwrap();
        let result = service.show("Student").unwrap();
        assert_eq!(result.outgoing_links.len(), 1);
        assert_eq!(result.outgoing_links[0].1.name, "Subject");
        assert_eq!(result.incoming_links.len(), 1);
        assert_eq!(result.incoming_links[0].1.name, "Teacher");
    }

    #[test]
    fn context_merges_links_and_preserves_direction_in_edge() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        let service = EntityService::with_workspace_root(conn, "/tmp/project");
        let entity_repo = EntityRepository::new(conn);

        entity_repo.create(&NewEntity { name: "student".into(), r#ref: Some("./docs/student.md".into()), description: Some("A learner".into()) }).unwrap();
        entity_repo.create(&NewEntity { name: "subject".into(), r#ref: Some("./docs/subject.md".into()), description: Some("A course".into()) }).unwrap();
        entity_repo.create(&NewEntity { name: "teacher".into(), r#ref: Some("./docs/teacher.md".into()), description: Some("An instructor".into()) }).unwrap();
        entity_repo.link(&parse_lookup("student"), &parse_lookup("subject"), "student has subjects").unwrap();
        entity_repo.link(&parse_lookup("teacher"), &parse_lookup("student"), "teacher mentors student").unwrap();

        let result = service.context("student", 1).unwrap();
        assert_eq!(result.linked_entities.len(), 2);
        assert!(result.linked_entities.iter().any(|item| item.entity.name == "subject" && item.link.from_entity_id != item.link.to_entity_id && item.depth == 1));
        assert!(result.linked_entities.iter().any(|item| item.entity.name == "teacher" && item.link.from_entity_id != item.link.to_entity_id && item.depth == 1));
    }

    #[test]
    fn context_traverses_link_depth_and_dedupes_entities() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        let service = EntityService::with_workspace_root(conn, "/tmp/project");
        let entity_repo = EntityRepository::new(conn);

        entity_repo.create(&NewEntity { name: "student".into(), r#ref: Some("./docs/student.md".into()), description: Some("A learner".into()) }).unwrap();
        entity_repo.create(&NewEntity { name: "subject".into(), r#ref: Some("./docs/subject.md".into()), description: Some("A course".into()) }).unwrap();
        entity_repo.create(&NewEntity { name: "semester".into(), r#ref: Some("./docs/semester.md".into()), description: Some("A term".into()) }).unwrap();
        entity_repo.create(&NewEntity { name: "teacher".into(), r#ref: Some("./docs/teacher.md".into()), description: Some("An instructor".into()) }).unwrap();
        entity_repo.create(&NewEntity { name: "department".into(), r#ref: Some("./docs/department.md".into()), description: Some("An org unit".into()) }).unwrap();

        entity_repo.link(&parse_lookup("student"), &parse_lookup("subject"), "student has subjects").unwrap();
        entity_repo.link(&parse_lookup("subject"), &parse_lookup("semester"), "subject is scheduled in semester").unwrap();
        entity_repo.link(&parse_lookup("semester"), &parse_lookup("student"), "semester contains student records").unwrap();
        entity_repo.link(&parse_lookup("teacher"), &parse_lookup("student"), "teacher mentors student").unwrap();
        entity_repo.link(&parse_lookup("department"), &parse_lookup("teacher"), "department employs teacher").unwrap();

        let result = service.context("student", 2).unwrap();
        assert_eq!(result.linked_entities.len(), 4);
        assert!(result.linked_entities.iter().any(|item| item.entity.name == "subject" && item.depth == 1));
        assert!(result.linked_entities.iter().any(|item| item.entity.name == "semester" && item.depth == 1));
        assert!(result.linked_entities.iter().any(|item| item.entity.name == "teacher" && item.depth == 1));
        assert!(result.linked_entities.iter().any(|item| item.entity.name == "department" && item.depth == 2));
        assert!(!result.linked_entities.iter().any(|item| item.entity.name == "student"));
    }

    #[test]
    fn context_includes_ref_files_snippet_and_links_by_default() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        let service = EntityService::with_workspace_root(conn, "/tmp/project");
        let entity_repo = EntityRepository::new(conn);
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "before line\nanchor line\nafter line\n").unwrap();
        let entity = entity_repo.create(&NewEntity {
            name: "student".into(),
            r#ref: Some("./docs/student.md".into()),
            description: Some("A learner".into()),
        }).unwrap();
        entity_repo.create(&NewEntity {
            name: "subject".into(),
            r#ref: Some("./docs/subject.md".into()),
            description: Some("A course".into()),
        }).unwrap();
        entity_repo.link(&parse_lookup("student"), &parse_lookup("subject"), "student has subjects").unwrap();
        let anchor = service.anchor_repo.create(1, file.to_string_lossy().as_ref(), Some(2), Some(0), Some(12)).unwrap();
        service.anchor_entity_repo.attach(anchor.anchor_id, entity.entity_id).unwrap();

        let result = service.context("student", 1).unwrap();
        assert_eq!(result.entity.r#ref.as_deref(), Some("./docs/student.md"));
        assert_eq!(result.files.len(), 1);
        assert!(result.files[0].contains("note.txt"));
        assert_eq!(result.anchors.len(), 1);
        assert_eq!(result.linked_entities.len(), 1);
        let snippet = result.anchors[0].snippet.as_deref().unwrap();
        assert!(snippet.contains("before line"));
        assert!(snippet.contains("anchor line"));
        assert!(snippet.contains("after line"));
    }
}

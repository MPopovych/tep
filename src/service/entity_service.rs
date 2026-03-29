use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use rusqlite::Connection;

use crate::anchor::Anchor;
use crate::entity::{Entity, NewEntity, ParsedEntityDeclaration, UpdateEntity, materialize_entity_declaration, parse_entity_declarations, parse_lookup};
use crate::filter::tep_ignore_filter::TepIgnoreFilter;
use crate::repository::anchor_entity_repository::AnchorEntityRepository;
use crate::repository::anchor_repository::AnchorRepository;
use crate::repository::entity_repository::EntityRepository;

pub struct EntityShowResult {
    pub entity: Entity,
    pub anchors: Vec<Anchor>,
}

pub struct EntityContextAnchor {
    pub anchor: Anchor,
    pub snippet: Option<String>,
}

pub struct EntityContextResult {
    pub entity: Entity,
    pub anchors: Vec<EntityContextAnchor>,
    pub files: Vec<String>,
}

pub struct EntityAutoResult {
    pub files_processed: usize,
    pub declarations_seen: usize,
    pub entities_ensured: usize,
    pub refs_filled: usize,
    pub anchors_created: usize,
    pub relations_synced: usize,
}

pub struct EntityService<'a> {
    repo: EntityRepository<'a>,
    anchor_repo: AnchorRepository<'a>,
    anchor_entity_repo: AnchorEntityRepository<'a>,
}

impl<'a> EntityService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            repo: EntityRepository::new(conn),
            anchor_repo: AnchorRepository::new(conn),
            anchor_entity_repo: AnchorEntityRepository::new(conn),
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

    pub fn auto(&self, paths: &[String]) -> Result<EntityAutoResult> {
        let workspace_root = std::env::current_dir().context("failed to determine current directory")?;
        let filter = TepIgnoreFilter::for_workspace_root(workspace_root);
        let files = filter.collect_paths(paths)?;

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
        Ok(EntityShowResult { entity, anchors })
    }

    pub fn context(&self, target: &str) -> Result<EntityContextResult> {
        let shown = self.show(target)?;
        let mut files = Vec::new();
        let mut anchors = Vec::new();

        for anchor in shown.anchors {
            if !files.contains(&anchor.file_path) {
                files.push(anchor.file_path.clone());
            }
            let snippet = extract_anchor_snippet(&anchor).ok().flatten();
            anchors.push(EntityContextAnchor { anchor, snippet });
        }

        Ok(EntityContextResult {
            entity: shown.entity,
            anchors,
            files,
        })
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

    fn auto_file(&self, path: &Path, result: &mut EntityAutoResult) -> Result<bool> {
        if !path.is_file() {
            return Ok(false);
        }

        let original = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let declarations = parse_entity_declarations(&original);
        if declarations.is_empty() {
            return Ok(false);
        }

        result.declarations_seen += declarations.len();

        let mut rewritten = String::with_capacity(original.len() + 64);
        let mut cursor = 0usize;
        let file_path = path.to_string_lossy().to_string();

        for declaration in declarations {
            rewritten.push_str(&original[cursor..declaration.start_offset]);
            let replacement = self.sync_declaration(&file_path, &declaration, result)?;
            rewritten.push_str(&replacement);
            cursor = declaration.start_offset + declaration.raw.len();
        }

        rewritten.push_str(&original[cursor..]);

        if rewritten != original {
            fs::write(path, rewritten)
                .with_context(|| format!("failed to write {}", path.display()))?;
        }

        Ok(true)
    }

    fn sync_declaration(
        &self,
        file_path: &str,
        declaration: &ParsedEntityDeclaration,
        result: &mut EntityAutoResult,
    ) -> Result<String> {
        let mut entity = self.repo.ensure(&NewEntity {
            name: declaration.name.clone(),
            r#ref: None,
        })?;
        result.entities_ensured += 1;

        if entity.r#ref.is_none() {
            entity = self.repo.update(
                &parse_lookup(&entity.entity_id.to_string()),
                &UpdateEntity {
                    name: None,
                    r#ref: Some(file_path.to_string()),
                },
            )?;
            result.refs_filled += 1;
        }

        let anchor = if declaration.version.is_some() {
            if let Some(existing) = self.anchor_repo.find_latest_for_entity_in_file(entity.entity_id, file_path)? {
                self.anchor_repo.update_location(
                    existing.anchor_id,
                    file_path,
                    Some(declaration.line),
                    Some(declaration.shift),
                    Some(declaration.start_offset as i64),
                )?
            } else {
                let created = self.anchor_repo.create(
                    declaration.version.unwrap_or(1),
                    file_path,
                    Some(declaration.line),
                    Some(declaration.shift),
                    Some(declaration.start_offset as i64),
                )?;
                result.anchors_created += 1;
                created
            }
        } else {
            let created = self.anchor_repo.create(
                1,
                file_path,
                Some(declaration.line),
                Some(declaration.shift),
                Some(declaration.start_offset as i64),
            )?;
            result.anchors_created += 1;
            created
        };

        self.anchor_entity_repo.attach(anchor.anchor_id, entity.entity_id)?;
        result.relations_synced += 1;

        Ok(materialize_entity_declaration(declaration, 1))
    }
}

fn extract_anchor_snippet(anchor: &Anchor) -> Result<Option<String>> {
    let text = fs::read_to_string(&anchor.file_path)
        .with_context(|| format!("failed to read {}", anchor.file_path))?;
    let offset = match anchor.offset {
        Some(value) if value >= 0 => value as usize,
        _ => return Ok(None),
    };
    if offset >= text.len() {
        return Ok(None);
    }

    let start = text[..offset].rfind('\n').map(|idx| idx + 1).unwrap_or(0);
    let end = text[offset..]
        .find('\n')
        .map(|idx| offset + idx)
        .unwrap_or(text.len());
    let line = text[start..end].trim();
    if line.is_empty() {
        return Ok(None);
    }

    Ok(Some(line.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::repository::entity_repository::EntityRepository;

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

    #[test]
    fn context_includes_ref_files_and_snippet() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        let service = EntityService::new(conn);
        let entity_repo = EntityRepository::new(conn);
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "hello\nanchor line\n").unwrap();
        let entity = entity_repo.create(&NewEntity {
            name: "student".into(),
            r#ref: Some("./docs/student.md".into()),
        }).unwrap();
        let anchor = service.anchor_repo.create(1, file.to_string_lossy().as_ref(), Some(2), Some(0), Some(6)).unwrap();
        service.anchor_entity_repo.attach(anchor.anchor_id, entity.entity_id).unwrap();

        let result = service.context("student").unwrap();
        assert_eq!(result.entity.r#ref.as_deref(), Some("./docs/student.md"));
        assert_eq!(result.files.len(), 1);
        assert!(result.files[0].contains("note.txt"));
        assert_eq!(result.anchors.len(), 1);
        assert_eq!(result.anchors[0].snippet.as_deref(), Some("anchor line"));
    }

    #[test]
    fn auto_creates_entity_fills_ref_and_creates_anchor_relation() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        let service = EntityService::new(conn);
        let entity_repo = EntityRepository::new(conn);
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "(#!#tep:Student)").unwrap();

        let result = service.auto(&[file.to_string_lossy().to_string()]).unwrap();
        assert_eq!(result.declarations_seen, 1);
        assert_eq!(result.entities_ensured, 1);
        assert_eq!(result.refs_filled, 1);
        assert_eq!(result.anchors_created, 1);
        assert_eq!(result.relations_synced, 1);

        let updated = std::fs::read_to_string(&file).unwrap();
        assert!(updated.contains("(#!#1#tep:Student)"));

        let entity = entity_repo.find(&parse_lookup("Student")).unwrap().unwrap();
        assert_eq!(entity.r#ref.as_deref(), Some(file.to_string_lossy().as_ref()));

        let anchors = service.show("Student").unwrap().anchors;
        assert_eq!(anchors.len(), 1);
    }

    #[test]
    fn auto_does_not_overwrite_existing_ref() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        let service = EntityService::new(conn);
        let entity_repo = EntityRepository::new(conn);
        entity_repo.create(&NewEntity {
            name: "Student".into(),
            r#ref: Some("./docs/student.md".into()),
        }).unwrap();

        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "(#!#tep:Student)").unwrap();

        let result = service.auto(&[file.to_string_lossy().to_string()]).unwrap();
        assert_eq!(result.refs_filled, 0);

        let entity = entity_repo.find(&parse_lookup("Student")).unwrap().unwrap();
        assert_eq!(entity.r#ref.as_deref(), Some("./docs/student.md"));
    }

    #[test]
    fn auto_reuses_anchor_for_materialized_declaration_on_rescan() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        let service = EntityService::new(conn);
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "(#!#tep:Student)").unwrap();

        let first = service.auto(&[file.to_string_lossy().to_string()]).unwrap();
        assert_eq!(first.anchors_created, 1);
        let first_anchors = service.show("Student").unwrap().anchors;
        assert_eq!(first_anchors.len(), 1);
        let first_anchor_id = first_anchors[0].anchor_id;

        let second = service.auto(&[file.to_string_lossy().to_string()]).unwrap();
        assert_eq!(second.anchors_created, 0);
        let second_anchors = service.show("Student").unwrap().anchors;
        assert_eq!(second_anchors.len(), 1);
        assert_eq!(second_anchors[0].anchor_id, first_anchor_id);
    }

    #[test]
    fn auto_reuses_anchor_after_materialization_shift() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        let service = EntityService::new(conn);
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "header\n(#!#tep:Student)\n").unwrap();

        let first = service.auto(&[file.to_string_lossy().to_string()]).unwrap();
        assert_eq!(first.anchors_created, 1);

        let second = service.auto(&[file.to_string_lossy().to_string()]).unwrap();
        assert_eq!(second.anchors_created, 0);
        let anchors = service.show("Student").unwrap().anchors;
        assert_eq!(anchors.len(), 1);
    }

    #[test]
    fn auto_multiple_files_for_same_entity_create_distinct_anchors() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        let service = EntityService::new(conn);
        let temp = tempfile::tempdir().unwrap();
        let one = temp.path().join("one.txt");
        let two = temp.path().join("two.txt");
        std::fs::write(&one, "(#!#tep:Student)").unwrap();
        std::fs::write(&two, "(#!#tep:Student)").unwrap();

        let result = service.auto(&[
            one.to_string_lossy().to_string(),
            two.to_string_lossy().to_string(),
        ]).unwrap();
        assert_eq!(result.anchors_created, 2);

        let anchors = service.show("Student").unwrap().anchors;
        assert_eq!(anchors.len(), 2);
        assert_ne!(anchors[0].file_path, anchors[1].file_path);
    }

    #[test]
    fn auto_two_declarations_in_same_file_for_different_entities_stay_distinct_on_rescan() {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        let service = EntityService::new(conn);
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "(#!#tep:Student)\n(#!#tep:Project)\n").unwrap();

        let first = service.auto(&[file.to_string_lossy().to_string()]).unwrap();
        assert_eq!(first.anchors_created, 2);

        let second = service.auto(&[file.to_string_lossy().to_string()]).unwrap();
        assert_eq!(second.anchors_created, 0);

        let student_anchors = service.show("Student").unwrap().anchors;
        let project_anchors = service.show("Project").unwrap().anchors;
        assert_eq!(student_anchors.len(), 1);
        assert_eq!(project_anchors.len(), 1);
        assert_ne!(student_anchors[0].anchor_id, project_anchors[0].anchor_id);
    }
}

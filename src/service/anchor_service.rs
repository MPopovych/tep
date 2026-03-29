use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use rusqlite::Connection;

use crate::anchor::{AnchorKind, ParsedAnchor, materialize_anchor, parse_anchors};
use crate::entity::parse_lookup;
use crate::filter::tep_ignore_filter::TepIgnoreFilter;
use crate::repository::anchor_entity_repository::AnchorEntityRepository;
use crate::repository::anchor_repository::AnchorRepository;
use crate::repository::entity_repository::EntityRepository;

#[derive(Debug, Clone)]
pub struct AnchorSyncResult {
    pub files_processed: usize,
    pub anchors_created: usize,
    pub anchors_seen: usize,
    pub anchors_dropped: usize,
    pub relations_synced: usize,
}

pub struct AnchorService<'a> {
    anchor_repo: AnchorRepository<'a>,
    anchor_entity_repo: AnchorEntityRepository<'a>,
    entity_repo: EntityRepository<'a>,
}

impl<'a> AnchorService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            anchor_repo: AnchorRepository::new(conn),
            anchor_entity_repo: AnchorEntityRepository::new(conn),
            entity_repo: EntityRepository::new(conn),
        }
    }

    pub fn sync_paths(&self, paths: &[String]) -> Result<AnchorSyncResult> {
        let workspace_root = std::env::current_dir().context("failed to determine current directory")?;
        let filter = TepIgnoreFilter::for_workspace_root(workspace_root);
        let files = filter.collect_paths(paths)?;

        let mut result = AnchorSyncResult {
            files_processed: 0,
            anchors_created: 0,
            anchors_seen: 0,
            anchors_dropped: 0,
            relations_synced: 0,
        };

        for file in files {
            let changed = self.sync_file(&file, &mut result)?;
            if changed {
                result.files_processed += 1;
            }
        }

        Ok(result)
    }

    pub fn attach_entity(&self, anchor_id: i64, entity_target: &str) -> Result<()> {
        let entity = self.resolve_entity_reference(entity_target)?;
        self.anchor_entity_repo.attach(anchor_id, entity.entity_id)
    }

    pub fn detach_entity(&self, anchor_id: i64, entity_target: &str) -> Result<()> {
        let entity = self.resolve_entity_reference(entity_target)?;
        self.anchor_entity_repo.detach(anchor_id, entity.entity_id)
    }

    fn sync_file(&self, path: &Path, result: &mut AnchorSyncResult) -> Result<bool> {
        if !path.is_file() {
            return Ok(false);
        }

        let original = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let parsed = parse_anchors(&original);
        if parsed.is_empty() {
            self.drop_missing_anchors(path, &HashSet::new(), result)?;
            return Ok(false);
        }

        self.ensure_no_duplicate_materialized_ids(&parsed)?;
        result.anchors_seen += parsed.len();

        let mut rewritten = String::with_capacity(original.len() + 64);
        let mut cursor = 0usize;
        let mut seen_materialized_ids = HashSet::new();

        for anchor in parsed {
            rewritten.push_str(&original[cursor..anchor.start_offset]);
            let replacement = self.sync_anchor(path, &anchor, result, &mut seen_materialized_ids)?;
            rewritten.push_str(&replacement);
            cursor = anchor.start_offset + anchor.raw.len();
        }
        rewritten.push_str(&original[cursor..]);

        self.drop_missing_anchors(path, &seen_materialized_ids, result)?;

        if rewritten != original {
            fs::write(path, rewritten)
                .with_context(|| format!("failed to write {}", path.display()))?;
        }

        Ok(true)
    }

    fn sync_anchor(
        &self,
        path: &Path,
        anchor: &ParsedAnchor,
        result: &mut AnchorSyncResult,
        seen_materialized_ids: &mut HashSet<i64>,
    ) -> Result<String> {
        let file_path = path.to_string_lossy();
        match anchor.kind() {
            AnchorKind::Incomplete => {
                let created = self.anchor_repo.create(
                    1,
                    &file_path,
                    Some(anchor.line),
                    Some(anchor.shift),
                    Some(anchor.start_offset as i64),
                )?;
                seen_materialized_ids.insert(created.anchor_id);
                self.sync_entity_relations(created.anchor_id, &anchor.entity_refs, result)?;
                result.anchors_created += 1;
                Ok(materialize_anchor(anchor, created.anchor_id, 1))
            }
            AnchorKind::Materialized => {
                let anchor_id = anchor.anchor_id.expect("materialized anchor should have id");
                seen_materialized_ids.insert(anchor_id);
                self.anchor_repo.update_location(
                    anchor_id,
                    &file_path,
                    Some(anchor.line),
                    Some(anchor.shift),
                    Some(anchor.start_offset as i64),
                )?;
                self.sync_entity_relations(anchor_id, &anchor.entity_refs, result)?;
                Ok(anchor.raw.clone())
            }
        }
    }

    fn sync_entity_relations(
        &self,
        anchor_id: i64,
        refs: &[String],
        result: &mut AnchorSyncResult,
    ) -> Result<()> {
        if refs.is_empty() {
            return Ok(());
        }

        let mut entity_ids = Vec::new();
        for entity_ref in refs {
            let entity = self.resolve_entity_reference(entity_ref)?;
            entity_ids.push(entity.entity_id);
        }

        self.anchor_entity_repo.replace_for_anchor(anchor_id, &entity_ids)?;
        result.relations_synced += entity_ids.len();
        Ok(())
    }

    fn resolve_entity_reference(&self, entity_ref: &str) -> Result<crate::entity::Entity> {
        let lookup = parse_lookup(entity_ref);
        match lookup {
            crate::entity::EntityLookup::Id(_) => self
                .entity_repo
                .find(&lookup)?
                .with_context(|| format!("entity not found: {entity_ref}")),
            crate::entity::EntityLookup::Name(name) => self.entity_repo.ensure(&crate::entity::NewEntity {
                name,
                r#ref: None,
            }),
        }
    }

    fn ensure_no_duplicate_materialized_ids(&self, parsed: &[ParsedAnchor]) -> Result<()> {
        let mut seen = HashSet::new();
        for anchor in parsed {
            if let Some(anchor_id) = anchor.anchor_id {
                if !seen.insert(anchor_id) {
                    bail!("duplicate materialized anchor {} found in the same file", anchor_id);
                }
            }
        }
        Ok(())
    }

    fn drop_missing_anchors(
        &self,
        path: &Path,
        seen_materialized_ids: &HashSet<i64>,
        result: &mut AnchorSyncResult,
    ) -> Result<()> {
        let file_path = path.to_string_lossy();
        let existing_ids = self.anchor_repo.list_ids_for_file(&file_path)?;
        for anchor_id in existing_ids {
            if !seen_materialized_ids.contains(&anchor_id) {
                self.anchor_repo.delete(anchor_id)?;
                result.anchors_dropped += 1;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use std::env;

    fn setup_service() -> AnchorService<'static> {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql())
            .expect("schema should apply");
        AnchorService::new(conn)
    }

    #[test]
    fn sync_file_materializes_incomplete_anchor() {
        let service = setup_service();
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let file = temp.path().join("note.txt");
        fs::write(&file, "hello [#!#tep:](student)").expect("should write file");

        let result = service
            .sync_paths(&[file.to_string_lossy().to_string()])
            .expect("sync should succeed");

        let updated = fs::read_to_string(&file).expect("should read file");
        assert!(updated.contains("[#!#1#tep:"));
        assert_eq!(result.anchors_created, 1);
        assert_eq!(result.anchors_seen, 1);
        assert_eq!(result.anchors_dropped, 0);
        assert_eq!(result.relations_synced, 1);
    }

    #[test]
    fn sync_directory_processes_dot_style_path() {
        let service = setup_service();
        let previous = env::current_dir().expect("current dir should exist");
        let temp = tempfile::tempdir().expect("temp dir should be created");
        env::set_current_dir(temp.path()).expect("should set current dir");
        fs::write("a.txt", "[#!#tep:]").expect("should write file");

        let result = service.sync_paths(&[".".into()]).expect("sync should succeed");
        assert_eq!(result.anchors_created, 1);

        env::set_current_dir(previous).expect("should restore current dir");
    }

    #[test]
    fn sync_multiple_entity_references_creates_relations() {
        let service = setup_service();
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let file = temp.path().join("note.txt");
        fs::write(&file, "[#!#tep:](student,basic-user)").expect("should write file");

        let result = service
            .sync_paths(&[file.to_string_lossy().to_string()])
            .expect("sync should succeed");

        assert_eq!(result.relations_synced, 2);
    }

    #[test]
    fn dropped_anchor_is_deleted_when_removed_from_file() {
        let service = setup_service();
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let file = temp.path().join("note.txt");
        fs::write(&file, "[#!#tep:]").expect("should write file");
        let first = service
            .sync_paths(&[file.to_string_lossy().to_string()])
            .expect("first sync should succeed");
        assert_eq!(first.anchors_created, 1);

        let updated = fs::read_to_string(&file).expect("should read file");
        assert!(updated.contains("[#!#1#tep:"));

        fs::write(&file, "no anchors now\n").expect("should rewrite file");
        let second = service
            .sync_paths(&[file.to_string_lossy().to_string()])
            .expect("second sync should succeed");
        assert_eq!(second.anchors_dropped, 1);
    }

    #[test]
    fn duplicate_materialized_anchor_ids_in_same_file_fail() {
        let service = setup_service();
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let file = temp.path().join("bad.txt");
        fs::write(&file, "[#!#1#tep:5]\n[#!#1#tep:5]\n").expect("should write file");

        let result = service.sync_paths(&[file.to_string_lossy().to_string()]);
        assert!(result.is_err());
    }
}

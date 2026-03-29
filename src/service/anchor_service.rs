// [#!#1#tep:47](service.anchor.sync)
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rusqlite::Connection;

use crate::anchor::{Anchor, AnchorKind, ParsedAnchor, materialize_anchor, parse_anchors};
use crate::entity::{Entity, parse_lookup};
use crate::filter::tep_ignore_filter::TepIgnoreFilter;
use crate::repository::anchor_entity_repository::AnchorEntityRepository;
use crate::repository::anchor_repository::AnchorRepository;
use crate::repository::entity_repository::EntityRepository;
use crate::utils::path::{display_path, resolve_from_workspace};

#[derive(Debug, Clone)]
pub struct AnchorSyncResult {
    pub files_processed: usize,
    pub anchors_created: usize,
    pub anchors_seen: usize,
    pub anchors_dropped: usize,
    pub relations_synced: usize,
}

pub struct AnchorShowResult {
    pub anchor: Anchor,
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone)]
struct ParsedFile {
    absolute_path: PathBuf,
}

pub struct AnchorService<'a> {
    workspace_root: PathBuf,
    anchor_repo: AnchorRepository<'a>,
    anchor_entity_repo: AnchorEntityRepository<'a>,
    entity_repo: EntityRepository<'a>,
}

impl<'a> AnchorService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::with_workspace_root(conn, workspace_root)
    }

    pub fn with_workspace_root(conn: &'a Connection, workspace_root: impl Into<PathBuf>) -> Self {
        let workspace_root = workspace_root.into();
        Self {
            workspace_root: workspace_root.clone(),
            anchor_repo: AnchorRepository::with_workspace_root(conn, workspace_root),
            anchor_entity_repo: AnchorEntityRepository::new(conn),
            entity_repo: EntityRepository::new(conn),
        }
    }

    pub fn sync_paths(&self, paths: &[String]) -> Result<AnchorSyncResult> {
        let files = self.collect_workspace_files(paths)?;

        let mut result = AnchorSyncResult {
            files_processed: 0,
            anchors_created: 0,
            anchors_seen: 0,
            anchors_dropped: 0,
            relations_synced: 0,
        };

        for file in files {
            let changed = self.sync_file(&file.absolute_path, &mut result)?;
            if changed {
                result.files_processed += 1;
            }
        }

        Ok(result)
    }

    pub fn show(&self, anchor_id: i64) -> Result<AnchorShowResult> {
        let anchor = self
            .anchor_repo
            .find_by_id(anchor_id)?
            .with_context(|| format!("anchor not found: {anchor_id}"))?;
        let entities = self.anchor_entity_repo.list_entities_for_anchor(anchor_id)?;
        Ok(AnchorShowResult { anchor, entities })
    }

    pub fn attach_entity(&self, anchor_id: i64, entity_target: &str) -> Result<()> {
        let entity = self.resolve_entity_reference(entity_target)?;
        self.anchor_entity_repo.attach(anchor_id, entity.entity_id)
    }

    pub fn detach_entity(&self, anchor_id: i64, entity_target: &str) -> Result<()> {
        let entity = self.resolve_entity_reference(entity_target)?;
        self.anchor_entity_repo.detach(anchor_id, entity.entity_id)
    }

    fn collect_workspace_files(&self, paths: &[String]) -> Result<Vec<ParsedFile>> {
        let filter = TepIgnoreFilter::for_workspace_root(&self.workspace_root);
        let files = filter.collect_paths(paths)?;
        Ok(files
            .into_iter()
            .map(|path| ParsedFile {
                absolute_path: resolve_from_workspace(&path, &self.workspace_root),
            })
            .collect())
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
        let file_path = display_path(path);
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
                description: None,
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
        let file_path = display_path(path);
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

// #tepignoreafter
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_service(workspace_root: &Path) -> AnchorService<'static> {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        db::ensure_schema(conn).expect("schema should apply");
        AnchorService::with_workspace_root(conn, workspace_root)
    }

    #[test]
    fn collect_workspace_files_resolves_absolute_paths() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        std::fs::write(temp.path().join("note.txt"), "[#!#tep:]").unwrap();
        let service = setup_service(temp.path());

        let files = service.collect_workspace_files(&["./note.txt".into()]).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].absolute_path, temp.path().join("./note.txt"));
    }

    #[test]
    fn sync_file_materializes_incomplete_anchor() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "hello [#!#tep:](student)").expect("should write file");

        let result = service
            .sync_paths(&["./note.txt".into()])
            .expect("sync should succeed");

        let updated = std::fs::read_to_string(&file).expect("should read file");
        assert!(updated.contains("[#!#1#tep:"));
        assert_eq!(result.anchors_created, 1);
        assert_eq!(result.anchors_seen, 1);
        assert_eq!(result.anchors_dropped, 0);
        assert_eq!(result.relations_synced, 1);
    }

    #[test]
    fn show_returns_related_entities() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let anchor = service.anchor_repo.create(1, "./file.txt", Some(1), Some(0), Some(0)).unwrap();
        let entity = service.entity_repo.ensure(&crate::entity::NewEntity { name: "student".into(), r#ref: None, description: None }).unwrap();
        service.anchor_entity_repo.attach(anchor.anchor_id, entity.entity_id).unwrap();

        let result = service.show(anchor.anchor_id).unwrap();
        assert_eq!(result.anchor.anchor_id, anchor.anchor_id);
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].name, "student");
    }

    #[test]
    fn sync_directory_processes_dot_style_path() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        std::fs::write(temp.path().join("a.txt"), "[#!#tep:]").expect("should write file");

        let result = service.sync_paths(&[".".into()]).expect("sync should succeed");
        assert_eq!(result.anchors_created, 1);
    }

    #[test]
    fn sync_multiple_entity_references_creates_relations() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "[#!#tep:](student,basic-user)").expect("should write file");

        let result = service
            .sync_paths(&["./note.txt".into()])
            .expect("sync should succeed");

        assert_eq!(result.relations_synced, 2);
    }

    #[test]
    fn dropped_anchor_is_deleted_when_removed_from_file() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "[#!#tep:]").expect("should write file");
        let first = service
            .sync_paths(&["./note.txt".into()])
            .expect("first sync should succeed");
        assert_eq!(first.anchors_created, 1);

        let updated = std::fs::read_to_string(&file).expect("should read file");
        assert!(updated.contains("[#!#1#tep:"));

        std::fs::write(&file, "no anchors now\n").expect("should rewrite file");
        let second = service
            .sync_paths(&["./note.txt".into()])
            .expect("second sync should succeed");
        assert_eq!(second.anchors_dropped, 1);
    }

    #[test]
    fn duplicate_materialized_anchor_ids_in_same_file_fail() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let file = temp.path().join("bad.txt");
        std::fs::write(&file, "[#!#1#tep:5]\n[#!#1#tep:5]\n").expect("should write file");

        let result = service.sync_paths(&["./bad.txt".into()]);
        assert!(result.is_err());
    }

}

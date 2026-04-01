// (#!#tep:anchor.sync)
// [#!#tep:anchor.sync](anchor.sync)
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rusqlite::Connection;

use crate::anchor::{Anchor, AnchorTarget, ParsedAnchor, parse_anchor_target, parse_anchors};
use crate::entity::{EntityLookup, NewEntity, parse_lookup};
use crate::repository::anchor_entity_repository::AnchorEntityRepository;
use crate::repository::anchor_repository::AnchorRepository;
use crate::repository::entity_repository::EntityRepository;
use crate::utils::path::display_path;
use crate::utils::workspace_scanner::collect_workspace_files;

#[derive(Debug, Clone, Default)]
pub struct AnchorSyncResult {
    pub files_processed: usize,
    pub anchors_created: usize,
    pub anchors_seen: usize,
    pub anchors_dropped: usize,
    pub relations_synced: usize,
}

pub struct AnchorShowResult {
    pub anchor: Anchor,
    pub entities: Vec<crate::entity::Entity>,
}

pub struct AnchorService<'a> {
    workspace_root: PathBuf,
    pub(crate) anchor_repo: AnchorRepository<'a>,
    pub(crate) anchor_entity_repo: AnchorEntityRepository<'a>,
    pub(crate) entity_repo: EntityRepository<'a>,
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

    // [#!#tep:anchor.sync.paths](anchor.sync,workspace.scanner)
    pub fn sync_paths(&self, paths: &[String]) -> Result<AnchorSyncResult> {
        let files = collect_workspace_files(&self.workspace_root, paths)?;
        let mut result = AnchorSyncResult::default();
        for file in files {
            self.sync_file(&file.absolute_path, &mut result)?;
        }
        Ok(result)
    }

    pub fn show(&self, target: &str) -> Result<AnchorShowResult> {
        let anchor = self.resolve_anchor_reference(target)?;
        let entities = self.anchor_entity_repo.list_entities_for_anchor(anchor.anchor_id)?;
        Ok(AnchorShowResult { anchor, entities })
    }

    pub fn list_all(&self) -> Result<Vec<Anchor>> {
        self.anchor_repo.list_all()
    }

    fn resolve_anchor_reference(&self, target: &str) -> Result<Anchor> {
        match parse_anchor_target(target) {
            AnchorTarget::Id(id) => self
                .anchor_repo
                .find_by_id(id)?
                .with_context(|| format!("anchor not found: {target}")),
            AnchorTarget::Name(name) => self
                .anchor_repo
                .find_by_name(&name)?
                .with_context(|| format!("anchor not found: {target}")),
        }
    }

    fn sync_file(&self, path: &Path, result: &mut AnchorSyncResult) -> Result<()> {
        if !path.is_file() {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let parsed = parse_anchors(&content);

        self.ensure_no_duplicate_names(&parsed)?;
        result.anchors_seen += parsed.len();

        let mut seen_ids = HashSet::new();
        for anchor in &parsed {
            self.sync_anchor(path, anchor, result, &mut seen_ids)?;
        }

        self.drop_missing_anchors(path, &seen_ids, result)?;

        if !parsed.is_empty() {
            result.files_processed += 1;
        }

        Ok(())
    }

    fn sync_anchor(
        &self,
        path: &Path,
        anchor: &ParsedAnchor,
        result: &mut AnchorSyncResult,
        seen_ids: &mut HashSet<i64>,
    ) -> Result<()> {
        let file_path = display_path(path);
        let name = &anchor.anchor_name;

        match self.anchor_repo.find_by_name(name)? {
            Some(existing) => {
                seen_ids.insert(existing.anchor_id);
                self.anchor_repo.update_location(
                    existing.anchor_id,
                    &file_path,
                    Some(anchor.line),
                    Some(anchor.shift),
                    Some(anchor.start_offset as i64),
                )?;
                self.sync_entity_relations(existing.anchor_id, &anchor.entity_refs, result)?;
            }
            None => {
                let created = self.anchor_repo.create_named(
                    name,
                    1,
                    &file_path,
                    Some(anchor.line),
                    Some(anchor.shift),
                    Some(anchor.start_offset as i64),
                )?;
                seen_ids.insert(created.anchor_id);
                self.sync_entity_relations(created.anchor_id, &anchor.entity_refs, result)?;
                result.anchors_created += 1;
            }
        }

        Ok(())
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
        match parse_lookup(entity_ref) {
            EntityLookup::Id(_) => {
                let lookup = parse_lookup(entity_ref);
                self.entity_repo
                    .find(&lookup)?
                    .with_context(|| format!("entity not found: {entity_ref}"))
            }
            EntityLookup::Name(name) => self.entity_repo.ensure(&NewEntity {
                name,
                r#ref: None,
                description: None,
            }),
        }
    }

    fn ensure_no_duplicate_names(&self, parsed: &[ParsedAnchor]) -> Result<()> {
        let mut seen_names: HashSet<String> = HashSet::new();
        for anchor in parsed {
            if !seen_names.insert(anchor.anchor_name.clone()) {
                bail!("duplicate anchor name '{}' found in the same file", anchor.anchor_name);
            }
        }
        Ok(())
    }

    fn drop_missing_anchors(
        &self,
        path: &Path,
        seen_ids: &HashSet<i64>,
        result: &mut AnchorSyncResult,
    ) -> Result<()> {
        let file_path = display_path(path);
        let existing_ids = self.anchor_repo.list_ids_for_file(&file_path)?;
        for anchor_id in existing_ids {
            if !seen_ids.contains(&anchor_id) {
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
        std::fs::write(temp.path().join("note.txt"), "[#!#tep:foo](student)").unwrap();

        let files = collect_workspace_files(&temp.path().to_path_buf(), &["./note.txt".into()]).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].absolute_path, temp.path().join("./note.txt"));
    }

    #[test]
    fn sync_file_tracks_named_anchor_with_entity_refs() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "hello [#!#tep:myanchor](student)").expect("should write file");

        let result = service
            .sync_paths(&["./note.txt".into()])
            .expect("sync should succeed");

        let updated = std::fs::read_to_string(&file).expect("should read file");
        assert_eq!(updated, "hello [#!#tep:myanchor](student)", "file should not be rewritten");
        assert_eq!(result.anchors_created, 1);
        assert_eq!(result.anchors_seen, 1);
        assert_eq!(result.anchors_dropped, 0);
        assert_eq!(result.relations_synced, 1);
    }

    #[test]
    fn sync_file_ignores_anchor_without_entity_refs() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "[#!#tep:foo]").expect("should write file");

        let result = service.sync_paths(&["./note.txt".into()]).expect("sync should succeed");

        assert_eq!(result.anchors_seen, 0);
        assert_eq!(result.anchors_created, 0);
        assert_eq!(result.relations_synced, 0);
    }

    #[test]
    fn show_returns_related_entities() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let anchor = service.anchor_repo.create(1, "./file.txt", Some(1), Some(0), Some(0)).unwrap();
        let entity = service.entity_repo.ensure(&crate::entity::NewEntity { name: "student".into(), r#ref: None, description: None }).unwrap();
        service.anchor_entity_repo.attach(anchor.anchor_id, entity.entity_id).unwrap();

        let result = service.show(&anchor.anchor_id.to_string()).unwrap();
        assert_eq!(result.anchor.anchor_id, anchor.anchor_id);
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].name, "student");
    }

    #[test]
    fn sync_directory_processes_dot_style_path() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        std::fs::write(temp.path().join("a.txt"), "[#!#tep:foo](student)").expect("should write file");

        let result = service.sync_paths(&[".".into()]).expect("sync should succeed");
        assert_eq!(result.anchors_created, 1);
    }

    #[test]
    fn sync_multiple_entity_references_creates_relations() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "[#!#tep:foo](student,basic_user)").expect("should write file");

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
        std::fs::write(&file, "[#!#tep:foo](student)").expect("should write file");
        let first = service
            .sync_paths(&["./note.txt".into()])
            .expect("first sync should succeed");
        assert_eq!(first.anchors_created, 1);

        std::fs::write(&file, "no anchors now\n").expect("should rewrite file");
        let second = service
            .sync_paths(&["./note.txt".into()])
            .expect("second sync should succeed");
        assert_eq!(second.anchors_dropped, 1);
    }

    #[test]
    fn duplicate_anchor_names_in_same_file_fail() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let file = temp.path().join("bad.txt");
        std::fs::write(&file, "[#!#tep:foo](student)\n[#!#tep:foo](teacher)\n").expect("should write file");

        let result = service.sync_paths(&["./bad.txt".into()]);
        assert!(result.is_err());
    }
}

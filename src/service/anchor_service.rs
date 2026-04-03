// #!#tep:(anchor.sync){description="Service for synchronizing anchors and anchor-entity attachments from files"}
// #!#tep:(anchor.sync)->(repo.anchor){description="uses for anchor persistence"}
// #!#tep:(anchor.sync)->(repo.entity){description="uses to ensure referenced entities exist"}
// #!#tep:[anchor.sync](anchor.sync,repo.anchor,repo.entity){description="Anchor sync service module entry"}
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::anchor::{Anchor, AnchorTarget, parse_anchor_target};
use crate::entity::{EntityLookup, NewEntity, parse_lookup};
use crate::repository::anchor_entity_repository::AnchorEntityRepository;
use crate::repository::anchor_repository::{AnchorRepository, NewAnchor};
use crate::repository::entity_repository::EntityRepository;
use crate::service::workspace_parse_service::{
    ParsedWorkspaceFile, collect_parsed_workspace_files,
};
use crate::tep_tag::ParsedAnchorTag;
use crate::utils::path::display_path;

#[derive(Debug, Clone, Default)]
pub struct AnchorSyncResult {
    pub files_processed: usize,
    pub anchors_created: usize,
    pub anchors_seen: usize,
    pub anchors_dropped: usize,
    pub relations_synced: usize,
    pub metadata_updated: usize,
    pub warnings: Vec<String>,
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

    pub fn sync_paths(&self, paths: &[String]) -> Result<AnchorSyncResult> {
        let (files, warnings) = collect_parsed_workspace_files(&self.workspace_root, paths)?;
        let mut result = AnchorSyncResult::default();
        result.warnings.extend(warnings);
        for file in files {
            self.sync_file(&file, &mut result)?;
        }
        Ok(result)
    }

    pub fn show(&self, target: &str) -> Result<AnchorShowResult> {
        let anchor = self.resolve_anchor_reference(target)?;
        let entities = self
            .anchor_entity_repo
            .list_entities_for_anchor(anchor.anchor_id)?;
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

    fn sync_file(&self, file: &ParsedWorkspaceFile, result: &mut AnchorSyncResult) -> Result<()> {
        if let Some(duplicate_name) = self.find_duplicate_name(&file.anchor_tags) {
            result.warnings.push(format!(
                "duplicate anchor name '{}' found in the same file {}",
                duplicate_name,
                file.file.absolute_path.display()
            ));
            return Ok(());
        }
        result.anchors_seen += file.anchor_tags.len();

        let mut seen_ids = HashSet::new();
        for anchor in &file.anchor_tags {
            self.sync_anchor(&file.file.absolute_path, anchor, result, &mut seen_ids)?;
        }

        self.drop_missing_anchors(&file.file.absolute_path, &seen_ids, result)?;
        if !file.anchor_tags.is_empty() {
            result.files_processed += 1;
        }

        Ok(())
    }

    fn sync_anchor(
        &self,
        path: &Path,
        anchor: &ParsedAnchorTag,
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
                self.sync_anchor_metadata(existing.anchor_id, anchor, result)?;
            }
            None => {
                let created = self.anchor_repo.create(&NewAnchor {
                    name: Some(name),
                    version: 1,
                    file_path: &file_path,
                    line: Some(anchor.line),
                    shift: Some(anchor.shift),
                    offset: Some(anchor.start_offset as i64),
                    description: anchor
                        .metadata
                        .fields
                        .get("description")
                        .map(String::as_str),
                })?;
                seen_ids.insert(created.anchor_id);
                self.sync_entity_relations(created.anchor_id, &anchor.entity_refs, result)?;
                self.sync_anchor_metadata(created.anchor_id, anchor, result)?;
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

        self.anchor_entity_repo
            .replace_for_anchor(anchor_id, &entity_ids)?;
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

    fn find_duplicate_name(&self, parsed: &[ParsedAnchorTag]) -> Option<String> {
        let mut seen_names: HashSet<String> = HashSet::new();
        for anchor in parsed {
            if !seen_names.insert(anchor.anchor_name.clone()) {
                return Some(anchor.anchor_name.clone());
            }
        }
        None
    }

    fn sync_anchor_metadata(
        &self,
        anchor_id: i64,
        anchor: &ParsedAnchorTag,
        result: &mut AnchorSyncResult,
    ) -> Result<()> {
        let description = anchor
            .metadata
            .fields
            .get("description")
            .map(String::as_str);
        if description.is_some() {
            self.anchor_repo
                .update_description(anchor_id, description)?;
            result.metadata_updated += 1;
        }
        for key in &anchor.metadata.duplicate_keys {
            result.warnings.push(format!(
                "duplicate metadata key '{}' in anchor {}",
                key, anchor.anchor_name
            ));
        }
        for key in &anchor.metadata.unknown_fields {
            result.warnings.push(format!(
                "unknown metadata field '{}' in anchor {}",
                key, anchor.anchor_name
            ));
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
    fn sync_file_tracks_named_anchor_with_entity_refs() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "hello #!#tep:[myanchor](student)").expect("should write file");

        let result = service.sync_paths(&["./note.txt".into()]).unwrap();
        assert_eq!(result.anchors_created, 1);
        assert_eq!(result.relations_synced, 1);
    }

    #[test]
    fn sync_persists_anchor_description_metadata() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let file = temp.path().join("note.txt");
        std::fs::write(
            &file,
            "#!#tep:[myanchor](student){description=\"Entry point\"}",
        )
        .unwrap();

        let result = service.sync_paths(&["./note.txt".into()]).unwrap();
        let anchor = service
            .anchor_repo
            .find_by_name("myanchor")
            .unwrap()
            .unwrap();
        assert_eq!(result.metadata_updated, 1);
        assert_eq!(anchor.description.as_deref(), Some("Entry point"));
    }

    #[test]
    fn sync_warns_on_unknown_anchor_metadata() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let file = temp.path().join("note.txt");
        std::fs::write(&file, "#!#tep:[myanchor](student){foo=\"bar\"}").unwrap();

        let result = service.sync_paths(&["./note.txt".into()]).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("unknown metadata field 'foo'"))
        );
    }

    #[test]
    fn duplicate_anchor_names_in_same_file_warn_and_continue() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let service = setup_service(temp.path());
        let file = temp.path().join("bad.txt");
        std::fs::write(&file, "#!#tep:[foo](student)\n#!#tep:[foo](teacher)\n").unwrap();

        let result = service.sync_paths(&["./bad.txt".into()]).unwrap();
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("duplicate anchor name 'foo'"))
        );
    }
}

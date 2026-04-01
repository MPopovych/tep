// (#!#tep:anchor.health)
// [#!#tep:anchor.health](anchor.health)
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::anchor::{ParsedAnchor, parse_anchors};
use crate::entity::parse_entity_declarations;
use crate::repository::anchor_repository::AnchorRepository;
use crate::repository::entity_repository::EntityRepository;
use crate::utils::workspace_scanner::{WorkspaceFile, collect_workspace_files};

#[derive(Debug, Clone, Default)]
pub struct HealthIssueCounts {
    pub anchors_moved: usize,
    pub anchors_missing: usize,
    pub duplicate_anchor_ids: usize,
    pub unknown_anchor_ids: usize,
    pub entities_without_anchors: usize,
    pub anchors_without_entities: usize,
}

#[derive(Debug, Clone, Default)]
pub struct HealthIssueGroups {
    pub moved_anchors: Vec<String>,
    pub missing_anchors: Vec<String>,
    pub duplicate_anchor_ids: Vec<String>,
    pub unknown_anchor_ids: Vec<String>,
    pub entities_without_anchors: Vec<String>,
    pub anchors_without_entities: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct HealthReport {
    pub files_scanned: usize,
    pub anchors_seen: usize,
    pub anchors_healthy: usize,
    pub issue_counts: HealthIssueCounts,
    pub groups: HealthIssueGroups,
}

#[derive(Debug, Default)]
struct HealthTracker {
    /// anchor name → file path where it was first seen
    seen_name_to_file: HashMap<String, String>,
    seen_anchor_ids: HashSet<i64>,
}

pub struct HealthService<'a> {
    workspace_root: PathBuf,
    anchor_repo: AnchorRepository<'a>,
    entity_repo: EntityRepository<'a>,
}

impl<'a> HealthService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::with_workspace_root(conn, workspace_root)
    }

    pub fn with_workspace_root(conn: &'a Connection, workspace_root: impl Into<PathBuf>) -> Self {
        let workspace_root = workspace_root.into();
        Self {
            workspace_root: workspace_root.clone(),
            anchor_repo: AnchorRepository::with_workspace_root(conn, workspace_root),
            entity_repo: EntityRepository::new(conn),
        }
    }

    // [#!#tep:anchor.health.audit](anchor.health,workspace.scanner)
    pub fn audit_paths(&self, paths: &[String]) -> Result<HealthReport> {
        let files = collect_workspace_files(&self.workspace_root, paths)?;
        let scoped_files = files
            .iter()
            .map(|f| self.anchor_repo.normalized_path_for(&f.display_path))
            .collect::<HashSet<_>>();

        let mut report = HealthReport::default();
        let mut tracker = HealthTracker::default();

        for file in files {
            self.inspect_file(&file, &mut report, &mut tracker)?;
        }

        self.report_missing_db_anchors(&scoped_files, &tracker.seen_anchor_ids, &mut report)?;
        self.report_entities_without_anchors(&mut report)?;
        self.report_anchors_without_entities(&mut report)?;
        Ok(report)
    }

    fn inspect_file(
        &self,
        file: &WorkspaceFile,
        report: &mut HealthReport,
        tracker: &mut HealthTracker,
    ) -> Result<()> {
        let content = fs::read_to_string(&file.absolute_path)
            .with_context(|| format!("failed to read {}", file.absolute_path.display()))?;
        let parsed_anchors = parse_anchors(&content);
        let parsed_declarations = parse_entity_declarations(&content);

        if parsed_anchors.is_empty() && parsed_declarations.is_empty() {
            return Ok(());
        }

        report.files_scanned += 1;
        report.anchors_seen += parsed_anchors.len() + parsed_declarations.len();

        let mut local_names = HashSet::new();
        for anchor in &parsed_anchors {
            self.inspect_named_anchor(
                anchor,
                &file.display_path,
                report,
                tracker,
                &mut local_names,
            )?;
        }

        for declaration in &parsed_declarations {
            if let Some(existing) = self
                .entity_repo
                .find(&crate::entity::EntityLookup::Name(declaration.name.clone()))?
                .and_then(|entity| {
                    self.anchor_repo
                        .find_latest_for_entity_in_file(entity.entity_id, &file.display_path)
                        .ok()
                        .flatten()
                })
            {
                tracker.seen_anchor_ids.insert(existing.anchor_id);
                report.anchors_healthy += 1;
            }
        }

        Ok(())
    }

    fn inspect_named_anchor(
        &self,
        anchor: &ParsedAnchor,
        file_path: &str,
        report: &mut HealthReport,
        tracker: &mut HealthTracker,
        local_names: &mut HashSet<String>,
    ) -> Result<()> {
        let name = &anchor.anchor_name;

        if !local_names.insert(name.clone()) {
            report.issue_counts.duplicate_anchor_ids += 1;
            report.groups.duplicate_anchor_ids.push(format!(
                "anchor '{}' appears multiple times in {}",
                name, file_path
            ));
            return Ok(());
        }

        if let Some(existing_file) = tracker.seen_name_to_file.get(name) {
            if existing_file != file_path {
                report.issue_counts.duplicate_anchor_ids += 1;
                report.groups.duplicate_anchor_ids.push(format!(
                    "anchor '{}' appears in multiple files: {} and {}",
                    name, existing_file, file_path
                ));
                return Ok(());
            }
        } else {
            tracker
                .seen_name_to_file
                .insert(name.clone(), file_path.to_string());
        }

        match self.anchor_repo.find_by_name(name)? {
            Some(stored) => {
                tracker.seen_anchor_ids.insert(stored.anchor_id);
                let normalized_file_path = self.anchor_repo.normalized_path_for(file_path);
                if normalized_file_path != stored.file_path
                    || stored.line != Some(anchor.line)
                    || stored.shift != Some(anchor.shift)
                    || stored.offset != Some(anchor.start_offset as i64)
                {
                    report.issue_counts.anchors_moved += 1;
                    report.groups.moved_anchors.push(format!(
                        "anchor '{}' metadata drifted in {} (db: {} {:?}:{:?} [{:?}], file: {} {}:{} [{}])",
                        name,
                        file_path,
                        stored.file_path,
                        stored.line,
                        stored.shift,
                        stored.offset,
                        file_path,
                        anchor.line,
                        anchor.shift,
                        anchor.start_offset
                    ));
                } else {
                    report.anchors_healthy += 1;
                }
            }
            None => {
                report.issue_counts.unknown_anchor_ids += 1;
                report.groups.unknown_anchor_ids.push(format!(
                    "anchor '{}' found in file but does not exist in the database ({})",
                    name, file_path
                ));
            }
        }

        Ok(())
    }

    fn report_missing_db_anchors(
        &self,
        scoped_files: &HashSet<String>,
        seen_anchor_ids: &HashSet<i64>,
        report: &mut HealthReport,
    ) -> Result<()> {
        for anchor in self.anchor_repo.list_all()? {
            if scoped_files.contains(&anchor.file_path)
                && !seen_anchor_ids.contains(&anchor.anchor_id)
            {
                report.issue_counts.anchors_missing += 1;
                report.groups.missing_anchors.push(format!(
                    "missing anchor {} recorded in db but not found in file {}",
                    anchor.anchor_id, anchor.file_path
                ));
            }
        }
        Ok(())
    }

    fn report_entities_without_anchors(&self, report: &mut HealthReport) -> Result<()> {
        let entities = self.entity_repo.list_without_anchors()?;
        report.issue_counts.entities_without_anchors = entities.len();
        report.groups.entities_without_anchors = entities
            .into_iter()
            .map(|entity| format!("{} ({})", entity.entity_id, entity.name))
            .collect();
        Ok(())
    }

    fn report_anchors_without_entities(&self, report: &mut HealthReport) -> Result<()> {
        let anchors = self.anchor_repo.list_without_entities()?;
        report.issue_counts.anchors_without_entities = anchors.len();
        report.groups.anchors_without_entities = anchors
            .into_iter()
            .map(|anchor| format!("{} {}", anchor.anchor_id, anchor.file_path))
            .collect();
        Ok(())
    }
}

// #tepignoreafter
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::entity::NewEntity;
    use crate::repository::anchor_entity_repository::AnchorEntityRepository;

    fn setup_service() -> HealthService<'static> {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql())
            .expect("schema should apply");
        HealthService::with_workspace_root(conn, "/tmp/project")
    }

    #[test]
    fn reports_entities_without_anchors() {
        let service = setup_service();
        service
            .entity_repo
            .create(&NewEntity {
                name: "student".into(),
                r#ref: None,
                description: None,
            })
            .unwrap();

        let report = service.audit_paths(&["src".into()]).unwrap();
        assert_eq!(report.issue_counts.entities_without_anchors, 1);
    }

    #[test]
    fn reports_anchors_without_entities() {
        let service = setup_service();
        service
            .anchor_repo
            .create_named(
                "my_anchor",
                1,
                "./docs/student.md",
                Some(1),
                Some(0),
                Some(0),
            )
            .unwrap();
        std::fs::create_dir_all("/tmp/project/docs").ok();
        std::fs::write(
            "/tmp/project/docs/student.md",
            "[#!#tep:my_anchor](student)",
        )
        .ok(); // #tepignore

        let report = service.audit_paths(&["./docs/student.md".into()]).unwrap();
        assert_eq!(report.issue_counts.anchors_without_entities, 1);
    }

    #[test]
    fn reports_healthy_anchor_when_attached() {
        let service = setup_service();
        let entity = service
            .entity_repo
            .create(&NewEntity {
                name: "student".into(),
                r#ref: None,
                description: None,
            })
            .unwrap();
        let anchor = service
            .anchor_repo
            .create_named(
                "my_anchor",
                1,
                "./docs/student.md",
                Some(1),
                Some(0),
                Some(0),
            )
            .unwrap();
        let rel = AnchorEntityRepository::new(service.anchor_repo.conn);
        rel.attach(anchor.anchor_id, entity.entity_id).unwrap();

        std::fs::create_dir_all("/tmp/project/docs").ok();
        std::fs::write(
            "/tmp/project/docs/student.md",
            "[#!#tep:my_anchor](student)",
        )
        .ok(); // #tepignore

        let report = service.audit_paths(&["./docs/student.md".into()]).unwrap();
        assert_eq!(report.anchors_healthy, 1);
    }
}

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::anchor::{AnchorKind, ParsedAnchor, parse_anchors};
use crate::filter::tep_ignore_filter::TepIgnoreFilter;
use crate::repository::anchor_repository::AnchorRepository;
use crate::repository::entity_repository::EntityRepository;
use crate::utils::path::{display_path, resolve_from_workspace};

#[derive(Debug, Clone)]
pub struct HealthIssueCounts {
    pub anchors_moved: usize,
    pub anchors_missing: usize,
    pub duplicate_anchor_ids: usize,
    pub unknown_anchor_ids: usize,
    pub entities_without_anchors: usize,
    pub anchors_without_entities: usize,
}

#[derive(Debug, Clone)]
pub struct HealthIssueGroups {
    pub moved_anchors: Vec<String>,
    pub missing_anchors: Vec<String>,
    pub duplicate_anchor_ids: Vec<String>,
    pub unknown_anchor_ids: Vec<String>,
    pub entities_without_anchors: Vec<String>,
    pub anchors_without_entities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HealthReport {
    pub files_scanned: usize,
    pub anchors_seen: usize,
    pub anchors_healthy: usize,
    pub issue_counts: HealthIssueCounts,
    pub groups: HealthIssueGroups,
}

#[derive(Debug, Clone)]
struct ParsedFile {
    absolute_path: PathBuf,
    display_path: String,
    anchors: Vec<ParsedAnchor>,
}

#[derive(Debug, Default)]
struct HealthTracker {
    seen_by_anchor_id: HashMap<i64, String>,
    seen_materialized_ids: HashSet<i64>,
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

    pub fn audit_paths(&self, paths: &[String]) -> Result<HealthReport> {
        let files = self.collect_workspace_files(paths)?;
        let scoped_files = files
            .iter()
            .map(|file| self.anchor_repo.normalized_path_for(&file.display_path))
            .collect::<HashSet<_>>();

        let mut report = HealthReport {
            files_scanned: 0,
            anchors_seen: 0,
            anchors_healthy: 0,
            issue_counts: HealthIssueCounts {
                anchors_moved: 0,
                anchors_missing: 0,
                duplicate_anchor_ids: 0,
                unknown_anchor_ids: 0,
                entities_without_anchors: 0,
                anchors_without_entities: 0,
            },
            groups: HealthIssueGroups {
                moved_anchors: Vec::new(),
                missing_anchors: Vec::new(),
                duplicate_anchor_ids: Vec::new(),
                unknown_anchor_ids: Vec::new(),
                entities_without_anchors: Vec::new(),
                anchors_without_entities: Vec::new(),
            },
        };

        let mut tracker = HealthTracker::default();
        for file in files {
            if self.inspect_file(&file, &mut report, &mut tracker)? {
                report.files_scanned += 1;
            }
        }

        self.report_missing_db_anchors(&scoped_files, &tracker.seen_materialized_ids, &mut report)?;
        self.report_entities_without_anchors(&mut report)?;
        self.report_anchors_without_entities(&mut report)?;
        Ok(report)
    }

    fn collect_workspace_files(&self, paths: &[String]) -> Result<Vec<ParsedFile>> {
        let filter = TepIgnoreFilter::for_workspace_root(&self.workspace_root);
        let files = filter.collect_paths(paths)?;
        Ok(files
            .into_iter()
            .map(|path| ParsedFile {
                absolute_path: resolve_from_workspace(&path, &self.workspace_root),
                display_path: display_path(&path),
                anchors: Vec::new(),
            })
            .collect())
    }

    fn inspect_file(&self, file: &ParsedFile, report: &mut HealthReport, tracker: &mut HealthTracker) -> Result<bool> {
        let parsed = self.read_parsed_file(file)?;
        if parsed.anchors.is_empty() {
            return Ok(false);
        }

        report.anchors_seen += parsed.anchors.len();
        let mut local_ids = HashSet::new();

        for anchor in &parsed.anchors {
            if let AnchorKind::Materialized = anchor.kind() {
                self.inspect_materialized_anchor(anchor, &parsed.display_path, report, tracker, &mut local_ids)?;
            }
        }

        Ok(true)
    }

    fn inspect_materialized_anchor(
        &self,
        anchor: &ParsedAnchor,
        file_path: &str,
        report: &mut HealthReport,
        tracker: &mut HealthTracker,
        local_ids: &mut HashSet<i64>,
    ) -> Result<()> {
        let anchor_id = anchor.anchor_id.expect("materialized anchor should have id");
        tracker.seen_materialized_ids.insert(anchor_id);

        if !local_ids.insert(anchor_id) {
            report.issue_counts.duplicate_anchor_ids += 1;
            report
                .groups
                .duplicate_anchor_ids
                .push(format!("duplicate materialized anchor {} found in the same file {}", anchor_id, file_path));
            return Ok(());
        }

        if let Some(existing_file) = tracker.seen_by_anchor_id.get(&anchor_id) {
            if existing_file != file_path {
                report.issue_counts.duplicate_anchor_ids += 1;
                report.groups.duplicate_anchor_ids.push(format!(
                    "anchor {} appears in multiple files: {} and {}",
                    anchor_id, existing_file, file_path
                ));
                return Ok(());
            }
        } else {
            tracker.seen_by_anchor_id.insert(anchor_id, file_path.to_string());
        }

        match self.anchor_repo.find_by_id(anchor_id)? {
            Some(stored) => {
                let normalized_file_path = self.anchor_repo.normalized_path_for(file_path);
                if normalized_file_path != stored.file_path
                    || stored.line != Some(anchor.line)
                    || stored.shift != Some(anchor.shift)
                    || stored.offset != Some(anchor.start_offset as i64)
                {
                    report.issue_counts.anchors_moved += 1;
                    report.groups.moved_anchors.push(format!(
                        "anchor {} metadata drifted in {} (db: {} {:?}:{:?} [{:?}], file: {} {}:{} [{}])",
                        anchor_id,
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
                    "materialized anchor {} was found in a file but does not exist in the database ({})",
                    anchor_id, file_path
                ));
            }
        }

        Ok(())
    }

    fn report_missing_db_anchors(
        &self,
        scoped_files: &HashSet<String>,
        seen_materialized_ids: &HashSet<i64>,
        report: &mut HealthReport,
    ) -> Result<()> {
        for anchor in self.anchor_repo.list_all()? {
            if scoped_files.contains(&anchor.file_path) && !seen_materialized_ids.contains(&anchor.anchor_id) {
                report.issue_counts.anchors_missing += 1;
                report
                    .groups
                    .missing_anchors
                    .push(format!("missing anchor {} recorded in db but not found in file {}", anchor.anchor_id, anchor.file_path));
            }
        }
        Ok(())
    }

    fn report_entities_without_anchors(&self, report: &mut HealthReport) -> Result<()> {
        for entity in self.entity_repo.list_without_anchors()? {
            report.issue_counts.entities_without_anchors += 1;
            report
                .groups
                .entities_without_anchors
                .push(format!("{} ({})", entity.entity_id, entity.name));
        }
        Ok(())
    }

    fn report_anchors_without_entities(&self, report: &mut HealthReport) -> Result<()> {
        for anchor in self.anchor_repo.list_without_entities()? {
            report.issue_counts.anchors_without_entities += 1;
            report
                .groups
                .anchors_without_entities
                .push(format!("{} {}", anchor.anchor_id, anchor.file_path));
        }
        Ok(())
    }

    fn read_parsed_file(&self, file: &ParsedFile) -> Result<ParsedFile> {
        let original = fs::read_to_string(&file.absolute_path)
            .with_context(|| format!("failed to read {}", file.absolute_path.display()))?;
        Ok(ParsedFile {
            absolute_path: file.absolute_path.clone(),
            display_path: file.display_path.clone(),
            anchors: parse_anchors(&original),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::entity::NewEntity;
    use crate::repository::anchor_entity_repository::AnchorEntityRepository;

    fn setup_health_service() -> HealthService<'static> {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        db::ensure_schema(conn).expect("schema should apply");
        HealthService::with_workspace_root(conn, "/tmp/project")
    }

    #[test]
    fn reports_entities_without_anchors() {
        let service = setup_health_service();
        service
            .entity_repo
            .create(&NewEntity {
                name: "orphan.entity".into(),
                r#ref: None,
                description: None,
            })
            .unwrap();

        let report = service.audit_paths(&[".".into()]).unwrap();
        assert_eq!(report.issue_counts.entities_without_anchors, 1);
        assert!(report.groups.entities_without_anchors.iter().any(|item| item.contains("orphan.entity")));
    }

    #[test]
    fn reports_anchors_without_entities() {
        let service = setup_health_service();
        service
            .anchor_repo
            .create(1, "./docs/orphan.md", Some(1), Some(0), Some(0))
            .unwrap();

        let report = service.audit_paths(&[".".into()]).unwrap();
        assert_eq!(report.issue_counts.anchors_without_entities, 1);
        assert!(report.groups.anchors_without_entities.iter().any(|item| item.contains("./docs/orphan.md")));
    }

    #[test]
    fn reports_healthy_anchor_when_attached() {
        let service = setup_health_service();
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
            .create(1, "./docs/student.md", Some(1), Some(0), Some(0))
            .unwrap();
        let rel = AnchorEntityRepository::new(service.anchor_repo.conn);
        rel.attach(anchor.anchor_id, entity.entity_id).unwrap();

        std::fs::create_dir_all("/tmp/project/docs").ok();
        std::fs::write("/tmp/project/docs/student.md", "[#!#1#tep:1](student)").ok(); // #tepignore

        let report = service.audit_paths(&["./docs/student.md".into()]).unwrap();
        assert_eq!(report.anchors_healthy, 1);
    }
}

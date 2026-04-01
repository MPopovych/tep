use crate::output::anchor_format::format_anchor_compact;
use crate::service::anchor_service::{AnchorShowResult, AnchorSyncResult};
use crate::service::health_service::HealthReport;

pub fn format_anchor_sync_result(result: &AnchorSyncResult) -> String {
    format!(
        "anchor sync complete\nfiles_processed: {}\nanchors_seen: {}\nanchors_created: {}\nanchors_dropped: {}\nrelations_synced: {}\n",
        result.files_processed,
        result.anchors_seen,
        result.anchors_created,
        result.anchors_dropped,
        result.relations_synced
    )
}

pub fn format_anchor_health_result(report: &HealthReport) -> String {
    let mut out = format!(
        "workspace health report\nfiles_scanned: {}\nanchors_seen: {}\nanchors_healthy: {}\nanchors_moved: {}\nanchors_missing: {}\nduplicate_anchor_ids: {}\nunknown_anchor_ids: {}\nentities_without_anchors: {}\nanchors_without_entities: {}\n",
        report.files_scanned,
        report.anchors_seen,
        report.anchors_healthy,
        report.issue_counts.anchors_moved,
        report.issue_counts.anchors_missing,
        report.issue_counts.duplicate_anchor_ids,
        report.issue_counts.unknown_anchor_ids,
        report.issue_counts.entities_without_anchors,
        report.issue_counts.anchors_without_entities
    );

    append_group(&mut out, "moved anchors", &report.groups.moved_anchors);
    append_group(&mut out, "missing anchors", &report.groups.missing_anchors);
    append_group(&mut out, "duplicate anchor ids", &report.groups.duplicate_anchor_ids);
    append_group(&mut out, "unknown anchor ids", &report.groups.unknown_anchor_ids);
    append_group(&mut out, "entities without anchors", &report.groups.entities_without_anchors);
    append_group(&mut out, "anchors without entities", &report.groups.anchors_without_entities);

    out
}

pub fn format_anchor_list(anchors: &[crate::anchor::Anchor]) -> String {
    use crate::output::anchor_format::format_anchor_compact;
    if anchors.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for anchor in anchors {
        out.push_str(&format_anchor_compact(anchor));
    }
    out
}

pub fn format_anchor_show(result: &AnchorShowResult) -> String {
    let mut out = format_anchor_compact(&result.anchor);
    for entity in &result.entities {
        out.push_str(&format!("{} ({})\n", entity.entity_id, entity.name));
    }
    out
}

fn append_group(out: &mut String, label: &str, items: &[String]) {
    if !items.is_empty() {
        out.push_str(&format!("{}:\n", label));
        for item in items {
            out.push_str(&format!("- {}\n", item));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::Anchor;
    use crate::entity::Entity;
    use crate::service::health_service::{HealthIssueCounts, HealthIssueGroups};

    #[test]
    fn formats_anchor_sync_result() {
        let rendered = format_anchor_sync_result(&AnchorSyncResult {
            files_processed: 2,
            anchors_seen: 3,
            anchors_created: 1,
            anchors_dropped: 1,
            relations_synced: 2,
        });

        assert!(rendered.contains("anchor sync complete"));
        assert!(rendered.contains("files_processed: 2"));
        assert!(rendered.contains("anchors_seen: 3"));
        assert!(rendered.contains("anchors_created: 1"));
        assert!(rendered.contains("anchors_dropped: 1"));
        assert!(rendered.contains("relations_synced: 2"));
    }

    #[test]
    fn formats_anchor_health_result() {
        let rendered = format_anchor_health_result(&HealthReport {
            files_scanned: 2,
            anchors_seen: 3,
            anchors_healthy: 1,
            issue_counts: HealthIssueCounts {
                anchors_moved: 1,
                anchors_missing: 1,
                duplicate_anchor_ids: 1,
                unknown_anchor_ids: 0,
                entities_without_anchors: 1,
                anchors_without_entities: 1,
            },
            groups: HealthIssueGroups {
                moved_anchors: vec!["anchor 7 metadata drifted".into()],
                missing_anchors: vec![],
                duplicate_anchor_ids: vec![],
                unknown_anchor_ids: vec![],
                entities_without_anchors: vec!["3 (orphan.entity)".into()],
                anchors_without_entities: vec!["9 ./docs/orphan.md".into()],
            },
        });

        assert!(rendered.contains("workspace health report"));
        assert!(rendered.contains("files_scanned: 2"));
        assert!(rendered.contains("anchors_moved: 1"));
        assert!(rendered.contains("entities_without_anchors: 1"));
        assert!(rendered.contains("anchors_without_entities: 1"));
        assert!(rendered.contains("moved anchors:"));
        assert!(rendered.contains("entities without anchors:"));
    }

    #[test]
    fn formats_anchor_list_empty() {
        assert_eq!(format_anchor_list(&[]), "");
    }

    #[test]
    fn formats_anchor_list_with_anchors() {
        use crate::anchor::Anchor;
        let anchors = vec![
            Anchor {
                anchor_id: 1,
                version: 1,
                name: Some("student_processor".into()),
                file_path: "./src/student.rs".into(),
                line: Some(10),
                shift: Some(0),
                offset: Some(100),
                created_at: "1".into(),
                updated_at: "2".into(),
            },
            Anchor {
                anchor_id: 2,
                version: 1,
                name: None,
                file_path: "./src/teacher.rs".into(),
                line: Some(5),
                shift: Some(0),
                offset: Some(50),
                created_at: "1".into(),
                updated_at: "2".into(),
            },
        ];
        let rendered = format_anchor_list(&anchors);
        assert!(rendered.contains("1 (student_processor)"));
        assert!(rendered.contains("./src/student.rs"));
        assert!(rendered.contains("2\n"));
        assert!(rendered.contains("./src/teacher.rs"));
    }

    #[test]
    fn formats_anchor_show() {
        let rendered = format_anchor_show(&AnchorShowResult {
            anchor: Anchor {
                anchor_id: 7,
                version: 1,
                name: Some("student_processor".into()),
                file_path: "./file.md".into(),
                line: Some(3),
                shift: Some(4),
                offset: Some(22),
                created_at: "1".into(),
                updated_at: "2".into(),
            },
            entities: vec![Entity {
                entity_id: 1,
                name: "student".into(),
                r#ref: None,
                description: None,
                created_at: "1".into(),
                updated_at: "2".into(),
            }],
        });

        assert!(rendered.contains("7 (student_processor)"));
        assert!(rendered.contains("./file.md"));
        assert!(rendered.contains("student_processor"));
        assert!(rendered.contains("\x1b[36m"));
        assert!(rendered.contains("\x1b[32m3\x1b[0m"));
        assert!(rendered.contains("\x1b[35m4\x1b[0m"));
        assert!(rendered.contains("1 (student)"));
    }
}

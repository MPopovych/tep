use crate::dto::{AnchorShowDto, AnchorSyncDto, HealthDto};
use crate::output::anchor_format::format_anchor_line;

pub fn format_anchor_sync_result(result: &AnchorSyncDto) -> String {
    format!(
        "anchor sync complete\nfiles_processed: {}\nanchors_seen: {}\nanchors_created: {}\nanchors_dropped: {}\nrelations_synced: {}\n",
        result.files_processed,
        result.anchors_seen,
        result.anchors_created,
        result.anchors_dropped,
        result.relations_synced
    )
}

pub fn format_anchor_health_result(report: &HealthDto) -> String {
    let mut out = format!(
        "workspace health report\nfiles_scanned: {}\nanchors_seen: {}\nanchors_healthy: {}\nanchors_moved: {}\nanchors_missing: {}\nduplicate_anchor_ids: {}\nunknown_anchor_ids: {}\nentities_without_anchors: {}\nanchors_without_entities: {}\n",
        report.files_scanned,
        report.anchors_seen,
        report.anchors_healthy,
        report.anchors_moved,
        report.anchors_missing,
        report.duplicate_anchor_ids,
        report.unknown_anchor_ids,
        report.entities_without_anchors,
        report.anchors_without_entities
    );
    if !report.issues.is_empty() {
        out.push_str("issues:\n");
        for issue in &report.issues {
            out.push_str(&format!("- {}\n", issue));
        }
    }
    out
}

pub fn format_anchor_list(anchors: &[crate::dto::AnchorDto]) -> String {
    if anchors.is_empty() {
        return String::new();
    }
    anchors.iter().map(format_anchor_line).collect()
}

pub fn format_anchor_show(result: &AnchorShowDto) -> String {
    let mut out = format_anchor_line(&result.anchor);
    for entity in &result.entities {
        out.push_str(&format!("{} ({})\n", entity.id, entity.name));
    }
    out
}

// #tepignoreafter
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{AnchorDto, AnchorShowDto, AnchorSyncDto, EntityDto, HealthDto};

    fn sample_anchor() -> AnchorDto {
        AnchorDto {
            id: 7,
            name: "student_processor".into(),
            file: "./file.md".into(),
            line: Some(3),
            shift: Some(4),
            offset: Some(22),
        }
    }

    #[test]
    fn formats_anchor_sync_result() {
        let rendered = format_anchor_sync_result(&AnchorSyncDto {
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
        let rendered = format_anchor_health_result(&HealthDto {
            files_scanned: 2,
            anchors_seen: 3,
            anchors_healthy: 1,
            anchors_moved: 1,
            anchors_missing: 1,
            duplicate_anchor_ids: 1,
            unknown_anchor_ids: 0,
            entities_without_anchors: 1,
            anchors_without_entities: 1,
            issues: vec![
                "anchor 7 metadata drifted".into(),
                "3 (orphan.entity)".into(),
            ],
        });
        assert!(rendered.contains("workspace health report"));
        assert!(rendered.contains("files_scanned: 2"));
        assert!(rendered.contains("anchors_moved: 1"));
        assert!(rendered.contains("entities_without_anchors: 1"));
        assert!(rendered.contains("issues:"));
        assert!(rendered.contains("anchor 7 metadata drifted"));
    }

    #[test]
    fn formats_anchor_list_empty() {
        assert_eq!(format_anchor_list(&[]), "");
    }

    #[test]
    fn formats_anchor_list_with_anchors() {
        let anchors = vec![
            AnchorDto {
                id: 1,
                name: "student_processor".into(),
                file: "./src/student.rs".into(),
                line: Some(10),
                shift: Some(0),
                offset: Some(100),
            },
            AnchorDto {
                id: 2,
                name: "teacher_service".into(),
                file: "./src/teacher.rs".into(),
                line: Some(5),
                shift: Some(0),
                offset: Some(50),
            },
        ];
        let rendered = format_anchor_list(&anchors);
        assert!(rendered.contains("anchor:1 student_processor"));
        assert!(rendered.contains("./src/student.rs"));
        assert!(rendered.contains("anchor:2 teacher_service"));
        assert!(rendered.contains("./src/teacher.rs"));
    }

    #[test]
    fn formats_anchor_show() {
        let rendered = format_anchor_show(&AnchorShowDto {
            anchor: sample_anchor(),
            entities: vec![EntityDto {
                id: 1,
                name: "student".into(),
                r#ref: None,
                description: None,
            }],
        });
        assert!(rendered.contains("anchor:7 student_processor"));
        assert!(rendered.contains("./file.md"));
        assert!(rendered.contains("1 (student)"));
    }
}

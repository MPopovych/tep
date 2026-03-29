use crate::anchor::Anchor;
use crate::service::anchor_service::{AnchorShowResult, AnchorSyncResult};

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_MAGENTA: &str = "\x1b[35m";

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

pub fn format_anchor_relation_result(prefix: &str, anchor_id: i64, entity_target: &str) -> String {
    format!("{prefix}\nanchor_id: {anchor_id}\nentity: {entity_target}\n")
}

pub fn format_anchor_show(result: &AnchorShowResult) -> String {
    let mut out = format_anchor_compact(&result.anchor);
    for entity in &result.entities {
        out.push_str(&format!("{} ({})\n", entity.entity_id, entity.name));
    }
    out
}

pub fn format_anchor_compact(anchor: &Anchor) -> String {
    format!(
        "{}\n{}{}{} ({}{}{}:{}{}{} ) {}[{}]{}\n",
        anchor.anchor_id,
        ANSI_CYAN,
        anchor.file_path,
        ANSI_RESET,
        ANSI_GREEN,
        anchor.line.unwrap_or(0),
        ANSI_RESET,
        ANSI_MAGENTA,
        anchor.shift.unwrap_or(0),
        ANSI_RESET,
        ANSI_YELLOW,
        anchor.offset.unwrap_or(0),
        ANSI_RESET
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Entity;

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
    fn formats_anchor_relation_result() {
        let rendered = format_anchor_relation_result("attached", 42, "student");
        assert!(rendered.contains("attached"));
        assert!(rendered.contains("anchor_id: 42"));
        assert!(rendered.contains("entity: student"));
    }

    #[test]
    fn formats_anchor_show() {
        let rendered = format_anchor_show(&AnchorShowResult {
            anchor: Anchor {
                anchor_id: 7,
                version: 1,
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
                created_at: "1".into(),
                updated_at: "2".into(),
            }],
        });

        assert!(rendered.contains("7"));
        assert!(rendered.contains("./file.md"));
        assert!(rendered.contains("\x1b[36m"));
        assert!(rendered.contains("\x1b[32m3\x1b[0m"));
        assert!(rendered.contains("\x1b[35m4\x1b[0m"));
        assert!(rendered.contains("1 (student)"));
    }
}

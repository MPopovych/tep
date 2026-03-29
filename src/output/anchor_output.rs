use crate::service::anchor_service::AnchorSyncResult;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}

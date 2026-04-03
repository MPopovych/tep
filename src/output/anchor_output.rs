use crate::dto::{AnchorShowDto, AnchorSyncDto, HealthDto};
use crate::output::anchor_format::format_anchor_line;

pub fn format_anchor_sync_result(result: &AnchorSyncDto) -> String {
    let mut out = format!(
        "anchor sync complete\nfiles_processed: {}\nanchors_seen: {}\nanchors_created: {}\nanchors_dropped: {}\nrelations_synced: {}\nmetadata_updated: {}\n",
        result.files_processed,
        result.anchors_seen,
        result.anchors_created,
        result.anchors_dropped,
        result.relations_synced,
        result.metadata_updated,
    );
    if !result.warnings.is_empty() {
        out.push_str("warnings:\n");
        for warning in &result.warnings {
            out.push_str(&format!("- {}\n", warning));
        }
    }
    out
}

pub fn format_anchor_health_result(report: &HealthDto) -> String {
    let mut out = format!(
        "workspace health report\nfiles_scanned: {}\nanchors_seen: {}\nanchors_healthy: {}\nanchors_moved: {}\nanchors_missing: {}\nduplicate_anchor_ids: {}\nunknown_anchor_ids: {}\nentities_without_anchors: {}\nanchors_without_entities: {}\nmetadata_warnings: {}\n",
        report.files_scanned,
        report.anchors_seen,
        report.anchors_healthy,
        report.anchors_moved,
        report.anchors_missing,
        report.duplicate_anchor_ids,
        report.unknown_anchor_ids,
        report.entities_without_anchors,
        report.anchors_without_entities,
        report.metadata_warnings,
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

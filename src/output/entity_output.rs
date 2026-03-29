use crate::anchor::Anchor;
use crate::entity::Entity;
use crate::output::anchor_format::{format_anchor_compact, format_anchor_location};
use crate::output::styles::{ANSI_CYAN, ANSI_YELLOW, paint};
use crate::service::entity_service::{EntityAutoResult, EntityContextResult, EntityShowResult};

pub fn format_entity_created(prefix: &str, entity: &Entity) -> String {
    format!("{prefix}\n{} ({})\n", entity.entity_id, entity.name)
}

pub fn format_entity_auto_result(result: &EntityAutoResult) -> String {
    format!(
        "entity auto complete\nfiles_processed: {}\ndeclarations_seen: {}\nentities_ensured: {}\nrefs_filled: {}\nanchors_created: {}\nrelations_synced: {}\n",
        result.files_processed,
        result.declarations_seen,
        result.entities_ensured,
        result.refs_filled,
        result.anchors_created,
        result.relations_synced
    )
}

pub fn format_entity_show(result: &EntityShowResult) -> String {
    let mut out = format!("{} ({})\n", result.entity.entity_id, result.entity.name);
    for anchor in &result.anchors {
        out.push_str(&format_anchor_compact(anchor));
    }
    out
}

pub fn format_entity_context(result: &EntityContextResult) -> String {
    let mut out = format!("{} ({})\n", result.entity.entity_id, result.entity.name);
    if let Some(entity_ref) = &result.entity.r#ref {
        out.push_str(&format!("{}\n", paint(ANSI_YELLOW, format!("ref: {}", paint(ANSI_CYAN, entity_ref)))));
    }
    out.push('\n');

    for item in &result.anchors {
        out.push_str(&format!("anchor {}\n", item.anchor.anchor_id));
        out.push_str(&format_anchor_location(&item.anchor));
        if let Some(snippet) = &item.snippet {
            out.push_str("snippet:\n");
            out.push_str(snippet);
            out.push('\n');
        }
        out.push('\n');
    }

    if !result.files.is_empty() {
        out.push_str("files:\n");
        for file in &result.files {
            out.push_str(&format!("- {}\n", paint(ANSI_CYAN, file)));
        }
    }

    out
}

pub fn format_entity_list(entities: &[Entity]) -> String {
    if entities.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for entity in entities {
        out.push_str(&format!("{} ({})\n", entity.entity_id, entity.name));
    }
    out
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::entity_service::EntityContextAnchor;

    fn sample_entity() -> Entity {
        Entity {
            entity_id: 42,
            name: "student".into(),
            r#ref: Some("./docs/student.md".into()),
            created_at: "1".into(),
            updated_at: "2".into(),
        }
    }

    fn sample_anchor() -> Anchor {
        Anchor {
            anchor_id: 7,
            version: 1,
            file_path: "./file.md".into(),
            line: Some(3),
            shift: Some(4),
            offset: Some(22),
            created_at: "1".into(),
            updated_at: "2".into(),
        }
    }

    #[test]
    fn formats_created_entity() {
        let rendered = format_entity_created("created", &sample_entity());
        assert!(rendered.contains("created"));
        assert!(rendered.contains("42 (student)"));
    }

    #[test]
    fn formats_entity_auto_result() {
        let rendered = format_entity_auto_result(&EntityAutoResult {
            files_processed: 1,
            declarations_seen: 2,
            entities_ensured: 2,
            refs_filled: 1,
            anchors_created: 2,
            relations_synced: 2,
        });
        assert!(rendered.contains("entity auto complete"));
        assert!(rendered.contains("declarations_seen: 2"));
        assert!(rendered.contains("refs_filled: 1"));
    }

    #[test]
    fn formats_entity_show_with_anchor() {
        let rendered = format_entity_show(&EntityShowResult {
            entity: sample_entity(),
            anchors: vec![sample_anchor()],
        });
        assert!(rendered.contains("42 (student)"));
        assert!(rendered.contains("7"));
        assert!(rendered.contains("./file.md"));
        assert!(rendered.contains("\x1b[36m"));
        assert!(rendered.contains("\x1b[32m3\x1b[0m"));
        assert!(rendered.contains("\x1b[35m4\x1b[0m"));
    }

    #[test]
    fn formats_entity_context() {
        let rendered = format_entity_context(&EntityContextResult {
            entity: sample_entity(),
            anchors: vec![EntityContextAnchor {
                anchor: sample_anchor(),
                snippet: Some("hello context".into()),
            }],
            files: vec!["./file.md".into()],
        });
        assert!(rendered.contains("42 (student)"));
        assert!(rendered.contains("ref:"));
        assert!(rendered.contains("./docs/student.md"));
        assert!(rendered.contains("anchor 7"));
        assert!(rendered.contains("snippet:"));
        assert!(rendered.contains("hello context"));
        assert!(rendered.contains("files:"));
        assert!(rendered.contains("- \x1b[36m./file.md\x1b[0m"));
    }

    #[test]
    fn formats_entity_list() {
        let rendered = format_entity_list(&[sample_entity()]);
        assert!(rendered.contains("42 (student)"));
    }
}

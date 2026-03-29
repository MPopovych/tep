use crate::entity::{Entity, EntityLink};
use crate::output::styles::{ANSI_CYAN, ANSI_YELLOW, paint};
use crate::service::entity_service::LinkedEntityContext;

pub fn render_entity_header(entity: &Entity) -> String {
    format!("{} ({})\n", entity.entity_id, entity.name)
}

pub fn append_entity_metadata(out: &mut String, entity: &Entity) {
    if let Some(entity_ref) = &entity.r#ref {
        out.push_str(&format!("{}\n", paint(ANSI_YELLOW, format!("ref: {}", paint(ANSI_CYAN, entity_ref)))));
    }
    if let Some(description) = &entity.description {
        out.push_str(&format!("description: {}\n", description));
    }
}

pub fn append_files_block(out: &mut String, files: &[String]) {
    if !files.is_empty() {
        out.push_str("files:\n");
        for file in files {
            out.push_str(&format!("- {}\n", paint(ANSI_CYAN, file)));
        }
    }
}

pub fn append_show_link_block(out: &mut String, header: &str, links: &[(EntityLink, Entity)], outgoing: bool) {
    if !links.is_empty() {
        out.push_str(header);
        out.push('\n');
        for (link, entity) in links {
            let arrow = if outgoing { "->" } else { "<-" };
            out.push_str(&format!("{} {} ({})\n   relation: {}\n", arrow, entity.entity_id, entity.name, link.relation));
        }
    }
}

pub fn append_context_link_block(out: &mut String, links: &[LinkedEntityContext]) {
    if !links.is_empty() {
        out.push_str("linked entities:\n");
        for item in links {
            out.push_str(&format!("- {} ({})\n", item.entity.entity_id, item.entity.name));
            if let Some(entity_ref) = &item.entity.r#ref {
                out.push_str(&format!("  ref: {}\n", entity_ref));
            }
            if let Some(description) = &item.entity.description {
                out.push_str(&format!("  description: {}\n", description));
            }
            out.push_str(&format!(
                "  edge: ({}->{})[{}] {}\n",
                item.link.from_entity_id,
                item.link.to_entity_id,
                item.depth,
                item.link.relation
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entity() -> Entity {
        Entity {
            entity_id: 42,
            name: "student".into(),
            r#ref: Some("./docs/student.md".into()),
            description: Some("A learner".into()),
            created_at: "1".into(),
            updated_at: "2".into(),
        }
    }

    #[test]
    fn renders_entity_header() {
        let rendered = render_entity_header(&sample_entity());
        assert_eq!(rendered, "42 (student)\n");
    }

    #[test]
    fn appends_entity_metadata() {
        let mut out = String::new();
        append_entity_metadata(&mut out, &sample_entity());
        assert!(out.contains("ref:"));
        assert!(out.contains("description: A learner"));
    }

    #[test]
    fn appends_files_block() {
        let mut out = String::new();
        append_files_block(&mut out, &["./docs/a.md".into()]);
        assert!(out.contains("files:"));
        assert!(out.contains("./docs/a.md"));
    }
}

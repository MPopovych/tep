use crate::anchor::Anchor;
use crate::entity::Entity;
use crate::service::entity_service::EntityShowResult;

pub fn format_entity_created(prefix: &str, entity: &Entity) -> String {
    format!("{prefix}\n{} ({})\n", entity.entity_id, entity.name)
}

pub fn format_entity_show(result: &EntityShowResult) -> String {
    let mut out = format!("{} ({})\n", result.entity.entity_id, result.entity.name);
    for anchor in &result.anchors {
        out.push_str(&format_anchor_compact(anchor));
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

pub fn format_anchor_compact(anchor: &Anchor) -> String {
    format!(
        "{}\n{} ({}:{}) [{}]\n",
        anchor.anchor_id,
        anchor.file_path,
        anchor.line.unwrap_or(0),
        anchor.shift.unwrap_or(0),
        anchor.offset.unwrap_or(0)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn formats_entity_show_with_anchor() {
        let rendered = format_entity_show(&EntityShowResult {
            entity: sample_entity(),
            anchors: vec![sample_anchor()],
        });
        assert!(rendered.contains("42 (student)"));
        assert!(rendered.contains("7"));
        assert!(rendered.contains("./file.md (3:4) [22]"));
    }

    #[test]
    fn formats_entity_list() {
        let rendered = format_entity_list(&[sample_entity()]);
        assert!(rendered.contains("42 (student)"));
    }
}

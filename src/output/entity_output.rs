use crate::entity::Entity;

pub fn format_entity(prefix: &str, entity: &Entity) -> String {
    format!(
        "{prefix}\nid: {}\nname: {}\nref: {}\ncreated_at: {}\nupdated_at: {}\n",
        entity.entity_id,
        entity.name,
        entity.r#ref.as_deref().unwrap_or("-"),
        entity.created_at,
        entity.updated_at
    )
}

pub fn format_entity_list(entities: &[Entity]) -> String {
    if entities.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for entity in entities {
        out.push_str(&format!(
            "{}\t{}\t{}\n",
            entity.entity_id,
            entity.name,
            entity.r#ref.as_deref().unwrap_or("-")
        ));
    }
    out
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

    #[test]
    fn formats_single_entity() {
        let rendered = format_entity("entity", &sample_entity());
        assert!(rendered.contains("entity"));
        assert!(rendered.contains("id: 42"));
        assert!(rendered.contains("name: student"));
        assert!(rendered.contains("ref: ./docs/student.md"));
    }

    #[test]
    fn formats_entity_list() {
        let rendered = format_entity_list(&[sample_entity()]);
        assert!(rendered.contains("42\tstudent\t./docs/student.md"));
    }
}

use crate::entity::Entity;
use crate::output::styles::{ANSI_CYAN, ANSI_YELLOW, paint};
use crate::service::entity_service::LinkedEntityContext;

pub fn render_entity_header(entity: &Entity) -> String {
    format!("{} ({})\n", entity.entity_id, entity.name)
}

pub fn append_entity_metadata(out: &mut String, entity: &Entity) {
    if let Some(entity_ref) = &entity.r#ref {
        out.push_str(&format!(
            "{}\n",
            paint(
                ANSI_YELLOW,
                format!("ref: {}", paint(ANSI_CYAN, entity_ref))
            )
        ));
    }
    if let Some(description) = &entity.description {
        out.push_str(&format!("description: {}\n", description));
    }
}

/// Shared link block used by both `entity show` and `entity context`.
/// Format per line: `-> id (name) [ref]  relation  [depth:N]`
/// Direction is derived from `link.from_entity_id` vs root entity id.
/// `[depth:N]` is only shown when depth > 1.
pub fn append_link_block(out: &mut String, root_entity_id: i64, links: &[LinkedEntityContext]) {
    if links.is_empty() {
        return;
    }
    out.push_str("links:\n");
    for item in links {
        let arrow = if item.link.from_entity_id == root_entity_id {
            "->"
        } else {
            "<-"
        };
        let ref_part = item
            .entity
            .r#ref
            .as_deref()
            .map(|r| format!(" [{}]", r))
            .unwrap_or_default();
        let depth_part = if item.depth > 1 {
            format!("  [depth:{}]", item.depth)
        } else {
            String::new()
        };
        out.push_str(&format!(
            "{} {} ({}){}  {}{}\n",
            arrow,
            item.entity.entity_id,
            item.entity.name,
            ref_part,
            item.link.relation,
            depth_part,
        ));
    }
}

// #tepignoreafter
#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Entity, EntityLink};
    use crate::service::entity_service::LinkedEntityContext;

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
    fn append_link_block_shows_direction_and_relation() {
        let mut out = String::new();
        let links = vec![
            LinkedEntityContext {
                link: EntityLink {
                    from_entity_id: 42,
                    to_entity_id: 10,
                    relation: "student has subjects".into(),
                    created_at: "1".into(),
                    updated_at: "2".into(),
                },
                entity: Entity {
                    entity_id: 10,
                    name: "subject".into(),
                    r#ref: Some("./docs/subject.md".into()),
                    description: None,
                    created_at: "1".into(),
                    updated_at: "2".into(),
                },
                depth: 1,
            },
            LinkedEntityContext {
                link: EntityLink {
                    from_entity_id: 5,
                    to_entity_id: 42,
                    relation: "teacher mentors student".into(),
                    created_at: "1".into(),
                    updated_at: "2".into(),
                },
                entity: Entity {
                    entity_id: 5,
                    name: "teacher".into(),
                    r#ref: None,
                    description: None,
                    created_at: "1".into(),
                    updated_at: "2".into(),
                },
                depth: 2,
            },
        ];
        append_link_block(&mut out, 42, &links);
        assert!(out.contains("links:"));
        assert!(out.contains("-> 10 (subject) [./docs/subject.md]  student has subjects"));
        assert!(out.contains("<- 5 (teacher)  teacher mentors student  [depth:2]"));
    }
}

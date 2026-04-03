use crate::dto::{EntityDto, LinkDto};
use crate::output::styles::{ANSI_CYAN, ANSI_YELLOW, paint};

pub fn render_entity_header(entity: &EntityDto) -> String {
    format!("{} ({})\n", entity.id, entity.name)
}

pub fn append_entity_metadata(out: &mut String, entity: &EntityDto) {
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
/// `[depth:N]` shown only when depth > 1.
pub fn append_link_block(out: &mut String, links: &[LinkDto]) {
    if links.is_empty() {
        return;
    }
    out.push_str("links:\n");
    for item in links {
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
            item.direction, item.entity.id, item.entity.name, ref_part, item.relation, depth_part,
        ));
    }
}

// #tepignoreafter
// #tepignoreafter
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{EntityDto, LinkDto};

    fn sample_entity_dto() -> EntityDto {
        EntityDto {
            id: 42,
            name: "student".into(),
            r#ref: Some("./docs/student.md".into()),
            description: Some("A learner".into()),
        }
    }

    #[test]
    fn renders_entity_header() {
        let rendered = render_entity_header(&sample_entity_dto());
        assert_eq!(rendered, "42 (student)\n");
    }

    #[test]
    fn appends_entity_metadata() {
        let mut out = String::new();
        append_entity_metadata(&mut out, &sample_entity_dto());
        assert!(out.contains("ref:"));
        assert!(out.contains("description: A learner"));
    }

    #[test]
    fn append_link_block_shows_direction_and_relation() {
        let mut out = String::new();
        let links = vec![
            LinkDto {
                direction: "->".into(),
                entity: EntityDto {
                    id: 10,
                    name: "subject".into(),
                    r#ref: Some("./docs/subject.md".into()),
                    description: None,
                },
                relation: "student has subjects".into(),
                depth: 1,
            },
            LinkDto {
                direction: "<-".into(),
                entity: EntityDto {
                    id: 5,
                    name: "teacher".into(),
                    r#ref: None,
                    description: None,
                },
                relation: "teacher mentors student".into(),
                depth: 2,
            },
        ];
        append_link_block(&mut out, &links);
        assert!(out.contains("links:"));
        assert!(out.contains("-> 10 (subject) [./docs/subject.md]  student has subjects"));
        assert!(out.contains("<- 5 (teacher)  teacher mentors student  [depth:2]"));
    }
}

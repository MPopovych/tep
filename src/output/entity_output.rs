use crate::dto::{EntityAutoDto, EntityContextDto, EntityDto, EntityShowDto};
use crate::output::anchor_format::format_anchor_line;
use crate::output::render::{append_entity_metadata, append_link_block, render_entity_header};

pub fn format_entity_created(prefix: &str, entity: &EntityDto) -> String {
    let mut out = format!("{prefix}\n");
    out.push_str(&render_entity_header(entity));
    if let Some(description) = &entity.description {
        out.push_str(&format!("description: {}\n", description));
    }
    out
}

pub fn format_entity_auto_result(result: &EntityAutoDto) -> String {
    format!(
        "entity auto complete\nfiles_processed: {}\ndeclarations_seen: {}\nentities_ensured: {}\nrefs_filled: {}\n",
        result.files_processed,
        result.declarations_seen,
        result.entities_ensured,
        result.refs_filled,
    )
}

pub fn format_entity_show(result: &EntityShowDto) -> String {
    let mut out = render_entity_header(&result.entity);
    append_entity_metadata(&mut out, &result.entity);
    for anchor in &result.anchors {
        out.push_str(&format_anchor_line(anchor));
    }
    append_link_block(&mut out, &result.links);
    out
}

pub fn format_entity_context(result: &EntityContextDto) -> String {
    let mut out = render_entity_header(&result.entity);
    append_entity_metadata(&mut out, &result.entity);
    if !result.anchors.is_empty() {
        out.push('\n');
        for item in &result.anchors {
            out.push_str(&format_anchor_line(&item.anchor));
            if let Some(snippet) = &item.snippet {
                for line in snippet.lines() {
                    out.push_str(&format!("  {}\n", line));
                }
                out.push('\n');
            }
        }
    }
    append_link_block(&mut out, &result.links);
    out
}

pub fn format_entity_context_files_only(result: &EntityContextDto) -> String {
    let mut out = render_entity_header(&result.entity);
    append_entity_metadata(&mut out, &result.entity);
    if !result.anchors.is_empty() {
        out.push('\n');
        for item in &result.anchors {
            out.push_str(&format_anchor_line(&item.anchor));
        }
    }
    append_link_block(&mut out, &result.links);
    out
}

pub fn format_entity_link_result(
    prefix: &str,
    from: &EntityDto,
    to: &EntityDto,
    relation: &str,
) -> String {
    format!(
        "{}\nfrom: {} ({})\nto: {} ({})\nrelation: {}\n",
        prefix, from.id, from.name, to.id, to.name, relation
    )
}

pub fn format_entity_unlink_result(prefix: &str, from: &EntityDto, to: &EntityDto) -> String {
    format!(
        "{}\nfrom: {} ({})\nto: {} ({})\n",
        prefix, from.id, from.name, to.id, to.name
    )
}

pub fn format_entity_list(entities: &[EntityDto]) -> String {
    if entities.is_empty() {
        return String::new();
    }
    entities
        .iter()
        .map(format_entity_list_line)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn format_entity_list_line(entity: &EntityDto) -> String {
    let mut out = format!("{} {}", entity.id, entity.name);
    if let Some(description) = &entity.description {
        out.push_str(&format!(" - \"{}\"", description));
    }
    if let Some(entity_ref) = &entity.r#ref {
        if !entity_ref.is_empty() {
            out.push_str(&format!(" ({})", entity_ref));
        }
    }
    out
}

// #tepignoreafter
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{
        AnchorContextDto, AnchorDto, AnchorSyncDto, EntityAutoDto, EntityContextDto, EntityShowDto,
        LinkDto,
    };

    fn sample_entity() -> EntityDto {
        EntityDto {
            id: 42,
            name: "student".into(),
            r#ref: Some("./docs/student.md".into()),
            description: Some("A learner".into()),
        }
    }

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

    fn outgoing_link() -> LinkDto {
        LinkDto {
            direction: "->".into(),
            entity: EntityDto {
                id: 10,
                name: "subject".into(),
                r#ref: None,
                description: None,
            },
            relation: "student has subjects".into(),
            depth: 1,
        }
    }

    fn incoming_link() -> LinkDto {
        LinkDto {
            direction: "<-".into(),
            entity: EntityDto {
                id: 5,
                name: "teacher".into(),
                r#ref: None,
                description: None,
            },
            relation: "teacher mentors student".into(),
            depth: 1,
        }
    }

    #[test]
    fn formats_created_entity() {
        let rendered = format_entity_created("created", &sample_entity());
        assert!(rendered.contains("created"));
        assert!(rendered.contains("42 (student)"));
        assert!(rendered.contains("description: A learner"));
    }

    #[test]
    fn formats_entity_auto_result() {
        let rendered = format_entity_auto_result(&EntityAutoDto {
            files_processed: 1,
            declarations_seen: 2,
            entities_ensured: 2,
            refs_filled: 1,
        });
        assert!(rendered.contains("entity auto complete"));
        assert!(rendered.contains("declarations_seen: 2"));
        assert!(rendered.contains("refs_filled: 1"));
    }

    #[test]
    fn formats_anchor_sync_result() {
        let rendered = crate::output::anchor_output::format_anchor_sync_result(&AnchorSyncDto {
            files_processed: 2,
            anchors_seen: 3,
            anchors_created: 1,
            anchors_dropped: 1,
            relations_synced: 2,
        });
        assert!(rendered.contains("anchor sync complete"));
        assert!(rendered.contains("files_processed: 2"));
        assert!(rendered.contains("anchors_seen: 3"));
    }

    #[test]
    fn formats_entity_show_with_links() {
        let rendered = format_entity_show(&EntityShowDto {
            entity: sample_entity(),
            anchors: vec![sample_anchor()],
            links: vec![outgoing_link(), incoming_link()],
        });
        assert!(rendered.contains("42 (student)"));
        assert!(rendered.contains("description: A learner"));
        assert!(rendered.contains("anchor:7 student_processor"));
        assert!(rendered.contains("./file.md"));
        assert!(rendered.contains("links:"));
        assert!(rendered.contains("-> 10 (subject)"));
        assert!(rendered.contains("student has subjects"));
        assert!(rendered.contains("<- 5 (teacher)"));
        assert!(rendered.contains("teacher mentors student"));
    }

    #[test]
    fn formats_entity_list_with_one_line_shape() {
        let entities = vec![
            sample_entity(),
            EntityDto {
                id: 99,
                name: "teacher".into(),
                r#ref: None,
                description: Some("An instructor".into()),
            },
        ];
        let rendered = format_entity_list(&entities);
        assert!(rendered.contains("42 student - \"A learner\" (./docs/student.md)"));
        assert!(rendered.contains("99 teacher - \"An instructor\""));
    }

    #[test]
    fn formats_context_links_with_depth() {
        let rendered = format_entity_context(&EntityContextDto {
            entity: sample_entity(),
            anchors: vec![],
            links: vec![LinkDto {
                direction: "->".into(),
                entity: EntityDto {
                    id: 10,
                    name: "subject".into(),
                    r#ref: Some("./docs/subject.md".into()),
                    description: None,
                },
                relation: "student has subjects".into(),
                depth: 2,
            }],
        });
        assert!(rendered.contains("links:"));
        assert!(
            rendered
                .contains("-> 10 (subject) [./docs/subject.md]  student has subjects  [depth:2]")
        );
    }

    #[test]
    fn format_context_files_only_shows_anchors_without_snippets() {
        let rendered = format_entity_context_files_only(&EntityContextDto {
            entity: sample_entity(),
            anchors: vec![AnchorContextDto {
                anchor: sample_anchor(),
                snippet: Some("some snippet".into()),
            }],
            links: vec![],
        });
        assert!(rendered.contains("anchor:7 student_processor"));
        assert!(!rendered.contains("some snippet"));
    }
}

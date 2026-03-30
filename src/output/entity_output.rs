use crate::entity::Entity;
use crate::output::anchor_format::{format_anchor_compact, format_anchor_location};
use crate::output::render::{append_context_link_block, append_entity_metadata, append_files_block, append_show_link_block, render_entity_header};
use crate::service::entity_service::{EntityAutoResult, EntityContextResult, EntityLinkResult, EntityShowResult};

pub fn format_entity_created(prefix: &str, entity: &Entity) -> String {
    let mut out = format!("{prefix}\n");
    out.push_str(&render_entity_header(entity));
    if let Some(description) = &entity.description {
        out.push_str(&format!("description: {}\n", description));
    }
    out
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
    let mut out = render_entity_header(&result.entity);
    append_entity_metadata(&mut out, &result.entity);
    for anchor in &result.anchors {
        out.push_str(&format_anchor_compact(anchor));
    }
    append_show_link_block(&mut out, "outgoing links:", &result.outgoing_links, true);
    append_show_link_block(&mut out, "incoming links:", &result.incoming_links, false);
    out
}

pub fn format_entity_context(result: &EntityContextResult) -> String {
    let mut out = render_entity_header(&result.entity);
    append_entity_metadata(&mut out, &result.entity);
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

    append_files_block(&mut out, &result.files);
    append_context_link_block(&mut out, &result.linked_entities);
    out
}

pub fn format_entity_context_files_only(result: &EntityContextResult) -> String {
    let mut out = render_entity_header(&result.entity);
    append_entity_metadata(&mut out, &result.entity);
    append_files_block(&mut out, &result.files);
    append_context_link_block(&mut out, &result.linked_entities);
    out
}

pub fn format_entity_link_result(prefix: &str, result: &EntityLinkResult) -> String {
    format!(
        "{}\nfrom: {} ({})\nto: {} ({})\nrelation: {}\n",
        prefix,
        result.from.entity_id,
        result.from.name,
        result.to.entity_id,
        result.to.name,
        result.relation
    )
}

pub fn format_entity_unlink_result(prefix: &str, from: &Entity, to: &Entity) -> String {
    format!(
        "{}\nfrom: {} ({})\nto: {} ({})\n",
        prefix,
        from.entity_id,
        from.name,
        to.entity_id,
        to.name
    )
}

pub fn format_entity_list(entities: &[Entity]) -> String {
    if entities.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    for entity in entities {
        out.push_str(&format_entity_list_line(entity));
        out.push('\n');
    }
    out
}

fn format_entity_list_line(entity: &Entity) -> String {
    let mut out = format!("{} {}", entity.entity_id, entity.name);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::Anchor;

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

    fn sample_anchor() -> Anchor {
        Anchor {
            anchor_id: 7,
            version: 1,
            name: Some("student_processor".into()),
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
        assert!(rendered.contains("description: A learner"));
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
    fn formats_entity_show_with_incoming_and_outgoing_links() {
        let rendered = format_entity_show(&EntityShowResult {
            entity: sample_entity(),
            anchors: vec![sample_anchor()],
            outgoing_links: vec![(
                crate::entity::EntityLink {
                    from_entity_id: 42,
                    to_entity_id: 10,
                    relation: "student has subjects".into(),
                    created_at: "1".into(),
                    updated_at: "2".into(),
                },
                Entity {
                    entity_id: 10,
                    name: "subject".into(),
                    r#ref: None,
                    description: None,
                    created_at: "1".into(),
                    updated_at: "2".into(),
                },
            )],
            incoming_links: vec![(
                crate::entity::EntityLink {
                    from_entity_id: 5,
                    to_entity_id: 42,
                    relation: "teacher mentors student".into(),
                    created_at: "1".into(),
                    updated_at: "2".into(),
                },
                Entity {
                    entity_id: 5,
                    name: "teacher".into(),
                    r#ref: None,
                    description: None,
                    created_at: "1".into(),
                    updated_at: "2".into(),
                },
            )],
        });
        assert!(rendered.contains("42 (student)"));
        assert!(rendered.contains("description: A learner"));
        assert!(rendered.contains("7 (student_processor)"));
        assert!(rendered.contains("./file.md"));
        assert!(rendered.contains("student_processor"));
        assert!(rendered.contains("outgoing links:"));
        assert!(rendered.contains("incoming links:"));
        assert!(rendered.contains("student has subjects"));
        assert!(rendered.contains("teacher mentors student"));
    }

    #[test]
    fn formats_entity_list_with_one_line_shape() {
        let rendered = format_entity_list(&[
            sample_entity(),
            Entity {
                entity_id: 99,
                name: "teacher".into(),
                r#ref: None,
                description: Some("An instructor".into()),
                created_at: "1".into(),
                updated_at: "2".into(),
            },
        ]);
        assert!(rendered.contains("42 student - \"A learner\" (./docs/student.md)"));
        assert!(rendered.contains("99 teacher - \"An instructor\""));
    }

    #[test]
    fn formats_context_links_with_edge_notation() {
        let rendered = format_entity_context(&EntityContextResult {
            entity: sample_entity(),
            anchors: vec![],
            files: vec!["./docs/student.md".into()],
            linked_entities: vec![crate::service::entity_service::LinkedEntityContext {
                link: crate::entity::EntityLink {
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
                    description: Some("A course".into()),
                    created_at: "1".into(),
                    updated_at: "2".into(),
                },
                depth: 2,
            }],
        });
        assert!(rendered.contains("linked entities:"));
        assert!(rendered.contains("edge: (42->10)[2] student has subjects"));
        assert!(rendered.contains("./docs/subject.md"));
    }
}

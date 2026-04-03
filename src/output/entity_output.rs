use crate::dto::{EntityAutoDto, EntityContextDto, EntityDto, EntityShowDto};
use crate::output::anchor_format::format_anchor_line;
use crate::output::render::{append_entity_metadata, append_link_block, render_entity_header};

pub fn format_entity_auto_result(result: &EntityAutoDto) -> String {
    let mut out = format!(
        "entity auto complete\nfiles_processed: {}\ndeclarations_seen: {}\nrelations_seen: {}\nentities_ensured: {}\nrefs_filled: {}\ndescriptions_filled: {}\nrelations_synced: {}\n",
        result.files_processed,
        result.declarations_seen,
        result.relations_seen,
        result.entities_ensured,
        result.refs_filled,
        result.descriptions_filled,
        result.relations_synced,
    );
    if !result.warnings.is_empty() {
        out.push_str("warnings:\n");
        for warning in &result.warnings {
            out.push_str(&format!("- {}\n", warning));
        }
    }
    out
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

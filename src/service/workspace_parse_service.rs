use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::anchor::{ParsedAnchor, parse_anchors};
use crate::entity::{ParsedEntityDeclaration, parse_entity_declarations};
use crate::tep_tag::{
    ParsedAnchorTag, ParsedEntityTag, ParsedRelationTag, parse_anchor_tags, parse_entity_tags,
    parse_relation_tags,
};
use crate::utils::workspace_scanner::{WorkspaceFile, collect_workspace_files};

#[derive(Debug, Clone)]
pub struct ParsedWorkspaceFile {
    pub file: WorkspaceFile,
    pub entity_tags: Vec<ParsedEntityTag>,
    pub relation_tags: Vec<ParsedRelationTag>,
    pub anchor_tags: Vec<ParsedAnchorTag>,
    pub parsed_anchors: Vec<ParsedAnchor>,
    pub parsed_declarations: Vec<ParsedEntityDeclaration>,
}

pub fn collect_parsed_workspace_files(
    workspace_root: &PathBuf,
    paths: &[String],
) -> Result<(Vec<ParsedWorkspaceFile>, Vec<String>)> {
    let files = collect_workspace_files(workspace_root, paths)?;
    let mut parsed_files = Vec::new();
    let mut warnings = Vec::new();

    for file in files {
        if let Some(parsed) = parse_workspace_file(file, &mut warnings)? {
            parsed_files.push(parsed);
        }
    }

    Ok((parsed_files, warnings))
}

fn parse_workspace_file(
    file: WorkspaceFile,
    warnings: &mut Vec<String>,
) -> Result<Option<ParsedWorkspaceFile>> {
    let content = match fs::read_to_string(&file.absolute_path) {
        Ok(content) => content,
        Err(err) => {
            warnings.push(format!(
                "skipping unreadable file {}: {}",
                file.absolute_path.display(),
                err
            ));
            return Ok(None);
        }
    };

    Ok(Some(ParsedWorkspaceFile {
        parsed_anchors: parse_anchors(&content),
        parsed_declarations: parse_entity_declarations(&content),
        entity_tags: parse_entity_tags(&content),
        relation_tags: parse_relation_tags(&content),
        anchor_tags: parse_anchor_tags(&content),
        file,
    }))
}

use std::collections::HashMap;

use crate::anchor::{normalize_anchor_name, validate_anchor_name, TEPIGNORE_AFTER_MARKER, TEPIGNORE_MARKER};
use crate::entity::{normalize_name, validate_name};
use crate::utils::parse::{line_contains_marker, parse_scan_limit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMetadata {
    pub fields: HashMap<String, String>,
    pub duplicate_keys: Vec<String>,
    pub unknown_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEntityTag {
    pub name: String,
    pub metadata: ParsedMetadata,
    pub raw: String,
    pub start_offset: usize,
    pub line: i64,
    pub shift: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRelationTag {
    pub from: String,
    pub to: String,
    pub metadata: ParsedMetadata,
    pub raw: String,
    pub start_offset: usize,
    pub line: i64,
    pub shift: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAnchorTag {
    pub anchor_name: String,
    pub entity_refs: Vec<String>,
    pub metadata: ParsedMetadata,
    pub raw: String,
    pub start_offset: usize,
    pub line: i64,
    pub shift: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedTepTag {
    Entity(ParsedEntityTag),
    Relation(ParsedRelationTag),
    Anchor(ParsedAnchorTag),
}

pub fn parse_tags(input: &str) -> Vec<ParsedTepTag> {
    let mut out = Vec::new();
    let mut i = 0usize;
    let scan_limit = parse_scan_limit(input, TEPIGNORE_AFTER_MARKER);

    while i < input.len() && i < scan_limit {
        let rest = &input[i..];
        if rest.starts_with("#!#tep:") {
            if let Some(parsed) = try_parse_tag(input, i) {
                i = next_offset(&parsed);
                out.push(parsed);
                continue;
            }
        }

        if let Some(ch) = rest.chars().next() {
            i += ch.len_utf8();
        } else {
            break;
        }
    }

    out
}

pub fn parse_entity_tags(input: &str) -> Vec<ParsedEntityTag> {
    parse_tags(input)
        .into_iter()
        .filter_map(|tag| match tag {
            ParsedTepTag::Entity(tag) => Some(tag),
            _ => None,
        })
        .collect()
}

pub fn parse_relation_tags(input: &str) -> Vec<ParsedRelationTag> {
    parse_tags(input)
        .into_iter()
        .filter_map(|tag| match tag {
            ParsedTepTag::Relation(tag) => Some(tag),
            _ => None,
        })
        .collect()
}

pub fn parse_anchor_tags(input: &str) -> Vec<ParsedAnchorTag> {
    parse_tags(input)
        .into_iter()
        .filter_map(|tag| match tag {
            ParsedTepTag::Anchor(tag) => Some(tag),
            _ => None,
        })
        .collect()
}

fn next_offset(tag: &ParsedTepTag) -> usize {
    match tag {
        ParsedTepTag::Entity(tag) => tag.start_offset + tag.raw.len(),
        ParsedTepTag::Relation(tag) => tag.start_offset + tag.raw.len(),
        ParsedTepTag::Anchor(tag) => tag.start_offset + tag.raw.len(),
    }
}

fn try_parse_tag(input: &str, start: usize) -> Option<ParsedTepTag> {
    if line_contains_marker(input, start, TEPIGNORE_MARKER) {
        return None;
    }

    let rest = &input[start..];
    if let Some(tag) = try_parse_anchor_tag(rest, start, input) {
        return Some(ParsedTepTag::Anchor(tag));
    }
    if let Some(tag) = try_parse_relation_tag(rest, start, input) {
        return Some(ParsedTepTag::Relation(tag));
    }
    if let Some(tag) = try_parse_entity_tag(rest, start, input) {
        return Some(ParsedTepTag::Entity(tag));
    }
    None
}

fn try_parse_entity_tag(rest: &str, start: usize, input: &str) -> Option<ParsedEntityTag> {
    let body = rest.strip_prefix("#!#tep:(")?;
    let close = body.find(')')?;
    let raw_name = &body[..close];
    validate_name(raw_name).ok()?;
    let name = normalize_name(raw_name);
    let after = &body[close + 1..];
    let (metadata, consumed) = parse_metadata_block(after, &["ref", "description"])?;
    let raw = format!("#!#tep:({}){}", raw_name, &after[..consumed]);
    let (line, shift) = compute_position(input, start);

    Some(ParsedEntityTag {
        name,
        metadata,
        raw,
        start_offset: start,
        line,
        shift,
    })
}

fn try_parse_relation_tag(rest: &str, start: usize, input: &str) -> Option<ParsedRelationTag> {
    let body = rest.strip_prefix("#!#tep:(")?;
    let first_close = body.find(')')?;
    let from_raw = &body[..first_close];
    validate_name(from_raw).ok()?;
    let after_from = body.get(first_close + 1..)?;
    let after_arrow = after_from.strip_prefix("->(")?;
    let second_close = after_arrow.find(')')?;
    let to_raw = &after_arrow[..second_close];
    validate_name(to_raw).ok()?;
    let after = &after_arrow[second_close + 1..];
    let (metadata, consumed) = parse_metadata_block(after, &["description"])?;
    let raw = format!("#!#tep:({})->({}){}", from_raw, to_raw, &after[..consumed]);
    let (line, shift) = compute_position(input, start);

    Some(ParsedRelationTag {
        from: normalize_name(from_raw),
        to: normalize_name(to_raw),
        metadata,
        raw,
        start_offset: start,
        line,
        shift,
    })
}

fn try_parse_anchor_tag(rest: &str, start: usize, input: &str) -> Option<ParsedAnchorTag> {
    let body = rest.strip_prefix("#!#tep:[")?;
    let close_bracket = body.find(']')?;
    let anchor_raw = &body[..close_bracket];
    validate_anchor_name(anchor_raw).ok()?;
    let anchor_name = normalize_anchor_name(anchor_raw);
    let after_name = body.get(close_bracket + 1..)?;
    let refs_body = after_name.strip_prefix('(')?;
    let refs_close = refs_body.find(')')?;
    let refs_raw = &refs_body[..refs_close];
    let entity_refs = parse_entity_refs(refs_raw)?;
    if entity_refs.is_empty() {
        return None;
    }
    let after = &refs_body[refs_close + 1..];
    let (metadata, consumed) = parse_metadata_block(after, &["description"])?;
    let raw = format!("#!#tep:[{}]({}){}", anchor_raw, refs_raw, &after[..consumed]);
    let (line, shift) = compute_position(input, start);

    Some(ParsedAnchorTag {
        anchor_name,
        entity_refs,
        metadata,
        raw,
        start_offset: start,
        line,
        shift,
    })
}

fn parse_entity_refs(input: &str) -> Option<Vec<String>> {
    let mut refs = Vec::new();
    for item in input.split(',') {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            continue;
        }
        validate_name(trimmed).ok()?;
        refs.push(normalize_name(trimmed));
    }
    Some(refs)
}

fn parse_metadata_block(input: &str, known_fields: &[&str]) -> Option<(ParsedMetadata, usize)> {
    let Some(rest) = input.strip_prefix('{') else {
        return Some((ParsedMetadata::default(), 0));
    };

    let close = find_metadata_end(rest)?;
    let inner = &rest[..close];
    let consumed = close + 2;
    Some((parse_metadata_fields(inner, known_fields), consumed))
}

fn find_metadata_end(input: &str) -> Option<usize> {
    let mut escaped = false;
    let mut in_quotes = false;
    for (idx, ch) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_quotes => escaped = true,
            '"' => in_quotes = !in_quotes,
            '}' if !in_quotes => return Some(idx),
            _ => {}
        }
    }
    None
}

fn parse_metadata_fields(input: &str, known_fields: &[&str]) -> ParsedMetadata {
    let mut fields = HashMap::new();
    let mut duplicate_keys = Vec::new();
    let mut unknown_fields = Vec::new();

    for segment in split_metadata_segments(input) {
        if let Some((key, value)) = parse_metadata_segment(&segment) {
            if fields.contains_key(&key) {
                duplicate_keys.push(key.clone());
            }
            if !known_fields.iter().any(|known| *known == key) {
                unknown_fields.push(key.clone());
            }
            fields.insert(key, value);
        }
    }

    ParsedMetadata {
        fields,
        duplicate_keys,
        unknown_fields,
    }
}

fn split_metadata_segments(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    let mut in_quotes = false;

    for ch in input.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_quotes => {
                current.push(ch);
                escaped = true;
            }
            '"' => {
                current.push(ch);
                in_quotes = !in_quotes;
            }
            ',' if !in_quotes => {
                let segment = current.trim();
                if !segment.is_empty() {
                    out.push(segment.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    let segment = current.trim();
    if !segment.is_empty() {
        out.push(segment.to_string());
    }
    out
}

fn parse_metadata_segment(segment: &str) -> Option<(String, String)> {
    let eq = segment.find('=')?;
    let key = segment[..eq].trim();
    if key.is_empty() {
        return None;
    }
    let raw_value = segment[eq + 1..].trim();
    let value = parse_quoted_value(raw_value)?;
    Some((key.to_string(), value))
}

fn parse_quoted_value(input: &str) -> Option<String> {
    let inner = input.strip_prefix('"')?.strip_suffix('"')?;
    Some(inner.replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn compute_position(input: &str, start: usize) -> (i64, i64) {
    let prefix = &input[..start];
    let line = prefix.bytes().filter(|b| *b == b'\n').count() as i64 + 1;
    let last_newline = prefix.rfind('\n').map(|idx| idx + 1).unwrap_or(0);
    let shift = (start - last_newline) as i64;
    (line, shift)
}

impl Default for ParsedMetadata {
    fn default() -> Self {
        Self {
            fields: HashMap::new(),
            duplicate_keys: Vec::new(),
            unknown_fields: Vec::new(),
        }
    }
}

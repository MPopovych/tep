// (#!#1#tep:module.entity)
// [#!#tep:entity.declaration.parser](entity.declaration.parser)
// [#!#1#tep:58](module.entity,entity.declaration.parser)
use crate::utils::parse::{line_contains_marker, parse_scan_limit};

pub const TEPIGNORE_MARKER: &str = "#tepignore";
pub const TEPIGNORE_AFTER_MARKER: &str = "#tepignoreafter";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub entity_id: i64,
    pub name: String,
    pub r#ref: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct NewEntity {
    pub name: String,
    pub r#ref: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateEntity {
    pub name: Option<String>,
    pub r#ref: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityLookup {
    Id(i64),
    Name(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityLink {
    pub from_entity_id: i64,
    pub to_entity_id: i64,
    pub relation: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEntityDeclaration {
    pub raw: String,
    pub version: Option<i64>,
    pub name: String,
    pub start_offset: usize,
    pub line: i64,
    pub shift: i64,
}

pub fn parse_lookup(input: &str) -> EntityLookup {
    if let Ok(id) = input.parse::<i64>() {
        EntityLookup::Id(id)
    } else {
        EntityLookup::Name(input.to_string())
    }
}

pub fn validate_name(name: &str) -> Result<(), &'static str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("entity name cannot be empty");
    }
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err("entity name cannot be purely numeric");
    }
    Ok(())
}

pub fn parse_entity_declarations(input: &str) -> Vec<ParsedEntityDeclaration> {
    let mut out = Vec::new();
    let mut i = 0usize;
    let scan_limit = parse_scan_limit(input, TEPIGNORE_AFTER_MARKER);

    while i < input.len() && i < scan_limit {
        let rest = &input[i..];
        if rest.starts_with("(#!#tep:") || rest.starts_with("(#!#") {
            if let Some(parsed) = try_parse_entity_declaration(input, i) {
                i = parsed.start_offset + parsed.raw.len();
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

fn try_parse_entity_declaration(input: &str, start: usize) -> Option<ParsedEntityDeclaration> {
    let rest = &input[start..];
    let close_idx = rest.find(')')?;
    let raw = &rest[..=close_idx];

    let body = raw.strip_prefix("(#!#")?.strip_suffix(')')?;
    let (version, name) = if let Some(name) = body.strip_prefix("tep:") {
        (None, name)
    } else {
        let (version_str, name) = body.split_once("#tep:")?;
        let version = version_str.parse::<i64>().ok()?;
        (Some(version), name)
    };

    validate_name(name).ok()?;

    let prefix = &input[..start];
    let line = prefix.bytes().filter(|b| *b == b'\n').count() as i64 + 1;
    let last_newline = prefix.rfind('\n').map(|idx| idx + 1).unwrap_or(0);
    let shift = (start - last_newline) as i64;

    if line_contains_marker(input, start, TEPIGNORE_MARKER) {
        return None;
    }

    Some(ParsedEntityDeclaration {
        raw: raw.to_string(),
        version,
        name: name.to_string(),
        start_offset: start,
        line,
        shift,
    })
}

pub fn materialize_entity_declaration(parsed: &ParsedEntityDeclaration, version: i64) -> String {
    format!("(#!#{}#tep:{})", version, parsed.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numeric_lookup_as_id() {
        assert_eq!(parse_lookup("42"), EntityLookup::Id(42));
    }

    #[test]
    fn parses_non_numeric_lookup_as_name() {
        assert_eq!(parse_lookup("student"), EntityLookup::Name("student".into()));
    }

    #[test]
    fn rejects_empty_name() {
        assert!(validate_name("   ").is_err());
    }

    #[test]
    fn rejects_numeric_name() {
        assert!(validate_name("123").is_err());
    }

    #[test]
    fn accepts_dotted_name() {
        assert!(validate_name("student.permissions").is_ok());
    }

    #[test]
    fn parses_incomplete_entity_declaration() {
        let parsed = parse_entity_declarations("abc (#!#tep:Student) xyz");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].version, None);
        assert_eq!(parsed[0].name, "Student");
    }

    #[test]
    fn parses_materialized_entity_declaration() {
        let parsed = parse_entity_declarations("(#!#1#tep:Student)");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].version, Some(1));
        assert_eq!(parsed[0].name, "Student");
    }

    #[test]
    fn materializes_entity_declaration() {
        let parsed = parse_entity_declarations("(#!#tep:Student)");
        let out = materialize_entity_declaration(&parsed[0], 1);
        assert_eq!(out, "(#!#1#tep:Student)");
    }

    #[test]
    fn ignores_numeric_entity_declaration_name() {
        let parsed = parse_entity_declarations("(#!#tep:123)");
        assert!(parsed.is_empty());
    }

    #[test]
    fn ignores_entity_declaration_when_line_contains_tepignore() {
        let parsed = parse_entity_declarations("example (#!#tep:Student) #tepignore");
        assert!(parsed.is_empty());
    }

    #[test]
    fn ignores_entity_declarations_after_tepignoreafter_marker() {
        let parsed = parse_entity_declarations("(#!#tep:Student)\n#tepignoreafter\n(#!#tep:Teacher)");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "Student");
    }
}

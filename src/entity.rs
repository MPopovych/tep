// #!#tep:(entity.declaration)
// #!#tep:[entity.declaration](entity.declaration)
use crate::tep_tag::parse_entity_tags;

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
    pub name: String,
    pub start_offset: usize,
    pub line: i64,
    pub shift: i64,
}

pub fn parse_lookup(input: &str) -> EntityLookup {
    if let Ok(id) = input.parse::<i64>() {
        EntityLookup::Id(id)
    } else {
        EntityLookup::Name(normalize_name(input))
    }
}

pub fn normalize_name(input: &str) -> String {
    input.trim().to_ascii_lowercase()
}

pub fn validate_name(name: &str) -> Result<(), &'static str> {
    let normalized = normalize_name(name);
    if normalized.is_empty() {
        return Err("entity name cannot be empty");
    }
    if normalized.chars().all(|c| c.is_ascii_digit()) {
        return Err("entity name cannot be purely numeric");
    }
    if !normalized
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_')
    {
        return Err(
            "entity name may only contain lowercase letters, numbers, dots, and underscores",
        );
    }
    Ok(())
}

pub fn validate_description(description: &str) -> Result<(), &'static str> {
    if description.contains('"') {
        return Err("entity description cannot contain quotes");
    }
    if description.contains('\n') || description.contains('\r') {
        return Err("entity description cannot contain newlines");
    }
    Ok(())
}

pub fn normalize_description(description: Option<String>) -> Result<Option<String>, &'static str> {
    match description {
        Some(value) => {
            validate_description(&value)?;
            Ok(Some(value.trim().to_string()))
        }
        None => Ok(None),
    }
}

// #!#tep:[entity.declaration.scan](entity.declaration)
pub fn parse_entity_declarations(input: &str) -> Vec<ParsedEntityDeclaration> {
    parse_entity_tags(input)
        .into_iter()
        .map(|tag| ParsedEntityDeclaration {
            raw: tag.raw,
            name: tag.name,
            start_offset: tag.start_offset,
            line: tag.line,
            shift: tag.shift,
        })
        .collect()
}

// #tepignoreafter
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numeric_lookup_as_id() {
        assert_eq!(parse_lookup("42"), EntityLookup::Id(42));
    }

    #[test]
    fn parses_non_numeric_lookup_as_normalized_name() {
        assert_eq!(
            parse_lookup("Student"),
            EntityLookup::Name("student".into())
        );
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
    fn rejects_name_with_dash() {
        assert!(validate_name("student-profile").is_err());
    }

    #[test]
    fn normalizes_mixed_case_name() {
        assert_eq!(normalize_name(" Student.Profile "), "student.profile");
    }

    #[test]
    fn accepts_uppercase_name_input_but_normalizes_it() {
        assert!(validate_name("STUDENT").is_ok());
        assert_eq!(normalize_name("STUDENT"), "student");
    }

    #[test]
    fn rejects_description_with_quotes() {
        assert!(validate_description("A \"learner\"").is_err());
    }

    #[test]
    fn rejects_description_with_newlines() {
        assert!(validate_description("A learner\nwith break").is_err());
    }

    #[test]
    fn accepts_description_with_spaces() {
        assert!(validate_description("A learner in the system").is_ok());
    }

    #[test]
    fn parses_entity_declaration() {
        let parsed = parse_entity_declarations("abc #!#tep:(Student) xyz");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "student");
    }

    #[test]
    fn ignores_entity_declaration_with_version_segment() {
        let parsed = parse_entity_declarations("(#!#1#tep:Student)");
        assert!(parsed.is_empty());
    }

    #[test]
    fn ignores_numeric_entity_declaration_name() {
        let parsed = parse_entity_declarations("#!#tep:(123)");
        assert!(parsed.is_empty());
    }

    #[test]
    fn parse_lookup_preserves_non_numeric_whitespace_input_as_name() {
        assert_eq!(parse_lookup(" 42a "), EntityLookup::Name("42a".into()));
    }

    #[test]
    fn ignores_entity_declaration_when_line_contains_tepignore() {
        let parsed = parse_entity_declarations("example #!#tep:(Student) #tepignore");
        assert!(parsed.is_empty());
    }

    #[test]
    fn ignores_entity_declarations_after_tepignoreafter_marker() {
        let parsed =
            parse_entity_declarations("#!#tep:(Student)\n#tepignoreafter\n#!#tep:(Teacher)");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "student");
    }
}

// (#!#tep:anchor.parser)
// [#!#tep:anchor.parser](anchor.parser)
use crate::tep_tag::parse_anchor_tags;

pub const TEPIGNORE_MARKER: &str = "#tepignore";
pub const TEPIGNORE_AFTER_MARKER: &str = "#tepignoreafter";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    pub anchor_id: i64,
    pub version: i64,
    pub name: Option<String>,
    pub file_path: String,
    pub line: Option<i64>,
    pub shift: Option<i64>,
    pub offset: Option<i64>,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAnchor {
    pub raw: String,
    pub anchor_name: String,
    pub entity_refs: Vec<String>,
    pub start_offset: usize,
    pub line: i64,
    pub shift: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorTarget {
    Id(i64),
    Name(String),
}

pub fn normalize_anchor_name(input: &str) -> String {
    input.trim().to_ascii_lowercase()
}

pub fn validate_anchor_name(name: &str) -> Result<(), &'static str> {
    let normalized = normalize_anchor_name(name);
    if normalized.is_empty() {
        return Err("anchor name cannot be empty");
    }
    if normalized.chars().all(|c| c.is_ascii_digit()) {
        return Err("anchor name cannot be purely numeric");
    }
    if !normalized
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_')
    {
        return Err(
            "anchor name may only contain lowercase letters, numbers, dots, and underscores",
        );
    }
    Ok(())
}

pub fn parse_anchor_target(input: &str) -> AnchorTarget {
    if let Ok(id) = input.parse::<i64>() {
        AnchorTarget::Id(id)
    } else {
        AnchorTarget::Name(normalize_anchor_name(input))
    }
}

// [#!#tep:anchor.parser.scan](anchor.parser)
pub fn parse_anchors(input: &str) -> Vec<ParsedAnchor> {
    parse_anchor_tags(input)
        .into_iter()
        .map(|tag| ParsedAnchor {
            raw: tag.raw,
            anchor_name: tag.anchor_name,
            entity_refs: tag.entity_refs,
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
    fn parses_named_anchor_with_entity_refs() {
        let parsed = parse_anchors("[#!#tep:foo](student,basic_user)");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].anchor_name, "foo");
        assert_eq!(parsed[0].entity_refs, vec!["student", "basic_user"]);
    }

    #[test]
    fn ignores_named_anchor_without_entity_refs() {
        let parsed = parse_anchors("[#!#tep:foo]");
        assert!(parsed.is_empty());
    }

    #[test]
    fn ignores_incomplete_anchor_no_name() {
        let parsed = parse_anchors("[#!#tep:](student)");
        assert!(parsed.is_empty());
    }

    #[test]
    fn ignores_anchor_with_version_segment() {
        let parsed = parse_anchors("[#!#1#tep:student_processor](student)");
        assert!(parsed.is_empty());
    }

    #[test]
    fn ignores_anchor_with_numeric_name() {
        let parsed = parse_anchors("[#!#tep:123](student)");
        assert!(parsed.is_empty());
    }

    #[test]
    fn parses_named_anchor_normalizes_case() {
        let parsed = parse_anchors("[#!#tep:Student_Processor](student)");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].anchor_name, "student_processor");
    }

    #[test]
    fn ignores_named_anchor_with_invalid_charset() {
        let parsed = parse_anchors("[#!#tep:student-processor]");
        assert!(parsed.is_empty());
    }

    #[test]
    fn computes_line_shift_and_offset() {
        let input = "hello\n  [#!#tep:foo](student)";
        let parsed = parse_anchors(input);
        assert_eq!(parsed[0].line, 2);
        assert_eq!(parsed[0].shift, 2);
        assert_eq!(parsed[0].start_offset, 8);
    }

    #[test]
    fn parses_anchor_after_unicode_text_without_panicking() {
        let input = "żółw 🐢\n[#!#tep:foo](student)";
        let parsed = parse_anchors(input);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].anchor_name, "foo");
    }

    #[test]
    fn computes_byte_offsets_after_unicode_prefix() {
        let input = "żółw [#!#tep:foo](student)";
        let parsed = parse_anchors(input);
        assert_eq!(parsed.len(), 1);
        let expected = input.find("[#!#tep:foo]").expect("anchor should exist");
        assert_eq!(parsed[0].start_offset, expected);
        assert_eq!(parsed[0].shift as usize, expected);
    }

    #[test]
    fn ignores_anchor_with_unclosed_entity_instruction() {
        let parsed = parse_anchors("[#!#tep:foo](student");
        assert!(parsed.is_empty());
    }

    #[test]
    fn ignores_random_bracket_noise() {
        let parsed = parse_anchors("[#!#not-an-anchor]");
        assert!(parsed.is_empty());
    }

    #[test]
    fn ignores_anchor_when_line_contains_tepignore() {
        let parsed = parse_anchors("example [#!#tep:foo](student) #tepignore");
        assert!(parsed.is_empty());
    }

    #[test]
    fn ignores_anchors_after_tepignoreafter_marker() {
        let parsed = parse_anchors("[#!#tep:foo](student)\n#tepignoreafter\n[#!#tep:bar](teacher)");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].anchor_name, "foo");
    }

    #[test]
    fn parse_anchor_target_uses_id_for_numeric_input() {
        assert_eq!(parse_anchor_target("42"), AnchorTarget::Id(42));
    }

    #[test]
    fn parse_anchor_target_uses_name_for_non_numeric_input() {
        assert_eq!(
            parse_anchor_target("student_processor"),
            AnchorTarget::Name("student_processor".into())
        );
    }

    #[test]
    fn parse_anchor_target_normalizes_name() {
        assert_eq!(
            parse_anchor_target("Student_Processor"),
            AnchorTarget::Name("student_processor".into())
        );
    }

    #[test]
    fn parse_anchor_target_trims_whitespace_as_name() {
        assert_eq!(
            parse_anchor_target(" 42a "),
            AnchorTarget::Name("42a".into())
        );
    }

    #[test]
    fn validate_anchor_name_rejects_empty() {
        assert!(validate_anchor_name("").is_err());
    }

    #[test]
    fn validate_anchor_name_rejects_purely_numeric() {
        assert!(validate_anchor_name("123").is_err());
    }

    #[test]
    fn validate_anchor_name_rejects_dash() {
        assert!(validate_anchor_name("student-processor").is_err());
    }

    #[test]
    fn validate_anchor_name_accepts_underscore_and_dot() {
        assert!(validate_anchor_name("student.processor_v2").is_ok());
    }
}

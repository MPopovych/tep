// (#!#1#tep:module.anchor)
// [#!#tep:anchor.parser](anchor.parser)
// [#!#1#tep:57](module.anchor,anchor.parser)
use crate::utils::parse::{line_contains_marker, parse_scan_limit};

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
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAnchor {
    pub raw: String,
    pub version: Option<i64>,
    pub anchor_id: Option<i64>,
    pub anchor_name: Option<String>,
    pub entity_refs: Vec<String>,
    pub start_offset: usize,
    pub line: i64,
    pub shift: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorKind {
    Incomplete,
    Materialized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorTarget {
    Id(i64),
    Name(String),
}

impl ParsedAnchor {
    pub fn kind(&self) -> AnchorKind {
        if self.version.is_some() && (self.anchor_id.is_some() || self.anchor_name.is_some()) {
            AnchorKind::Materialized
        } else {
            AnchorKind::Incomplete
        }
    }
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
        return Err("anchor name may only contain lowercase letters, numbers, dots, and underscores");
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

pub fn parse_anchors(input: &str) -> Vec<ParsedAnchor> {
    let mut out = Vec::new();
    let mut i = 0usize;
    let scan_limit = parse_scan_limit(input, TEPIGNORE_AFTER_MARKER);

    while i < input.len() && i < scan_limit {
        let rest = &input[i..];
        if rest.starts_with("[#!#tep:]") || rest.starts_with("[#!#") {
            if let Some(parsed) = try_parse_anchor(input, i) {
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

fn try_parse_anchor(input: &str, start: usize) -> Option<ParsedAnchor> {
    let rest = &input[start..];
    let close_idx = rest.find(']')?;
    let head = &rest[..=close_idx];
    let after_head = &rest[close_idx + 1..];

    let entity_refs = if after_head.starts_with('(') {
        let close_paren = after_head.find(')')?;
        let inside = &after_head[1..close_paren];
        inside
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let suffix_len = if after_head.starts_with('(') {
        after_head.find(')').map(|idx| idx + 1).unwrap_or(0)
    } else {
        0
    };

    let raw = format!("{}{}", head, &after_head[..suffix_len]);
    let (version, anchor_id, anchor_name) = parse_anchor_head(head)?;

    let prefix = &input[..start];
    let line = prefix.bytes().filter(|b| *b == b'\n').count() as i64 + 1;
    let last_newline = prefix.rfind('\n').map(|idx| idx + 1).unwrap_or(0);
    let shift = (start - last_newline) as i64;

    if line_contains_marker(input, start, TEPIGNORE_MARKER) {
        return None;
    }

    Some(ParsedAnchor {
        raw,
        version,
        anchor_id,
        anchor_name,
        entity_refs,
        start_offset: start,
        line,
        shift,
    })
}

/// Returns (version, anchor_id, anchor_name).
/// For incomplete tags: all None.
/// For materialized numeric: (Some(v), Some(id), None).
/// For materialized named: (Some(v), None, Some(name)).
fn parse_anchor_head(head: &str) -> Option<(Option<i64>, Option<i64>, Option<String>)> {
    if head == "[#!#tep:]" {
        return Some((None, None, None));
    }
    parse_materialized_head(head)
}

fn parse_materialized_head(head: &str) -> Option<(Option<i64>, Option<i64>, Option<String>)> {
    let body = head.strip_prefix("[#!#")?.strip_suffix(']')?;
    let (version_str, rest) = body.split_once("#tep:")?;
    let version = version_str.parse::<i64>().ok()?;
    if rest.is_empty() {
        // [#!#1#tep:] — treat as incomplete with a version hint; reject
        return None;
    }
    if let Ok(id) = rest.parse::<i64>() {
        return Some((Some(version), Some(id), None));
    }
    // non-numeric: a name hint or materialized name
    let name = normalize_anchor_name(rest);
    validate_anchor_name(&name).ok()?;
    Some((Some(version), None, Some(name)))
}

/// Materialize an incomplete anchor using a numeric id.
pub fn materialize_anchor(parsed: &ParsedAnchor, new_anchor_id: i64, version: i64) -> String {
    if parsed.entity_refs.is_empty() {
        format!("[#!#{}#tep:{}]", version, new_anchor_id)
    } else {
        format!(
            "[#!#{}#tep:{}]({})",
            version,
            new_anchor_id,
            parsed.entity_refs.join(",")
        )
    }
}

/// Materialize an incomplete anchor using a name.
pub fn materialize_anchor_named(parsed: &ParsedAnchor, name: &str, version: i64) -> String {
    if parsed.entity_refs.is_empty() {
        format!("[#!#{}#tep:{}]", version, name)
    } else {
        format!(
            "[#!#{}#tep:{}]({})",
            version,
            name,
            parsed.entity_refs.join(",")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_incomplete_anchor_without_entity_refs() {
        let parsed = parse_anchors("abc [#!#tep:] xyz");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].kind(), AnchorKind::Incomplete);
        assert!(parsed[0].entity_refs.is_empty());
        assert_eq!(parsed[0].anchor_name, None);
    }

    #[test]
    fn parses_incomplete_anchor_with_multiple_entity_refs() {
        let parsed = parse_anchors("[#!#tep:](student,basic-user)");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].entity_refs, vec!["student", "basic-user"]);
    }

    #[test]
    fn parses_materialized_anchor() {
        let parsed = parse_anchors("[#!#1#tep:123](student)");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].kind(), AnchorKind::Materialized);
        assert_eq!(parsed[0].version, Some(1));
        assert_eq!(parsed[0].anchor_id, Some(123));
        assert_eq!(parsed[0].anchor_name, None);
    }

    #[test]
    fn parses_named_anchor() {
        let parsed = parse_anchors("[#!#1#tep:student_processor](student)");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].kind(), AnchorKind::Materialized);
        assert_eq!(parsed[0].anchor_id, None);
        assert_eq!(parsed[0].anchor_name, Some("student_processor".into()));
        assert_eq!(parsed[0].entity_refs, vec!["student"]);
    }

    #[test]
    fn parses_named_anchor_normalizes_case() {
        let parsed = parse_anchors("[#!#1#tep:Student_Processor]");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].anchor_name, Some("student_processor".into()));
    }

    #[test]
    fn ignores_named_anchor_with_invalid_charset() {
        let parsed = parse_anchors("[#!#1#tep:student-processor]");
        assert!(parsed.is_empty());
    }

    #[test]
    fn parses_short_alpha_name_as_named_anchor() {
        let parsed = parse_anchors("[#!#1#tep:abc](student)");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].anchor_name, Some("abc".into()));
    }

    #[test]
    fn computes_line_shift_and_offset() {
        let input = "hello\n  [#!#tep:](student)";
        let parsed = parse_anchors(input);
        assert_eq!(parsed[0].line, 2);
        assert_eq!(parsed[0].shift, 2);
        assert_eq!(parsed[0].start_offset, 8);
    }

    #[test]
    fn materializes_anchor() {
        let parsed = parse_anchors("[#!#tep:](student,basic-user)");
        let out = materialize_anchor(&parsed[0], 42, 1);
        assert_eq!(out, "[#!#1#tep:42](student,basic-user)");
    }

    #[test]
    fn materializes_anchor_with_name() {
        let parsed = parse_anchors("[#!#tep:](student)");
        let out = materialize_anchor_named(&parsed[0], "student_processor", 1);
        assert_eq!(out, "[#!#1#tep:student_processor](student)");
    }

    #[test]
    fn parses_anchor_after_unicode_text_without_panicking() {
        let input = "żółw 🐢\n[#!#tep:](student)";
        let parsed = parse_anchors(input);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].kind(), AnchorKind::Incomplete);
    }

    #[test]
    fn computes_byte_offsets_after_unicode_prefix() {
        let input = "żółw [#!#tep:](student)";
        let parsed = parse_anchors(input);
        assert_eq!(parsed.len(), 1);
        let expected = input.find("[#!#tep:]").expect("anchor should exist");
        assert_eq!(parsed[0].start_offset, expected);
        assert_eq!(parsed[0].shift as usize, expected);
    }

    #[test]
    fn ignores_anchor_with_unclosed_entity_instruction() {
        let parsed = parse_anchors("[#!#tep:](student");
        assert!(parsed.is_empty());
    }

    #[test]
    fn ignores_random_bracket_noise() {
        let parsed = parse_anchors("[#!#not-an-anchor]");
        assert!(parsed.is_empty());
    }

    #[test]
    fn ignores_anchor_when_line_contains_tepignore() {
        let parsed = parse_anchors("example [#!#tep:](student) #tepignore");
        assert!(parsed.is_empty());
    }

    #[test]
    fn ignores_anchors_after_tepignoreafter_marker() {
        let parsed = parse_anchors("[#!#tep:](student)\n#tepignoreafter\n[#!#tep:](teacher)");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].entity_refs, vec!["student"]);
    }

    #[test]
    fn parse_anchor_target_uses_id_for_numeric_input() {
        assert_eq!(parse_anchor_target("42"), AnchorTarget::Id(42));
    }

    #[test]
    fn parse_anchor_target_uses_name_for_non_numeric_input() {
        assert_eq!(parse_anchor_target("student_processor"), AnchorTarget::Name("student_processor".into()));
    }

    #[test]
    fn parse_anchor_target_normalizes_name() {
        assert_eq!(parse_anchor_target("Student_Processor"), AnchorTarget::Name("student_processor".into()));
    }

    #[test]
    fn parse_anchor_target_trims_whitespace_as_name() {
        assert_eq!(parse_anchor_target(" 42a "), AnchorTarget::Name("42a".into()));
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

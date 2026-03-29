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

impl ParsedAnchor {
    pub fn kind(&self) -> AnchorKind {
        if self.anchor_id.is_some() && self.version.is_some() {
            AnchorKind::Materialized
        } else {
            AnchorKind::Incomplete
        }
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
    let (version, anchor_id) = parse_anchor_head(head)?;

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
        entity_refs,
        start_offset: start,
        line,
        shift,
    })
}

fn parse_anchor_head(head: &str) -> Option<(Option<i64>, Option<i64>)> {
    if head == "[#!#tep:]" {
        return Some((None, None));
    }
    parse_materialized_head(head)
}

fn parse_materialized_head(head: &str) -> Option<(Option<i64>, Option<i64>)> {
    let body = head.strip_prefix("[#!#")?.strip_suffix(']')?;
    let (version_str, rest) = body.split_once("#tep:")?;
    let version = version_str.parse::<i64>().ok()?;
    let anchor_id = rest.parse::<i64>().ok()?;
    Some((Some(version), Some(anchor_id)))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_incomplete_anchor_without_entity_refs() {
        let parsed = parse_anchors("abc [#!#tep:] xyz");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].kind(), AnchorKind::Incomplete);
        assert!(parsed[0].entity_refs.is_empty());
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
    fn ignores_malformed_materialized_anchor() {
        let parsed = parse_anchors("[#!#1#tep:abc](student)");
        assert!(parsed.is_empty());
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
}

// #!#tep:(entity.context){description="Snippet extraction around anchors for entity context output"}
// #!#tep:(entity.context)->(anchor.sync){description="consumes anchor locations produced by"}
// #!#tep:[entity.context](entity.context,anchor.sync){description="Entity context helper module entry"}
use std::fs;

use anyhow::{Context, Result};

use crate::anchor::Anchor;

pub const SNIPPET_LINES_BEFORE: usize = 3;
pub const SNIPPET_LINES_AFTER: usize = 6;
/// Safety cap: max bytes per line before we consider a line "degenerate" and bail
const MAX_BYTES_PER_LINE: usize = 512;

pub fn extract_anchor_snippet(anchor: &Anchor) -> Result<Option<String>> {
    let anchor_line = match anchor.line {
        Some(line) if line >= 1 => line as usize,
        _ => return Ok(None),
    };

    let text = fs::read_to_string(&anchor.file_path)
        .with_context(|| format!("failed to read {}", anchor.file_path))?;

    // Safety guard: max bytes we're willing to scan
    let max_scan = MAX_BYTES_PER_LINE * (SNIPPET_LINES_BEFORE + SNIPPET_LINES_AFTER + 1);
    let lines: Vec<&str> = text
        .lines()
        .take(anchor_line + SNIPPET_LINES_AFTER)
        .collect();

    if anchor_line > lines.len() {
        return Ok(None);
    }

    let first = anchor_line.saturating_sub(SNIPPET_LINES_BEFORE + 1);
    let last = (anchor_line + SNIPPET_LINES_AFTER).min(lines.len());

    let snippet_lines = &lines[first..last];

    // Apply byte budget
    let mut total_bytes = 0usize;
    let mut safe_end = 0usize;
    for (i, line) in snippet_lines.iter().enumerate() {
        total_bytes += line.len() + 1; // +1 for newline
        if total_bytes > max_scan {
            break;
        }
        safe_end = i + 1;
    }

    let snippet = snippet_lines[..safe_end].join("\n");
    if snippet.is_empty() {
        return Ok(None);
    }

    Ok(Some(snippet))
}

// #tepignoreafter
#[cfg(test)]
mod tests {
    use super::*;

    fn anchor(file_path: String, line: i64) -> Anchor {
        Anchor {
            anchor_id: 1,
            version: 1,
            name: None,
            file_path,
            line: Some(line),
            shift: Some(0),
            offset: Some(0),
            description: None,
            created_at: "1".into(),
            updated_at: "1".into(),
        }
    }

    #[test]
    fn snippet_respects_file_start_boundary() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("start.txt");
        std::fs::write(&file, "anchor at start\nrest\n").unwrap();
        let snippet = extract_anchor_snippet(&anchor(file.to_string_lossy().into(), 1))
            .unwrap()
            .unwrap();
        assert!(snippet.contains("anchor at start"));
    }

    #[test]
    fn snippet_respects_file_end_boundary() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("end.txt");
        std::fs::write(&file, "before\nanchor at end").unwrap();
        let snippet = extract_anchor_snippet(&anchor(file.to_string_lossy().into(), 2))
            .unwrap()
            .unwrap();
        assert!(snippet.contains("anchor at end"));
    }

    #[test]
    fn snippet_snaps_to_line_boundaries() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("lines.txt");
        std::fs::write(&file, "one\ntwo anchor target\nthree\n").unwrap();
        let snippet = extract_anchor_snippet(&anchor(file.to_string_lossy().into(), 2))
            .unwrap()
            .unwrap();
        assert!(snippet.contains("two anchor target"));
        // should not start mid-word
        assert!(snippet.starts_with("one") || snippet.starts_with("two"));
    }
}

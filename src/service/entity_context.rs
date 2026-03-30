use std::fs;

use anyhow::{Context, Result};

use crate::anchor::Anchor;

const CONTEXT_WINDOW_BYTES: usize = 120;

pub fn extract_anchor_snippet(anchor: &Anchor) -> Result<Option<String>> {
    let text = fs::read_to_string(&anchor.file_path)
        .with_context(|| format!("failed to read {}", anchor.file_path))?;
    let offset = match anchor.offset {
        Some(value) if value >= 0 => value as usize,
        _ => return Ok(None),
    };
    if offset >= text.len() {
        return Ok(None);
    }

    let raw_start = offset.saturating_sub(CONTEXT_WINDOW_BYTES);
    let raw_end = (offset + CONTEXT_WINDOW_BYTES).min(text.len());

    let start = snap_start_to_line_boundary(&text, raw_start);
    let end = snap_end_to_line_boundary(&text, raw_end);
    if start >= end {
        return Ok(None);
    }

    let snippet = text[start..end].trim();
    if snippet.is_empty() {
        return Ok(None);
    }

    Ok(Some(snippet.to_string()))
}

fn floor_char_boundary(text: &str, mut idx: usize) -> usize {
    idx = idx.min(text.len());
    while idx > 0 && !text.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

fn ceil_char_boundary(text: &str, mut idx: usize) -> usize {
    idx = idx.min(text.len());
    while idx < text.len() && !text.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}

fn snap_start_to_line_boundary(text: &str, raw_start: usize) -> usize {
    let start = floor_char_boundary(text, raw_start);
    match text[..start].rfind('\n') {
        Some(idx) => idx + 1,
        None => 0,
    }
}

fn snap_end_to_line_boundary(text: &str, raw_end: usize) -> usize {
    let end = ceil_char_boundary(text, raw_end);
    match text[end..].find('\n') {
        Some(idx) => end + idx,
        None => text.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchor(file_path: String, offset: i64) -> Anchor {
        Anchor {
            anchor_id: 1,
            version: 1,
            name: None,
            file_path,
            line: Some(1),
            shift: Some(0),
            offset: Some(offset),
            created_at: "1".into(),
            updated_at: "1".into(),
        }
    }

    #[test]
    fn snippet_respects_file_start_boundary() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("start.txt");
        std::fs::write(&file, "anchor at start\nrest\n").unwrap();
        let snippet = extract_anchor_snippet(&anchor(file.to_string_lossy().into(), 0)).unwrap().unwrap();
        assert!(snippet.starts_with("anchor at start"));
        assert!(!snippet.starts_with('\n'));
    }

    #[test]
    fn snippet_respects_file_end_boundary() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("end.txt");
        std::fs::write(&file, "before\nanchor at end").unwrap();
        let offset = "before\n".len() as i64;
        let snippet = extract_anchor_snippet(&anchor(file.to_string_lossy().into(), offset)).unwrap().unwrap();
        assert!(snippet.ends_with("anchor at end"));
        assert!(!snippet.ends_with('\n'));
    }

    #[test]
    fn snippet_snaps_to_line_boundaries() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("lines.txt");
        std::fs::write(&file, "one\ntwo anchor target\nthree\n").unwrap();
        let offset = ("one\n".len() + 4) as i64;
        let snippet = extract_anchor_snippet(&anchor(file.to_string_lossy().into(), offset)).unwrap().unwrap();
        assert!(snippet.starts_with("one\n") || snippet.starts_with("two anchor target"));
        assert!(snippet.ends_with("three") || snippet.ends_with("two anchor target"));
        assert!(!snippet.starts_with("ne"));
        assert!(!snippet.ends_with("thr"));
    }
}

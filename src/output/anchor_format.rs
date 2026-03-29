use crate::anchor::Anchor;
use crate::output::styles::{ANSI_CYAN, ANSI_GREEN, ANSI_MAGENTA, ANSI_YELLOW, paint};

pub fn format_anchor_location(anchor: &Anchor) -> String {
    let file = paint(ANSI_CYAN, &anchor.file_path);
    let line = paint(ANSI_GREEN, anchor.line.unwrap_or(0).to_string());
    let shift = paint(ANSI_MAGENTA, anchor.shift.unwrap_or(0).to_string());
    let offset = paint(ANSI_YELLOW, anchor.offset.unwrap_or(0).to_string());
    format!("{} ({}:{}) [{}]\n", file, line, shift, offset)
}

pub fn format_anchor_compact(anchor: &Anchor) -> String {
    format!("{}\n{}", anchor.anchor_id, format_anchor_location(anchor))
}

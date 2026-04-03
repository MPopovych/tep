use crate::dto::AnchorDto;
use crate::output::styles::{ANSI_CYAN, ANSI_GREEN, ANSI_MAGENTA, ANSI_YELLOW, paint};

/// Single-line anchor format: `anchor:7 student_processor ./file.md (3:4) [22]`
pub fn format_anchor_line(anchor: &AnchorDto) -> String {
    let file = paint(ANSI_CYAN, &anchor.file);
    let line = paint(ANSI_GREEN, anchor.line.unwrap_or(0).to_string());
    let shift = paint(ANSI_MAGENTA, anchor.shift.unwrap_or(0).to_string());
    let offset = paint(ANSI_YELLOW, anchor.offset.unwrap_or(0).to_string());
    format!(
        "anchor:{} {} {} ({}:{}) [{}]\n",
        anchor.id, anchor.name, file, line, shift, offset
    )
}

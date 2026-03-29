use crate::anchor::Anchor;
use crate::output::styles::{ANSI_CYAN, ANSI_GREEN, ANSI_MAGENTA, ANSI_RESET, ANSI_YELLOW};

pub fn format_anchor_location(anchor: &Anchor) -> String {
    format!(
        "{}{}{} ({}{}{}:{}{}{} ) {}[{}]{}\n",
        ANSI_CYAN,
        anchor.file_path,
        ANSI_RESET,
        ANSI_GREEN,
        anchor.line.unwrap_or(0),
        ANSI_RESET,
        ANSI_MAGENTA,
        anchor.shift.unwrap_or(0),
        ANSI_RESET,
        ANSI_YELLOW,
        anchor.offset.unwrap_or(0),
        ANSI_RESET
    )
}

pub fn format_anchor_compact(anchor: &Anchor) -> String {
    format!("{}\n{}", anchor.anchor_id, format_anchor_location(anchor))
}

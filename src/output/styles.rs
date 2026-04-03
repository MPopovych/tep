pub const ANSI_RESET: &str = "\x1b[0m";
pub const ANSI_CYAN: &str = "\x1b[36m";
pub const ANSI_GREEN: &str = "\x1b[32m";
pub const ANSI_YELLOW: &str = "\x1b[33m";
pub const ANSI_MAGENTA: &str = "\x1b[35m";

pub fn paint(color: &str, text: impl AsRef<str>) -> String {
    if use_color() {
        format!("{}{}{}", color, text.as_ref(), ANSI_RESET)
    } else {
        text.as_ref().to_string()
    }
}

fn use_color() -> bool {
    std::env::var("NO_COLOR").is_err() && std::io::IsTerminal::is_terminal(&std::io::stdout())
}

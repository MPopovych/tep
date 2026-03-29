use std::ops::Range;

pub fn parse_scan_limit(input: &str, ignore_after_marker: &str) -> usize {
    input.find(ignore_after_marker).unwrap_or(input.len())
}

pub fn line_range_at(input: &str, offset: usize) -> Range<usize> {
    let line_start = input[..offset].rfind('\n').map(|idx| idx + 1).unwrap_or(0);
    let line_end = input[offset..]
        .find('\n')
        .map(|idx| offset + idx)
        .unwrap_or(input.len());
    line_start..line_end
}

pub fn line_contains_marker(input: &str, offset: usize, marker: &str) -> bool {
    let range = line_range_at(input, offset);
    input[range].contains(marker)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_scan_limit_stops_at_first_ignoreafter() {
        let input = "a\n#tepignoreafter\nb\n#tepignoreafter\nc";
        assert_eq!(parse_scan_limit(input, "#tepignoreafter"), 2);
    }

    #[test]
    fn parse_scan_limit_defaults_to_full_len() {
        let input = "a\nb\nc";
        assert_eq!(parse_scan_limit(input, "#tepignoreafter"), input.len());
    }

    #[test]
    fn line_range_at_finds_current_line() {
        let input = "one\ntwo marker\nthree";
        let offset = input.find("marker").unwrap();
        let range = line_range_at(input, offset);
        assert_eq!(&input[range], "two marker");
    }

    #[test]
    fn line_contains_marker_checks_only_current_line() {
        let input = "one #tepignore\ntwo anchor\nthree #tepignore";
        let offset = input.find("anchor").unwrap();
        assert!(!line_contains_marker(input, offset, "#tepignore"));
    }
}

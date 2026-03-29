pub fn now_utc() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_secs();

    secs.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_utc_returns_numeric_unix_seconds_string() {
        let value = now_utc();
        assert!(!value.is_empty());
        assert!(value.chars().all(|c| c.is_ascii_digit()));
    }
}

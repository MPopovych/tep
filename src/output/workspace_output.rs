use crate::service::workspace_service::InitResult;

pub fn format_init_result(result: &InitResult) -> String {
    format!(
        "Initialized empty tep workspace in {}\nDatabase: {}\nIgnore file: {}\n",
        result.tep_dir, result.db_file, result.ignore_file
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_init_result() {
        let rendered = format_init_result(&InitResult {
            tep_dir: ".tep".into(),
            db_file: ".tep/tep.db".into(),
            ignore_file: ".tep_ignore".into(),
        });

        assert!(rendered.contains("Initialized empty tep workspace in .tep"));
        assert!(rendered.contains("Database: .tep/tep.db"));
        assert!(rendered.contains("Ignore file: .tep_ignore"));
    }
}

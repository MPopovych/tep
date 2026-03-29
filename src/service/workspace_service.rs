use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::db::{self, DEFAULT_DB_FILE, DEFAULT_IGNORE_FILE, DEFAULT_TEP_DIR};

#[derive(Debug, Clone)]
pub struct InitResult {
    pub tep_dir: String,
    pub db_file: String,
    pub ignore_file: String,
}

pub struct WorkspaceService;

impl WorkspaceService {
    pub fn init() -> Result<InitResult> {
        let tep_dir = Path::new(DEFAULT_TEP_DIR);
        fs::create_dir_all(tep_dir).with_context(|| format!("failed to create {}", DEFAULT_TEP_DIR))?;

        let ignore_path = Path::new(DEFAULT_IGNORE_FILE);
        if !ignore_path.exists() {
            fs::write(ignore_path, default_ignore_contents())
                .with_context(|| format!("failed to create {}", DEFAULT_IGNORE_FILE))?;
        }

        let conn = db::open_workspace_db()?;
        conn.execute_batch(db::schema_sql())
            .context("failed to apply database schema")?;

        Ok(InitResult {
            tep_dir: DEFAULT_TEP_DIR.into(),
            db_file: DEFAULT_DB_FILE.into(),
            ignore_file: DEFAULT_IGNORE_FILE.into(),
        })
    }
}

fn default_ignore_contents() -> &'static str {
    "# tep scan ignore rules\n.tep/\n.git/\ntarget/\nnode_modules/\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn init_creates_workspace_files_in_current_directory() {
        let previous = env::temp_dir();
        let temp = tempfile::tempdir().expect("temp dir should be created");
        env::set_current_dir(temp.path()).expect("should change current dir");

        let result = WorkspaceService::init().expect("init should succeed");

        assert_eq!(result.tep_dir, ".tep");
        assert!(temp.path().join(".tep").exists());
        assert!(temp.path().join(".tep/tep.db").exists());
        assert!(temp.path().join(".tep_ignore").exists());

        env::set_current_dir(previous).expect("should restore current dir");
    }
}

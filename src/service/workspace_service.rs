use std::fs;

use anyhow::{Context, Result};

use crate::db::{self, CURRENT_SCHEMA_VERSION, DEFAULT_DB_FILE, DEFAULT_IGNORE_FILE, DEFAULT_TEP_DIR};

#[derive(Debug, Clone)]
pub struct InitResult {
    pub tep_dir: String,
    pub db_file: String,
    pub ignore_file: String,
    pub schema_version: i64,
}

pub struct WorkspaceService;

impl WorkspaceService {
    pub fn init() -> Result<InitResult> {
        let cwd = std::env::current_dir().context("failed to determine current directory")?;
        let paths = db::workspace_paths_for(&cwd);

        fs::create_dir_all(&paths.tep_dir)
            .with_context(|| format!("failed to create {}", paths.tep_dir.display()))?;

        if !paths.ignore_file.exists() {
            fs::write(&paths.ignore_file, default_ignore_contents())
                .with_context(|| format!("failed to create {}", paths.ignore_file.display()))?;
        }

        let conn = db::open_workspace_db_in(&cwd)?;
        db::ensure_schema(&conn)
            .context("failed to apply database schema")?;

        Ok(InitResult {
            tep_dir: DEFAULT_TEP_DIR.into(),
            db_file: DEFAULT_DB_FILE.into(),
            ignore_file: DEFAULT_IGNORE_FILE.into(),
            schema_version: CURRENT_SCHEMA_VERSION,
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
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let previous = env::current_dir().unwrap_or_else(|_| temp.path().to_path_buf());
        env::set_current_dir(temp.path()).expect("should change current dir");

        let result = WorkspaceService::init().expect("init should succeed");

        assert_eq!(result.tep_dir, ".tep");
        assert_eq!(result.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(temp.path().join(".tep").exists());
        assert!(temp.path().join(".tep/tep.db").exists());
        assert!(temp.path().join(".tep_ignore").exists());

        env::set_current_dir(previous).expect("should restore current dir");
    }
}

use std::path::{Path, PathBuf};

use anyhow::Result;
use ignore::WalkBuilder;

pub struct TepIgnoreFilter {
    workspace_root: PathBuf,
}

impl TepIgnoreFilter {
    pub fn for_workspace_root(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }

    pub fn collect_paths(&self, inputs: &[String]) -> Result<Vec<PathBuf>> {
        let mut out = Vec::new();

        for input in inputs {
            let path = self.resolve_input(input);
            if path.is_file() {
                if self.is_special_internal_file(&path) {
                    continue;
                }
                out.push(path);
                continue;
            }

            if path.is_dir() {
                let mut builder = WalkBuilder::new(&path);
                builder.hidden(false);
                builder.git_ignore(false);
                builder.git_exclude(false);
                builder.git_global(false);
                builder.ignore(false);
                builder.add_custom_ignore_filename(".tep_ignore");

                for entry in builder.build() {
                    let entry = entry?;
                    let entry_path = entry.path();
                    if entry_path.is_file() && !self.is_special_internal_file(entry_path) {
                        out.push(entry_path.to_path_buf());
                    }
                }
            }
        }

        out.sort();
        out.dedup();
        Ok(out)
    }

    fn resolve_input(&self, input: &str) -> PathBuf {
        let path = Path::new(input);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.workspace_root.join(path)
        }
    }

    fn is_special_internal_file(&self, path: &Path) -> bool {
        matches!(path.file_name().and_then(|s| s.to_str()), Some(".tep_ignore"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn collects_files_from_explicit_directory() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        fs::write(temp.path().join("a.txt"), "x").expect("should write file");
        fs::write(temp.path().join("b.txt"), "y").expect("should write file");

        let filter = TepIgnoreFilter::for_workspace_root(temp.path());
        let paths = filter
            .collect_paths(&[".".into()])
            .expect("collection should succeed");

        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn respects_tep_ignore_file_entries() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        fs::write(temp.path().join(".tep_ignore"), "ignored.txt\n").expect("should write ignore file");
        fs::write(temp.path().join("ignored.txt"), "x").expect("should write file");
        fs::write(temp.path().join("included.txt"), "y").expect("should write file");

        let filter = TepIgnoreFilter::for_workspace_root(temp.path());
        let paths = filter
            .collect_paths(&[".".into()])
            .expect("collection should succeed");

        let names = paths
            .iter()
            .filter_map(|p| p.file_name().and_then(|s| s.to_str()))
            .collect::<Vec<_>>();
        assert!(names.contains(&"included.txt"));
        assert!(!names.contains(&"ignored.txt"));
        assert!(!names.contains(&".tep_ignore"));
    }

    #[test]
    fn does_not_honor_gitignore() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        fs::write(temp.path().join(".gitignore"), "ignored.txt\n").expect("should write gitignore");
        fs::write(temp.path().join("ignored.txt"), "x").expect("should write file");

        let filter = TepIgnoreFilter::for_workspace_root(temp.path());
        let paths = filter
            .collect_paths(&[".".into()])
            .expect("collection should succeed");

        let names = paths
            .iter()
            .filter_map(|p| p.file_name().and_then(|s| s.to_str()))
            .collect::<Vec<_>>();
        assert!(names.contains(&"ignored.txt"));
    }
}

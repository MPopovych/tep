use std::path::{Path, PathBuf};

use anyhow::Result;
use ignore::WalkBuilder;

use crate::utils::path::{normalize_to_workspace, resolve_from_workspace};

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
            let absolute_path = resolve_from_workspace(&path, &self.workspace_root);
            if absolute_path.is_file() {
                if self.is_special_internal_file(&absolute_path) {
                    continue;
                }
                let normalized = self.normalize_path(&absolute_path);
                if self.is_ignored(&normalized)? {
                    continue;
                }
                out.push(normalized);
                continue;
            }

            if absolute_path.is_dir() {
                let mut builder = WalkBuilder::new(&absolute_path);
                builder.hidden(false);
                builder.git_ignore(false);
                builder.git_exclude(false);
                builder.git_global(false);
                builder.ignore(false);
                builder.add_custom_ignore_filename(".tepignore");

                for entry in builder.build() {
                    let entry = entry?;
                    let entry_path = entry.path();
                    if entry_path.is_file() && !self.is_special_internal_file(entry_path) {
                        out.push(self.normalize_path(entry_path));
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
            self.normalize_path(path)
        }
    }

    fn normalize_path(&self, path: &Path) -> PathBuf {
        normalize_to_workspace(path, &self.workspace_root)
    }

    fn is_special_internal_file(&self, path: &Path) -> bool {
        matches!(
            path.file_name().and_then(|s| s.to_str()),
            Some(".tepignore")
        )
    }

    fn is_ignored(&self, normalized_path: &Path) -> Result<bool> {
        let mut builder = WalkBuilder::new(&self.workspace_root);
        builder.hidden(false);
        builder.git_ignore(false);
        builder.git_exclude(false);
        builder.git_global(false);
        builder.ignore(false);
        builder.add_custom_ignore_filename(".tepignore");

        let candidate = self.workspace_root.join(normalized_path);
        let mut matched = false;
        for entry in builder.build() {
            let entry = entry?;
            if entry.path() == candidate {
                matched = true;
                break;
            }
        }
        Ok(!matched)
    }
}

// #tepignoreafter
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
        assert!(paths.iter().all(|p| p.starts_with(".")));
    }

    #[test]
    fn normalizes_relative_input_to_workspace_relative_path() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        fs::write(temp.path().join("a.txt"), "x").expect("should write file");

        let previous = std::env::current_dir().expect("current dir should exist");
        std::env::set_current_dir(temp.path()).expect("should change current dir");

        let filter = TepIgnoreFilter::for_workspace_root(temp.path());
        let paths = filter.collect_paths(&["./a.txt".into()]).unwrap();
        assert_eq!(paths, vec![PathBuf::from("./a.txt")]);

        std::env::set_current_dir(previous).expect("should restore current dir");
    }

    #[test]
    fn respects_tep_ignore_file_entries() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        fs::write(temp.path().join(".tepignore"), "ignored.txt\n")
            .expect("should write ignore file");
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
        assert!(!names.contains(&".tepignore"));
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

    #[test]
    fn explicit_file_input_still_respects_tepignore() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        fs::write(temp.path().join(".tepignore"), "README.md\n").expect("should write ignore file");
        fs::write(temp.path().join("README.md"), "x").expect("should write file");

        let filter = TepIgnoreFilter::for_workspace_root(temp.path());
        let paths = filter
            .collect_paths(&["README.md".into()])
            .expect("collection should succeed");

        assert!(paths.is_empty());
    }
}

// (#!#1#tep:utils.path)
// [#!#1#tep:49](path.normalization)
// [#!#1#tep:63](utils.path,path.normalization)
use std::path::{Component, Path, PathBuf};

pub fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub fn normalize_to_workspace(path: &Path, workspace_root: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        normalize_lexically(path)
    } else {
        normalize_lexically(&workspace_root.join(path))
    };

    if let Ok(relative) = absolute.strip_prefix(workspace_root) {
        if relative.as_os_str().is_empty() {
            PathBuf::from(".")
        } else {
            PathBuf::from(".").join(relative)
        }
    } else {
        absolute
    }
}

pub fn resolve_from_workspace(path: &Path, workspace_root: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

pub fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_lexically_removes_dot_and_parent_segments() {
        let path = Path::new("./docs/../README.md");
        assert_eq!(normalize_lexically(path), PathBuf::from("README.md"));
    }

    #[test]
    fn normalize_to_workspace_returns_workspace_relative_path() {
        let workspace = Path::new("/tmp/project");
        let path = Path::new("/tmp/project/./docs/../README.md");
        assert_eq!(normalize_to_workspace(path, workspace), PathBuf::from("./README.md"));
    }

    #[test]
    fn normalize_to_workspace_keeps_outside_absolute_path() {
        let workspace = Path::new("/tmp/project");
        let path = Path::new("/var/tmp/file.md");
        assert_eq!(normalize_to_workspace(path, workspace), PathBuf::from("/var/tmp/file.md"));
    }

    #[test]
    fn resolve_from_workspace_joins_relative_path() {
        let workspace = Path::new("/tmp/project");
        assert_eq!(resolve_from_workspace(Path::new("./docs/a.md"), workspace), PathBuf::from("/tmp/project/./docs/a.md"));
    }
}

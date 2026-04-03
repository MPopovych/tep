// #!#tep:(workspace.scanner){description="Workspace file collection respecting .tepignore rules"}
// #!#tep:(workspace.scanner)->(workspace){description="supports workspace-wide scans for"}
// #!#tep:[workspace.scanner](workspace.scanner,workspace){description="Workspace scanner helper module entry"}
use std::path::PathBuf;

use anyhow::Result;

use crate::filter::tep_ignore_filter::TepIgnoreFilter;
use crate::utils::path::{display_path, resolve_from_workspace};

#[derive(Debug, Clone)]
pub struct WorkspaceFile {
    pub absolute_path: PathBuf,
    pub display_path: String,
}

pub fn collect_workspace_files(
    workspace_root: &PathBuf,
    paths: &[String],
) -> Result<Vec<WorkspaceFile>> {
    let filter = TepIgnoreFilter::for_workspace_root(workspace_root);
    let files = filter.collect_paths(paths)?;
    Ok(files
        .into_iter()
        .map(|path| WorkspaceFile {
            absolute_path: resolve_from_workspace(&path, workspace_root),
            display_path: display_path(&path),
        })
        .collect())
}

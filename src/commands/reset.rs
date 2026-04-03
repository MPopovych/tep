use std::io::{self, Write};

use crate::service::workspace_service::{ResetResult, WorkspaceService};

pub fn run(yes: bool) -> anyhow::Result<()> {
    if !yes && !confirm()? {
        println!("reset cancelled");
        return Ok(());
    }

    let result = WorkspaceService::reset()?;
    print!("{}", format_reset_result(&result));
    Ok(())
}

fn confirm() -> anyhow::Result<bool> {
    print!("this will delete and recreate the tep database. continue? [y/N] ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

fn format_reset_result(result: &ResetResult) -> String {
    let e = &result.entity_auto;
    let a = &result.anchor_auto;
    let mut out = format!(
        "reset complete\n\
         entity auto\n\
         files_processed: {}\n\
         declarations_seen: {}\n\
         relations_seen: {}\n\
         entities_ensured: {}\n\
         refs_filled: {}\n\
         descriptions_filled: {}\n\
         relations_synced: {}\n\
         \n\
         anchor auto\n\
         files_processed: {}\n\
         anchors_seen: {}\n\
         anchors_created: {}\n\
         relations_synced: {}\n\
         metadata_updated: {}\n",
        e.files_processed,
        e.declarations_seen,
        e.relations_seen,
        e.entities_ensured,
        e.refs_filled,
        e.descriptions_filled,
        e.relations_synced,
        a.files_processed,
        a.anchors_seen,
        a.anchors_created,
        a.relations_synced,
        a.metadata_updated,
    );
    if !e.warnings.is_empty() || !a.warnings.is_empty() {
        out.push_str("warnings:\n");
        for warning in &e.warnings {
            out.push_str(&format!("- {}\n", warning));
        }
        for warning in &a.warnings {
            out.push_str(&format!("- {}\n", warning));
        }
    }
    out
}

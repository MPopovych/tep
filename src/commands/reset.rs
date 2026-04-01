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
    format!(
        "reset complete\n\
         entity auto\n\
         files_processed: {}\n\
         declarations_seen: {}\n\
         entities_ensured: {}\n\
         \n\
         anchor auto\n\
         files_processed: {}\n\
         anchors_seen: {}\n\
         anchors_created: {}\n\
         relations_synced: {}\n",
        e.files_processed, e.declarations_seen, e.entities_ensured,
        a.files_processed, a.anchors_seen, a.anchors_created, a.relations_synced,
    )
}

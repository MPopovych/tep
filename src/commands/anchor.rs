use anyhow::Context;

use crate::cli::AnchorArgs;
use crate::db;
use crate::output::anchor_output::{format_anchor_relation_result, format_anchor_sync_result};
use crate::service::anchor_service::AnchorService;

pub fn run(args: AnchorArgs) -> anyhow::Result<()> {
    let conn = db::open_workspace_db()?;
    conn.execute_batch(db::schema_sql())
        .context("failed to apply database schema")?;

    let service = AnchorService::new(&conn);
    let result = service.sync_paths(&args.paths)?;
    print!("{}", format_anchor_sync_result(&result));
    Ok(())
}

pub fn attach(anchor_id: i64, entity_target: &str) -> anyhow::Result<()> {
    let conn = db::open_workspace_db()?;
    conn.execute_batch(db::schema_sql())
        .context("failed to apply database schema")?;

    let service = AnchorService::new(&conn);
    service.attach_entity(anchor_id, entity_target)?;
    print!("{}", format_anchor_relation_result("attached", anchor_id, entity_target));
    Ok(())
}

pub fn detach(anchor_id: i64, entity_target: &str) -> anyhow::Result<()> {
    let conn = db::open_workspace_db()?;
    conn.execute_batch(db::schema_sql())
        .context("failed to apply database schema")?;

    let service = AnchorService::new(&conn);
    service.detach_entity(anchor_id, entity_target)?;
    print!("{}", format_anchor_relation_result("detached", anchor_id, entity_target));
    Ok(())
}

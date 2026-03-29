use anyhow::Context;
use rusqlite::Connection;

use crate::db;
use crate::service::anchor_service::AnchorService;
use crate::service::entity_service::EntityService;

pub fn open_ready_workspace_db() -> anyhow::Result<Connection> {
    let conn = db::open_workspace_db()?;
    db::ensure_schema(&conn)
        .context("failed to apply database schema")?;
    Ok(conn)
}

pub fn with_entity_service<F>(f: F) -> anyhow::Result<()>
where
    F: FnOnce(&EntityService<'_>) -> anyhow::Result<()>,
{
    let conn = open_ready_workspace_db()?;
    let service = EntityService::new(&conn);
    f(&service)
}

pub fn with_anchor_service<F>(f: F) -> anyhow::Result<()>
where
    F: FnOnce(&AnchorService<'_>) -> anyhow::Result<()>,
{
    let conn = open_ready_workspace_db()?;
    let service = AnchorService::new(&conn);
    f(&service)
}

pub fn print_rendered(rendered: String) {
    print!("{}", rendered);
}

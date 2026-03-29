use anyhow::Context;
use rusqlite::Connection;

use crate::db;

pub fn open_ready_workspace_db() -> anyhow::Result<Connection> {
    let conn = db::open_workspace_db()?;
    conn.execute_batch(db::schema_sql())
        .context("failed to apply database schema")?;
    Ok(conn)
}

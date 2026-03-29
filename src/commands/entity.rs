use crate::cli::{EditEntityArgs, EntityCommands, UpsertEntityArgs};
use crate::db;
use crate::output::entity_output::{format_entity, format_entity_list};
use crate::service::entity_service::EntityService;

pub fn run(command: EntityCommands) -> anyhow::Result<()> {
    let conn = db::open_workspace_db()?;
    conn.execute_batch(db::schema_sql())
        .context("failed to apply database schema")?;

    let service = EntityService::new(&conn);

    match command {
        EntityCommands::Create(args) => create(&service, args),
        EntityCommands::Ensure(args) => ensure(&service, args),
        EntityCommands::Read { target } => read(&service, &target),
        EntityCommands::Edit(args) => edit(&service, args),
        EntityCommands::List => list(&service),
    }
}

use anyhow::Context;

fn create(service: &EntityService<'_>, args: UpsertEntityArgs) -> anyhow::Result<()> {
    let entity = service.create(args.name, args.r#ref)?;
    print!("{}", format_entity("created", &entity));
    Ok(())
}

fn ensure(service: &EntityService<'_>, args: UpsertEntityArgs) -> anyhow::Result<()> {
    let entity = service.ensure(args.name, args.r#ref)?;
    print!("{}", format_entity("ensured", &entity));
    Ok(())
}

fn read(service: &EntityService<'_>, target: &str) -> anyhow::Result<()> {
    let entity = service.read(target)?;
    print!("{}", format_entity("entity", &entity));
    Ok(())
}

fn edit(service: &EntityService<'_>, args: EditEntityArgs) -> anyhow::Result<()> {
    let entity = service.edit(&args.target, args.name, args.r#ref)?;
    print!("{}", format_entity("updated", &entity));
    Ok(())
}

fn list(service: &EntityService<'_>) -> anyhow::Result<()> {
    let entities = service.list()?;
    print!("{}", format_entity_list(&entities));
    Ok(())
}

use anyhow::Context;

use crate::cli::{EditEntityArgs, EntityAutoArgs, EntityCommands, UpsertEntityArgs};
use crate::db;
use crate::output::entity_output::{format_entity_auto_result, format_entity_context, format_entity_created, format_entity_list, format_entity_show};
use crate::service::entity_service::EntityService;

pub fn run(command: EntityCommands) -> anyhow::Result<()> {
    let conn = db::open_workspace_db()?;
    conn.execute_batch(db::schema_sql())
        .context("failed to apply database schema")?;

    let service = EntityService::new(&conn);

    match command {
        EntityCommands::Create(args) => create(&service, args),
        EntityCommands::Ensure(args) => ensure(&service, args),
        EntityCommands::Auto(args) => auto(&service, args),
        EntityCommands::Show { target } => show(&service, &target),
        EntityCommands::Context { target } => context(&service, &target),
        EntityCommands::Edit(args) => edit(&service, args),
        EntityCommands::List => list(&service),
    }
}

fn create(service: &EntityService<'_>, args: UpsertEntityArgs) -> anyhow::Result<()> {
    let entity = service.create(args.name, args.r#ref)?;
    print!("{}", format_entity_created("created", &entity));
    Ok(())
}

fn ensure(service: &EntityService<'_>, args: UpsertEntityArgs) -> anyhow::Result<()> {
    let entity = service.ensure(args.name, args.r#ref)?;
    print!("{}", format_entity_created("ensured", &entity));
    Ok(())
}

fn auto(service: &EntityService<'_>, args: EntityAutoArgs) -> anyhow::Result<()> {
    let result = service.auto(&args.paths)?;
    print!("{}", format_entity_auto_result(&result));
    Ok(())
}

fn show(service: &EntityService<'_>, target: &str) -> anyhow::Result<()> {
    let result = service.show(target)?;
    print!("{}", format_entity_show(&result));
    Ok(())
}

fn context(service: &EntityService<'_>, target: &str) -> anyhow::Result<()> {
    let result = service.context(target)?;
    print!("{}", format_entity_context(&result));
    Ok(())
}

fn edit(service: &EntityService<'_>, args: EditEntityArgs) -> anyhow::Result<()> {
    let entity = service.edit(&args.target, args.name, args.r#ref)?;
    print!("{}", format_entity_created("updated", &entity));
    Ok(())
}

fn list(service: &EntityService<'_>) -> anyhow::Result<()> {
    let entities = service.list()?;
    print!("{}", format_entity_list(&entities));
    Ok(())
}

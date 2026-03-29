use crate::cli::{EditEntityArgs, EntityAutoArgs, EntityCommands, EntityContextArgs, UpsertEntityArgs};
use crate::commands::support::open_ready_workspace_db;
use crate::output::entity_output::{format_entity_auto_result, format_entity_context, format_entity_context_files_only, format_entity_created, format_entity_link_result, format_entity_list, format_entity_show, format_entity_unlink_result};
use crate::service::entity_service::EntityService;

pub fn run(command: EntityCommands) -> anyhow::Result<()> {
    let conn = open_ready_workspace_db()?;

    let service = EntityService::new(&conn);

    match command {
        EntityCommands::Create(args) => create(&service, args),
        EntityCommands::Ensure(args) => ensure(&service, args),
        EntityCommands::Auto(args) => auto(&service, args),
        EntityCommands::Show { target } => show(&service, &target),
        EntityCommands::Context(args) => context(&service, args),
        EntityCommands::Edit(args) => edit(&service, args),
        EntityCommands::Link { from, to, relation } => link(&service, &from, &to, &relation),
        EntityCommands::Unlink { from, to } => unlink(&service, &from, &to),
        EntityCommands::List => list(&service),
    }
}

fn create(service: &EntityService<'_>, args: UpsertEntityArgs) -> anyhow::Result<()> {
    let entity = service.create(args.name, args.r#ref, args.description)?;
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

fn context(service: &EntityService<'_>, args: EntityContextArgs) -> anyhow::Result<()> {
    let result = service.context(&args.target, args.include_links, args.link_depth)?;
    if args.files_only {
        print!("{}", format_entity_context_files_only(&result));
    } else {
        print!("{}", format_entity_context(&result));
    }
    Ok(())
}

fn edit(service: &EntityService<'_>, args: EditEntityArgs) -> anyhow::Result<()> {
    let entity = service.edit(&args.target, args.name, args.r#ref, args.description)?;
    print!("{}", format_entity_created("updated", &entity));
    Ok(())
}

fn link(service: &EntityService<'_>, from: &str, to: &str, relation: &str) -> anyhow::Result<()> {
    let result = service.link(from, to, relation)?;
    print!("{}", format_entity_link_result("linked", &result));
    Ok(())
}

fn unlink(service: &EntityService<'_>, from: &str, to: &str) -> anyhow::Result<()> {
    let (from_entity, to_entity) = service.unlink(from, to)?;
    print!("{}", format_entity_unlink_result("unlinked", &from_entity, &to_entity));
    Ok(())
}

fn list(service: &EntityService<'_>) -> anyhow::Result<()> {
    let entities = service.list()?;
    print!("{}", format_entity_list(&entities));
    Ok(())
}

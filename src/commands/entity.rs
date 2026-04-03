use crate::cli::{EntityAutoArgs, EntityCommands, EntityContextArgs};
use crate::commands::support::{print_json, print_rendered, with_entity_service};
use crate::dto::{entity_auto_to_dto, entity_context_to_dto, entity_show_to_dto, entity_to_dto};
use crate::output::entity_output::{
    format_entity_auto_result, format_entity_context, format_entity_context_files_only,
    format_entity_list, format_entity_show,
};
use crate::service::entity_service::EntityService;

pub fn run(command: EntityCommands, json: bool) -> anyhow::Result<()> {
    with_entity_service(|service| match command {
        EntityCommands::Auto(args) => auto(service, args, json),
        EntityCommands::Show { target } => show(service, &target, json),
        EntityCommands::Context(args) => context(service, args, json),
        EntityCommands::List => list(service, json),
    })
}

fn auto(service: &EntityService<'_>, args: EntityAutoArgs, json: bool) -> anyhow::Result<()> {
    let dto = entity_auto_to_dto(&service.auto(&args.paths)?);
    if json {
        print_json(&dto);
    } else {
        print_rendered(format_entity_auto_result(&dto));
    }
    Ok(())
}

fn show(service: &EntityService<'_>, target: &str, json: bool) -> anyhow::Result<()> {
    let dto = entity_show_to_dto(&service.show(target)?);
    if json {
        print_json(&dto);
    } else {
        print_rendered(format_entity_show(&dto));
    }
    Ok(())
}

fn context(service: &EntityService<'_>, args: EntityContextArgs, json: bool) -> anyhow::Result<()> {
    let dto = entity_context_to_dto(&service.context(&args.target, args.link_depth)?);
    if json {
        print_json(&dto);
    } else if args.files_only {
        print_rendered(format_entity_context_files_only(&dto));
    } else {
        print_rendered(format_entity_context(&dto));
    }
    Ok(())
}

fn list(service: &EntityService<'_>, json: bool) -> anyhow::Result<()> {
    let entities: Vec<_> = service.list()?.iter().map(entity_to_dto).collect();
    if json {
        print_json(&entities);
    } else {
        print_rendered(format_entity_list(&entities));
    }
    Ok(())
}

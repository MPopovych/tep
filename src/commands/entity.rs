use crate::cli::{
    EditEntityArgs, EntityAutoArgs, EntityCommands, EntityContextArgs, UpsertEntityArgs,
};
use crate::commands::support::{print_json, print_rendered, with_entity_service};
use crate::dto::{entity_auto_to_dto, entity_context_to_dto, entity_show_to_dto, entity_to_dto};
use crate::output::entity_output::{
    format_entity_auto_result, format_entity_context, format_entity_context_files_only,
    format_entity_created, format_entity_link_result, format_entity_list, format_entity_show,
    format_entity_unlink_result,
};
use crate::service::entity_service::EntityService;

pub fn run(command: EntityCommands, json: bool) -> anyhow::Result<()> {
    with_entity_service(|service| match command {
        EntityCommands::Create(args) => create(service, args, json),
        EntityCommands::Ensure(args) => ensure(service, args, json),
        EntityCommands::Auto(args) => auto(service, args, json),
        EntityCommands::Show { target } => show(service, &target, json),
        EntityCommands::Context(args) => context(service, args, json),
        EntityCommands::Edit(args) => edit(service, args, json),
        EntityCommands::Link { from, to, relation } => link(service, &from, &to, &relation, json),
        EntityCommands::Unlink { from, to } => unlink(service, &from, &to, json),
        EntityCommands::List => list(service, json),
    })
}

fn create(service: &EntityService<'_>, args: UpsertEntityArgs, json: bool) -> anyhow::Result<()> {
    let entity = entity_to_dto(&service.create(args.name, args.r#ref, args.description)?);
    if json {
        print_json(&entity);
    } else {
        print_rendered(format_entity_created("created", &entity));
    }
    Ok(())
}

fn ensure(service: &EntityService<'_>, args: UpsertEntityArgs, json: bool) -> anyhow::Result<()> {
    let entity = entity_to_dto(&service.ensure(args.name, args.r#ref)?);
    if json {
        print_json(&entity);
    } else {
        print_rendered(format_entity_created("ensured", &entity));
    }
    Ok(())
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

fn edit(service: &EntityService<'_>, args: EditEntityArgs, json: bool) -> anyhow::Result<()> {
    let entity =
        entity_to_dto(&service.edit(&args.target, args.name, args.r#ref, args.description)?);
    if json {
        print_json(&entity);
    } else {
        print_rendered(format_entity_created("updated", &entity));
    }
    Ok(())
}

fn link(
    service: &EntityService<'_>,
    from: &str,
    to: &str,
    relation: &str,
    json: bool,
) -> anyhow::Result<()> {
    let result = service.link(from, to, relation)?;
    let from_dto = entity_to_dto(&result.from);
    let to_dto = entity_to_dto(&result.to);
    if json {
        print_json(&serde_json::json!({
            "from": from_dto,
            "to": to_dto,
            "relation": result.relation,
        }));
    } else {
        print_rendered(format_entity_link_result(
            "linked",
            &from_dto,
            &to_dto,
            &result.relation,
        ));
    }
    Ok(())
}

fn unlink(service: &EntityService<'_>, from: &str, to: &str, json: bool) -> anyhow::Result<()> {
    let (from_entity, to_entity) = service.unlink(from, to)?;
    let from_dto = entity_to_dto(&from_entity);
    let to_dto = entity_to_dto(&to_entity);
    if json {
        print_json(&serde_json::json!({
            "from": from_dto,
            "to": to_dto,
        }));
    } else {
        print_rendered(format_entity_unlink_result("unlinked", &from_dto, &to_dto));
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

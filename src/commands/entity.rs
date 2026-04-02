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
    let entity = service.create(args.name, args.r#ref, args.description)?;
    if json {
        print_json(&entity_to_dto(&entity));
    } else {
        print_rendered(format_entity_created("created", &entity));
    }
    Ok(())
}

fn ensure(service: &EntityService<'_>, args: UpsertEntityArgs, json: bool) -> anyhow::Result<()> {
    let entity = service.ensure(args.name, args.r#ref)?;
    if json {
        print_json(&entity_to_dto(&entity));
    } else {
        print_rendered(format_entity_created("ensured", &entity));
    }
    Ok(())
}

fn auto(service: &EntityService<'_>, args: EntityAutoArgs, json: bool) -> anyhow::Result<()> {
    let result = service.auto(&args.paths)?;
    if json {
        print_json(&entity_auto_to_dto(&result));
    } else {
        print_rendered(format_entity_auto_result(&result));
    }
    Ok(())
}

fn show(service: &EntityService<'_>, target: &str, json: bool) -> anyhow::Result<()> {
    let result = service.show(target)?;
    if json {
        print_json(&entity_show_to_dto(&result));
    } else {
        print_rendered(format_entity_show(&result));
    }
    Ok(())
}

fn context(service: &EntityService<'_>, args: EntityContextArgs, json: bool) -> anyhow::Result<()> {
    let result = service.context(&args.target, args.link_depth)?;
    if json {
        print_json(&entity_context_to_dto(&result));
    } else if args.files_only {
        print_rendered(format_entity_context_files_only(&result));
    } else {
        print_rendered(format_entity_context(&result));
    }
    Ok(())
}

fn edit(service: &EntityService<'_>, args: EditEntityArgs, json: bool) -> anyhow::Result<()> {
    let entity = service.edit(&args.target, args.name, args.r#ref, args.description)?;
    if json {
        print_json(&entity_to_dto(&entity));
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
    if json {
        print_json(&serde_json::json!({
            "from": entity_to_dto(&result.from),
            "to": entity_to_dto(&result.to),
            "relation": result.relation,
        }));
    } else {
        print_rendered(format_entity_link_result("linked", &result));
    }
    Ok(())
}

fn unlink(service: &EntityService<'_>, from: &str, to: &str, json: bool) -> anyhow::Result<()> {
    let (from_entity, to_entity) = service.unlink(from, to)?;
    if json {
        print_json(&serde_json::json!({
            "from": entity_to_dto(&from_entity),
            "to": entity_to_dto(&to_entity),
        }));
    } else {
        print_rendered(format_entity_unlink_result(
            "unlinked",
            &from_entity,
            &to_entity,
        ));
    }
    Ok(())
}

fn list(service: &EntityService<'_>, json: bool) -> anyhow::Result<()> {
    let entities = service.list()?;
    if json {
        let dto: Vec<_> = entities.iter().map(entity_to_dto).collect();
        print_json(&dto);
    } else {
        print_rendered(format_entity_list(&entities));
    }
    Ok(())
}

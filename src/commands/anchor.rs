use crate::cli::AnchorCommands;
use crate::commands::support::{print_rendered, with_anchor_service};
use crate::output::anchor_output::{format_anchor_list, format_anchor_relation_result, format_anchor_show, format_anchor_sync_result};

pub fn run(command: AnchorCommands) -> anyhow::Result<()> {
    with_anchor_service(|service| match command {
        AnchorCommands::Auto(args) => {
            let result = service.sync_paths(&args.paths)?;
            print_rendered(format_anchor_sync_result(&result));
            Ok(())
        }
        AnchorCommands::Show { target } => {
            let result = service.show(&target)?;
            print_rendered(format_anchor_show(&result));
            Ok(())
        }
        AnchorCommands::Edit(args) => {
            let updated = service.edit_name(args.anchor_id, &args.name, true)?;
            print_rendered(format_anchor_show(&crate::service::anchor_service::AnchorShowResult {
                anchor: updated,
                entities: vec![],
            }));
            Ok(())
        }
        AnchorCommands::List => {
            let anchors = service.list_all()?;
            print_rendered(format_anchor_list(&anchors));
            Ok(())
        }
    })
}

pub fn attach(anchor_target: &str, entity_target: &str) -> anyhow::Result<()> {
    with_anchor_service(|service| {
        service.attach_entity(anchor_target, entity_target)?;
        print_rendered(format_anchor_relation_result("attached", anchor_target, entity_target));
        Ok(())
    })
}

pub fn detach(anchor_target: &str, entity_target: &str) -> anyhow::Result<()> {
    with_anchor_service(|service| {
        service.detach_entity(anchor_target, entity_target)?;
        print_rendered(format_anchor_relation_result("detached", anchor_target, entity_target));
        Ok(())
    })
}

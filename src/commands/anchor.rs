use crate::cli::AnchorCommands;
use crate::commands::support::{print_rendered, with_anchor_service};
use crate::output::anchor_output::{format_anchor_relation_result, format_anchor_show, format_anchor_sync_result};
use crate::service::anchor_service::AnchorService;

pub fn run(command: AnchorCommands) -> anyhow::Result<()> {
    with_anchor_service(|service| match command {
        AnchorCommands::Auto(args) => {
            let result = service.sync_paths(&args.paths)?;
            print_rendered(format_anchor_sync_result(&result));
            Ok(())
        }
        AnchorCommands::Show { anchor_id } => {
            let result = service.show(anchor_id)?;
            print_rendered(format_anchor_show(&result));
            Ok(())
        }
    })
}

pub fn attach(anchor_id: i64, entity_target: &str) -> anyhow::Result<()> {
    with_anchor_service(|service| {
        service.attach_entity(anchor_id, entity_target)?;
        print_rendered(format_anchor_relation_result("attached", anchor_id, entity_target));
        Ok(())
    })
}

pub fn detach(anchor_id: i64, entity_target: &str) -> anyhow::Result<()> {
    with_anchor_service(|service| {
        service.detach_entity(anchor_id, entity_target)?;
        print_rendered(format_anchor_relation_result("detached", anchor_id, entity_target));
        Ok(())
    })
}

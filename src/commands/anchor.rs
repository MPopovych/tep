use crate::cli::AnchorCommands;
use crate::commands::support::open_ready_workspace_db;
use crate::output::anchor_output::{format_anchor_relation_result, format_anchor_show, format_anchor_sync_result};
use crate::service::anchor_service::AnchorService;

pub fn run(command: AnchorCommands) -> anyhow::Result<()> {
    let conn = open_ready_workspace_db()?;
    let service = AnchorService::new(&conn);

    match command {
        AnchorCommands::Auto(args) => {
            let result = service.sync_paths(&args.paths)?;
            print!("{}", format_anchor_sync_result(&result));
        }
        AnchorCommands::Show { anchor_id } => {
            let result = service.show(anchor_id)?;
            print!("{}", format_anchor_show(&result));
        }
    }
    Ok(())
}

pub fn attach(anchor_id: i64, entity_target: &str) -> anyhow::Result<()> {
    let conn = open_ready_workspace_db()?;
    let service = AnchorService::new(&conn);
    service.attach_entity(anchor_id, entity_target)?;
    print!("{}", format_anchor_relation_result("attached", anchor_id, entity_target));
    Ok(())
}

pub fn detach(anchor_id: i64, entity_target: &str) -> anyhow::Result<()> {
    let conn = open_ready_workspace_db()?;
    let service = AnchorService::new(&conn);
    service.detach_entity(anchor_id, entity_target)?;
    print!("{}", format_anchor_relation_result("detached", anchor_id, entity_target));
    Ok(())
}

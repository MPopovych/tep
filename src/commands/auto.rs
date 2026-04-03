use crate::cli::AutoArgs;
use crate::commands::support::{open_ready_workspace_db, print_json, print_rendered};
use crate::dto::{anchor_sync_to_dto, entity_auto_to_dto};
use crate::output::anchor_output::format_anchor_sync_result;
use crate::output::entity_output::format_entity_auto_result;
use crate::service::anchor_service::AnchorService;
use crate::service::entity_service::EntityService;

pub fn run(args: AutoArgs, json: bool) -> anyhow::Result<()> {
    let conn = open_ready_workspace_db()?;
    let entity_service = EntityService::new(&conn);
    let anchor_service = AnchorService::new(&conn);

    let entity_result = entity_auto_to_dto(&entity_service.auto(&args.paths)?);
    let anchor_result = anchor_sync_to_dto(&anchor_service.sync_paths(&args.paths)?);

    if json {
        print_json(&serde_json::json!({
            "entity_auto": entity_result,
            "anchor_auto": anchor_result,
        }));
    } else {
        let mut out = String::new();
        out.push_str(&format_entity_auto_result(&entity_result));
        out.push('\n');
        out.push_str(&format_anchor_sync_result(&anchor_result));
        print_rendered(out);
    }
    Ok(())
}

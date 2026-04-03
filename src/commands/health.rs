use crate::cli::HealthArgs;
use crate::commands::support::{open_ready_workspace_db, print_json};
use crate::dto::health_to_dto;
use crate::output::anchor_output::format_anchor_health_result;
use crate::service::health_service::HealthService;

pub fn run(args: HealthArgs, json: bool) -> anyhow::Result<()> {
    let conn = open_ready_workspace_db()?;
    let service = HealthService::new(&conn);
    let dto = health_to_dto(&service.audit_paths(&[args.path])?);
    let has_issues = dto.anchors_moved > 0
        || dto.anchors_missing > 0
        || dto.duplicate_anchor_ids > 0
        || dto.unknown_anchor_ids > 0
        || dto.entities_without_anchors > 0
        || dto.anchors_without_entities > 0
        || dto.metadata_warnings > 0;
    if json {
        print_json(&dto);
    } else {
        print!("{}", format_anchor_health_result(&dto));
    }
    if args.check && has_issues {
        anyhow::bail!("health check failed");
    }
    Ok(())
}

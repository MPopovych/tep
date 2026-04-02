use crate::cli::HealthArgs;
use crate::commands::support::{open_ready_workspace_db, print_json};
use crate::dto::health_to_dto;
use crate::output::anchor_output::format_anchor_health_result;
use crate::service::health_service::HealthService;

pub fn run(args: HealthArgs, json: bool) -> anyhow::Result<()> {
    let conn = open_ready_workspace_db()?;
    let service = HealthService::new(&conn);
    let dto = health_to_dto(&service.audit_paths(&[args.path])?);
    if json {
        print_json(&dto);
    } else {
        print!("{}", format_anchor_health_result(&dto));
    }
    Ok(())
}

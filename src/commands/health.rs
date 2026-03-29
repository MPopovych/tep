use crate::cli::HealthArgs;
use crate::commands::support::open_ready_workspace_db;
use crate::output::anchor_output::format_anchor_health_result;
use crate::service::health_service::HealthService;

pub fn run(args: HealthArgs) -> anyhow::Result<()> {
    let conn = open_ready_workspace_db()?;
    let service = HealthService::new(&conn);
    let result = service.audit_paths(&[args.path])?;
    print!("{}", format_anchor_health_result(&result));
    Ok(())
}

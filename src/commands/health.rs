use crate::cli::HealthArgs;
use crate::commands::support::open_ready_workspace_db;
use crate::output::anchor_output::format_anchor_health_result;
use crate::service::anchor_service::AnchorService;

pub fn run(args: HealthArgs) -> anyhow::Result<()> {
    let conn = open_ready_workspace_db()?;
    let service = AnchorService::new(&conn);
    let result = service.health_paths(&[args.path])?;
    print!("{}", format_anchor_health_result(&result));
    Ok(())
}

use crate::output::workspace_output::format_init_result;
use crate::service::workspace_service::WorkspaceService;

pub fn run() -> anyhow::Result<()> {
    let result = WorkspaceService::init()?;
    print!("{}", format_init_result(&result));
    Ok(())
}

use crate::cli::HealthArgs;
use crate::commands::support::{print_rendered, with_anchor_service};
use crate::output::anchor_output::format_anchor_health_result;

pub fn run(args: HealthArgs) -> anyhow::Result<()> {
    with_anchor_service(|service| {
        let result = service.health_paths(&[args.path])?;
        print_rendered(format_anchor_health_result(&result));
        Ok(())
    })
}

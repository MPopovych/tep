use crate::cli::AnchorCommands;
use crate::commands::support::{print_rendered, with_anchor_service};
use crate::output::anchor_output::{
    format_anchor_list, format_anchor_show, format_anchor_sync_result,
};

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
        AnchorCommands::List => {
            let anchors = service.list_all()?;
            print_rendered(format_anchor_list(&anchors));
            Ok(())
        }
    })
}

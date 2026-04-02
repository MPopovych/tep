use crate::cli::AnchorCommands;
use crate::commands::support::{print_json, print_rendered, with_anchor_service};
use crate::dto::{anchor_show_to_dto, anchor_sync_to_dto, anchor_to_dto};
use crate::output::anchor_output::{
    format_anchor_list, format_anchor_show, format_anchor_sync_result,
};

pub fn run(command: AnchorCommands, json: bool) -> anyhow::Result<()> {
    with_anchor_service(|service| match command {
        AnchorCommands::Auto(args) => {
            let result = service.sync_paths(&args.paths)?;
            if json {
                print_json(&anchor_sync_to_dto(&result));
            } else {
                print_rendered(format_anchor_sync_result(&result));
            }
            Ok(())
        }
        AnchorCommands::Show { target } => {
            let result = service.show(&target)?;
            if json {
                print_json(&anchor_show_to_dto(&result));
            } else {
                print_rendered(format_anchor_show(&result));
            }
            Ok(())
        }
        AnchorCommands::List => {
            let anchors = service.list_all()?;
            if json {
                let dto: Vec<_> = anchors.iter().map(anchor_to_dto).collect();
                print_json(&dto);
            } else {
                print_rendered(format_anchor_list(&anchors));
            }
            Ok(())
        }
    })
}

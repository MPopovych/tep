use crate::cli::AnchorCommands;
use crate::commands::support::{print_json, print_rendered, with_anchor_service};
use crate::dto::{anchor_show_to_dto, anchor_sync_to_dto, anchor_to_dto};
use crate::output::anchor_output::{
    format_anchor_list, format_anchor_show, format_anchor_sync_result,
};

pub fn run(command: AnchorCommands, json: bool) -> anyhow::Result<()> {
    with_anchor_service(|service| match command {
        AnchorCommands::Auto(args) => {
            let dto = anchor_sync_to_dto(&service.sync_paths(&args.paths)?);
            if json {
                print_json(&dto);
            } else {
                print_rendered(format_anchor_sync_result(&dto));
            }
            Ok(())
        }
        AnchorCommands::Show { target } => {
            let dto = anchor_show_to_dto(&service.show(&target)?);
            if json {
                print_json(&dto);
            } else {
                print_rendered(format_anchor_show(&dto));
            }
            Ok(())
        }
        AnchorCommands::List => {
            let anchors: Vec<_> = service.list_all()?.iter().map(anchor_to_dto).collect();
            if json {
                print_json(&anchors);
            } else {
                print_rendered(format_anchor_list(&anchors));
            }
            Ok(())
        }
    })
}

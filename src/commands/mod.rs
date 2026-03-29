mod anchor;
mod entity;
mod init;

use crate::cli::{Cli, Commands};

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Init => init::run(),
        Commands::Scan => not_implemented("scan"),
        Commands::Entity { command } | Commands::E { command } => entity::run(command),
        Commands::Anchor { command } => anchor::run(command),
        Commands::Attach { entity_id, anchor_id } => {
            let anchor_id = anchor_id.parse::<i64>()?;
            anchor::attach(anchor_id, &entity_id)
        }
        Commands::Detach { entity_id, anchor_id } => {
            let anchor_id = anchor_id.parse::<i64>()?;
            anchor::detach(anchor_id, &entity_id)
        }
        Commands::Link { command } => not_implemented(&format!("link {command:?}")),
        Commands::Resolve { target } => not_implemented(&format!("resolve target={target}")),
        Commands::Graph { entity_id, depth } => {
            not_implemented(&format!("graph entity_id={entity_id} depth={depth}"))
        }
        Commands::Context { command } => not_implemented(&format!("context {command:?}")),
        Commands::Status => not_implemented("status"),
        Commands::Doctor => not_implemented("doctor"),
    }
}

fn not_implemented(name: &str) -> anyhow::Result<()> {
    println!("tep {name} is not implemented yet");
    Ok(())
}

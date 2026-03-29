mod anchor;
mod entity;
mod init;
mod support;

use crate::cli::{Cli, Commands};

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Init => init::run(),
        Commands::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Entity { command } | Commands::E { command } => entity::run(command),
        Commands::Anchor { command } | Commands::A { command } => anchor::run(command),
        Commands::Attach { entity_id, anchor_id } => {
            let anchor_id = anchor_id.parse::<i64>()?;
            anchor::attach(anchor_id, &entity_id)
        }
        Commands::Detach { entity_id, anchor_id } => {
            let anchor_id = anchor_id.parse::<i64>()?;
            anchor::detach(anchor_id, &entity_id)
        }
    }
}

mod anchor;
mod entity;
mod health;
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
        Commands::Health(args) => health::run(args),
        Commands::Entity { command } | Commands::E { command } => entity::run(command),
        Commands::Anchor { command } | Commands::A { command } => anchor::run(command),
        Commands::Attach { entity_id, anchor_id } => {
            anchor::attach(&anchor_id, &entity_id)
        }
        Commands::Detach { entity_id, anchor_id } => {
            anchor::detach(&anchor_id, &entity_id)
        }
    }
}

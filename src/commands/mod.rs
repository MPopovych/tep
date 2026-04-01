mod anchor;
mod entity;
mod health;
mod init;
mod reset;
mod support;

use crate::cli::{Cli, Commands};

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Init => init::run(),
        Commands::Reset { yes } => reset::run(yes),
        Commands::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Health(args) => health::run(args),
        Commands::Entity { command } | Commands::E { command } => entity::run(command),
        Commands::Anchor { command } | Commands::A { command } => anchor::run(command),
    }
}

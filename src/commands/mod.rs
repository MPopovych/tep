mod anchor;
mod entity;
mod health;
mod init;
mod reset;
mod support;

use crate::cli::Commands;

pub fn run(command: Commands, json: bool) -> anyhow::Result<()> {
    match command {
        Commands::Init => init::run(),
        Commands::Reset { yes } => reset::run(yes),
        Commands::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Health(args) => health::run(args, json),
        Commands::Entity { command } | Commands::E { command } => entity::run(command, json),
        Commands::Anchor { command } | Commands::A { command } => anchor::run(command, json),
    }
}

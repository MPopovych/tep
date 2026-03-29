mod anchor;
mod cli;
mod commands;
mod db;
mod entity;
mod errors;
mod filter;
mod output {
    pub mod anchor_format;
    pub mod anchor_output;
    pub mod entity_output;
    pub mod styles;
    pub mod workspace_output;
}
mod repository;
mod service {
    pub mod anchor_service;
    pub mod entity_context;
    pub mod entity_service;
    pub mod workspace_service;
}

use clap::Parser;
use cli::Cli;

fn main() -> anyhow::Result<()> {
    init_tracing();

    let cli = Cli::parse();
    commands::run(cli)
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tep=info".into()),
        )
        .with_target(false)
        .try_init();
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke_test() {
        assert_eq!(2 + 2, 4);
    }
}

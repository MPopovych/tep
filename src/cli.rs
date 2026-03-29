use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "tep")]
#[command(about = "text entity pointers")]
#[command(version, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init,
    Version,
    Entity {
        #[command(subcommand)]
        command: EntityCommands,
    },
    E {
        #[command(subcommand)]
        command: EntityCommands,
    },
    Anchor {
        #[command(subcommand)]
        command: AnchorCommands,
    },
    Attach {
        entity_id: String,
        anchor_id: String,
    },
    Detach {
        entity_id: String,
        anchor_id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum AnchorCommands {
    Auto(AnchorAutoArgs),
    Show { anchor_id: i64 },
}

#[derive(Debug, Args, Clone)]
pub struct AnchorAutoArgs {
    #[arg(required = true)]
    pub paths: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum EntityCommands {
    Create(UpsertEntityArgs),
    Ensure(UpsertEntityArgs),
    Auto(EntityAutoArgs),
    Show { target: String },
    Edit(EditEntityArgs),
    List,
}

#[derive(Debug, Args, Clone)]
pub struct UpsertEntityArgs {
    pub name: String,
    #[arg(long)]
    pub r#ref: Option<String>,
}

#[derive(Debug, Args, Clone)]
pub struct EntityAutoArgs {
    #[arg(required = true)]
    pub paths: Vec<String>,
}

#[derive(Debug, Args, Clone)]
pub struct EditEntityArgs {
    pub target: String,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub r#ref: Option<String>,
}

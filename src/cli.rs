use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "tep")]
#[command(about = "text entity pointers")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init,
    Scan,
    Status,
    Doctor,
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
    Link {
        #[command(subcommand)]
        command: LinkCommands,
    },
    Resolve {
        target: String,
    },
    Graph {
        entity_id: String,
        #[arg(long, default_value_t = 1)]
        depth: usize,
    },
    Context {
        #[command(subcommand)]
        command: ContextCommands,
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
pub struct EditEntityArgs {
    pub target: String,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub r#ref: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum LinkCommands {
    Add {
        from_entity_id: String,
        relation_type: String,
        to_entity_id: String,
        #[arg(long, default_value_t = 1)]
        priority: u32,
    },
    List {
        entity_id: String,
    },
    Remove {
        from_entity_id: String,
        relation_type: String,
        to_entity_id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ContextCommands {
    Get {
        target: String,
        #[arg(long, default_value_t = 1)]
        depth: usize,
    },
}

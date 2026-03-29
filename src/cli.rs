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
    #[command(about = "Initialize a tep workspace")]
    Init,
    #[command(about = "Print the tep version")]
    Version,
    #[command(about = "Work with entities")]
    Entity {
        #[command(subcommand)]
        command: EntityCommands,
    },
    #[command(about = "Shorthand for entity")]
    E {
        #[command(subcommand)]
        command: EntityCommands,
    },
    #[command(about = "Work with anchors")]
    Anchor {
        #[command(subcommand)]
        command: AnchorCommands,
    },
    #[command(about = "Shorthand for anchor")]
    A {
        #[command(subcommand)]
        command: AnchorCommands,
    },
    #[command(about = "Attach an entity to an anchor")]
    Attach {
        entity_id: String,
        anchor_id: String,
    },
    #[command(about = "Detach an entity from an anchor")]
    Detach {
        entity_id: String,
        anchor_id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum AnchorCommands {
    #[command(about = "Materialize and sync anchors in files")]
    Auto(AnchorAutoArgs),
    #[command(about = "Show one anchor and its related entities")]
    Show { anchor_id: i64 },
}

#[derive(Debug, Args, Clone)]
pub struct AnchorAutoArgs {
    #[arg(required = true)]
    pub paths: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum EntityCommands {
    #[command(about = "Create a new entity")]
    Create(UpsertEntityArgs),
    #[command(about = "Ensure an entity exists")]
    Ensure(UpsertEntityArgs),
    #[command(about = "Auto-declare entities from files")]
    Auto(EntityAutoArgs),
    #[command(about = "Show one entity and its related anchors")]
    Show { target: String },
    #[command(about = "Edit an existing entity")]
    Edit(EditEntityArgs),
    #[command(about = "List entities")]
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

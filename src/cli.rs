use clap::{Args, Parser, Subcommand};

const ABOUT: &str = "text entity pointers";
const ENTITY_ABOUT: &str = "Work with entities and directional links";
const ANCHOR_ABOUT: &str = "Work with anchors and anchor-entity attachments";

#[derive(Debug, Parser)]
#[command(name = "tep")]
#[command(about = ABOUT)]
#[command(version, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a tep workspace in the current directory")]
    Init,
    #[command(about = "Print the tep version")]
    Version,
    #[command(about = "Audit workspace health and graph integrity")]
    Health(HealthArgs),
    #[command(about = ENTITY_ABOUT)]
    Entity {
        #[command(subcommand)]
        command: EntityCommands,
    },
    #[command(about = "Shorthand for entity")]
    E {
        #[command(subcommand)]
        command: EntityCommands,
    },
    #[command(about = ANCHOR_ABOUT)]
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
        #[arg(help = "Entity name or id")]
        entity_id: String,
        #[arg(help = "Anchor id or name")]
        anchor_id: String,
    },
    #[command(about = "Detach an entity from an anchor")]
    Detach {
        #[arg(help = "Entity name or id")]
        entity_id: String,
        #[arg(help = "Anchor id or name")]
        anchor_id: String,
    },
}

#[derive(Debug, Args, Clone)]
pub struct HealthArgs {
    #[arg(default_value = ".", help = "File or directory to audit relative to the workspace")]
    pub path: String,
}

#[derive(Debug, Subcommand)]
pub enum AnchorCommands {
    #[command(about = "Materialize and sync anchors in files")]
    Auto(AnchorAutoArgs),
    #[command(about = "Show one anchor and its related entities")]
    Show {
        #[arg(help = "Anchor id or name")]
        target: String,
    },
    #[command(about = "Edit an anchor (set or rename its name)")]
    Edit(EditAnchorArgs),
    #[command(about = "List all anchors in the workspace")]
    List,
}

#[derive(Debug, Args, Clone)]
pub struct AnchorAutoArgs {
    #[arg(required = true, help = "Files or directories to scan")]
    pub paths: Vec<String>,
}

#[derive(Debug, Args, Clone)]
pub struct EditAnchorArgs {
    #[arg(help = "Anchor id")]
    pub anchor_id: i64,
    #[arg(long, help = "Set or rename the anchor's name")]
    pub name: String,
}

#[derive(Debug, Subcommand)]
pub enum EntityCommands {
    #[command(about = "Create a new entity")]
    Create(UpsertEntityArgs),
    #[command(about = "Ensure an entity exists")]
    Ensure(UpsertEntityArgs),
    #[command(about = "Auto-declare entities from files")]
    Auto(EntityAutoArgs),
    #[command(about = "Show one entity and its related anchors and links")]
    Show {
        #[arg(help = "Entity name or id")]
        target: String,
    },
    #[command(about = "Show one entity with snippets, files, and linked entities")]
    Context(EntityContextArgs),
    #[command(about = "Edit an existing entity")]
    Edit(EditEntityArgs),
    #[command(about = "Create or update a directional entity link")]
    Link {
        #[arg(help = "Source entity name or id")]
        from: String,
        #[arg(help = "Target entity name or id")]
        to: String,
        #[arg(long, help = "Relation text stored on the directional link")]
        relation: String,
    },
    #[command(about = "Remove a directional entity link")]
    Unlink {
        #[arg(help = "Source entity name or id")]
        from: String,
        #[arg(help = "Target entity name or id")]
        to: String,
    },
    #[command(about = "List entities")]
    List,
}

#[derive(Debug, Args, Clone)]
pub struct UpsertEntityArgs {
    #[arg(help = "Entity name")]
    pub name: String,
    #[arg(long, help = "Reference file path for the entity")]
    pub r#ref: Option<String>,
    #[arg(long, help = "Human-readable description")]
    pub description: Option<String>,
}

#[derive(Debug, Args, Clone)]
pub struct EntityAutoArgs {
    #[arg(required = true, help = "Files or directories to scan")]
    pub paths: Vec<String>,
}

#[derive(Debug, Args, Clone)]
pub struct EntityContextArgs {
    #[arg(help = "Entity name or id")]
    pub target: String,
    #[arg(long, help = "Show files and linked entities without anchor snippets")]
    pub files_only: bool,
    #[arg(long, default_value_t = 1, help = "Traverse directional links up to this depth")]
    pub link_depth: usize,
}

#[derive(Debug, Args, Clone)]
pub struct EditEntityArgs {
    #[arg(help = "Entity name or id")]
    pub target: String,
    #[arg(long, help = "Replace the entity name")]
    pub name: Option<String>,
    #[arg(long, help = "Replace the entity reference path")]
    pub r#ref: Option<String>,
    #[arg(long, help = "Replace the entity description")]
    pub description: Option<String>,
}

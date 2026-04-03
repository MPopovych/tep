use clap::{Args, Parser, Subcommand};

const ABOUT: &str =
    "text entity pointers — connect concepts to locations in your codebase and docs";
const ENTITY_ABOUT: &str = "Inspect entities, context, descriptions, refs, and links";
const ANCHOR_ABOUT: &str = "Work with anchors, names, and anchor-entity attachments";

#[derive(Debug, Parser)]
#[command(name = "tep")]
#[command(about = ABOUT)]
#[command(version, propagate_version = true)]
pub struct Cli {
    #[arg(long, global = true, help = "Output as JSON")]
    pub json: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a tep workspace in the current directory")]
    Init,
    #[command(about = "Reset the tep database and re-index the workspace")]
    Reset {
        #[arg(long, help = "Skip confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Auto-sync entities, relations, and anchors from files")]
    Auto(AutoArgs),
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
}

#[derive(Debug, Args, Clone)]
pub struct AutoArgs {
    #[arg(required = true, help = "Files or directories to scan")]
    pub paths: Vec<String>,
}

#[derive(Debug, Args, Clone)]
pub struct HealthArgs {
    #[arg(
        default_value = ".",
        help = "File or directory to audit relative to the workspace"
    )]
    pub path: String,
    #[arg(
        long,
        help = "Exit with status 1 when health issues or warnings are present"
    )]
    pub check: bool,
}

#[derive(Debug, Subcommand)]
pub enum AnchorCommands {
    #[command(about = "Sync anchors in files")]
    Auto(AnchorAutoArgs),
    #[command(about = "Show one anchor and its related entities")]
    Show {
        #[arg(help = "Anchor id or name")]
        target: String,
    },
    #[command(about = "List all anchors in the workspace")]
    List,
}

#[derive(Debug, Args, Clone)]
pub struct AnchorAutoArgs {
    #[arg(required = true, help = "Files or directories to scan")]
    pub paths: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum EntityCommands {
    #[command(about = "Auto-declare entities from files")]
    Auto(EntityAutoArgs),
    #[command(about = "Show one entity and its related anchors and links")]
    Show {
        #[arg(help = "Entity name or id")]
        target: String,
    },
    #[command(about = "Show one entity with snippets, files, and linked entities")]
    Context(EntityContextArgs),
    #[command(about = "List entities")]
    List,
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
    #[arg(
        long,
        default_value_t = 1,
        help = "Traverse directional links up to this depth"
    )]
    pub link_depth: usize,
}

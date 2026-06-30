use clap::{Parser, Subcommand};

pub const VERSION: &str = env!("ENVFORGE_VERSION");

#[derive(Parser)]
#[command(name = "envforge")]
#[command(about = "Create and manage reproducible development environments from YAML configs", long_about = None)]
#[command(version = VERSION)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(global = true, long, short = 'n', help = "Show actions without executing")]
    pub dry_run: bool,

    #[arg(global = true, long, short = 'v', help = "Enable verbose logging")]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Init {
        name: String,
    },
    Create {
        path: String,
    },
    Enter {
        profile: String,

        #[arg(long, short = 'e', help = "Print export commands for evaluation")]
        export: bool,
    },
    List,
    Remove {
        profile: String,
    },
    Doctor,
}

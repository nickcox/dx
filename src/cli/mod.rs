use clap::{Parser, Subcommand};

use crate::resolve::Resolver;

mod resolve;
mod stacks;

#[derive(Debug, Parser)]
#[command(name = "dx", version, about = "Directory navigation helper")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Resolve {
        query: String,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        json: bool,
    },
    Push {
        path: String,
        #[arg(long)]
        session: Option<String>,
    },
    Pop {
        #[arg(long)]
        session: Option<String>,
    },
    Undo {
        #[arg(long)]
        session: Option<String>,
    },
    Redo {
        #[arg(long)]
        session: Option<String>,
    },
}

pub fn run() -> i32 {
    let cli = Cli::parse();
    let resolver = Resolver::from_environment();

    match cli.command {
        Commands::Resolve { query, list, json } => {
            resolve::run_resolve(&resolver, &query, list, json)
        }
        Commands::Push { path, session } => stacks::run_push(&path, session.as_deref()),
        Commands::Pop { session } => stacks::run_pop(session.as_deref()),
        Commands::Undo { session } => stacks::run_undo(session.as_deref()),
        Commands::Redo { session } => stacks::run_redo(session.as_deref()),
    }
}

use clap::{Parser, Subcommand};

use crate::resolve::Resolver;

mod bookmarks;
mod complete;
mod init;
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
    Init {
        shell: String,
        #[arg(long = "command-not-found")]
        command_not_found: bool,
    },
    Complete {
        #[command(subcommand)]
        command: complete::CompleteCommand,
    },
    Navigate {
        mode: complete::NavigateMode,
        selector: Option<String>,
        #[arg(long)]
        session: Option<String>,
    },
    Mark {
        name: String,
        path: Option<String>,
    },
    Unmark {
        name: String,
    },
    Bookmarks {
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
        Commands::Init {
            shell,
            command_not_found,
        } => init::run_init(&shell, command_not_found),
        Commands::Resolve { query, list, json } => {
            resolve::run_resolve(&resolver, &query, list, json)
        }
        Commands::Complete { command } => complete::run_complete(&resolver, command),
        Commands::Navigate {
            mode,
            selector,
            session,
        } => complete::run_navigate(mode, selector.as_deref(), session.as_deref()),
        Commands::Mark { name, path } => bookmarks::run_mark(&name, path.as_deref()),
        Commands::Unmark { name } => bookmarks::run_unmark(&name),
        Commands::Bookmarks { json } => bookmarks::run_list(json),
        Commands::Push { path, session } => stacks::run_push(&path, session.as_deref()),
        Commands::Pop { session } => stacks::run_pop(session.as_deref()),
        Commands::Undo { session } => stacks::run_undo(session.as_deref()),
        Commands::Redo { session } => stacks::run_redo(session.as_deref()),
    }
}

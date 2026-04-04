use clap::{Parser, Subcommand};

use crate::resolve::Resolver;

mod bookmarks;
mod complete;
mod init;
mod menu;
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
        #[arg(long)]
        menu: bool,
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
    Bookmarks {
        /// Output as JSON
        #[arg(long, global = true)]
        json: bool,
        #[command(subcommand)]
        command: Option<bookmarks::BookmarksCommand>,
    },
    Stack(stacks::StackCommandArgs),
    Menu(menu::MenuCommand),
}

pub fn run() -> i32 {
    let cli = Cli::parse();
    let resolver = Resolver::from_environment();

    match cli.command {
        Commands::Init {
            shell,
            command_not_found,
            menu,
        } => init::run_init(&shell, command_not_found, menu),
        Commands::Resolve { query, list, json } => {
            resolve::run_resolve(&resolver, &query, list, json)
        }
        Commands::Complete { command } => complete::run_complete(&resolver, command),
        Commands::Navigate {
            mode,
            selector,
            session,
        } => complete::run_navigate(mode, selector.as_deref(), session.as_deref()),
        Commands::Bookmarks { json, command } => bookmarks::run_bookmarks(command, json),
        Commands::Stack(args) => stacks::run_stack(args),
        Commands::Menu(cmd) => menu::run_menu(&resolver, cmd),
    }
}

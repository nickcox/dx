use clap::{CommandFactory, Parser, Subcommand, ValueHint};

use crate::hooks::Shell;
use crate::resolve::Resolver;

mod bookmarks;
mod complete;
mod error;
mod init;
mod menu;
mod resolve;
mod stacks;

pub use error::CliError;

#[derive(Debug, Parser)]
#[command(name = "dx", version, about = "Directory navigation helper")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Resolve {
        #[arg(value_hint = ValueHint::DirPath)]
        query: String,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        json: bool,
    },
    Init {
        shell: Shell,
        #[arg(long = "command-not-found")]
        command_not_found: bool,
        #[arg(long, conflicts_with = "native_menu")]
        menu: bool,
        #[arg(long = "native-menu", conflicts_with = "menu")]
        native_menu: bool,
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

pub fn completion_script(shell: Shell) -> String {
    use clap_complete::{generate, shells};

    let mut command = Cli::command();
    let mut output = Vec::new();
    match shell {
        Shell::Bash => generate(shells::Bash, &mut command, "dx", &mut output),
        Shell::Zsh => generate(shells::Zsh, &mut command, "dx", &mut output),
        Shell::Fish => generate(shells::Fish, &mut command, "dx", &mut output),
        Shell::Pwsh => {
            generate(shells::PowerShell, &mut command, "dx", &mut output);
        }
    }
    let script = String::from_utf8(output).expect("clap completion scripts are UTF-8");

    match shell {
        // Taken verbatim. `dx` deliberately exposes no hidden arguments, because
        // `clap_complete` ignores `Arg::hide` and would offer them as
        // completions — see `MenuCommand::psreadline_mode`.
        Shell::Bash | Shell::Zsh | Shell::Fish => script,
        // Clap's PowerShell script compares the final AST token with the
        // completion word. PowerShell may pass a differently quoted word, so
        // always treat the final token as the incomplete value. The `using`
        // statements it emits are relocated by the hook assembler rather than
        // rewritten away — see `hooks::pwsh::hoist_using_statements`.
        Shell::Pwsh => patch(
            script,
            "$element.Value -eq $wordToComplete)",
            "$i -eq ($commandElements.Count - 1))",
        ),
    }
}

/// Rewrites clap's generated completion script.
///
/// # Panics
///
/// Panics when `needle` is absent. A silently skipped replacement would ship a
/// subtly broken completion script; failing loudly turns a clap upgrade into a
/// test failure instead, since the `hooks` tests generate every shell's script.
fn patch(script: String, needle: &str, replacement: &str) -> String {
    assert!(
        script.contains(needle),
        "clap completion script no longer contains {needle:?}; \
         the patch in cli::completion_script needs updating for this clap version"
    );
    script.replace(needle, replacement)
}

pub fn run() -> i32 {
    match dispatch(Cli::parse()) {
        Ok(()) => 0,
        Err(error) => {
            if !error.is_silent() {
                eprintln!("{error}");
            }
            1
        }
    }
}

fn dispatch(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Commands::Init {
            shell,
            command_not_found,
            menu,
            native_menu,
        } => init::run_init(shell, command_not_found, menu, native_menu),
        Commands::Resolve { query, list, json } => {
            with_resolver(|resolver| resolve::run_resolve(resolver, &query, list, json))
        }
        Commands::Complete { command } => {
            with_resolver(|resolver| complete::run_complete(resolver, command))
        }
        Commands::Navigate {
            mode,
            selector,
            session,
        } => complete::run_navigate(mode, selector.as_deref(), session.as_deref()),
        Commands::Bookmarks { json, command } => bookmarks::run_bookmarks(command, json),
        Commands::Stack(args) => stacks::run_stack(args),
        Commands::Menu(cmd) => with_resolver(|resolver| menu::run_menu(resolver, cmd)),
    }
}

fn with_resolver(run: impl FnOnce(&Resolver) -> Result<(), CliError>) -> Result<(), CliError> {
    run(&Resolver::from_environment()?)
}

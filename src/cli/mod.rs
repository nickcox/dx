use clap::{CommandFactory, Parser, Subcommand, ValueHint};

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
        #[arg(value_hint = ValueHint::DirPath)]
        query: String,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        json: bool,
    },
    Init {
        shell: init::InitShell,
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

pub fn completion_script(shell: crate::hooks::Shell) -> String {
    use clap_complete::{generate, shells};

    let mut command = Cli::command();
    let mut output = Vec::new();
    match shell {
        crate::hooks::Shell::Bash => generate(shells::Bash, &mut command, "dx", &mut output),
        crate::hooks::Shell::Zsh => generate(shells::Zsh, &mut command, "dx", &mut output),
        crate::hooks::Shell::Fish => generate(shells::Fish, &mut command, "dx", &mut output),
        crate::hooks::Shell::Pwsh => {
            generate(shells::PowerShell, &mut command, "dx", &mut output);
        }
    }
    let script = String::from_utf8(output).expect("clap completion scripts are UTF-8");

    match shell {
        crate::hooks::Shell::Bash => script.replace(" --psreadline-mode", ""),
        crate::hooks::Shell::Zsh | crate::hooks::Shell::Fish => script
            .lines()
            .filter(|line| !line.contains("psreadline-mode"))
            .collect::<Vec<_>>()
            .join("\n"),
        crate::hooks::Shell::Pwsh => script
            .lines()
            .filter(|line| !line.contains("psreadline-mode"))
            .collect::<Vec<_>>()
            .join("\n")
            .replace("using namespace System.Management.Automation\n", "")
            .replace(
                "using namespace System.Management.Automation.Language\n",
                "",
            )
            .replace(
                "[CompletionResult]",
                "[System.Management.Automation.CompletionResult]",
            )
            .replace(
                "[CompletionResultType]",
                "[System.Management.Automation.CompletionResultType]",
            )
            .replace(
                "[StringConstantExpressionAst]",
                "[System.Management.Automation.Language.StringConstantExpressionAst]",
            )
            .replace(
                "[StringConstantType]",
                "[System.Management.Automation.Language.StringConstantType]",
            )
            // Clap's PowerShell script compares the final AST token with the
            // completion word. PowerShell may pass a differently quoted word,
            // so always treat the final token as the incomplete value.
            .replace(
                "$element.Value -eq $wordToComplete)",
                "$i -eq ($commandElements.Count - 1))",
            ),
    }
}

pub fn run() -> i32 {
    let cli = Cli::parse();

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

fn with_resolver(run: impl FnOnce(&Resolver) -> i32) -> i32 {
    match Resolver::from_environment() {
        Ok(resolver) => run(&resolver),
        Err(error) => {
            eprintln!("dx: {error}");
            1
        }
    }
}

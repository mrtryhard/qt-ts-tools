use crate::extract::{extract_main, ExtractArgs};
use crate::merge::{merge_main, MergeArgs};
use crate::sort::{sort_main, SortArgs};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Generator, Shell};
use clap_complete_nushell::Nushell;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[command(name = env!("CARGO_PKG_NAME"), about, version)]
enum Commands {
    Sort(SortArgs),
    Extract(ExtractArgs),
    Merge(MergeArgs),
    /// Print a shell completion for supported shells
    #[command(name = "shell-completion")]
    ShellCompletion {
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
}

#[derive(Debug, Clone, ValueEnum)]
#[value(rename_all = "lower")]
pub enum GenShell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    PowerShell,
    Zsh,
}

impl Generator for GenShell {
    fn file_name(&self, name: &str) -> String {
        match self {
            // clap_complete
            Self::Bash => Shell::Bash.file_name(name),
            Self::Elvish => Shell::Elvish.file_name(name),
            Self::Fish => Shell::Fish.file_name(name),
            Self::PowerShell => Shell::PowerShell.file_name(name),
            Self::Zsh => Shell::Zsh.file_name(name),

            // clap_complete_nushell
            Self::Nushell => Nushell.file_name(name),
        }
    }

    fn generate(&self, cmd: &clap::Command, buf: &mut dyn std::io::prelude::Write) {
        match self {
            // clap_complete
            Self::Bash => Shell::Bash.generate(cmd, buf),
            Self::Elvish => Shell::Elvish.generate(cmd, buf),
            Self::Fish => Shell::Fish.generate(cmd, buf),
            Self::PowerShell => Shell::PowerShell.generate(cmd, buf),
            Self::Zsh => Shell::Zsh.generate(cmd, buf),

            // clap_complete_nushell
            Self::Nushell => Nushell.generate(cmd, buf),
        }
    }
}

pub fn get_cli_result() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sort(args) => sort_main(&args),
        Commands::Extract(args) => extract_main(&args),
        Commands::Merge(args) => merge_main(&args),
        Commands::ShellCompletion { shell } => {
            shell.generate(&mut Cli::command(), &mut std::io::stdout());
            Ok(())
        }
    }
}

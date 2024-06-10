use clap::{ArgAction, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Generator, Shell};
use clap_complete_nushell::Nushell;

use crate::commands::extract::{extract_main, ExtractArgs};
use crate::commands::merge::{merge_main, MergeArgs};
use crate::commands::sort::{sort_main, SortArgs};
use crate::locale::tr;

#[derive(Parser)]
#[command(author,
    version,
    about = tr("cli-about"),
    disable_help_flag = true,
    disable_help_subcommand = true,
    disable_version_flag = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, action = ArgAction::Help, help = tr("cli-help"), help_heading = tr("cli-headers-options"))]
    pub help: bool,
    #[arg(short, long, short_alias = 'v', action = ArgAction::Version, help = tr("cli-version"))]
    version: bool,
}

#[derive(Subcommand)]
#[command(subcommand_help_heading = tr("cli-headers-commands"),
    next_help_heading = tr("cli-headers-options"))]
enum Commands {
    #[command(about = tr("cli-sort-desc"))]
    Sort(SortArgs),
    #[command(about = tr("cli-extract-desc"))]
    Extract(ExtractArgs),
    #[command(about = tr("cli-merge-desc"))]
    Merge(MergeArgs),
    #[command(name = "shell-completion", about = tr("cli-shell-completion-desc"), disable_help_flag = true)]
    ShellCompletion {
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
        #[arg(short, long, action = ArgAction::Help, help = tr("cli-help"), help_heading = tr("cli-headers-options"))]
        help: bool,
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
        Commands::ShellCompletion { shell, help: _ } => {
            shell.generate(&mut Cli::command(), &mut std::io::stdout());
            Ok(())
        }
    }
}

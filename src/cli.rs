use clap::{ArgAction, Parser, Subcommand};

use crate::commands::extract::{ExtractArgs, extract_main};
use crate::commands::merge::{MergeArgs, merge_main};
use crate::commands::release::{ReleaseArgs, release_main};
use crate::commands::shell_completion::{ShellCompletionArgs, shell_completion_main};
use crate::commands::sort::{SortArgs, sort_main};
use crate::commands::stat::{StatArgs, stat_main};
use crate::commands::strip::{StripArgs, strip_main};
use crate::locale::tr;

#[derive(Parser)]
#[command(author,
    version,
    about = tr!("cli-about"),
    disable_help_flag = true,
    disable_help_subcommand = true,
    disable_version_flag = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, action = ArgAction::Help, help = tr!("cli-help"), help_heading = tr!("cli-headers-options"))]
    pub help: Option<bool>,
    #[arg(short, long, short_alias = 'v', action = ArgAction::Version, help = tr!("cli-version"))]
    version: Option<bool>,
}

#[derive(Subcommand)]
#[command(subcommand_help_heading = tr!("cli-headers-commands"),
    next_help_heading = tr!("cli-headers-options"))]
enum Commands {
    #[command(about = tr!("cli-extract-desc"))]
    Extract(ExtractArgs),
    #[command(about = tr!("cli-merge-desc"))]
    Merge(MergeArgs),
    #[command(about = tr!("cli-release-desc"))]
    Release(ReleaseArgs),
    #[command(about = tr!("cli-sort-desc"))]
    Sort(SortArgs),
    #[command(about = tr!("cli-stat-desc"))]
    Stat(StatArgs),
    #[command(about = tr!("cli-strip-desc"))]
    Strip(StripArgs),
    // Want to have shell-completion as the very last option displayed
    #[command(name = "shell-completion", about = tr!("cli-shell-completion-desc"))]
    ShellCompletion(ShellCompletionArgs),
}

pub fn get_cli_result() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Extract(args) => extract_main(&args),
        Commands::Merge(args) => merge_main(&args),
        Commands::Release(args) => release_main(&args),
        Commands::Sort(args) => sort_main(&args),
        Commands::Stat(args) => stat_main(&args),
        Commands::Strip(args) => strip_main(&args),
        Commands::ShellCompletion(args) => shell_completion_main(&args),
    }
}

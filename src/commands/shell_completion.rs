use clap::{ArgAction, Args, CommandFactory, ValueEnum};
use clap_complete::{Generator, Shell};
use clap_complete_nushell::Nushell;

use crate::cli::Cli;
use crate::locale::tr;

#[derive(Args)]
#[command(disable_help_flag = true)]
pub struct ShellCompletionArgs {
    #[arg(value_enum)]
    shell: clap_complete_command::Shell,
    #[arg(short, long, action = ArgAction::Help, help = tr("cli-help"), help_heading = tr("cli-headers-options"))]
    help: bool,
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

pub fn shell_completion_main(args: &ShellCompletionArgs) -> Result<(), String> {
    args.shell
        .generate(&mut Cli::command(), &mut std::io::stdout());

    Ok(())
}

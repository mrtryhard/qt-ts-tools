use std::io::Write;

use clap::{ArgAction, Args, CommandFactory, ValueEnum};
use clap_complete::{Generator, Shell};
use clap_complete_nushell::Nushell;

use crate::cli::Cli;
use crate::locale::{tr, tr_args};

#[derive(Args)]
#[command(disable_help_flag = true)]
pub struct ShellCompletionArgs {
    #[arg(value_enum, help = tr("cli-shell-completion-shell"))]
    shell: clap_complete_command::Shell,
    #[arg(short, long, help = tr("cli-shell-completion-install"))]
    output_path: Option<String>,
    #[arg(short, long, action = ArgAction::Help, help = tr("cli-help"), help_heading = tr("cli-headers-options"))]
    help: Option<bool>,
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
    let mut buf = Vec::<u8>::new();

    {
        let mut writer = std::io::BufWriter::new(&mut buf);
        args.shell.generate(&mut Cli::command(), &mut writer);
    }

    match &args.output_path {
        None => match &mut buf.is_empty() {
            true => Err(tr_args(
                "cli-shell-completion-error-get-shell",
                [("shell", format!("{:?}", args.shell).into())].into(),
            )),
            false => Ok(()),
        },
        Some(output_path) => write_to_file(&mut buf, output_path),
    }
}

fn write_to_file(buf: &mut [u8], output_path: &String) -> Result<(), String> {
    let file = std::fs::File::create(output_path);

    match file {
        Ok(mut file) => match file.write(buf) {
            Ok(sz) => {
                if sz == buf.len() {
                    Ok(())
                } else {
                    Err(tr_args(
                        "cli-shell-completion-error-write-to-file",
                        [("file", output_path.into())].into(),
                    ))
                }
            }
            Err(err) => Err(tr_args(
                "cli-shell-completion-error-write-privilege",
                [("error", err.to_string().into())].into(),
            )),
        },
        Err(err) => Err(tr_args(
            "cli-shell-completion-error-open",
            [("error", err.to_string().into())].into(),
        )),
    }
}

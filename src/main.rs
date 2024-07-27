use crate::cli::get_cli_result;
use crate::locale::initialize_locale;
use crate::logging::initialize_logging;
use i18n_embed::LanguageLoader;
use log::*;

mod cli;
mod commands;
mod locale;
mod logging;
mod ts;

fn main() {
    initialize_locale();
    initialize_logging();

    debug!(
        "Using localization language: {}",
        locale::current_loader().current_language()
    );

    if let Err(e) = get_cli_result() {
        error!("Command returned error: {e}");
        eprintln!("{e}");
        std::process::exit(1);
    }

    info!("Tool exits normally");
}

use log::*;

use crate::cli::get_cli_result;
use crate::logging::initialize_logging;

mod cli;
mod commands;
mod locale;
mod logging;
mod ts;

fn main() {
    initialize_logging();

    debug!(
        "Using localization language: {}",
        locale::current_lang().language.to_string()
    );

    if let Err(e) = get_cli_result() {
        error!("Command returned error: {e}");
        eprintln!("{e}");
        std::process::exit(1);
    }

    info!("Tool exits normally");
}

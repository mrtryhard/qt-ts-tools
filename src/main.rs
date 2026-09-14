use i18n_embed::LanguageLoader;
use log::*;
use qt_ts_tools::cli::get_cli_result;
use qt_ts_tools::locale::{self, initialize_locale};
use qt_ts_tools::logging::initialize_logging;

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

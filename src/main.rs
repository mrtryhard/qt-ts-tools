use tracing::{debug, error, info};

use crate::cli::get_cli_result;

mod cli;
mod extract;
mod locale;
mod merge;
mod sort;
mod ts;

fn main() {
    initialize_logging();

    debug!(
        "Using localization language: {}",
        locale::CURRENT_LANG.language.to_string()
    );

    if let Err(e) = get_cli_result() {
        error!("Command returned error: {e}");
        println!("{e}");
        std::process::exit(1);
    }

    info!("Tool exits normally");
}

/// Initializes the logging in the application.
/// Logging will only be active when environment variable `RUST_LOG` is set.
/// ### Example
/// **Activating log**
/// ```bash
/// RUST_LOG="trace" ./qt-ts-tools
/// ```
/// **Setting the output directory**
/// ```bash
/// RUST_LOG="trace" LOG_DIR=. ./qt-ts-tools
/// ```
///
/// ### Output
/// A file name `qt_ts_tools.log` should be output at `LOG_DIR` location.
fn initialize_logging() {
    if let Ok(log_level) = std::env::var("RUST_LOG") {
        let crate_name = env!("CARGO_PKG_NAME").replace('-', "_");

        let log_file = std::env::var("LOG_DIR").unwrap_or(".".to_owned());
        let file_appender = tracing_appender::rolling::never(log_file, format!("{crate_name}.log"));

        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::new(format!(
                "{}={log_level}",
                crate_name
            )))
            .with_writer(file_appender)
            .with_ansi(false)
            .pretty()
            .init();
    }
}

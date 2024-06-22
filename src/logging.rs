use std::fs::File;
use std::str::FromStr;

use log::LevelFilter;

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
pub fn initialize_logging() {
    if let Ok(log_level) = std::env::var("RUST_LOG") {
        let mut crate_name = env!("CARGO_PKG_NAME").replace('-', "_");
        crate_name.push_str(".log");

        let log_file = std::path::PathBuf::new()
            .join(std::env::var("LOG_DIR").unwrap_or("./".to_owned()))
            .join(crate_name);

        let target = Box::new(File::create(log_file).expect("Can't create file"));

        env_logger::Builder::new()
            .target(env_logger::Target::Pipe(target))
            .filter(
                None,
                LevelFilter::from_str(&log_level).unwrap_or(LevelFilter::Off),
            )
            .init();
    }
}

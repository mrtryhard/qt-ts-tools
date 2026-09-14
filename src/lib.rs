//! Library facade of `qt-ts-tools`.
//!
//! The command line application lives in `src/main.rs` and is a thin wrapper around
//! the modules exposed here. Exposing them as a library also allows the benchmarks
//! in `benches/` to exercise the parsing, transformation and serialization code
//! without going through the process boundary.

pub mod cli;
pub mod commands;
pub mod locale;
pub mod logging;
pub mod parser;

#![allow(dead_code)]

//! Logging module for SpecGen CLI.
//!
//! Provides structured logging with configurable levels and formats.
//! API key is always redacted from logs.

use std::io::IsTerminal;
use tracing::Level;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

/// Initialize the logging subsystem.
pub fn init(level: Level, log_format: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("specgen=info"))
        .add_directive(level.into());

    match log_format {
        "json" => {
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_thread_ids(false)
                        .with_file(true)
                        .with_line_number(true)
                        .json(),
                )
                .init();
        }
        _ => {
            let ansi = std::io::stdout().is_terminal();
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_thread_ids(false)
                        .with_file(true)
                        .with_line_number(true)
                        .with_ansi(ansi),
                )
                .init();
        }
    }
}

/// Convert CLI verbosity to tracing level.
pub fn verbosity_to_level(verbose: u8) -> Level {
    match verbose {
        0 => Level::ERROR,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    }
}

/// Redact sensitive data from strings for safe logging.
pub fn redact_api_key(value: &str) -> String {
    if value.len() > 8 {
        format!("{}...{}", &value[..4], &value[value.len() - 4..])
    } else {
        "***".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_to_level() {
        assert_eq!(verbosity_to_level(0), Level::ERROR);
        assert_eq!(verbosity_to_level(1), Level::INFO);
        assert_eq!(verbosity_to_level(2), Level::DEBUG);
        assert_eq!(verbosity_to_level(3), Level::TRACE);
    }

    #[test]
    fn test_redact_api_key() {
        let key = "abcdefghijklmnop";
        let redacted = redact_api_key(key);
        assert!(redacted.starts_with("abcd"));
        assert!(redacted.ends_with("nop"));
        assert_eq!(redacted.len(), 11); // abcd + ... + nop = 4 + 3 + 4 = 11

        let short = "abc";
        assert_eq!(redact_api_key(short), "***");
    }
}

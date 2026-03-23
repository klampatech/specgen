//! Terminal UI for SpecGen CLI using Ratatui.
//!
//! This module provides the TUI components for the interactive interview
//! and spec generation workflow.

#![allow(dead_code)]
#![allow(clippy::uninlined_format_args)]

pub mod theme;

use crate::error::SpecGenError;

/// TUI application result type.
pub type TuiResult<T> = Result<T, SpecGenError>;

/// Application state for the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AppState {
    /// Initial state - waiting for user input.
    Idle,
    /// Detecting domain.
    DetectingDomain,
    /// Running interview.
    Interview,
    /// Generating specs.
    Generating,
    /// Showing diff/merge.
    Diffing,
    /// Complete.
    Complete,
}

/// Main TUI application.
#[derive(Debug)]
#[allow(dead_code)]
pub struct App {
    /// Current state.
    pub state: AppState,
    /// Current question number.
    pub question_num: usize,
    /// Total questions.
    pub question_total: usize,
    /// Current section being generated.
    pub current_section: Option<String>,
}

impl App {
    /// Create a new application.
    pub fn new() -> Self {
        App {
            state: AppState::Idle,
            question_num: 0,
            question_total: 0,
            current_section: None,
        }
    }

    /// Set the state.
    pub fn set_state(&mut self, state: AppState) {
        self.state = state;
    }

    /// Update question progress.
    pub fn set_progress(&mut self, current: usize, total: usize) {
        self.question_num = current;
        self.question_total = total;
    }

    /// Set the current section being generated.
    pub fn set_current_section(&mut self, section: impl Into<String>) {
        // Note: In real implementation, would store section name
        let _ = section.into();
    }

    /// Check if terminal supports color.
    pub fn supports_color() -> bool {
        // Check NO_COLOR and TERM environment variables
        std::env::var("NO_COLOR").is_err()
            && std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Format progress string.
#[allow(dead_code)]
pub fn format_progress(current: usize, total: usize) -> String {
    format!("Question {} of ~{}", current, total)
}

/// Format progress bar.
#[allow(dead_code)]
pub fn format_progress_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64) as usize;
    let empty = width - filled;

    format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
}

/// Check if running in a terminal.
#[allow(dead_code)]
pub fn is_terminal() -> bool {
    atty::is(atty::Stream::Stdin)
}

/// Initialize the terminal for TUI mode.
#[allow(dead_code)]
pub fn init_terminal() -> Result<(), SpecGenError> {
    // Setup terminal - in a full implementation, would use ratatui::Terminal
    if !App::supports_color() {
        return Err(SpecGenError::UiError(
            "Terminal does not support color output".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert_eq!(app.state, AppState::Idle);
    }

    #[test]
    fn test_format_progress() {
        let progress = format_progress(3, 10);
        assert_eq!(progress, "Question 3 of ~10");
    }

    #[test]
    fn test_format_progress_bar() {
        let bar = format_progress_bar(50.0, 10);
        assert_eq!(bar.len(), 12); // [=====     ]
    }

    #[test]
    fn test_theme_colors() {
        // Test that theme colors are accessible
        let primary = theme::accent_primary();
        assert!(matches!(primary, ratatui::style::Color::Rgb(_, _, _)));
    }
}

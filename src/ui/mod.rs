//! Terminal UI for SpecGen CLI using Ratatui.
//!
//! This module provides the TUI components for the interactive interview
//! and spec generation workflow.

#![allow(dead_code)]
#![allow(clippy::uninlined_format_args)]

pub mod theme;

use std::io;
use std::sync::Arc;

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use tokio::sync::mpsc;

use crate::error::SpecGenError;
use crate::interview::answers::Answer;
use crate::interview::orchestrator::InterviewSession;
use crate::interview::questions::Question;
use crate::spec::SpecSection;

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

/// Events that can occur in the TUI.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TuiEvent {
    /// User submitted an answer.
    SubmitAnswer(String),
    /// User wants to skip the current question.
    SkipQuestion,
    /// User pressed a key to navigate.
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    /// User wants to quit.
    Quit,
    /// Window was resized.
    Resize(u16, u16),
    /// Tick event for animations.
    Tick,
}

/// Section progress tracking.
#[derive(Debug, Clone)]
pub struct SectionProgress {
    /// Section name.
    pub name: String,
    /// Display name.
    pub display_name: String,
    /// Current status.
    pub status: theme::ProgressStatus,
    /// Current content (for preview).
    pub content: Option<String>,
}

impl SectionProgress {
    /// Create a new section progress tracker.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            display_name: Self::format_name(name),
            status: theme::ProgressStatus::Queued,
            content: None,
        }
    }

    /// Format section name for display.
    fn format_name(name: &str) -> String {
        name.replace('_', " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
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
    /// Input buffer for answer.
    input_buffer: String,
    /// Current question (if in interview).
    current_question: Option<Question>,
    /// Interview session (if active).
    interview_session: Option<InterviewSession>,
    /// Section progress for generation.
    section_progress: Vec<SectionProgress>,
    /// Preview content.
    preview_content: String,
    /// Focused panel (0 = interview, 1 = progress, 2 = preview).
    focused_panel: usize,
    /// Message to display (errors, hints).
    message: Option<String>,
    /// Channel for events.
    event_tx: Option<mpsc::Sender<TuiEvent>>,
}

impl App {
    /// Create a new application.
    pub fn new() -> Self {
        App {
            state: AppState::Idle,
            question_num: 0,
            question_total: 0,
            current_section: None,
            input_buffer: String::new(),
            current_question: None,
            interview_session: None,
            section_progress: Vec::new(),
            preview_content: String::new(),
            focused_panel: 0,
            message: None,
            event_tx: None,
        }
    }

    /// Set the event channel.
    pub fn set_event_channel(&mut self, tx: mpsc::Sender<TuiEvent>) {
        self.event_tx = Some(tx);
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
        self.current_section = Some(section.into());
    }

    /// Check if terminal supports color.
    pub fn supports_color() -> bool {
        !theme::use_plain_text()
    }

    /// Start an interview session.
    pub fn start_interview(&mut self, idea: String, domain: crate::domain::Domain) {
        self.state = AppState::Interview;
        self.interview_session = Some(InterviewSession::new(idea, domain));
        if let Some(session) = &self.interview_session {
            let (current, total) = session.progress();
            self.question_num = current;
            self.question_total = total;
            self.current_question = session.current_question().cloned();
        }
    }

    /// Submit an answer to the interview.
    pub fn submit_answer(&mut self, answer_text: &str) -> Result<bool, SpecGenError> {
        if let Some(session) = self.interview_session.as_mut() {
            let answer = if answer_text.trim().is_empty() || answer_text.to_lowercase() == "skip" {
                // Skip question
                if let Some(ref q) = self.current_question {
                    Answer::skipped(q.id)
                } else {
                    return Err(SpecGenError::Unexpected("No active question".to_string()));
                }
            } else {
                Answer::new(
                    self.current_question
                        .as_ref()
                        .map(|q| q.id)
                        .unwrap_or_default(),
                    answer_text.to_string(),
                )
            };

            session.submit_answer(answer)?;

            // Update progress
            let (current, total) = session.progress();
            self.question_num = current;
            self.question_total = total;
            self.current_question = session.current_question().cloned();

            // Check if interview is complete
            Ok(session.completed)
        } else {
            Err(SpecGenError::Unexpected(
                "No active interview session".to_string(),
            ))
        }
    }

    /// Get interview answers.
    pub fn get_answers(&self) -> Vec<Answer> {
        self.interview_session
            .as_ref()
            .map(|s| s.answers.clone())
            .unwrap_or_default()
    }

    /// Initialize section progress tracking.
    pub fn init_sections(&mut self) {
        self.section_progress = SpecSection::all()
            .iter()
            .map(|s| SectionProgress::new(s.filename().trim_end_matches(".md")))
            .collect();
    }

    /// Update section status.
    pub fn set_section_status(&mut self, section_name: &str, status: theme::ProgressStatus) {
        if let Some(progress) = self
            .section_progress
            .iter_mut()
            .find(|p| p.name == section_name)
        {
            progress.status = status;
        }
    }

    /// Update section content for preview.
    pub fn set_section_content(&mut self, section_name: &str, content: &str) {
        if let Some(progress) = self
            .section_progress
            .iter_mut()
            .find(|p| p.name == section_name)
        {
            progress.content = Some(content.to_string());
        }
    }

    /// Get current preview content.
    pub fn get_preview(&self) -> &str {
        &self.preview_content
    }

    /// Set preview content.
    pub fn set_preview(&mut self, content: &str) {
        self.preview_content = content.to_string();
    }

    /// Get input buffer.
    pub fn get_input(&self) -> &str {
        &self.input_buffer
    }

    /// Set input buffer.
    pub fn set_input(&mut self, input: &str) {
        self.input_buffer = input.to_string();
    }

    /// Clear input buffer.
    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
    }

    /// Append to input buffer.
    pub fn append_input(&mut self, text: &str) {
        self.input_buffer.push_str(text);
    }

    /// Set message to display.
    pub fn set_message(&mut self, msg: Option<String>) {
        self.message = msg;
    }

    /// Focus next panel.
    pub fn focus_next_panel(&mut self) {
        self.focused_panel = (self.focused_panel + 1) % 3;
    }

    /// Focus previous panel.
    pub fn focus_previous_panel(&mut self) {
        self.focused_panel = if self.focused_panel == 0 {
            2
        } else {
            self.focused_panel - 1
        };
    }

    /// Get focused panel index.
    pub fn focused_panel(&self) -> usize {
        self.focused_panel
    }

    /// Run the TUI application.
    pub async fn run(
        mut self,
        idea: String,
        domain: crate::domain::Domain,
        _client: Arc<dyn crate::ai::client::AiClient>,
    ) -> Result<Vec<Answer>, SpecGenError> {
        // Create event channel
        let (tx, mut rx) = mpsc::channel::<TuiEvent>(100);
        self.set_event_channel(tx);

        // Initialize terminal - use setup to handle raw mode automatically
        let backend = CrosstermBackend::new(io::stderr());
        let mut terminal = Terminal::new(backend)?;

        // Start the interview
        self.start_interview(idea, domain);
        self.init_sections();

        // Render loop
        loop {
            // Draw the UI
            terminal.draw(|f| self.draw(f))?;

            // Handle events
            match rx.recv().await {
                Some(TuiEvent::SubmitAnswer(text)) => {
                    let completed = self.submit_answer(&text)?;
                    self.clear_input();

                    if completed {
                        // Interview complete - break and return answers
                        break;
                    }
                }
                Some(TuiEvent::SkipQuestion) => {
                    let _ = self.submit_answer("skip");
                }
                Some(TuiEvent::Quit) => {
                    break;
                }
                Some(TuiEvent::NavigateUp) => {
                    self.focus_previous_panel();
                }
                Some(TuiEvent::NavigateDown) => {
                    self.focus_next_panel();
                }
                Some(TuiEvent::Tick) => {
                    // Animation tick - could update spinners here
                }
                None => {
                    // Channel closed
                    break;
                }
                _ => {}
            }
        }

        // Terminal drop will restore alternate screen automatically
        Ok(self.get_answers())
    }

    /// Draw the UI.
    fn draw(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Header
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // Footer
            ])
            .split(f.size());

        // Draw header
        self.draw_header(f, chunks[0]);

        // Draw main content (three panels)
        self.draw_main_content(f, chunks[1]);

        // Draw footer
        self.draw_footer(f, chunks[2]);
    }

    /// Draw the header.
    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let title = match self.state {
            AppState::Idle => "SpecGen CLI",
            AppState::DetectingDomain => "Detecting Domain...",
            AppState::Interview => "Interview",
            AppState::Generating => "Generating Specifications",
            AppState::Diffing => "Reviewing Changes",
            AppState::Complete => "Complete",
        };

        let block = Block::new()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::border_inactive()));

        f.render_widget(block, area);
    }

    /// Draw the main content area with three panels.
    fn draw_main_content(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33), // Interview panel
                Constraint::Percentage(33), // Progress panel
                Constraint::Percentage(34), // Preview panel
            ])
            .split(area);

        // Draw each panel
        self.draw_interview_panel(f, chunks[0]);
        self.draw_progress_panel(f, chunks[1]);
        self.draw_preview_panel(f, chunks[2]);
    }

    /// Draw the interview panel (left).
    fn draw_interview_panel(&self, f: &mut Frame, area: Rect) {
        let border_style = if self.focused_panel == 0 {
            Style::default().fg(theme::border_active())
        } else {
            Style::default().fg(theme::border_inactive())
        };

        let block = Block::new()
            .title("Interview")
            .borders(Borders::ALL)
            .border_style(border_style);

        // If we have a current question, show it
        let content = if let Some(ref question) = self.current_question {
            let progress_text =
                format!("Question {} of ~{}", self.question_num, self.question_total);
            let question_text = format!("\n{}\n\n{}", progress_text, question.text);

            let input_hint = if self.input_buffer.is_empty() {
                "Type your answer... (Tab to skip)"
            } else {
                "Press Enter to submit"
            };

            format!(
                "{}\n\nAnswer: {}\n\n{}",
                question_text,
                if self.input_buffer.is_empty() {
                    "[waiting for input...]".to_string()
                } else {
                    self.input_buffer.clone()
                },
                input_hint
            )
        } else if self.state == AppState::Complete {
            "Interview complete!".to_string()
        } else {
            "Starting interview...".to_string()
        };

        let paragraph = Paragraph::new(content)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(theme::text_primary()));

        f.render_widget(&block, area);
        f.render_widget(paragraph, block.inner(area));
    }

    /// Draw the progress panel (center).
    fn draw_progress_panel(&self, f: &mut Frame, area: Rect) {
        let border_style = if self.focused_panel == 1 {
            Style::default().fg(theme::border_active())
        } else {
            Style::default().fg(theme::border_inactive())
        };

        let block = Block::new()
            .title("Generation Progress")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);

        // Build progress lines
        let mut lines = Vec::new();
        for progress in &self.section_progress {
            let indicator = progress.status.indicator();
            let indicator_color = progress.status.color();
            let status_text = match progress.status {
                theme::ProgressStatus::Complete => "Done",
                theme::ProgressStatus::InProgress => "Generating...",
                theme::ProgressStatus::Queued => "Queued",
                theme::ProgressStatus::Error => "Failed",
            };

            lines.push(Line::from(vec![
                Span::raw(indicator).fg(indicator_color),
                Span::raw(" "),
                Span::raw(&progress.display_name),
                Span::raw(" "),
                Span::raw(status_text).fg(theme::text_muted()),
            ]));
        }

        let paragraph = Paragraph::new(lines).style(Style::default().fg(theme::text_primary()));

        f.render_widget(block, area);
        f.render_widget(paragraph, inner);
    }

    /// Draw the preview panel (right).
    fn draw_preview_panel(&self, f: &mut Frame, area: Rect) {
        let border_style = if self.focused_panel == 2 {
            Style::default().fg(theme::border_active())
        } else {
            Style::default().fg(theme::border_inactive())
        };

        let block = Block::new()
            .title("Preview")
            .borders(Borders::ALL)
            .border_style(border_style);

        let content = if self.preview_content.is_empty() {
            "No content to preview yet...".to_string()
        } else {
            // Truncate long content for display
            let preview: String = self.preview_content.chars().take(500).collect();
            if self.preview_content.len() > 500 {
                format!("{}\n\n... (truncated)", preview)
            } else {
                preview
            }
        };

        let paragraph = Paragraph::new(content)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(theme::text_primary()));

        f.render_widget(&block, area);
        f.render_widget(paragraph, block.inner(area));
    }

    /// Draw the footer with key bindings.
    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::border_inactive()));

        // Build key binding display
        let bindings = [
            ("Enter", "Submit"),
            ("Tab", "Skip"),
            ("↑↓", "Navigate"),
            ("?", "Help"),
            ("Ctrl-C", "Quit"),
        ];

        let mut spans = Vec::new();
        for (i, (key, desc)) in bindings.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" | "));
            }
            spans.push(Span::raw(*key).fg(theme::accent_primary()));
            spans.push(Span::raw(" "));
            spans.push(Span::raw(*desc).fg(theme::text_muted()));
        }

        let line = Line::from(spans).centered();
        let paragraph = Paragraph::new(line).style(Style::default().fg(theme::text_secondary()));

        f.render_widget(block, area);
        f.render_widget(paragraph, area);
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

    #[test]
    fn test_section_progress_new() {
        let progress = SectionProgress::new("requirements");
        assert_eq!(progress.name, "requirements");
        assert_eq!(progress.display_name, "Requirements");
        assert_eq!(progress.status, theme::ProgressStatus::Queued);
    }

    #[test]
    fn test_app_input_buffer() {
        let mut app = App::new();
        app.append_input("Hello ");
        app.append_input("World");
        assert_eq!(app.get_input(), "Hello World");
        app.clear_input();
        assert_eq!(app.get_input(), "");
    }
}

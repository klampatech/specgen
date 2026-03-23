//! Session persistence for SpecGen CLI.
//!
//! This module handles reading and writing session state to disk.

#![allow(dead_code)]
#![allow(clippy::uninlined_format_args)]

use crate::error::SpecGenError;
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::SystemTime;

/// Session file location relative to project root.
const SESSION_DIR: &str = ".specgen";
const SESSION_FILE: &str = "session.json";

/// Session schema version.
const SESSION_VERSION: &str = "1";

/// Session data structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Session {
    /// Schema version.
    pub version: String,
    /// Creation timestamp.
    pub created_at: u64,
    /// Last update timestamp.
    pub updated_at: u64,
    /// Original project idea.
    pub idea: String,
    /// Detected domain.
    pub domain: String,
    /// Interview Q&A pairs.
    pub interview: Vec<InterviewEntry>,
    /// Generated sections.
    pub generated_sections: Vec<String>,
    /// Refinement history.
    pub refinement_history: Vec<RefinementEntry>,
}

/// A single interview question-answer entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct InterviewEntry {
    /// The question text.
    pub question: String,
    /// The answer text.
    pub answer: String,
    /// Whether this was assumed by AI.
    pub assumed: bool,
}

/// A refinement history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RefinementEntry {
    /// Timestamp.
    pub timestamp: u64,
    /// Instruction.
    pub instruction: String,
    /// Sections refined.
    pub sections: Vec<String>,
}

impl Session {
    /// Create a new session.
    pub fn new(idea: String, domain: String) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Session {
            version: SESSION_VERSION.to_string(),
            created_at: now,
            updated_at: now,
            idea,
            domain,
            interview: Vec::new(),
            generated_sections: Vec::new(),
            refinement_history: Vec::new(),
        }
    }

    /// Add an interview entry.
    pub fn add_interview_entry(&mut self, question: String, answer: String, assumed: bool) {
        self.interview.push(InterviewEntry {
            question,
            answer,
            assumed,
        });
        self.update_timestamp();
    }

    /// Mark a section as generated.
    pub fn mark_section_generated(&mut self, section: &str) {
        if !self.generated_sections.contains(&section.to_string()) {
            self.generated_sections.push(section.to_string());
        }
        self.update_timestamp();
    }

    /// Add a refinement entry.
    pub fn add_refinement(&mut self, instruction: String, sections: Vec<String>) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.refinement_history.push(RefinementEntry {
            timestamp: now,
            instruction,
            sections,
        });
        self.update_timestamp();
    }

    /// Update the timestamp.
    fn update_timestamp(&mut self) {
        self.updated_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(self.updated_at);
    }
}

/// Get the session file path for a project directory.
#[allow(dead_code)]
pub fn get_session_path(project_dir: &Utf8Path) -> Utf8PathBuf {
    project_dir.join(SESSION_DIR).join(SESSION_FILE)
}

/// Check if a session exists for a project.
#[allow(dead_code)]
pub fn session_exists(project_dir: &Utf8Path) -> bool {
    get_session_path(project_dir).exists()
}

/// Load a session from disk.
#[allow(dead_code)]
pub fn load_session(project_dir: &Utf8Path) -> Result<Session, SpecGenError> {
    let path = get_session_path(project_dir);

    if !path.exists() {
        return Err(SpecGenError::SessionError("No session found".to_string()));
    }

    let content = fs::read_to_string(&path)?;
    let session: Session = serde_json::from_str(&content)
        .map_err(|e| SpecGenError::SessionError(format!("Failed to parse session: {e}")))?;

    Ok(session)
}

/// Save a session to disk atomically.
#[allow(dead_code)]
pub fn save_session(project_dir: &Utf8Path, session: &Session) -> Result<(), SpecGenError> {
    let session_dir = project_dir.join(SESSION_DIR);

    // Create directory if it doesn't exist
    if !session_dir.exists() {
        fs::create_dir_all(&session_dir)?;
    }

    let session_path = get_session_path(project_dir);

    // Write to temp file first
    let temp_path = session_path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(session)
        .map_err(|e| SpecGenError::SessionError(format!("Failed to serialize session: {e}")))?;

    fs::write(&temp_path, content)?;

    // Atomic rename
    fs::rename(&temp_path, &session_path)?;

    Ok(())
}

/// Get the session directory path.
#[allow(dead_code)]
pub fn get_session_dir(project_dir: &Utf8Path) -> Utf8PathBuf {
    project_dir.join(SESSION_DIR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        let session = Session::new("test idea".to_string(), "WebApp".to_string());
        assert_eq!(session.idea, "test idea");
        assert_eq!(session.domain, "WebApp");
        assert_eq!(session.version, "1");
    }

    #[test]
    fn test_session_add_entry() {
        let mut session = Session::new("test".to_string(), "WebApp".to_string());
        session.add_interview_entry("Q1".to_string(), "A1".to_string(), false);
        assert_eq!(session.interview.len(), 1);
    }

    #[test]
    fn test_session_mark_generated() {
        let mut session = Session::new("test".to_string(), "WebApp".to_string());
        session.mark_section_generated("requirements.md");
        assert!(session
            .generated_sections
            .contains(&"requirements.md".to_string()));
    }
}

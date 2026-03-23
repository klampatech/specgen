//! Answer handling and validation for the interview engine.

#[allow(dead_code)]
use crate::error::SpecGenError;
use serde::{Deserialize, Serialize};

/// Minimum words required for a valid answer.
#[allow(dead_code)]
const MIN_WORDS: usize = 3;

/// Represents an answer from the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    /// The question ID this answer responds to.
    pub question_id: u32,
    /// The answer text.
    pub text: String,
    /// Whether the user marked this as "skip" or "unsure".
    pub skipped: bool,
    /// Whether this answer was AI-assumed (for non-interactive mode).
    pub assumed: bool,
}

#[allow(dead_code)]
impl Answer {
    /// Create a new answer.
    pub fn new(question_id: u32, text: impl Into<String>) -> Self {
        Answer {
            question_id,
            text: text.into(),
            skipped: false,
            assumed: false,
        }
    }

    /// Create a skipped answer.
    pub fn skipped(question_id: u32) -> Self {
        Answer {
            question_id,
            text: String::new(),
            skipped: true,
            assumed: false,
        }
    }

    /// Create an assumed answer (for non-interactive mode).
    pub fn assumed(question_id: u32, text: impl Into<String>) -> Self {
        Answer {
            question_id,
            text: text.into(),
            skipped: false,
            assumed: true,
        }
    }

    /// Check if the answer has meaningful content.
    pub fn has_content(&self) -> bool {
        !self.text.is_empty() && !self.skipped
    }
}

/// Validate an answer against quality requirements.
///
/// Returns Ok(()) if valid, or an error with details.
pub fn validate_answer(answer: &Answer) -> Result<(), SpecGenError> {
    if answer.skipped {
        return Ok(());
    }

    let word_count = answer.text.split_whitespace().count();

    if word_count < MIN_WORDS {
        return Err(SpecGenError::InterviewError(format!(
            "Answer must be at least {MIN_WORDS} words, got {word_count}",
        )));
    }

    Ok(())
}

/// Count the number of words in text.
#[allow(dead_code)]
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Check if text meets the minimum word requirement.
#[allow(dead_code)]
pub fn meets_minimum(text: &str) -> bool {
    count_words(text) >= MIN_WORDS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_answer_new() {
        let answer = Answer::new(1, "This is a test answer");
        assert_eq!(answer.question_id, 1);
        assert_eq!(answer.text, "This is a test answer");
        assert!(!answer.skipped);
        assert!(!answer.assumed);
    }

    #[test]
    fn test_answer_skipped() {
        let answer = Answer::skipped(1);
        assert_eq!(answer.question_id, 1);
        assert!(answer.skipped);
        assert!(answer.text.is_empty());
    }

    #[test]
    fn test_answer_assumed() {
        let answer = Answer::assumed(1, "AI generated answer");
        assert!(answer.assumed);
    }

    #[test]
    fn test_validate_answer_valid() {
        let answer = Answer::new(1, "This is a valid answer with enough words");
        assert!(validate_answer(&answer).is_ok());
    }

    #[test]
    fn test_validate_answer_skipped() {
        let answer = Answer::skipped(1);
        assert!(validate_answer(&answer).is_ok());
    }

    #[test]
    fn test_validate_answer_too_short() {
        let answer = Answer::new(1, "Too short");
        let result = validate_answer(&answer);
        assert!(result.is_err());
    }

    #[test]
    fn test_count_words() {
        assert_eq!(count_words("one two three"), 3);
        assert_eq!(count_words("  leading and trailing  "), 3);
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn test_meets_minimum() {
        assert!(meets_minimum("one two three"));
        assert!(!meets_minimum("one two"));
    }
}

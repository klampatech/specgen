//! Interview orchestrator for managing the Q/A flow.

use crate::domain::Domain;
use crate::error::SpecGenError;
use serde::{Deserialize, Serialize};

use super::answers::{validate_answer, Answer};
use super::questions::{generate_questions, Question};

/// Interview session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewSession {
    /// The project idea.
    pub idea: String,
    /// Detected or selected domain.
    pub domain: Domain,
    /// All questions to be asked.
    pub questions: Vec<Question>,
    /// All collected answers.
    pub answers: Vec<Answer>,
    /// Current question index.
    pub current_index: usize,
    /// Whether the interview is complete.
    pub completed: bool,
}

#[allow(dead_code)]
impl InterviewSession {
    /// Create a new interview session.
    #[allow(dead_code)]
    pub fn new(idea: String, domain: Domain) -> Self {
        let questions = generate_questions(domain);

        InterviewSession {
            idea,
            domain,
            questions,
            answers: Vec::new(),
            current_index: 0,
            completed: false,
        }
    }

    /// Get the current question.
    pub fn current_question(&self) -> Option<&Question> {
        self.questions.get(self.current_index)
    }

    /// Get progress info for display.
    pub fn progress(&self) -> (usize, usize) {
        let current = self.current_index + 1;
        let total = self.questions.len();
        (current, total)
    }

    /// Submit an answer for the current question.
    pub fn submit_answer(&mut self, answer: Answer) -> Result<(), SpecGenError> {
        // Validate the answer
        validate_answer(&answer)?;

        self.answers.push(answer);
        self.current_index += 1;

        // Check if interview is complete
        if self.current_index >= self.questions.len() {
            self.completed = true;
        }

        Ok(())
    }

    /// Skip the current question.
    pub fn skip_current(&mut self) {
        let question = match self.current_question() {
            Some(q) => q,
            None => return,
        };

        let answer = Answer::skipped(question.id);
        self.answers.push(answer);
        self.current_index += 1;

        if self.current_index >= self.questions.len() {
            self.completed = true;
        }
    }

    /// Check if interview is done.
    pub fn is_complete(&self) -> bool {
        self.completed
    }

    /// Get all collected answers.
    pub fn get_answers(&self) -> &[Answer] {
        &self.answers
    }

    /// Get answer for a specific question.
    pub fn get_answer_for(&self, question_id: u32) -> Option<&Answer> {
        self.answers.iter().find(|a| a.question_id == question_id)
    }

    /// Build context string for spec generation.
    pub fn build_context(&self) -> String {
        let mut context = String::new();

        context.push_str(&format!("Project Idea: {}\n", self.idea));
        context.push_str(&format!("Domain: {}\n\n", self.domain.display_name()));

        context.push_str("## Interview Q&A\n\n");

        for answer in &self.answers {
            if let Some(question) = self.questions.iter().find(|q| q.id == answer.question_id) {
                context.push_str(&format!("**Q{}: {}**\n", question.id, question.text));

                if answer.skipped {
                    context.push_str("*Skipped*\n\n");
                } else {
                    context.push_str(&format!("{}\n\n", answer.text));
                }
            }
        }

        context
    }
}

/// Calculate completion percentage.
#[allow(dead_code)]
pub fn calculate_completion(session: &InterviewSession) -> f32 {
    if session.questions.is_empty() {
        return 100.0;
    }

    let answered = session.answers.len();
    let total = session.questions.len();

    (answered as f32 / total as f32) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interview_session_new() {
        let session = InterviewSession::new("Build a web app".to_string(), Domain::WebApp);

        assert_eq!(session.idea, "Build a web app");
        assert_eq!(session.domain, Domain::WebApp);
        assert!(!session.questions.is_empty());
        assert!(!session.completed);
    }

    #[test]
    fn test_interview_progress() {
        let session = InterviewSession::new("Build a web app".to_string(), Domain::WebApp);

        let (current, total) = session.progress();
        assert_eq!(current, 1);
        assert!(total > 1);
    }

    #[test]
    fn test_submit_answer() {
        let mut session = InterviewSession::new("Build a web app".to_string(), Domain::WebApp);

        let answer = Answer::new(1, "A web application for users");
        let result = session.submit_answer(answer);

        assert!(result.is_ok());
        assert_eq!(session.answers.len(), 1);
    }

    #[test]
    fn test_submit_invalid_answer() {
        let mut session = InterviewSession::new("Build a web app".to_string(), Domain::WebApp);

        // Answer too short
        let answer = Answer::new(1, "short");
        let result = session.submit_answer(answer);

        assert!(result.is_err());
    }

    #[test]
    fn test_skip_question() {
        let mut session = InterviewSession::new("Build a web app".to_string(), Domain::WebApp);

        session.skip_current();
        assert_eq!(session.answers.len(), 1);
    }

    #[test]
    fn test_build_context() {
        let mut session = InterviewSession::new("Build a web app".to_string(), Domain::WebApp);

        let answer = Answer::new(1, "A web application for users");
        session.submit_answer(answer).unwrap();

        let context = session.build_context();
        assert!(context.contains("Build a web app"));
        assert!(context.contains("Web Application"));
    }

    #[test]
    fn test_calculate_completion() {
        let mut session = InterviewSession::new("Build a web app".to_string(), Domain::WebApp);

        let answer = Answer::new(1, "A web application for users");
        session.submit_answer(answer).unwrap();

        let completion = calculate_completion(&session);
        assert!(completion > 0.0);
    }
}

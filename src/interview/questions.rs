//! Question generation and management for the interview engine.

#[allow(dead_code)]
use crate::domain::Domain;
use serde::{Deserialize, Serialize};

/// Minimum words required for a valid answer.
#[allow(dead_code)]
pub const MIN_ANSWER_WORDS: usize = 3;

/// Represents a question in the interview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    /// Unique identifier for the question.
    pub id: u32,
    /// The question text.
    pub text: String,
    /// Category for organizing questions.
    pub category: QuestionCategory,
    /// Whether this question is required.
    pub required: bool,
}

/// Categories for interview questions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionCategory {
    /// Project overview questions.
    Overview,
    /// Technical stack questions.
    Technical,
    /// Requirements and features questions.
    Requirements,
    /// User experience questions.
    UserExperience,
    /// Infrastructure questions.
    Infrastructure,
    /// Security questions.
    Security,
}

#[allow(dead_code)]
impl Question {
    /// Create a new question.
    pub fn new(id: u32, text: impl Into<String>, category: QuestionCategory) -> Self {
        Question {
            id,
            text: text.into(),
            category,
            required: true,
        }
    }

    /// Create a new optional question.
    pub fn optional(id: u32, text: impl Into<String>, category: QuestionCategory) -> Self {
        Question {
            id,
            text: text.into(),
            category,
            required: false,
        }
    }
}

/// Generate questions for the interview based on the detected domain.
///
/// Returns a vector of questions appropriate for the project type.
#[allow(dead_code)]
pub fn generate_questions(domain: Domain) -> Vec<Question> {
    let mut questions = Vec::new();
    let mut id = 1u32;

    // Always start with overview questions
    questions.push(Question::new(
        id,
        "What is the main purpose of this project? Describe what problem it solves.",
        QuestionCategory::Overview,
    ));
    id += 1;

    questions.push(Question::new(
        id,
        "Who are the target users or customers for this application?",
        QuestionCategory::Overview,
    ));
    id += 1;

    // Domain-specific questions
    match domain {
        Domain::WebApp => {
            questions.extend(webapp_questions(&mut id));
        }
        Domain::RestApi => {
            questions.extend(restapi_questions(&mut id));
        }
        Domain::GraphQLApi => {
            questions.extend(graphql_questions(&mut id));
        }
        Domain::Cli => {
            questions.extend(cli_questions(&mut id));
        }
        Domain::MobileApp => {
            questions.extend(mobile_questions(&mut id));
        }
        Domain::DataPipeline => {
            questions.extend(pipeline_questions(&mut id));
        }
        Domain::MachineLearning => {
            questions.extend(ml_questions(&mut id));
        }
        Domain::EmbeddedSystem => {
            questions.extend(embedded_questions(&mut id));
        }
        Domain::GameDev => {
            questions.extend(gamedev_questions(&mut id));
        }
        Domain::DesktopApp => {
            questions.extend(desktop_questions(&mut id));
        }
        Domain::Unknown => {
            questions.extend(default_questions(&mut id));
        }
    }

    // Common technical questions
    questions.push(Question::new(
        id,
        "Are there any specific programming languages or frameworks you want to use?",
        QuestionCategory::Technical,
    ));
    id += 1;

    questions.push(Question::new(
        id,
        "Do you need to integrate with any existing systems or APIs?",
        QuestionCategory::Technical,
    ));
    id += 1;

    // Add security and infrastructure questions
    questions.push(Question::new(
        id,
        "What are your security requirements? (e.g., authentication, encryption, compliance)",
        QuestionCategory::Security,
    ));
    id += 1;

    questions.push(Question::new(
        id,
        "What is your deployment target? (e.g., cloud, on-premise, serverless)",
        QuestionCategory::Infrastructure,
    ));

    questions
}

/// Get the estimated total number of questions for display.
#[allow(dead_code)]
pub fn get_estimated_question_count(domain: Domain) -> usize {
    generate_questions(domain).len()
}

#[allow(dead_code)]
fn webapp_questions(id: &mut u32) -> Vec<Question> {
    let mut qs = Vec::new();

    qs.push(Question::new(
        *id,
        "Will this be a single-page application (SPA) or server-rendered?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "Do you need user authentication and authorization?",
        QuestionCategory::UserExperience,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "What browser support do you need?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs
}

fn restapi_questions(id: &mut u32) -> Vec<Question> {
    let mut qs = Vec::new();

    qs.push(Question::new(
        *id,
        "What will the API consume and produce? (JSON, XML, etc.)",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "Do you need API versioning from the start?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::optional(
        *id,
        "Will you need rate limiting or throttling?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs
}

fn graphql_questions(id: &mut u32) -> Vec<Question> {
    let mut qs = Vec::new();

    qs.push(Question::new(
        *id,
        "What is the primary use case for your GraphQL API?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "Do you need real-time subscriptions?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs
}

fn cli_questions(id: &mut u32) -> Vec<Question> {
    let mut qs = Vec::new();

    qs.push(Question::new(
        *id,
        "What operating systems should be supported?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "Will the CLI need to read/write files or interact with other tools?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs
}

fn mobile_questions(id: &mut u32) -> Vec<Question> {
    let mut qs = Vec::new();

    qs.push(Question::new(
        *id,
        "Which platforms do you need to support? (iOS, Android, or both)",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "Do you need offline functionality?",
        QuestionCategory::Requirements,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "Will this integrate with device hardware? (camera, GPS, etc.)",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs
}

fn pipeline_questions(id: &mut u32) -> Vec<Question> {
    let mut qs = Vec::new();

    qs.push(Question::new(
        *id,
        "What data sources will feed into this pipeline?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "What is the expected data volume and frequency?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "Do you need real-time streaming or batch processing?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs
}

fn ml_questions(id: &mut u32) -> Vec<Question> {
    let mut qs = Vec::new();

    qs.push(Question::new(
        *id,
        "What type of ML problem is this? (classification, regression, NLP, etc.)",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "Do you have existing training data, or will it need to be collected?",
        QuestionCategory::Requirements,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "What is the inference target? (edge devices, cloud, browser)",
        QuestionCategory::Infrastructure,
    ));
    *id += 1;

    qs
}

fn embedded_questions(id: &mut u32) -> Vec<Question> {
    let mut qs = Vec::new();

    qs.push(Question::new(
        *id,
        "What hardware platform are you targeting?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "What are the real-time requirements?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "Do you have memory or storage constraints?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs
}

fn gamedev_questions(id: &mut u32) -> Vec<Question> {
    let mut qs = Vec::new();

    qs.push(Question::new(
        *id,
        "Is this a 2D or 3D game?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "What platforms do you need to target?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "Will this be single-player or multiplayer?",
        QuestionCategory::Requirements,
    ));
    *id += 1;

    qs
}

fn desktop_questions(id: &mut u32) -> Vec<Question> {
    let mut qs = Vec::new();

    qs.push(Question::new(
        *id,
        "Which desktop platforms do you need to support?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "Does the app need to work offline?",
        QuestionCategory::Requirements,
    ));
    *id += 1;

    qs
}

fn default_questions(id: &mut u32) -> Vec<Question> {
    let mut qs = Vec::new();

    qs.push(Question::new(
        *id,
        "What are the core features you want to build?",
        QuestionCategory::Requirements,
    ));
    *id += 1;

    qs.push(Question::new(
        *id,
        "Do you have any constraints on technology choices?",
        QuestionCategory::Technical,
    ));
    *id += 1;

    qs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_questions_webapp() {
        let qs = generate_questions(Domain::WebApp);
        assert!(!qs.is_empty());
        // Should have overview + webapp-specific + common questions
        assert!(qs.len() > 5);
    }

    #[test]
    fn test_generate_questions_cli() {
        let qs = generate_questions(Domain::Cli);
        assert!(!qs.is_empty());
    }

    #[test]
    fn test_min_answer_words() {
        assert_eq!(MIN_ANSWER_WORDS, 3);
    }

    #[test]
    fn test_estimated_count() {
        let count = get_estimated_question_count(Domain::WebApp);
        assert!(count > 0);
    }
}

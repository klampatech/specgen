//! Spec section generation for SpecGen CLI.
//!
//! This module generates all 10 specification sections from interview data.

#![allow(dead_code)]
#![allow(clippy::uninlined_format_args)]

mod output;

pub use output::{get_default_output_dir, write_all_sections};

use crate::ai::client::AiClient;
use crate::ai::models::{ChatRequest, Message};
use crate::domain::Domain;
use crate::error::SpecGenError;
use crate::interview::answers::Answer;
use std::sync::Arc;
use std::time::SystemTime;

/// All 10 spec section files to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SpecSection {
    /// System requirements and functional requirements.
    Requirements,
    /// Architecture and component design.
    Architecture,
    /// Feature specifications.
    Features,
    /// TDD strategy and test cases.
    TddStrategy,
    /// Sequence diagrams (Mermaid).
    SequenceDiagrams,
    /// UI/UX design scheme.
    DesignScheme,
    /// Security strategy and threat model.
    SecurityStrategy,
    /// SDLC and CI/CD.
    Sdlc,
    /// Acceptance criteria.
    AcceptanceCriteria,
    /// Testing strategy.
    TestingStrategy,
}

impl SpecSection {
    /// Get all sections in generation order.
    pub fn all() -> &'static [SpecSection] {
        &[
            SpecSection::Requirements,
            SpecSection::Architecture,
            SpecSection::Features,
            SpecSection::TddStrategy,
            SpecSection::SequenceDiagrams,
            SpecSection::DesignScheme,
            SpecSection::SecurityStrategy,
            SpecSection::Sdlc,
            SpecSection::AcceptanceCriteria,
            SpecSection::TestingStrategy,
        ]
    }

    /// Get the filename for this section.
    pub fn filename(&self) -> &'static str {
        match self {
            SpecSection::Requirements => "requirements.md",
            SpecSection::Architecture => "architecture.md",
            SpecSection::Features => "features.md",
            SpecSection::TddStrategy => "tdd_strategy.md",
            SpecSection::SequenceDiagrams => "sequence_diagrams.md",
            SpecSection::DesignScheme => "design_scheme.md",
            SpecSection::SecurityStrategy => "security_strategy.md",
            SpecSection::Sdlc => "sdlc.md",
            SpecSection::AcceptanceCriteria => "acceptance_criteria.md",
            SpecSection::TestingStrategy => "testing_strategy.md",
        }
    }

    /// Get the display name for this section.
    pub fn display_name(&self) -> &'static str {
        match self {
            SpecSection::Requirements => "Requirements",
            SpecSection::Architecture => "Architecture",
            SpecSection::Features => "Features",
            SpecSection::TddStrategy => "TDD Strategy",
            SpecSection::SequenceDiagrams => "Sequence Diagrams",
            SpecSection::DesignScheme => "Design Scheme",
            SpecSection::SecurityStrategy => "Security Strategy",
            SpecSection::Sdlc => "SDLC",
            SpecSection::AcceptanceCriteria => "Acceptance Criteria",
            SpecSection::TestingStrategy => "Testing Strategy",
        }
    }
}

/// Interview context for spec generation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InterviewContext {
    /// The original project idea.
    pub idea: String,
    /// Detected project domain.
    pub domain: Domain,
    /// All question-answer pairs.
    pub answers: Vec<Answer>,
}

/// Generate all spec sections concurrently.
///
/// This is the main entry point for spec generation after interview completion.
pub async fn generate_all_sections(
    client: Arc<dyn AiClient>,
    context: InterviewContext,
) -> Result<Vec<(SpecSection, String)>, SpecGenError> {
    let sections = SpecSection::all();

    // Generate all sections concurrently using tokio::join
    let mut handles = Vec::new();

    for section in sections {
        let client = Arc::clone(&client);
        let context = context.clone();

        let handle = tokio::spawn(async move { generate_section(client, *section, context).await });

        handles.push(handle);
    }

    // Wait for all sections to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle
            .await
            .map_err(|e| SpecGenError::SpecError(e.to_string()))?;
        results.push(result?);
    }

    // Sort results by section order
    results.sort_by_key(|(section, _)| {
        sections
            .iter()
            .position(|s| s == section)
            .unwrap_or(usize::MAX)
    });

    Ok(results)
}

/// Generate a single spec section.
pub async fn generate_section(
    client: Arc<dyn AiClient>,
    section: SpecSection,
    context: InterviewContext,
) -> Result<(SpecSection, String), SpecGenError> {
    let system_prompt = build_system_prompt(section, &context);
    let user_prompt = build_user_prompt(section, &context);

    let request = ChatRequest::new_spec_request(vec![
        Message::system(system_prompt),
        Message::user(user_prompt),
    ]);

    let response = client.chat(request).await?;

    let content = post_process_response(&section, &response, &context);

    Ok((section, content))
}

/// Build the system prompt for a specific section.
fn build_system_prompt(section: SpecSection, _context: &InterviewContext) -> String {
    match section {
        SpecSection::Requirements => {
            "You are a software requirements analyst. Generate a comprehensive requirements document that includes:
- Project overview and goals
- Functional requirements (FR-001, FR-002, etc.)
- Non-functional requirements (NFR-001, NFR-002, etc.)

Use the interview answers to derive specific requirements. Format each requirement as 'FR-XXX: [description]' or 'NFR-XXX: [description]' with clear, testable criteria.".to_string()
        }
        SpecSection::Architecture => {
            "You are a software architect. Generate a comprehensive architecture document that includes:
- High-level component design
- Technology stack recommendations with rationale
- Data flow diagrams
- Integration points

Consider the detected domain and suggest appropriate technologies.".to_string()
        }
        SpecSection::Features => {
            "You are a product manager. Generate a feature specification document that includes:
- Core features (F-01, F-02, etc.)
- Feature descriptions with acceptance criteria
- Priority levels (P0, P1, P2)
- Dependencies between features

Base these on the interview responses about project goals and requirements.".to_string()
        }
        SpecSection::TddStrategy => {
            "You are a test-driven development expert. Generate a TDD strategy document that includes:
- Test categories (unit, integration, E2E)
- Testing methodology
- Example test case stubs for core features
- Test coverage targets

Use the feature specifications to derive test cases.".to_string()
        }
        SpecSection::SequenceDiagrams => {
            "You are a technical writer. Generate sequence diagrams in Mermaid syntax that include:
- User interaction flows
- API calls and responses
- Database operations
- External integrations

Use descriptive participant names (no abbreviations). Include 'participant' declarations explicitly.".to_string()
        }
        SpecSection::DesignScheme => {
            "You are a UI/UX designer. Generate a design scheme document that includes:
- UI/UX principles
- Color palette recommendations
- Typography guidelines
- Component design patterns

Consider the target users and domain from the interview.".to_string()
        }
        SpecSection::SecurityStrategy => {
            "You are a security engineer. Generate a security strategy document that includes:
- Threat model (STRIDE analysis)
- Security requirements
- Authentication and authorization approach
- Encryption and data protection
- Compliance considerations

Tailor to the detected domain and interview responses about security needs.".to_string()
        }
        SpecSection::Sdlc => {
            "You are a DevOps engineer. Generate an SDLC document that includes:
- Git branching strategy
- CI/CD pipeline stages
- Deployment strategy
- Environment management
- Code quality gates

Consider the project type and team size from interview responses.".to_string()
        }
        SpecSection::AcceptanceCriteria => {
            "You are a QA engineer. Generate acceptance criteria derived from the TDD strategy. Each criterion should be:
- Specific and measurable
- Linked to a feature or requirement
- Testable in automated tests

Format as 'AC-XXX: [description] - Test: [test name]'.".to_string()
        }
        SpecSection::TestingStrategy => {
            "You are a test automation engineer. Generate a comprehensive testing strategy that includes:
- Test pyramid (unit, integration, E2E distribution)
- Testing tools and frameworks
- Test data management
- CI integration for tests
- Coverage targets

Consider the technology stack from architecture.".to_string()
        }
    }
}

/// Build the user prompt with context for a specific section.
fn build_user_prompt(section: SpecSection, context: &InterviewContext) -> String {
    let idea = &context.idea;
    let domain = context.domain.display_name();
    let answers = format_answers(&context.answers);

    match section {
        SpecSection::Requirements => {
            format!(
                "Generate requirements for the following project:\n\nProject: {idea}\nDomain: {domain}\n\nInterview Responses:\n{answers}\n\nProvide a comprehensive requirements document."
            )
        }
        SpecSection::Architecture => {
            format!(
                "Generate architecture for the following project:\n\nProject: {idea}\nDomain: {domain}\n\nInterview Responses:\n{answers}\n\nProvide a comprehensive architecture document with technology recommendations."
            )
        }
        SpecSection::Features => {
            format!(
                "Generate feature specifications for the following project:\n\nProject: {idea}\nDomain: {domain}\n\nInterview Responses:\n{answers}\n\nProvide detailed feature descriptions."
            )
        }
        _ => {
            format!(
                "Generate content for the following project:\n\nProject: {idea}\nDomain: {domain}\n\nInterview Responses:\n{answers}\n\nProvide comprehensive content."
            )
        }
    }
}

/// Format answers for inclusion in prompts.
fn format_answers(answers: &[Answer]) -> String {
    answers
        .iter()
        .map(|a| format!("Q ID: {}\nA: {}\n", a.question_id, a.text))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Post-process the AI response to add metadata and format.
fn post_process_response(
    section: &SpecSection,
    response: &str,
    context: &InterviewContext,
) -> String {
    let version = env!("CARGO_PKG_VERSION");
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Add generation header
    let header = format!("<!-- Generated by SpecGen CLI v{version} on {date} -->\n\n");

    // Add section title
    let title = format!("# {}\n\n", section.display_name());

    // Combine header, title, and content
    let mut content = header;
    content.push_str(&title);
    content.push_str(response);

    // Add metadata footer
    let footer = format!(
        "\n\n---\n*Generated from: {} | Domain: {}*",
        context.idea,
        context.domain.display_name()
    );
    content.push_str(&footer);

    content
}

/// Get the current timestamp for session metadata.
pub fn get_current_timestamp() -> Result<String, SpecGenError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| SpecGenError::Unexpected(e.to_string()))?;

    Ok(format!("{}", now.as_secs()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_sections_count() {
        assert_eq!(SpecSection::all().len(), 10);
    }

    #[test]
    fn test_section_filename() {
        assert_eq!(SpecSection::Requirements.filename(), "requirements.md");
        assert_eq!(SpecSection::Architecture.filename(), "architecture.md");
        assert_eq!(SpecSection::TddStrategy.filename(), "tdd_strategy.md");
    }

    #[test]
    fn test_section_display_name() {
        assert_eq!(SpecSection::Requirements.display_name(), "Requirements");
        assert_eq!(SpecSection::Architecture.display_name(), "Architecture");
    }

    #[test]
    fn test_format_answers() {
        let answers = vec![
            Answer::new(1, "Answer 1".to_string()),
            Answer::new(2, "Answer 2".to_string()),
        ];

        let formatted = format_answers(&answers);
        assert!(formatted.contains("Q ID: 1"));
        assert!(formatted.contains("A: Answer 1"));
    }
}

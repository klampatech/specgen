//! Domain detection for SpecGen CLI.
//!
//! This module detects the type of software project based on user input
//! using keyword-based heuristics and AI fallback.

use crate::ai::client::AiClient;
use crate::ai::models::{ChatRequest, Message};
use crate::error::SpecGenError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Domain types that can be detected for a software project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    /// Web application (SaaS, webapp, etc.)
    WebApp,
    /// REST API backend
    RestApi,
    /// GraphQL API
    GraphQLApi,
    /// Command-line interface tool
    Cli,
    /// Mobile application (iOS, Android, React Native)
    MobileApp,
    /// Data pipeline / ETL
    DataPipeline,
    /// Machine learning system
    MachineLearning,
    /// Embedded system / firmware
    EmbeddedSystem,
    /// Game development
    GameDev,
    /// Desktop application
    DesktopApp,
    /// Unknown domain
    #[default]
    Unknown,
}

#[allow(dead_code)]
impl Domain {
    /// Get a human-readable name for the domain.
    pub fn display_name(&self) -> &'static str {
        match self {
            Domain::WebApp => "Web Application",
            Domain::RestApi => "REST API",
            Domain::GraphQLApi => "GraphQL API",
            Domain::Cli => "Command-Line Interface",
            Domain::MobileApp => "Mobile Application",
            Domain::DataPipeline => "Data Pipeline",
            Domain::MachineLearning => "Machine Learning System",
            Domain::EmbeddedSystem => "Embedded System",
            Domain::GameDev => "Game Development",
            Domain::DesktopApp => "Desktop Application",
            Domain::Unknown => "Unknown",
        }
    }

    /// Get all possible domains.
    pub fn all() -> &'static [Domain; 11] {
        &[
            Domain::WebApp,
            Domain::RestApi,
            Domain::GraphQLApi,
            Domain::Cli,
            Domain::MobileApp,
            Domain::DataPipeline,
            Domain::MachineLearning,
            Domain::EmbeddedSystem,
            Domain::GameDev,
            Domain::DesktopApp,
            Domain::Unknown,
        ]
    }
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Keyword mappings for domain detection.
#[allow(dead_code)]
struct DomainKeywords {
    keywords: &'static [(Domain, &'static [&'static str])],
}

impl DomainKeywords {
    const fn new() -> Self {
        DomainKeywords {
            keywords: &[
                // More specific domains first
                (Domain::GraphQLApi, &["graphql", "apollo", "relay", "gql"]),
                (
                    Domain::EmbeddedSystem,
                    &[
                        "embedded",
                        "firmware",
                        "iot",
                        "arduino",
                        "raspberry pi",
                        "microcontroller",
                        "rtos",
                        "driver",
                        "kernel",
                        "bare metal",
                    ],
                ),
                (
                    Domain::MachineLearning,
                    &[
                        "machine learning",
                        "artificial intelligence",
                        "deep learning",
                        "neural network",
                        "tensorflow",
                        "pytorch",
                        "llm",
                        "gpt",
                        "bert",
                        "nlp",
                    ],
                ),
                (
                    Domain::GameDev,
                    &[
                        "game",
                        "gaming",
                        "unity",
                        "unreal",
                        "godot",
                        "3d game",
                        "2d game",
                        "virtual reality",
                        "augmented reality",
                    ],
                ),
                (
                    Domain::MobileApp,
                    &[
                        "mobile app",
                        "ios app",
                        "android app",
                        "react native",
                        "flutter",
                        "xamarin",
                        "cordova",
                        "ionic",
                        "swift",
                        "kotlin",
                    ],
                ),
                (
                    Domain::Cli,
                    &[
                        "cli",
                        "command-line",
                        "command line tool",
                        "terminal tool",
                        "console tool",
                        "utility",
                        "command line",
                    ],
                ),
                (
                    Domain::DataPipeline,
                    &[
                        "pipeline",
                        "etl",
                        "data pipeline",
                        "stream processing",
                        "kafka",
                        "spark",
                        "airflow",
                        "dag",
                        "data warehouse",
                        "data lake",
                    ],
                ),
                (
                    Domain::DesktopApp,
                    &[
                        "desktop app",
                        "desktop application",
                        "gui app",
                        "electron",
                        "qt",
                        "gtk",
                        "wxwidgets",
                    ],
                ),
                (
                    Domain::WebApp,
                    &[
                        "web app",
                        "saas",
                        "website",
                        "webapp",
                        "frontend",
                        "fullstack",
                        "react",
                        "vue",
                        "angular",
                        "next.js",
                        "django",
                        "rails",
                        "spring",
                        "express",
                        "dashboard",
                        "portal",
                        "cms",
                        "blog",
                        "ecommerce",
                    ],
                ),
                (
                    Domain::RestApi,
                    &[
                        "rest api",
                        "restful api",
                        "api backend",
                        "api endpoint",
                        "microservice",
                        "api server",
                        "grpc",
                        "openapi",
                        "swagger",
                    ],
                ),
            ],
        }
    }

    /// Detect domain from idea string using keyword matching.
    fn detect(&self, idea: &str) -> Domain {
        let idea_lower = idea.to_lowercase();

        // Find the domain with the most keyword matches
        let mut best_domain = Domain::Unknown;
        let mut best_score = 0usize;

        for (domain, keywords) in self.keywords {
            let count = keywords
                .iter()
                .filter(|kw| idea_lower.contains(*kw))
                .count();

            if count > best_score {
                best_score = count;
                best_domain = *domain;
            }
        }

        best_domain
    }
}

/// Detect the domain of a software project from user input.
///
/// Uses keyword-based heuristics for initial classification.
/// Returns the detected domain.
#[allow(dead_code)]
pub fn detect_domain(idea: &str) -> Domain {
    let keywords = DomainKeywords::new();
    keywords.detect(idea)
}

/// Check if the domain detection is ambiguous (needs AI fallback).
///
/// Returns true if the domain is Unknown or confidence is low.
#[allow(dead_code)]
pub fn needs_ai_fallback(detected: Domain) -> bool {
    detected == Domain::Unknown
}

/// Detect domain using AI when keyword-based detection is ambiguous.
///
/// This is called when `needs_ai_fallback()` returns true. It sends the
/// project idea to the AI and parses the response to extract a domain.
pub async fn detect_domain_with_ai(
    client: Arc<dyn AiClient>,
    idea: &str,
) -> Result<Domain, SpecGenError> {
    let system_prompt = "You are a software domain classifier. Given a project description, \
identify the most appropriate domain from this list: WebApp, RestApi, GraphQLApi, Cli, \
MobileApp, DataPipeline, MachineLearning, EmbeddedSystem, GameDev, DesktopApp. \
Respond ONLY with the domain name, nothing else.";

    let user_prompt = format!("Classify this project: {}", idea);

    let request = ChatRequest::new_spec_request(vec![
        Message::system(system_prompt),
        Message::user(user_prompt),
    ]);

    let response = client.chat(request).await?;
    parse_ai_domain_response(&response)
}

/// Prompt user to confirm or change the detected domain.
///
/// Returns the final domain (either confirmed or user-selected).
pub fn confirm_domain(detected: Domain) -> Domain {
    println!("\nDetected domain: {}", detected.display_name());
    println!("Is this correct? (y/n): ");

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
        let input = input.trim().to_lowercase();
        if input == "y" || input.is_empty() {
            return detected;
        }
    }

    // User said no - return Unknown to trigger domain selection
    // In a full implementation, this would prompt for domain selection
    println!("Please specify the domain manually (or we'll use Unknown).");
    Domain::Unknown
}

/// Parse the AI response to extract the domain enum value.
fn parse_ai_domain_response(response: &str) -> Result<Domain, SpecGenError> {
    let response = response.trim();

    // Try to match the response to a known domain
    match response {
        s if s.eq_ignore_ascii_case("webapp") => Ok(Domain::WebApp),
        s if s.eq_ignore_ascii_case("restapi") => Ok(Domain::RestApi),
        s if s.eq_ignore_ascii_case("graphqlapi") || s.eq_ignore_ascii_case("graphql") => {
            Ok(Domain::GraphQLApi)
        }
        s if s.eq_ignore_ascii_case("cli") || s.eq_ignore_ascii_case("command-line") => {
            Ok(Domain::Cli)
        }
        s if s.eq_ignore_ascii_case("mobileapp") || s.eq_ignore_ascii_case("mobile") => {
            Ok(Domain::MobileApp)
        }
        s if s.eq_ignore_ascii_case("datapipeline") || s.eq_ignore_ascii_case("data pipeline") => {
            Ok(Domain::DataPipeline)
        }
        s if s.eq_ignore_ascii_case("machinelearning") || s.eq_ignore_ascii_case("ml") => {
            Ok(Domain::MachineLearning)
        }
        s if s.eq_ignore_ascii_case("embeddedsystem") || s.eq_ignore_ascii_case("embedded") => {
            Ok(Domain::EmbeddedSystem)
        }
        s if s.eq_ignore_ascii_case("gamedev") || s.eq_ignore_ascii_case("game") => {
            Ok(Domain::GameDev)
        }
        s if s.eq_ignore_ascii_case("desktopapp") || s.eq_ignore_ascii_case("desktop") => {
            Ok(Domain::DesktopApp)
        }
        _ => Ok(Domain::Unknown),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_web_app() {
        let idea = "Build a SaaS web application with React frontend";
        assert_eq!(detect_domain(idea), Domain::WebApp);
    }

    #[test]
    fn test_detect_rest_api() {
        let idea = "Create a REST API backend with Node.js";
        assert_eq!(detect_domain(idea), Domain::RestApi);
    }

    #[test]
    fn test_detect_graphql() {
        let idea = "GraphQL API server with Apollo";
        assert_eq!(detect_domain(idea), Domain::GraphQLApi);
    }

    #[test]
    fn test_detect_cli() {
        let idea = "Command line tool for file processing";
        assert_eq!(detect_domain(idea), Domain::Cli);
    }

    #[test]
    fn test_detect_mobile() {
        let idea = "iOS mobile app in Swift";
        assert_eq!(detect_domain(idea), Domain::MobileApp);
    }

    #[test]
    fn test_detect_ml() {
        let idea = "Machine learning model for image classification";
        assert_eq!(detect_domain(idea), Domain::MachineLearning);
    }

    #[test]
    fn test_detect_unknown() {
        let idea = "Some random project";
        assert_eq!(detect_domain(idea), Domain::Unknown);
    }

    #[test]
    fn test_needs_ai_fallback() {
        assert!(!needs_ai_fallback(Domain::WebApp));
        assert!(needs_ai_fallback(Domain::Unknown));
    }

    #[test]
    fn test_domain_display_name() {
        assert_eq!(Domain::WebApp.display_name(), "Web Application");
        assert_eq!(Domain::Cli.display_name(), "Command-Line Interface");
    }

    #[test]
    fn test_domain_serialization() {
        let domain = Domain::WebApp;
        let serialized = serde_json::to_string(&domain).unwrap();
        assert_eq!(serialized, "\"web_app\"");

        let deserialized: Domain = serde_json::from_str("\"web_app\"").unwrap();
        assert_eq!(deserialized, Domain::WebApp);
    }
}

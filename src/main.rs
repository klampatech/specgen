//! SpecGen CLI - AI-powered specification generator.
//!
//! A command-line tool that interviews developers and generates comprehensive
//! specifications for software projects using MiniMax AI.

mod ai;
mod api_key;
mod diff;
mod domain;
mod error;
mod interview;
mod session;
mod spec;
mod ui;

use std::io::Write;
use std::sync::Arc;

use ai::client::{AiClient, MinimaxClient};
use clap::Parser;
use domain::{detect_domain, needs_ai_fallback, Domain};

/// CLI argument parser.
#[derive(Parser, Debug)]
#[command(name = "specgen")]
#[command(version = "0.1.0")]
#[command(
    about = "AI-powered CLI tool that interviews developers and generates comprehensive specifications"
)]
struct Args {
    /// Subcommand to execute.
    #[command(subcommand)]
    command: Command,
}

/// Available subcommands.
#[derive(Parser, Debug)]
enum Command {
    /// Start a new specification interview.
    New {
        /// The project idea description.
        #[arg(default_value = "")]
        idea: String,
    },
    /// Refine existing specification.
    Refine {
        /// Instruction for refinement.
        instruction: String,
    },
    /// Show specification status.
    Status,
    /// Show diff against existing specs.
    Diff,
    /// Export all specs to single file.
    Export,
}

/// Validate the API key exists in environment.
fn validate_api_key() -> Result<api_key::ApiKey, error::SpecGenError> {
    api_key::read_api_key_from_env()
}

/// Run the new spec generation flow.
async fn run_new_command(
    idea: String,
    client: Arc<dyn AiClient>,
    _api_key: api_key::ApiKey,
) -> Result<(), error::SpecGenError> {
    // Get idea from user if not provided
    let idea = if idea.is_empty() {
        println!("\n=== SpecGen CLI ===\n");
        print!("Enter your project idea: ");
        std::io::stdout()
            .flush()
            .map_err(|e| error::SpecGenError::IoError(e.to_string()))?;
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| error::SpecGenError::IoError(e.to_string()))?;
        input.trim().to_string()
    } else {
        idea
    };

    if idea.is_empty() {
        return Err(error::SpecGenError::Unexpected(
            "Project idea cannot be empty".to_string(),
        ));
    }

    println!("\nAnalyzing project: {idea}\n");

    // Detect domain using keyword-based detection
    let mut domain = detect_domain(&idea);
    println!("Detected domain: {}", domain.display_name());

    // If domain is unknown, try AI fallback
    if domain == Domain::Unknown || needs_ai_fallback(domain) {
        println!("Domain unclear, using AI classifier...");
        match domain::detect_domain_with_ai(Arc::clone(&client), &idea).await {
            Ok(ai_domain) => {
                if ai_domain != Domain::Unknown {
                    domain = ai_domain;
                    println!("AI classified as: {}", domain.display_name());
                }
            }
            Err(e) => {
                println!("AI classification failed: {e}, using Unknown");
            }
        }
    }

    // Confirm domain with user
    println!("\nIs this correct? (y/n): ");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| error::SpecGenError::IoError(e.to_string()))?;
    let input = input.trim().to_lowercase();
    if input != "y" && input.is_empty() {
        domain = Domain::Unknown;
    }

    println!("\nDomain confirmed: {}\n", domain.display_name());

    // Start interview session
    println!("=== Starting Interview ===\n");
    let mut session = interview::orchestrator::InterviewSession::new(idea.clone(), domain);

    // Run Q/A loop (simplified - in production this would use TUI)
    while !session.completed {
        let (current, total) = session.progress();
        if let Some(question) = session.current_question() {
            println!("[Question {}/{}] {}\n", current, total, question.text);
            print!("Answer (or 'skip' to skip): ");
            std::io::stdout()
                .flush()
                .map_err(|e| error::SpecGenError::IoError(e.to_string()))?;

            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .map_err(|e| error::SpecGenError::IoError(e.to_string()))?;
            let input = input.trim().to_string();

            if input.is_empty() || input.to_lowercase() == "skip" {
                // Skip question - mark as assumed
                let answer = interview::answers::Answer::skipped(question.id);
                let _ = session.submit_answer(answer);
            } else {
                let answer = interview::answers::Answer::new(question.id, input);
                match session.submit_answer(answer) {
                    Ok(_) => println!("Answer recorded.\n"),
                    Err(e) => println!("Invalid answer: {e}\n"),
                }
            }
        }
    }

    println!("Interview complete! Generating specifications...\n");

    // Build interview context for spec generation
    let context = spec::InterviewContext {
        idea: idea.clone(),
        domain,
        answers: session.answers.clone(),
    };

    // Generate all spec sections concurrently
    let sections = spec::generate_all_sections(client, context).await?;

    // Write all spec files
    let output_dir = spec::get_default_output_dir();
    let section_tuples: Vec<(&str, &str)> = sections
        .iter()
        .map(|(section, content)| (section.filename().trim_end_matches(".md"), content.as_str()))
        .collect();

    let written_paths = spec::write_all_sections(&section_tuples, &output_dir, false)?;

    println!("\n=== Spec Generation Complete ===\n");
    println!("Generated {} specification files:", written_paths.len());
    for path in &written_paths {
        println!("  - {path}");
    }
    println!("\nAll specs saved to: {output_dir}\n");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), error::SpecGenError> {
    // Validate API key before any other operations
    let api_key = validate_api_key()?;

    let args = Args::parse();

    // Create AI client
    let client: Arc<dyn AiClient> = Arc::new(MinimaxClient::new(api_key.clone()));

    // Dispatch to command handlers
    match args.command {
        Command::New { idea } => {
            run_new_command(idea, client, api_key).await?;
        }
        Command::Refine { instruction } => {
            println!("Refining with instruction: {instruction}");
            // TODO: Implement refine command
        }
        Command::Status => {
            println!("Checking spec status...");
            // TODO: Implement status command
        }
        Command::Diff => {
            println!("Showing diff...");
            // TODO: Implement diff command
        }
        Command::Export => {
            println!("Exporting specs...");
            // TODO: Implement export command
        }
    }

    Ok(())
}

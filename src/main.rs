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
mod logging;
mod session;
mod spec;
mod ui;

use std::io::Write;
use std::sync::Arc;

use ai::client::{AiClient, MinimaxClient};
use camino::Utf8PathBuf;
use clap::Parser;
use domain::{detect_domain, needs_ai_fallback, Domain};
use serde::Serialize;
use spec::SpecSection;

// Re-export SpecGenError for internal use
use crate::error::SpecGenError;
use crate::logging::{init as init_logging, verbosity_to_level};

/// Status output for a single spec section.
#[derive(Debug, Serialize)]
struct SectionStatus {
    name: String,
    filename: String,
    exists: bool,
}

/// Complete status output.
#[derive(Debug, Serialize)]
struct SpecStatus {
    total: usize,
    present: usize,
    missing: usize,
    completeness: u8,
    sections: Vec<SectionStatus>,
}

/// CLI argument parser.
#[derive(Parser, Debug)]
#[command(name = "specgen")]
#[command(version = "0.1.0")]
#[command(
    about = "AI-powered CLI tool that interviews developers and generates comprehensive specifications"
)]
struct Args {
    /// Verbose output (-v INFO, -vv DEBUG, -vvv TRACE).
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: Option<u8>,
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
        /// Skip interactive interview, use AI-assumed answers.
        #[arg(long, short)]
        no_interview: bool,
    },
    /// Refine existing specification.
    Refine {
        /// Instruction for refinement.
        instruction: String,
        /// Specific sections to refine (comma-separated).
        #[arg(long, short)]
        sections: Option<String>,
    },
    /// Show specification status.
    Status {
        /// Output as JSON.
        #[arg(long, short)]
        json: bool,
    },
    /// Show diff against existing specs.
    Diff,
    /// Export all specs to single file.
    Export,
}

/// Validate the API key exists in environment.
fn validate_api_key() -> Result<api_key::ApiKey, SpecGenError> {
    api_key::read_api_key_from_env()
}

/// Run the new spec generation flow.
async fn run_new_command(
    idea: String,
    client: Arc<dyn AiClient>,
    _api_key: api_key::ApiKey,
    no_interview: bool,
) -> Result<(), SpecGenError> {
    // Get idea from user if not provided
    let idea = if idea.is_empty() {
        if no_interview {
            // Try to read from stdin (pipe mode)
            let piped_input = std::io::read_to_string(&mut std::io::stdin())
                .map_err(|e| SpecGenError::IoError(e.to_string()))?;
            piped_input.trim().to_string()
        } else {
            println!("\n=== SpecGen CLI ===\n");
            print!("Enter your project idea: ");
            std::io::stdout()
                .flush()
                .map_err(|e| SpecGenError::IoError(e.to_string()))?;
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .map_err(|e| SpecGenError::IoError(e.to_string()))?;
            input.trim().to_string()
        }
    } else {
        idea
    };

    if idea.is_empty() {
        return Err(SpecGenError::Unexpected(
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

    // Confirm domain with user (skip in no-interview mode)
    if !no_interview {
        println!("\nIs this correct? (y/n): ");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| SpecGenError::IoError(e.to_string()))?;
        let input = input.trim().to_lowercase();
        if input != "y" && input.is_empty() {
            domain = Domain::Unknown;
        }
    }

    println!("\nDomain confirmed: {}\n", domain.display_name());

    // Start interview session
    println!("=== Starting Interview ===\n");
    let mut session = interview::orchestrator::InterviewSession::new(idea.clone(), domain);

    // In no-interview mode, skip all questions with assumed answers
    if no_interview {
        println!("[Non-interactive mode: AI-assuming all answers]\n");
        // Collect question IDs first to avoid borrow issues
        let question_ids: Vec<_> = session.questions.iter().map(|q| q.id).collect();
        for qid in question_ids {
            let answer = interview::answers::Answer::assumed(qid, "[ASSUMED]".to_string());
            let _ = session.submit_answer(answer);
        }
    } else {
        // Run Q/A loop (simplified - in production this would use TUI)
        while !session.completed {
            let (current, total) = session.progress();
            if let Some(question) = session.current_question() {
                println!("[Question {}/{}] {}\n", current, total, question.text);
                print!("Answer (or 'skip' to skip): ");
                std::io::stdout()
                    .flush()
                    .map_err(|e| SpecGenError::IoError(e.to_string()))?;

                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .map_err(|e| SpecGenError::IoError(e.to_string()))?;
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
    }

    println!("Interview complete! Generating specifications...\n");

    // Build interview context for spec generation
    let context = spec::InterviewContext {
        idea: idea.clone(),
        domain,
        answers: session.answers.clone(),
    };

    // Save interview entries to session
    let mut session_data = session::Session::new(idea.clone(), domain.to_string());
    for answer in &session.answers {
        let question_text = session
            .questions
            .iter()
            .find(|q| q.id == answer.question_id)
            .map(|q| q.text.clone())
            .unwrap_or_default();
        session_data.add_interview_entry(
            question_text,
            answer.text.clone(),
            answer.skipped || answer.assumed,
        );
    }

    // Generate all spec sections concurrently
    let sections = spec::generate_all_sections(client, context).await?;

    // Write all spec files
    let output_dir = spec::get_default_output_dir();
    let section_tuples: Vec<(&str, &str)> = sections
        .iter()
        .map(|(section, content)| (section.filename().trim_end_matches(".md"), content.as_str()))
        .collect();

    let written_paths = spec::write_all_sections(&section_tuples, &output_dir, false)?;

    // Mark sections as generated and save session
    for (section, _) in &sections {
        session_data.mark_section_generated(section.filename());
    }

    // Save session to disk
    let project_dir = Utf8PathBuf::from_path_buf(
        std::env::current_dir().map_err(|e| SpecGenError::IoError(e.to_string()))?,
    )
    .map_err(|_| SpecGenError::Unexpected("Invalid path".to_string()))?;
    session::save_session(&project_dir, &session_data)?;

    println!("\n=== Spec Generation Complete ===\n");
    println!("Generated {} specification files:", written_paths.len());
    for path in &written_paths {
        println!("  - {path}");
    }
    println!("\nAll specs saved to: {output_dir}\n");

    Ok(())
}

/// Run the status command to show spec completeness.
fn run_status_command(json_output: bool) -> Result<(), SpecGenError> {
    let output_dir = spec::get_default_output_dir();
    let sections = SpecSection::all();

    let mut section_statuses = Vec::new();
    let mut present_count = 0;

    for section in sections {
        let filename = section.filename();
        let file_path = output_dir.join(filename);
        let exists = file_path.exists();

        if exists {
            present_count += 1;
        }

        section_statuses.push(SectionStatus {
            name: section.display_name().to_string(),
            filename: filename.to_string(),
            exists,
        });
    }

    let total = sections.len();
    let missing = total - present_count;
    let completeness = if total > 0 {
        ((present_count as f64 / total as f64) * 100.0) as u8
    } else {
        0
    };

    let status = SpecStatus {
        total,
        present: present_count,
        missing,
        completeness,
        sections: section_statuses,
    };

    if json_output {
        // JSON output
        let json = serde_json::to_string_pretty(&status)
            .map_err(|e| SpecGenError::Unexpected(e.to_string()))?;
        println!("{json}");
    } else {
        // Human-readable output
        println!("\n=== Spec Status ===\n");
        println!("Output directory: {output_dir}");
        println!("Completeness: {completeness}% ({present_count}/{total} sections)\n");

        println!("Sections:");
        for s in &status.sections {
            let status_marker = if s.exists { "[x]" } else { "[ ]" };
            println!("  {} {:30} ({})", status_marker, s.name, s.filename);
        }

        if missing > 0 {
            println!("\nMissing {missing} section(s). Run 'specgen new' to generate.");
        } else {
            println!("\nAll sections complete!");
        }
    }

    Ok(())
}

/// Run the diff command to show changes between generated and existing specs.
fn run_diff_command() -> Result<(), SpecGenError> {
    let output_dir = spec::get_default_output_dir();
    let sections = SpecSection::all();

    println!("\n=== Spec Diff ===\n");

    for section in sections {
        let filename = section.filename();
        let file_path = output_dir.join(filename);

        if file_path.exists() {
            println!("[x] {} - exists", section.display_name());
        } else {
            println!("[ ] {} - missing", section.display_name());
        }
    }

    println!("\nDiff comparison not yet implemented.");
    Ok(())
}

/// Run the refine command to regenerate specific sections.
async fn run_refine_command(
    instruction: String,
    sections: Option<String>,
    client: Arc<dyn AiClient>,
) -> Result<(), SpecGenError> {
    // Load session data to get context
    let project_dir = Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .map_err(|_| SpecGenError::Unexpected("Invalid path".to_string()))?;

    let session = match session::load_session(&project_dir) {
        Ok(s) => s,
        Err(e) => {
            return Err(SpecGenError::Unexpected(format!(
                "No session found. Run 'specgen new' first. Error: {e}"
            )));
        }
    };

    // Determine which sections to refine
    let sections_to_refine = if let Some(s) = sections {
        spec::SpecSection::parse_sections(&s)?
    } else {
        spec::SpecSection::all().to_vec()
    };

    println!("Refining {} section(s)...", sections_to_refine.len());

    // Build interview context from session
    let domain = domain::Domain::from_string(&session.domain);
    let context = spec::InterviewContext {
        idea: session.idea,
        domain,
        answers: Vec::new(), // Not needed for refinement
    };

    // Generate refined sections
    for section in &sections_to_refine {
        println!("Regenerating {}...", section.display_name());

        let result = spec::generate_section_with_instruction(
            Arc::clone(&client),
            *section,
            context.clone(),
            Some(instruction.clone()),
        )
        .await?;

        // Write the refined section (with overwrite)
        let output_dir = spec::get_default_output_dir();
        let file_path = output_dir.join(section.filename());
        spec::write_spec_file(&file_path, &result.1, true)?;

        println!("  Updated: {}", section.filename());
    }

    println!("\nRefinement complete!");
    Ok(())
}

/// Run the export command to bundle all specs into a single file.
fn run_export_command() -> Result<(), SpecGenError> {
    let output_dir = spec::get_default_output_dir();
    let sections = SpecSection::all();
    let mut all_content = String::new();

    // Add header
    all_content.push_str("# Specification Document\n\n");
    all_content.push_str("*Generated by SpecGen CLI*\n\n");
    all_content.push_str("---\n\n");

    for section in sections {
        let filename = section.filename();
        let file_path = output_dir.join(filename);

        if file_path.exists() {
            let content = std::fs::read_to_string(&file_path)?;
            all_content.push_str(&content);
            all_content.push_str("\n\n---\n\n");
        }
    }

    // Write to SPEC.md
    let spec_path = output_dir.join("SPEC.md");
    std::fs::write(&spec_path, &all_content)?;

    println!("Exported to: {spec_path}");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), SpecGenError> {
    let args = Args::parse();

    // Initialize logging based on verbosity
    let level = verbosity_to_level(args.verbose.unwrap_or(0));
    let log_format = std::env::var("SPECGEN_LOG_FORMAT").unwrap_or_default();
    init_logging(level, &log_format);

    // Parse command first to determine if we need API key
    let needs_api_key = matches!(args.command, Command::New { .. } | Command::Refine { .. });

    // Only validate API key for commands that need it
    let api_key = if needs_api_key {
        validate_api_key()?
    } else {
        // Return early for commands that don't need API key
        match args.command {
            Command::Status { json } => {
                run_status_command(json)?;
                return Ok(());
            }
            Command::Diff => {
                run_diff_command()?;
                return Ok(());
            }
            Command::Export => {
                run_export_command()?;
                return Ok(());
            }
            _ => {
                return Err(SpecGenError::Unexpected("Unexpected command".to_string()));
            }
        }
    };

    // Create AI client
    let client: Arc<dyn AiClient> = Arc::new(MinimaxClient::new(api_key.clone()));

    // Dispatch to command handlers
    match args.command {
        Command::New { idea, no_interview } => {
            run_new_command(idea, client, api_key, no_interview).await?;
        }
        Command::Refine {
            instruction,
            sections,
        } => {
            run_refine_command(instruction, sections, client).await?;
        }
        _ => {}
    }

    Ok(())
}

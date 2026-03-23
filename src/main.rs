//! SpecGen CLI - AI-powered specification generator.
//!
//! A command-line tool that interviews developers and generates comprehensive
//! specifications for software projects using MiniMax AI.

mod api_key;
mod domain;
mod error;

use clap::Parser;

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

#[tokio::main]
async fn main() -> Result<(), error::SpecGenError> {
    // Validate API key before any other operations
    let _api_key = validate_api_key()?;

    let args = Args::parse();

    // Dispatch to command handlers
    match args.command {
        Command::New { idea } => {
            println!("Starting new spec generation for: {idea}");
            // TODO: Implement new command
            Ok(())
        }
        Command::Refine { instruction } => {
            println!("Refining with instruction: {instruction}");
            // TODO: Implement refine command
            Ok(())
        }
        Command::Status => {
            println!("Checking spec status...");
            // TODO: Implement status command
            Ok(())
        }
        Command::Diff => {
            println!("Showing diff...");
            // TODO: Implement diff command
            Ok(())
        }
        Command::Export => {
            println!("Exporting specs...");
            // TODO: Implement export command
            Ok(())
        }
    }
}

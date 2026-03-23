# SpecGen

AI-powered CLI tool that interviews developers and generates comprehensive, production-ready specifications.

## Overview

SpecGen streamlines the specification process by asking targeted questions about your project idea, then generating a complete 10-section specification document using MiniMax AI. The generated spec covers everything from high-level requirements through to testing strategy — ready to hand off to your team or use as a foundation for TDD.

## Features

- **Interactive Interview Mode** — Step-by-step guided questioning tailored to your project domain
- **Non-Interactive (Pipe) Mode** — Pass project ideas via stdin or CLI args for CI/CD integration
- **AI-Powered Generation** — Uses MiniMax AI to generate structured, domain-aware specifications
- **Domain Detection** — Automatic project domain classification (web, mobile, CLI, library, etc.) with keyword matching and AI fallback
- **10-Section Spec Coverage** — Requirements, Architecture, Features, TDD Strategy, Sequence Diagrams, Design Scheme, Security, SDLC, Acceptance Criteria, Testing Strategy
- **Session Persistence** — Tracks interview answers and generated sections across sessions
- **Refinement Engine** — Regenerate specific sections with additional instructions
- **Spec Export** — Bundle all sections into a single `SPEC.md` file
- **Diff View** — Track which sections exist vs. missing
- **Structured Logging** — Verbose logging with `-v` / `-vv` / `-vvv` flags, JSON output support
- **Typed Error Handling** — Domain-specific error types with descriptive messages

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/specgen/specgen.git
cd specgen

# Build with Cargo
cargo build --release

# Install globally
cargo install --path .
```

### Prerequisites

- Rust 1.88 or later
- A MiniMax API key

### Environment Setup

Set your MiniMax API key in your environment:

```bash
export MINIMAX_API_KEY="your-api-key-here"
```

Or use a `.env` file (ensure it is gitignored):

```bash
echo "MINIMAX_API_KEY=your-api-key-here" > .env
```

## Quick Start

```bash
# Start an interactive interview for your project idea
specgen new "A CLI tool for generating API documentation from OpenAPI specs"

# Or pipe a project idea directly (non-interactive mode)
echo "A REST API for managing task lists with authentication" | specgen new --no-interview

# Check which spec sections have been generated
specgen status

# Export all sections into a single SPEC.md file
specgen export
```

## CLI Commands

### `specgen new` — Start a New Specification

Generate a complete specification by answering interview questions about your project.

```bash
# Interactive mode — enter project idea when prompted
specgen new

# Interactive mode — pass project idea on the command line
specgen new "A websocket-based chat server with room management"

# Non-interactive mode — skips the interview, uses AI-assumed defaults
# Useful for CI/CD pipelines or quick drafts
specgen new "A markdown note-taking app" --no-interview

# Non-interactive mode — pipe project idea via stdin
echo "A GitHub PR review automation tool" | specgen new --no-interview
```

**Flags:**
- `[idea]` — Optional project idea description (prompts if omitted)
- `--no-interview`, `-n` — Skip interactive Q&A, use AI-assumed answers

**Output:** Generates 10 markdown files in the `specs/` directory, one per specification section.

---

### `specgen refine` — Refine Existing Specification

Regenerate specific sections with additional instructions or corrections.

```bash
# Refine all sections with a general instruction
specgen refine "Add error handling requirements"

# Refine specific sections only
specgen refine "Add rate limiting to the API design" --sections acceptance,security
```

**Flags:**
- `<instruction>` — (required) The refinement instruction/prompt
- `--sections`, `-s` — Comma-separated list of section names to regenerate

**Sections:** `requirements`, `architecture`, `features`, `tdd`, `diagrams`, `design`, `security`, `sdlc`, `acceptance`, `testing`

**Note:** Requires an existing session (run `specgen new` first).

---

### `specgen status` — Show Specification Status

Display the completeness of your specification — which sections exist and which are missing.

```bash
# Human-readable status output
specgen status
```

**Output:**
```
=== Spec Status ===

Output directory: ./specs
Completeness: 40% (4/10 sections)

Sections:
  [x] Requirements         (01-requirements.md)
  [x] Architecture          (02-architecture.md)
  [x] Features              (03-features.md)
  [ ] TDD Strategy          (04-tdd.md)
  [x] Sequence Diagrams     (05-diagrams.md)
  [ ] Design Scheme         (06-design.md)
  [ ] Security              (07-security.md)
  [ ] SDLC                  (08-sdlc.md)
  [ ] Acceptance Criteria   (09-acceptance.md)
  [ ] Testing Strategy       (10-testing.md)

Missing 6 section(s). Run 'specgen new' to generate.
```

```bash
# JSON output for scripting / integration
specgen status --json
```

**Flags:**
- `--json`, `-j` — Output status as machine-readable JSON

---

### `specgen diff` — Compare Specification State

Show which spec sections exist versus which are missing.

```bash
specgen diff
```

**Output:**
```
=== Spec Diff ===

[x] Requirements      - exists
[x] Architecture      - exists
[ ] TDD Strategy      - missing
[x] Sequence Diagrams  - exists
...
```

---

### `specgen export` — Export All Specs

Bundle all generated section files into a single `SPEC.md` document.

```bash
specgen export
```

**Output:** Writes `specs/SPEC.md` containing all sections joined with markdown dividers.

---

## Logging & Verbosity

Control log verbosity with the `-v` flag (can be stacked):

```bash
specgen new "My project idea" -v      # INFO level
specgen new "My project idea" -vv     # DEBUG level
specgen new "My project idea" -vvv    # TRACE level
```

For JSON-formatted logs (useful for log aggregation):

```bash
export SPECGEN_LOG_FORMAT=json
specgen new "My project idea"
```

## Project Structure

```
specgen/
├── src/
│   ├── main.rs          # CLI entry point, argument parsing, command dispatch
│   ├── ai/              # MiniMax AI client integration
│   ├── api_key.rs       # API key validation from environment
│   ├── diff.rs          # Diff generation and comparison
│   ├── domain.rs        # Project domain detection
│   ├── error.rs         # Typed error handling
│   ├── interview.rs     # Interview orchestration and Q&A flow
│   ├── logging.rs       # Structured logging initialization
│   ├── session.rs       # Session persistence
│   ├── spec.rs          # Spec section generation and file I/O
│   └── ui/              # Terminal UI components
├── specs/               # Generated specification files (output directory)
└── Cargo.toml
```

## Specification Sections

Each `specgen new` run generates 10 sections:

| # | Section | File |
|---|---------|------|
| 1 | Requirements | `01-requirements.md` |
| 2 | Architecture | `02-architecture.md` |
| 3 | Features | `03-features.md` |
| 4 | TDD Strategy | `04-tdd.md` |
| 5 | Sequence Diagrams | `05-diagrams.md` |
| 6 | Design Scheme | `06-design.md` |
| 7 | Security | `07-security.md` |
| 8 | SDLC | `08-sdlc.md` |
| 9 | Acceptance Criteria | `09-acceptance.md` |
| 10 | Testing Strategy | `10-testing.md` |

## License

MIT
## Architecture

SpecGen follows a layered, async-first architecture built in Rust. It orchestrates a multi-step pipeline from developer input to a complete specification document, with each module owning a single responsibility.

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI Entry Point                          │
│                        (main.rs, clap)                          │
└──────────────────────────┬──────────────────────────────────────┘
                           │
          ┌────────────────▼────────────────┐
          │     Domain Detection Layer     │
          │   (keyword + AI fallback)      │
          └────────────────┬───────────────┘
                           │
          ┌────────────────▼────────────────┐
          │      Interview Orchestrator      │
          │  (questions → answers → session) │
          └────────────────┬───────────────┘
                           │
          ┌────────────────▼────────────────┐
          │     Spec Generation Engine       │
          │  (10 sections, concurrent AI)   │
          └────────────────┬───────────────┘
                           │
          ┌────────────────▼────────────────┐
          │      File Output Layer          │
          │  (atomic writes, diff/merge)    │
          └─────────────────────────────────┘
```

### Module Breakdown

#### `ai/` — MiniMax API Client

Provides a typed REST client for the MiniMax chat completions API with streaming support.

- **`client.rs`** — Implements `AiClient` trait (for testability) and `MinimaxClient` concrete type. Handles HTTP communication, retry logic with exponential backoff for rate limits (429) and server errors (5xx), and configurable timeouts (30s request, 5s connect).
- **`models.rs`** — Typed request/response models: `ChatRequest`, `ChatResponse`, `StreamChunk`, `Message`, `Role`. Uses Serde for serialization with `skip_serializing_if` for optional fields.
- **`streaming.rs`** — SSE (Server-Sent Events) parser for streaming API responses. Converts byte streams into text chunks, extracts delta content from MiniMax's SSE format, and handles the `[DONE]` signal.

The client uses `Arc<dyn AiClient>` for dependency injection, enabling mock clients in tests.

#### `domain/` — Domain Detection

Detects the software domain from a project idea using a two-tier approach.

- Keyword-based detection maps the input string against curated keyword lists for 10 domains (WebApp, RestApi, GraphQLApi, Cli, MobileApp, DataPipeline, MachineLearning, EmbeddedSystem, GameDev, DesktopApp). More-specific domains are checked first to avoid false positives.
- AI fallback is invoked when keyword detection returns `Unknown` or when confidence is low. It sends a classification prompt to MiniMax and parses the single-word response into a `Domain` enum.

All 11 domain variants (including `Unknown`) implement `display_name()`, `all()`, and `from_string()` with serde serialization.

#### `interview/` — Adaptive Interview Engine

Manages the Q/A flow, generating domain-appropriate questions and collecting validated answers.

- **`orchestrator.rs`** — `InterviewSession` holds the session state: idea, domain, questions, answers, current index, and completion flag. Key methods: `submit_answer()` validates answers and advances state; `skip_current()` handles optional skips; `build_context()` formats Q/A into a string for spec generation.
- **`questions.rs`** — `generate_questions()` builds a domain-adaptive question list. Each domain adds 2-4 specific questions (e.g., GraphQL gets subscription questions, MobileApp gets platform/offline questions). Common questions (tech stack, integrations, security, deployment) are always appended.
- **`answers.rs`** — `Answer` struct tracks question ID, text, and flags (`skipped`, `assumed`). `validate_answer()` enforces a minimum 3-word threshold to prevent empty or trivial responses.

#### `spec/` — 10-Section Spec Generation

Converts interview data into a complete specification document. Each of the 10 sections has its own system prompt, user prompt template, and output filename.

**The 10 sections:**

| Section | File | Role |
|---------|------|------|
| Requirements | `requirements.md` | FR/NFR requirements with IDs (FR-001, ...) |
| Architecture | `architecture.md` | Component design, tech stack, data flow |
| Features | `features.md` | Feature specs with priorities (P0-P2) |
| TDD Strategy | `tdd_strategy.md` | Testing methodology, coverage targets |
| Sequence Diagrams | `sequence_diagrams.md` | Mermaid syntax for key flows |
| Design Scheme | `design_scheme.md` | UI/UX principles, color, typography |
| Security Strategy | `security_strategy.md` | STRIDE analysis, auth, encryption |
| SDLC | `sdlc.md` | Git strategy, CI/CD, deployment |
| Acceptance Criteria | `acceptance_criteria.md` | Testable criteria linked to features |
| Testing Strategy | `testing_strategy.md` | Test pyramid, tools, CI integration |

- **`mod.rs`** — `generate_all_sections()` spawns all 10 generation tasks concurrently via `tokio::spawn`, collecting results and sorting them back into generation order. `generate_section_with_instruction()` builds section-specific prompts and applies post-processing (metadata header/footer).
- **`output.rs`** — `write_spec_file()` uses `NamedTempFile` + `persist()` for atomic writes (no partial files on crash). `write_all_sections()` pre-validates all paths for path-traversal attacks before writing. `validate_path()` rejects any path containing `..`.

#### `session/` — Interview State Persistence

Persists interview context and generation history to disk in `.specgen/session.json`.

- `Session` stores: version, timestamps, idea, domain, interview Q/A entries, generated section list, and refinement history.
- `save_session()` writes atomically via a temp file + rename. `load_session()` reads and parses the JSON.
- Sessions are identified by the current working directory, enabling per-project state management.

#### `diff/` — Text Diff and Merge Engine

Provides semantic diffing and user-edit preservation.

- `diff()` uses the `similar` crate's patience diff algorithm to compare old vs. new content, returning a `DiffResult` with change counts and merged output.
- `merge()` detects `<!-- user-edited -->` markers in existing content and preserves those sections across regeneration.
- `create_conflict()` generates conflict markers (`<!-- CONFLICT: review required -->`) for unresolved differences, including both the AI-generated and existing versions.
- `has_user_edits()`, `extract_user_edits()`, and `has_conflicts()` support conflict detection in the refinement pipeline.

#### `ui/` — Terminal UI Components

Typed theming for the SpecGen TUI built on `ratatui`.

- **`theme.rs`** — All colors defined as named constants with light/dark terminal adaptation. `detect_terminal_background()` reads `COLORFGBG` and `TERM` env vars; `use_plain_text()` respects `NO_COLOR` and `TERM=dumb`. Each semantic color (accent, text, border, background) has light and dark variants resolved at runtime.
- `ProgressStatus` enum (Complete, InProgress, Queued, Error) with emoji indicators and theme-aware colors.
- `KeyBinding` struct with common navigation bindings (arrows, Tab, Ctrl-C, etc.).

#### `logging/` — Structured Logging

Configured via `tracing_subscriber` with support for text and JSON output.

- `init()` sets up the subscriber with an `EnvFilter` and per-target directives. Text mode auto-detects ANSI support; JSON mode omits threading/file info.
- `verbosity_to_level()` maps CLI flags (`-v` → INFO, `-vv` → DEBUG, `-vvv` → TRACE) to `tracing::Level`.
- `redact_api_key()` masks API keys in logs (shows first/last 4 chars only), preventing accidental secret exposure.

#### `api_key/` — Secure API Key Management

Wraps the MiniMax API key using the `secrecy` crate's `SecretString` type to prevent logging or display.

- `read_api_key_from_env()` reads `MINIMAX_API_KEY`, returning a typed error if missing or empty.
- `ApiKey::validate()` enforces minimum length (10 chars) and character restrictions (alphanumeric, `-`, `_`, `.`).

#### `error.rs` — Typed Error Hierarchy

Uses `thiserror` to define a flat enum of `SpecGenError` variants covering all failure modes:

- API errors: `MissingApiKey`, `InvalidApiKey`, `NetworkError`, `HttpError`, `RateLimited`, `InvalidResponse`, `StreamError`
- Domain errors: `InterviewError`, `DomainError`
- File errors: `IoError`, `FileExists`, `InvalidPath`, `SessionError`
- Spec errors: `SpecError`, `DiffError`, `MergeError`
- UI errors: `UiError`, `ConfigError`
- Catch-all: `Unexpected`

Implements `From<std::io::Error>` and `From<reqwest::Error>` for ergonomic error propagation.

### Data Flow

```
User Input (idea)
       │
       ▼
┌──────────────────┐
│  Domain Detection │  keyword match → Domain
│   (domain/mod)    │  Unknown/low confidence → AI fallback
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Interview Session │  domain-specific questions
│ (interview/mod)   │  user answers (or --no-interview assumed)
└────────┬─────────┘
         │
         ├──────────────────────┐
         ▼                      ▼
┌──────────────────┐  ┌──────────────────┐
│ Session Persist  │  │ InterviewContext │
│ (.specgen/*.json) │  │  (idea + domain  │
└──────────────────┘  │   + answers)     │
                      └────────┬─────────┘
                               │
         ┌─────────────────────┴──────────────────────┐
         │                                              │
         ▼                                              ▼
  ┌─────────────────────────────────────────────────────────┐
  │           Spec Generation (concurrent, tokio)           │
  │                                                         │
  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
  │  │ Req.    │ │ Arch.    │ │ Features │ │ TDD      │  │
  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
  │  │ Seq.    │ │ Design   │ │ Security │ │ SDLC     │  │
  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘  │
  │  ┌──────────┐ ┌──────────┐                            │
  │  │ Accept.  │ │ Testing  │                            │
  │  └──────────┘ └──────────┘                            │
  └─────────────────────────────────────────────────────────┘
                               │
                               ▼
                      ┌─────────────────┐
                      │ Atomic File Write│
                      │  (specs/*.md)    │
                      └─────────────────┘
```

### Concurrency Model

All I/O-bound operations (API calls, file I/O) are async via `tokio`. Spec generation is parallelized: all 10 sections are generated concurrently using `tokio::spawn`, then sorted back into canonical order. The AI client uses an async HTTP client (`reqwest`) with SSE streaming. Retry logic for rate limits and server errors is implemented as an async retry loop with exponential backoff, never blocking the tokio runtime.

### CLI Command Architecture

```
specgen new [--no-interview] [idea]   # Full pipeline: detect → interview → generate
specgen refine <instruction> [--sections] # Regenerate specific sections
specgen status [--json]               # Show spec completeness
specgen diff                          # Compare generated vs existing
specgen export                        # Bundle all sections into SPEC.md
```

Commands that don't need the AI (status, diff, export) skip API key validation entirely. The `--no-interview` flag enables pipe mode: the idea is read from stdin and all interview answers are assumed as `[ASSUMED]`, allowing fully automated spec generation from a script.
## Development

### Prerequisites

- **Rust 1.88.0+** — Install via [rustup](https://rustup.rs):

  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup update
  ```

- **MINIMAX_API_KEY** — Required for AI-powered commands (`new`, `refine`). Obtain from [MiniMax](https://platform.minimaxi.com).

### Setup

Clone the repository and install dependencies:

```sh
git clone https://github.com/specgen/specgen.git
cd specgen
cargo fetch
```

Set the required environment variable:

```sh
export MINIMAX_API_KEY="your-api-key-here"
```

For convenience, add it to your shell profile or use a `.env` file (not committed):

```sh
# .env (gitignored)
MINIMAX_API_KEY=your-api-key-here
```

### Build

```sh
# Development build (fast compile, debug symbols)
cargo build

# Release build (optimized, stripped)
cargo build --release

# Build with colored output feature
cargo build --features colored
```

The release binary is located at `target/release/specgen` and includes LTO, opt-level=z, and binary stripping for minimal size.

### Test

```sh
# Run all tests including unit and integration tests
cargo test

# Run tests with output visible (don't capture stdout)
cargo test -- --nocapture

# Run only doc tests
cargo test --doc

# Run with verbose output
cargo test -v
```

The project uses `pretty_assertions`, `proptest`, `insta` (snapshot testing with YAML), and `wiremock` for HTTP mocking in tests.

### Environment Variables

| Variable | Required | Description |
|---|---|---|
| `MINIMAX_API_KEY` | For `new`/`refine` commands | MiniMax API key for AI generation |
| `SPECGEN_LOG_FORMAT` | No | Log output format: `json` for structured JSON, empty for pretty print |
| `RUST_LOG` | No | Override default `specgen=info` log filter (used by `tracing_subscriber`) |

Verbosity is controlled via CLI flags (`-v` / `-vv` / `-vvv`) and maps to logging levels as follows:

- No flag → `ERROR`
- `-v` → `INFO`
- `-vv` → `DEBUG`
- `-vvv` → `TRACE`

### Project Structure

```
src/
├── main.rs           # CLI entry point, argument parsing, command dispatch
├── error.rs          # Typed error hierarchy (thiserror)
├── logging.rs        # Structured logging initialization (tracing)
├── ai/               # MiniMax AI client and streaming
│   ├── client.rs     # HTTP client for MiniMax API
│   ├── models.rs     # Request/response models
│   ├── streaming.rs  # SSE stream parsing
│   └── mod.rs
├── api_key/          # API key loading and validation
├── domain/           # Domain detection (keyword + AI fallback)
├── interview/        # Q&A session management
│   ├── orchestrator.rs
│   ├── questions.rs
│   ├── answers.rs
│   └── mod.rs
├── spec/             # Spec generation and file output
│   ├── mod.rs        # Section definitions (10 sections)
│   ├── output.rs     # File writing utilities
│   └── mod.rs
├── session/          # Session persistence (JSON to disk)
├── diff/             # Diff and merge engine (similar crate)
└── ui/               # Terminal UI (ratatui-based)
    ├── mod.rs
    └── theme.rs
```

The codebase follows domain-driven design with clear separation: AI integration in `ai/`, business logic in `domain/` and `interview/`, specification generation in `spec/`, and infrastructure concerns (persistence, logging, errors) isolated from domain logic.

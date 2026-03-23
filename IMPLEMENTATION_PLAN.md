# Implementation Plan

This document tracks the implementation of SpecGen CLI against the specifications in `specs/*.md`.

## Project Foundation (Prerequisites)

These tasks must be completed first as other tasks depend on them.

- [ ] **Setup Rust project structure** - Initialize Cargo.toml with all dependencies from spec 02_architecture.md
- [ ] **Create rust-toolchain.toml** - Pin MSRV to 1.75.0 as specified
- [ ] **Create .cargo/config.toml** - Enable deny(warnings) for release builds
- [ ] **Setup basic error handling** - Implement SpecGenError enum with thiserror
- [ ] **Verify empty project compiles** - cargo build passes with no warnings

## Core P0 Features (Must Implement First)

### F-01 — API Key Validation

- [ ] **Implement ApiKey type** - Wrap MINIMAX_API_KEY in SecretString from secrecy crate
- [ ] **Implement environment variable reading** - Read key at startup, fail fast if missing
- [ ] **Add CLI argument parsing** - Setup clap with new, refine, status, diff, export commands
- [ ] **Implement credential validation** - Ping MiniMax API to validate key before interview
- [ ] **Write unit tests for API key validation** - Test missing key, empty key, invalid key

### F-02 — Domain Detection

- [ ] **Define Domain enum** - WebApp, RestApi, GraphQLApi, CLI, MobileApp, DataPipeline, MachineLearning, EmbeddedSystem, GameDev, DesktopApp, Unknown
- [ ] **Implement keyword-based detection** - Heuristic classifier for common keywords (web, SaaS, API, CLI, mobile, etc.)
- [ ] **Implement MiniMax fallback classifier** - Call API for ambiguous ideas
- [ ] **Create domain confirmation prompt** - Show detected domain to user before interview
- [ ] **Write unit tests for domain detection** - 11 domain variants, serialization

### F-03 — Adaptive Interview Engine

- [ ] **Implement question generator** - Generate questions via MiniMax API based on context
- [ ] **Implement answer quality gate** - Reject answers < 3 words
- [ ] **Implement skip/unsure handling** - Flag questions for assumption generation
- [ ] **Implement question counter** - Show "Question N of ~M" progress
- [ ] **Implement interview orchestrator** - Manage Q/A flow, collect all answers
- [ ] **Write unit tests for interview engine** - Question count bounds, answer validation, skip handling

### F-04 — MiniMax Streaming Client

- [ ] **Define AiClient trait** - Enable mock injection for testing
- [ ] **Implement MinimaxClient** - Typed REST client with reqwest
- [ ] **Implement SSE streaming** - Parse streaming tokens from MiniMax API
- [ ] **Implement retry logic** - Exponential backoff for 429/5xx errors (max 3 retries)
- [ ] **Add TLS configuration** - Enforce rustls with TLS 1.2+
- [ ] **Write integration tests** - Retry behavior, stream parsing, error handling

### F-05 — Spec Section Generation

- [ ] **Implement spec module orchestration** - Dispatch 10 concurrent generation tasks
- [ ] **Implement requirements.md generator** - System prompt + context → Markdown
- [ ] **Implement architecture.md generator** - Include tech stack recommendation table
- [ ] **Implement features.md generator** - Per-feature software requirements
- [ ] **Implement tdd_strategy.md generator** - Test-first plan with example stubs
- [ ] **Implement sequence_diagrams.md generator** - Mermaid syntax with descriptive names
- [ ] **Implement design_scheme.md generator** - UI/UX conventions
- [ ] **Implement security_strategy.md generator** - Threat model table
- [ ] **Implement sdlc.md generator** - CI/CD pipeline, branching
- [ ] **Implement acceptance_criteria.md generator** - Derived from TDD strategy
- [ ] **Implement testing_strategy.md generator** - Test pyramid, tools
- [ ] **Write integration tests** - All sections generated correctly, concurrent generation

### F-06 — Atomic File Output

- [ ] **Implement atomic file writer** - Write to temp file, then rename
- [ ] **Implement file existence check** - Block if files exist without diff
- [ ] **Implement Utf8PathBuf usage** - No raw PathBuf in file I/O
- [ ] **Add generation header comment** - Include version and date in each file
- [ ] **Write unit tests** - Atomic writes, path traversal prevention

## P1 Features (Important)

### F-07 — Ratatui Rich UI

- [ ] **Implement UI theme constants** - Colors defined in ui/theme.rs
- [ ] **Implement main TUI app** - Three-panel layout as specified
- [ ] **Implement interview panel** - Question display + answer input
- [ ] **Implement progress panel** - Per-section progress bars
- [ ] **Implement preview panel** - Syntax-highlighted Markdown preview
- [ ] **Implement footer bar** - Key bindings display
- [ ] **Handle NO_COLOR and TERM=dumb** - Graceful degradation to plain text
- [ ] **Handle terminal resize** - No crashes on resize events

### F-08 — Iterative Refinement

- [ ] **Implement refinement classifier** - Identify target sections from instruction
- [ ] **Implement targeted regeneration** - Only regenerate targeted sections
- [ ] **Implement refinement context** - Include original content + interview + instruction
- [ ] **Write integration tests** - Section targeting, non-targeted sections unchanged

### F-09 — Diff and Merge Engine

- [ ] **Implement semantic diff** - Using similar crate with patience algorithm
- [ ] **Implement user-edited preservation** - Detect and preserve <!-- user-edited --> markers
- [ ] **Implement conflict marker injection** - Wrap conflicts in <!-- CONFLICT --> markers
- [ ] **Implement merge output** - Compose valid Markdown output
- [ ] **Write unit tests** - User-edited sections preserved, conflict markers correct

### F-10 — Session Persistence & Resume

- [ ] **Implement session storage** - Read/write .specgen/session.json
- [ ] **Implement atomic session writes** - Temp file + rename
- [ ] **Implement session schema** - Version, created_at, idea, domain, interview array
- [ ] **Implement session resume** - Offer to resume existing session
- [ ] **Implement schema migration** - Handle version upgrades
- [ ] **Write integration tests** - Persistence, resume, corruption handling

## P2 Features (Nice to Have)

### F-11 — Status Command

- [ ] **Implement status command** - List all spec files with existence/missing
- [ ] **Implement completeness score** - Calculate 0-100% based on generated sections
- [ ] **Implement JSON output** - --json flag for machine-readable output
- [ ] **Write E2E tests** - Status output validation

### F-12 — Export Command

- [ ] **Implement export command** - Bundle all sections into single SPEC.md
- [ ] **Implement TOC generation** - Table of contents with anchor links
- [ ] **Implement metadata block** - Include project name, date, version
- [ ] **Write E2E tests** - Export bundle validation

### F-13 — Non-interactive / Pipe Mode

- [ ] **Implement --no-interview flag** - Skip interactive interview
- [ ] **Implement AI-assumed answers** - Mark all answers as [ASSUMED]
- [ ] **Handle piped input** - Accept idea from stdin
- [ ] **Write E2E tests** - Pipe mode validation

### F-14 — Verbose Logging

- [ ] **Implement tracing setup** - Structured logging with tracing crate
- [ ] **Implement -v flags** - INFO, DEBUG, TRACE levels
- [ ] **Implement JSON log format** - SPECGEN_LOG_FORMAT=json support
- [ ] **Ensure API key never logged** - Redact in all code paths
- [ ] **Write integration tests** - Log output validation

## CI/CD & DevOps

- [ ] **Create .github/workflows/ci.yml** - PR pipeline with test, clippy, fmt, audit
- [ ] **Create .github/workflows/release.yml** - Release pipeline with cross-compile
- [ ] **Setup cargo-llvm-cov** - Coverage gate (80% line, 70% branch)
- [ ] **Setup cargo-audit** - Security scanning in CI

## Testing Infrastructure

- [ ] **Create test fixtures** - Idea inputs and API response fixtures
- [ ] **Setup wiremock** - Mock MiniMax API for integration tests
- [ ] **Setup insta** - Snapshot testing
- [ ] **Setup proptest** - Property-based testing

## In Progress

<!-- Tasks currently being worked on -->

## Completed

<!-- Completed tasks (can be periodically cleaned out) -->

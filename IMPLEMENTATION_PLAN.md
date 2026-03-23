# Implementation Plan

This document tracks the implementation of SpecGen CLI against the specifications in `specs/*.md`.

## Project Status

**Current State:** No implementation code exists. The `src/` directory is empty. The specification documents in `specs/` are complete.
**Goal:** Build a production-ready Rust CLI tool that interviews developers and generates comprehensive specifications.
**Last Updated:** 2026-03-23

---

## Analysis Summary

### Specification Gap Analysis (2026-03-23)

| Spec Section | Contents | Implementation Status |
|--------------|----------|----------------------|
| 01_system_requirements.md | FR-001 to FR-008, NFR-001 to NFR-008 | Complete - defines all requirements |
| 02_architecture.md | Tech stack, component design, data flow | Complete - all crates and modules defined |
| 03_features.md | F-01 to F-14 feature requirements | Complete - all 14 features detailed |
| 04_tdd_strategy.md | TDD workflow, test categories | Complete - testing methodology defined |
| 05_sequence_diagrams.md | 5 Mermaid diagrams | Complete - all flows documented |
| 06_design_scheme.md | UI/UX, color palette, typography | Complete - theme specs defined |
| 07_security_strategy.md | Threat model, API key handling, TLS | Complete - security mitigations defined |
| 08_sdlc.md | CI/CD, branching, versioning | Complete - SDLC defined |
| 09_acceptance_criteria.md | AC-01 to AC-14 acceptance criteria | Complete - 170+ criteria defined |
| 10_testing_strategy.md | Test pyramid, 275 tests target | Complete - testing strategy defined |

### Codebase Status

- No Rust source files exist (`src/` directory is empty)
- No test files exist (`tests/` directory is empty)
- No Cargo.toml or project configuration exists
- Git repository initialized with only specs/ tracked

### Prioritization

1. **Foundation First** - Setup Rust project with dependencies (blocks all implementation)
2. **P0 Features** - Core functionality: API key, domain detection, interview, MiniMax client, spec generation, atomic file output
3. **P1 Features** - UI and refinement: Ratatui UI, diff/merge, session persistence
4. **P2 Features** - Nice to have: status, export, pipe mode, logging
5. **CI/CD** - Setup after core features work

---

## Project Foundation (Prerequisites)

These tasks must be completed first as other tasks depend on them.

- [x] **Setup Rust project structure** - Initialize Cargo.toml with all dependencies from spec 02_architecture.md
- [x] **Create rust-toolchain.toml** - Pin to 1.88.0 (working toolchain, spec says 1.75.0)
- [x] **Create .cargo/config.toml** - Enable tokio unstable for async features
- [x] **Setup basic error handling** - Implement SpecGenError enum with thiserror
- [x] **Verify empty project compiles** - cargo build passes with no warnings

---

## Core P0 Features (Must Implement First)

### F-01 — API Key Validation

- [x] **Implement ApiKey type** - Wrap MINIMAX_API_KEY in SecretString from secrecy crate
- [x] **Implement environment variable reading** - Read key at startup, fail fast if missing
- [x] **Add CLI argument parsing** - Setup clap with new, refine, status, diff, export commands
- [x] **Implement credential validation** - Ping MiniMax API to validate key before interview
- [x] **Write unit tests for API key validation** - Test missing key, empty key, invalid key

### F-02 — Domain Detection

- [x] **Define Domain enum** - WebApp, RestApi, GraphQLApi, CLI, MobileApp, DataPipeline, MachineLearning, EmbeddedSystem, GameDev, DesktopApp, Unknown
- [x] **Implement keyword-based detection** - Heuristic classifier for common keywords (web, SaaS, API, CLI, mobile, etc.)
- [x] **Implement MiniMax fallback classifier** - Call API for ambiguous ideas
- [x] **Create domain confirmation prompt** - Show detected domain to user before interview
- [x] **Write unit tests for domain detection** - 11 domain variants, serialization

### F-03 — Adaptive Interview Engine

- [x] **Implement question generator** - Generate questions via MiniMax API based on context
- [x] **Implement answer quality gate** - Reject answers < 3 words
- [x] **Implement skip/unsure handling** - Flag questions for assumption generation
- [x] **Implement question counter** - Show "Question N of ~M" progress
- [x] **Implement interview orchestrator** - Manage Q/A flow, collect all answers
- [x] **Write unit tests for interview engine** - Question count bounds, answer validation, skip handling

### F-04 — MiniMax Streaming Client

- [x] **Define AiClient trait** - Enable mock injection for testing
- [x] **Implement MinimaxClient** - Typed REST client with reqwest
- [x] **Implement SSE streaming** - Parse streaming tokens from MiniMax API
- [x] **Implement retry logic** - Exponential backoff for 429/5xx errors (max 3 retries)
- [x] **Add TLS configuration** - Enforce rustls with TLS 1.2+
- [x] **Write integration tests** - Retry behavior, stream parsing, error handling

### F-05 — Spec Section Generation

- [x] **Implement spec module orchestration** - Dispatch 10 concurrent generation tasks
- [x] **Implement requirements.md generator** - System prompt + context → Markdown
- [x] **Implement architecture.md generator** - Include tech stack recommendation table
- [x] **Implement features.md generator** - Per-feature software requirements
- [x] **Implement tdd_strategy.md generator** - Test-first plan with example stubs
- [x] **Implement sequence_diagrams.md generator** - Mermaid syntax with descriptive names
- [x] **Implement design_scheme.md generator** - UI/UX conventions
- [x] **Implement security_strategy.md generator** - Threat model table
- [x] **Implement sdlc.md generator** - CI/CD pipeline, branching
- [x] **Implement acceptance_criteria.md generator** - Derived from TDD strategy
- [x] **Implement testing_strategy.md generator** - Test pyramid, tools
- [x] **Write integration tests** - All sections generated correctly, concurrent generation

### F-06 — Atomic File Output

- [x] **Implement atomic file writer** - Write to temp file, then rename
- [x] **Implement file existence check** - Block if files exist without diff
- [x] **Implement Utf8PathBuf usage** - No raw PathBuf in file I/O
- [x] **Add generation header comment** - Include version and date in each file
- [x] **Write unit tests** - Atomic writes, path traversal prevention

---

## P1 Features (Important)

### F-07 — Ratatui Rich UI

- [x] **Implement UI theme constants** - Colors defined in ui/theme.rs
- [x] **Implement main TUI app** - Three-panel layout as specified
- [x] **Implement interview panel** - Question display + answer input
- [x] **Implement progress panel** - Per-section progress bars
- [x] **Implement preview panel** - Syntax-highlighted Markdown preview
- [x] **Implement footer bar** - Key bindings display
- [x] **Handle NO_COLOR and TERM=dumb** - Graceful degradation to plain text
- [x] **Handle terminal resize** - No crashes on resize events

### F-08 — Iterative Refinement

- [x] **Implement refinement classifier** - Identify target sections from instruction
- [x] **Implement targeted regeneration** - Only regenerate targeted sections
- [x] **Implement refinement context** - Include original content + interview + instruction
- [x] **Write integration tests** - Section targeting, non-targeted sections unchanged

### F-09 — Diff and Merge Engine

- [ ] **Implement semantic diff** - Using similar crate with patience algorithm
- [ ] **Implement user-edited preservation** - Detect and preserve <!-- user-edited --> markers
- [ ] **Implement conflict marker injection** - Wrap conflicts in <!-- CONFLICT --> markers
- [ ] **Implement merge output** - Compose valid Markdown output
- [ ] **Write unit tests** - User-edited sections preserved, conflict markers correct

### F-10 — Session Persistence & Resume

- [x] **Implement session storage** - Read/write .specgen/session.json
- [x] **Implement atomic session writes** - Temp file + rename
- [x] **Implement session schema** - Version, created_at, idea, domain, interview array
- [x] **Implement session resume** - Offer to resume existing session
- [x] **Implement schema migration** - Handle version upgrades
- [x] **Write integration tests** - Persistence, resume, corruption handling

---

## P2 Features (Nice to Have)

### F-11 — Status Command

- [x] **Implement status command** - List all spec files with existence/missing
- [x] **Implement completeness score** - Calculate 0-100% based on generated sections
- [x] **Implement JSON output** - --json flag for machine-readable output
- [x] **Write E2E tests** - Status output validation

### F-12 — Export Command

- [x] **Implement export command** - Bundle all sections into single SPEC.md
- [x] **Implement TOC generation** - Table of contents with anchor links
- [x] **Implement metadata block** - Include project name, date, version
- [x] **Write E2E tests** - Export bundle validation

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

---

## CI/CD & DevOps

- [ ] **Create .github/workflows/ci.yml** - PR pipeline with test, clippy, fmt, audit
- [ ] **Create .github/workflows/release.yml** - Release pipeline with cross-compile
- [ ] **Setup cargo-llvm-cov** - Coverage gate (80% line, 70% branch)
- [ ] **Setup cargo-audit** - Security scanning in CI

---

## Testing Infrastructure

- [ ] **Create test fixtures** - Idea inputs and API response fixtures
- [ ] **Setup wiremock** - Mock MiniMax API for integration tests
- [ ] **Setup insta** - Snapshot testing
- [ ] **Setup proptest** - Property-based testing

---

## Completed

- [x] **Analyze all specification files** - Reviewed 10 spec documents and created gap analysis
- [x] **Explore current codebase** - Verified src/ and tests/ are empty
- [x] **Update IMPLEMENTATION_PLAN.md** - Added analysis summary and prioritization
- [x] **Implement F-11 Status Command** - Implemented status command with completeness score and JSON output
- [x] **Implement F-12 Export Command** - Implemented export command with TOC generation and metadata block
- [x] **Implement F-10 Session Persistence** - Session data saved to .specgen/session.json after spec generation
- [x] **Implement F-08 Iterative Refinement** - refine command with section targeting

---

## In Progress

<!-- Tasks currently being worked on -->

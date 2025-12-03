# Implementation Tasks: Core Infrastructure

**Feature**: Core Infrastructure | **Priority**: P1, P2, P3, P4 | **Status**: Ready to Implement
**Plan**: [Implementation Plan](plan.md) | **Spec**: [Feature Specification](spec.md) | **Task Board**: tasks.md
**Input**: User stories with independent test criteria and acceptance scenarios
**Strategy**: MVP-first with incremental delivery by user story priority (P1 then P2 then P3 then P4)

## Dependencies & Parallel Execution

**Execution Order**: (P1) Setup → Foundational → P2 → P3 → P4
**Parallel Opportunities**: All model/service implementations can run in parallel after foundational phase
**Cross-Story Dependencies**: US2 (Document Sync) requires US1 (LSP Lifecycle) to be functional
**MVP Scope**: US1 (LSP Lifecycle) provides independently testable functionality

## Phase 1 - Project Setup

Setup tasks needed before implementation can begin.

- [X] T001 Create project workspace with crates/php-lsp/ and Cargo.toml
- [X] T002 Add dependencies to Cargo.toml: async-lsp 0.2.2, tokio runtime, tree-sitter-php 0.24.2, ropey, lsp-types 0.97, tracing, thiserror/anyhow
- [X] T003 Create initial project structure per plan: src/main.rs, src/lib.rs, src/server/ directory
- [X] T004 Create zed-php-lsp crate with extension.toml and WASM configuration
- [X] T005 [P] Create server module files: lifecycle.rs, document_sync.rs, parsing.rs, transport.rs, types.rs

## Phase 2 - Foundational Implementation

Foundational tasks that block all user stories - must complete before any user story implementation.

- [X] T006 Define basic server state structure (DashMap for documents, capabilities, initialization flags)
- [X] T007 Implement command-line argument parsing for `--stdio` flag
- [X] T008 Set up tracing/logging infrastructure per constitution requirements
- [X] T009 Create error types with thiserror crate per constitution requirements
- [X] T010 Define core data types per data model: Document, AST, TransportMessage
- [X] T011 Create AST wrapper structure with tree-sitter integration
- [X] T012 Implement Document structure with ropey integration
- [X] T013 Set up stdio transport infrastructure
- [X] T014 Implement basic async-lsp server setup with placeholder handlers

## Phase 3 - User Story 1: LSP Lifecycle (Priority: P1)

As a Zed user, I want the PHP LSP to properly initialize and shutdown so that my editor integrates smoothly.

**Independent Test**: The LSP server can be started, responds to initialization requests with its capabilities, and shuts down cleanly when requested.

- [X] T015 [US1] Implement initialize request handler with required capabilities (FR-001)
- [X] T016 [US1] Add server capabilities configuration (textDocumentSync, hoverProvider, etc.)
- [X] T017 [US1] Implement initialized notification handler (FR-002)
- [X] T018 [US1] Implement shutdown request handler (FR-003)
- [X] T019 [US1] Implement exit notification handler (FR-004)
- [X] T020 [US1] Add server state flags for initialization and shutdown tracking
- [X] T021 [US1] Add server initialization performance metrics (SC-001)
- [X] T022 [US1] Test LSP lifecycle with mock client

## Phase 4 - User Story 2: Document Synchronization (Priority: P2)

As a developer, I want my PHP files to be tracked by the LSP so that I get real-time feedback.

**Independent Test**: The LSP can open, track changes to, and close PHP documents correctly, maintaining internal state that reflects the current content.

- [X] T023 [US2] Implement textDocument/didOpen notification handler (FR-005)
- [X] T024 [US2] Implement document storage in DashMap with URI as key
- [X] T025 [US2] Implement textDocument/didChange notification handler with incremental sync (FR-006)
- [X] T026 [US2] Implement textDocument/didClose notification handler (FR-007)
- [X] T027 [US2] Add document synchronization performance tracking (SC-002)
- [X] T028 [US2] Implement document validation (URI, version, UTF-8 content checks)
- [X] T029 [US2] Add memory usage monitoring for document storage (SC-004)
- [X] T030 [US2] Test document synchronization with mock client

## Phase 5 - User Story 3: PHP Parsing (Priority: P3)

As a developer, I want PHP files to be parsed correctly so that all language features work.

**Independent Test**: The LSP can parse a PHP document and create an AST representation that other features can use for analysis.

- [ ] T031 [US3] Integrate tree-sitter-php parsing library
- [ ] T032 [US3] Implement document parsing function with tree-sitter (FR-008)
- [ ] T033 [US3] Implement AST caching with document version tracking
- [ ] T034 [US3] Add error handling for parse errors with partial AST support (FR-012)
- [ ] T035 [US3] Add support for PHP 8+ syntax features (FR-013, FR-015)
- [ ] T036 [US3] Add PHP 7.4+ syntax validation and support
- [ ] T037 [US3] Implement parsing performance metrics and success rate tracking (SC-003)
- [ ] T038 [US3] Test PHP parsing with various syntax versions and error conditions

## Phase 6 - User Story 4: stdio Transport (Priority: P4)

As a Zed extension, I want the LSP to communicate via stdio so that it integrates with Zed.

**Independent Test**: The LSP can start with `--stdio` flag and properly read requests from stdin and write responses to stdout.

- [ ] T039 [US4] Implement command-line flag parsing for `--stdio` (FR-009)
- [ ] T040 [US4] Implement JSON-RPC request reading from stdin (FR-010)
- [ ] T041 [US4] Implement JSON-RPC response writing to stdout (FR-011)
- [ ] T042 [US4] Add transport error handling for malformed JSON-RPC messages
- [ ] T043 [US4] Implement graceful degradation for partial results (FR-016)
- [ ] T044 [US4] Add structured logging for transport layer (FR-014)
- [ ] T045 [US4] Test stdio communication with integration tests (SC-005)
- [ ] T046 [US4] Test long-running server stability (8+ hours)

## Phase 7 - Polish & Cross-Cutting Concerns

Final implementation and cross-cutting concerns to complete the feature.

- [ ] T047 Implement support for up to 1,000 documents with performance monitoring (SC-006)
- [ ] T048 Implement memory-efficient processing for large documents (>10,000 lines) with configurable limits and progress reporting (FR-017, Edge Case: Large file handling)
- [ ] T049 Implement graceful degradation for syntax errors (FR-016)
- [ ] T050 Define and implement performance benchmarks for key metrics: initialization time, document sync operations, parsing speed, and memory usage per document count (FR-017)
- [ ] T051 Add comprehensive logging for observability with structured JSON format and performance metrics (FR-018)
- [ ] T052 Write integration tests covering all user stories
- [ ] T053 Document the API and usage in README
- [ ] T054 Test end-to-end integration with Zed editor
- [ ] T055 Performance optimization and memory usage validation
- [ ] T056 Implement error handling for malformed JSON-RPC messages with appropriate logging and client responses (Edge Case: Malformed JSON-RPC)
- [ ] T057 Implement error handling for severe syntax errors with partial AST generation and graceful degradation (Edge Case: Severe syntax errors)

## Implementation Strategy (MVP First)

The MVP scope includes only Phase 1 (Setup), Phase 2 (Foundational), and Phase 3 (LSP Lifecycle). This delivers independently testable functionality where the LSP server can be started, handles initialization, and shuts down cleanly. Each subsequent phase builds on the previous to incrementally deliver value according to user story priorities.

**MVP Completion Criteria**:
- Server starts with `--stdio` flag
- Responds to `initialize` request with proper capabilities
- Handles `initialized`, `shutdown`, and `exit` notifications
- Properly tracks initialization/shutdown state
- All handlers are async and non-blocking
- Uses structured tracing for logging
- Proper error handling with thiserror/anyhow
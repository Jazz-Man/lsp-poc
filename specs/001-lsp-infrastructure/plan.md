# Implementation Plan: Core Infrastructure

**Branch**: `001-lsp-infrastructure` | **Date**: 2025-12-03 | **Spec**: [Feature Specification](spec.md)
**Input**: Feature specification from `/specs/001-lsp-infrastructure/spec.md`

**Note**: This template is filled in by the `/sp.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement the foundational LSP server infrastructure that handles lifecycle management, document synchronization, and PHP parsing with tree-sitter. This includes LSP initialization/shutdown, document tracking, PHP 7.4+ parsing with tree-sitter, and stdio communication for Zed editor integration.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: async-lsp 0.2.2, tokio runtime, tree-sitter-php 0.24.2, ropey, lsp-types 0.97, tracing, thiserror/anyhow
**Storage**: In-memory document storage using DashMap, no persistent storage
**Testing**: cargo test with unit and integration tests
**Target Platform**: Linux/macOS/WASM (for Zed integration via WebAssembly extension)
**Project Type**: Single binary LSP server
**Performance Goals**: <5s initialization, <100ms sync operations for documents up to 10,000 lines, memory usage <200MB for 50 documents, support up to 1,000 documents
**Constraints**: <200MB memory for 50 documents, <5s initialization time, support for PHP 7.4+ syntax
**Scale/Scope**: Support up to 1,000 simultaneous documents, 8+ hours continuous operation without crashes

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Based on the PHP LSP Server Constitution, this implementation plan must adhere to the following principles:
1. DOCUMENTATION FIRST: Before writing any code, read documentation in `target/doc-md/` for available crates and APIs (async-lsp, tree-sitter, etc.)
2. ITERATIVE DEVELOPMENT: Write MAX 20-30 lines at a time, run `cargo check` after every change, fix errors immediately
3. USE EXISTING FUNCTIONALITY: Leverage async-lsp framework capabilities rather than implementing LSP protocol from scratch
4. CODE QUALITY STANDARDS: Use async handlers, proper error handling with thiserror/anyhow, structured tracing instead of println
5. PHP & WORDPRESS SPECIFICS: Support PHP 7.4+ syntax fully, including PHP 8+ features like attributes and named arguments
6. SPECIFICATION-DRIVEN DEVELOPMENT: Follow the detailed specifications created in the previous phase

## Project Structure

### Documentation (this feature)

```text
specs/001-lsp-infrastructure/
├── plan.md              # This file (/sp.plan command output)
├── research.md          # Phase 0 output (/sp.plan command)
├── data-model.md        # Phase 1 output (/sp.plan command)
├── quickstart.md        # Phase 1 output (/sp.plan command)
├── contracts/           # Phase 1 output (/sp.plan command)
└── tasks.md             # Phase 2 output (/sp.tasks command - NOT created by /sp.plan)
```

### Source Code (repository root)

```text
crates/
├── php-lsp/             # Main LSP server
│   ├── src/
│   │   ├── main.rs      # Entry point with --stdio
│   │   ├── lib.rs       # Library exports
│   │   └── server/      # Server modules
│   │       ├── lifecycle.rs      # LSP lifecycle handlers
│   │       ├── document_sync.rs  # Document synchronization
│   │       ├── parsing.rs        # PHP parsing with tree-sitter
│   │       ├── transport.rs      # Transport layer (stdio)
│   │       └── types.rs          # Type definitions
│   └── Cargo.toml
├── zed-php-lsp/         # Zed extension (WASM)
│   ├── src/lib.rs
│   ├── extension.toml
│   └── Cargo.toml
├── Cargo.toml           # Workspace
└── .scripts/
    └── regen-docs.sh    # Generate documentation for AI
```

**Structure Decision**: Single binary LSP server in Rust using async-lsp framework with a Zed extension for WASM integration. The implementation follows the documented architecture with separate modules for each major function area (lifecycle, document sync, parsing, transport).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |

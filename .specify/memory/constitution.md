<!-- 
Sync Impact Report:
- Version change: N/A → 1.0.0
- Modified principles: N/A (new constitution)
- Added sections: All principles and sections
- Removed sections: N/A
- Templates requiring updates: ⚠ pending - .specify/templates/plan-template.md, .specify/templates/spec-template.md, .specify/templates/tasks-template.md, .specify/templates/commands/*.md
- Follow-up TODOs: RATIFICATION_DATE needs to be set to actual ratification date
-->
# PHP LSP Server Constitution

## Core Principles

### DOCUMENTATION FIRST
Before writing ANY code: Read documentation from `target/doc-md/` for available crates and APIs; Use `cargo info <crate>` to check versions and features; DO NOT invent APIs — use ONLY what exists in documentation; If documentation is missing, run `.scripts/regen-docs.sh`

### ITERATIVE DEVELOPMENT
Write MAX 20-30 lines at a time; After EVERY change: `cargo check`; If errors: FIX IMMEDIATELY before continuing; After fix confirmed: `git commit -m "..."; NEVER proceed with broken code; NEVER say "errors exist but out of scope"

### USE EXISTING FUNCTIONALITY
Check if feature exists in dependencies before implementing; async-lsp likely has what you need — READ ITS DOCS; Don't reinvent wheels; Prefer composition over custom implementations

### CODE QUALITY STANDARDS
All handlers must be async and non-blocking; Error handling with thiserror/anyhow (no unwrap in production code); Tracing for all logging (not println!); Tests for each module; Documentation comments for public APIs

### PHP & WORDPRESS SPECIFICS
Support PHP 7.4+ syntax fully with special attention to PHP 8+ features; Parse PHPDoc annotations: @param, @return, @var, @template, @psalm-*, @phpstan-*; WordPress Hook API: all 18 functions with go-to-definition; composer.json integration: PHP version detection, ext-* warnings, PSR-4/PSR-0 autoload

### SPECIFICATION-DRIVEN DEVELOPMENT (SDD)
Follow Specification-Driven Development (SDD) principles with spec-kit-plus; Create detailed specs before implementation; Break down features into testable tasks; Ensure all outputs strictly follow user intent

## Technology Stack Requirements
Language: Rust (edition 2021); LSP Framework: async-lsp 0.2.2 with tokio runtime; PHP Parser: tree-sitter-php 0.24.2; Text Handling: ropey for rope-based incremental text; LSP Types: lsp-types 0.97; Target Editor: Zed (via WebAssembly extension using zed_extension_api)

## Development Workflow
Follow feature phases in order: 1) Core Infrastructure (LSP lifecycle, document sync, PHP parsing), 2) Symbol Navigation (Document symbols, go-to-definition, references), 3) Code Completion (Variables, members, classes, signature help), 4) WordPress Hooks (Hook navigation, completion, hover), 5) Composer Integration (PHP version, autoload, vendor navigation); Each task follows implementation contract: Read docs → Write code (max 30 lines) → cargo check → Fix errors → Commit → Next task

## Governance
Constitution supersedes all other practices; Amendments require documentation, approval, migration plan; All PRs/reviews must verify compliance; Complexity must be justified; Use development principles for runtime guidance

**Version**: 1.0.0 | **Ratified**: TODO(RATIFICATION_DATE): Original adoption date unknown | **Last Amended**: 2025-12-03
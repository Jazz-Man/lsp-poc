# Technology Stack

## Architecture

Rust-based workspace with multiple crates: a core LSP implementation library and a Zed editor extension. Follows LSP protocol standards with potential for additional PHP-specific features like WordPress hook recognition.

## Core Technologies

- **Language**: Rust (edition 2021)
- **LSP Framework**: async-lsp (with async-io and tokio support)
- **Parsing**: Tree-sitter (tree-sitter-php parser)
- **Editor Integration**: Zed Extension API

## Key Libraries

- **async-lsp**: Asynchronous LSP implementation framework with stdio and TCP transport
- **tree-sitter**: Incremental parsing library for efficient syntax tree management
- **tree-sitter-php**: PHP grammar for Tree-sitter parser
- **zed_extension_api**: Safe Rust API for Zed editor extensions

## Development Standards

### Type Safety
- Rust's ownership and type system for memory safety
- No manual memory management required
- Compile-time error checking

### Code Quality
- Standard Rust formatting (rustfmt)
- Clippy linting for best practices
- Cargo for dependency management

### Testing
- Standard Rust testing framework (cargo test)
- Integration tests for LSP protocol compliance
- Language parsing validation tests

## Development Environment

### Required Tools
- Rust toolchain (stable)
- Cargo package manager
- Tree-sitter CLI (for grammar development)

### Common Commands
```bash
# Dev: cargo run (for development)
# Build: cargo build --release
# Test: cargo test
# Check: cargo check
```

## Key Technical Decisions

- **Rust for LSP Implementation**: Memory safety and performance benefits over traditional PHP-based LSP servers
- **Tree-sitter Integration**: Efficient incremental parsing for real-time language analysis
- **Workspace Structure**: Multi-crate approach separating core LSP functionality from editor integrations
- **Async-first Architecture**: Asynchronous processing for improved responsiveness

---
_Document standards and patterns, not every dependency_
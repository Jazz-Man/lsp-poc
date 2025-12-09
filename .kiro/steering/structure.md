# Project Structure

## Organization Philosophy

Multi-crate Rust workspace following the cargo workspace pattern. Separates the core LSP implementation from editor-specific integrations while maintaining a unified build and dependency management system.

## Directory Patterns

### Core LSP Implementation (`/crates/php-lsp/`)
**Purpose**: Core Language Server Protocol implementation for PHP
**Files**: 
- `src/lib.rs`: Main library entry point with server module
- `src/main.rs`: Binary entry point for standalone server
- `src/server/`: LSP server implementation
- `Cargo.toml`: Dependencies and package configuration

### Zed Editor Extension (`/crates/zed-php-lsp/`)
**Purpose**: Zed editor extension that wraps the core LSP server
**Files**:
- `src/lib.rs`: Zed extension implementation
- `Cargo.toml`: Extension package configuration
- `extension.toml`: Zed-specific extension metadata

### Workspace Root (`/`)
**Purpose**: Top-level configuration and documentation
**Files**:
- `Cargo.toml`: Workspace configuration
- `README.md`: Project overview
- `.kiro/`: Kiro framework specifications and steering

## Naming Conventions

- **Crates**: Lowercase with hyphens (e.g., php-lsp, zed-php-lsp)
- **Rust Modules**: Lowercase with underscores (e.g., server, language_features)
- **Rust Functions**: Snake case (e.g., run_server, language_server_command)
- **Rust Structs/Traits**: PascalCase (e.g., PhpLspExtension, Extension)

## Import Organization

```rust
// Standard library first
use std::collections::HashMap;

// External crates
use async-lsp::Server;

// Workspace crates
use php-lsp::server;

// Local modules
use crate::some_module;
```

**Path Aliases**:
- Standard Rust module system (`crate::`, `super::`, `self::`)

## Code Organization Principles

- **Separation of Concerns**: Core LSP logic separate from editor integration
- **Single Responsibility**: Each crate has a focused purpose
- **LSP Protocol Compliance**: Adherence to LSP specification
- **Async Architecture**: Asynchronous design for responsiveness

---
_Document patterns, not file trees. New files following patterns shouldn't require updates_
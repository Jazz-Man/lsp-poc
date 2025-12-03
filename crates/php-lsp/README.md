# PHP Language Server Protocol (LSP) Implementation

A robust and efficient Language Server Protocol implementation for PHP, designed for integration with modern editors like Zed.

## Features

- **LSP Lifecycle Management**: Full support for initialize, initialized, shutdown, and exit requests/notifications
- **Document Synchronization**: Real-time tracking of document changes with incremental updates
- **PHP Parsing**: Powered by tree-sitter-php for accurate syntax analysis of PHP 7.4+ code with special attention to PHP 8+ features
- **Stdio Transport**: Compatible with editor integration via standard input/output streams
- **Performance Optimized**: Efficient handling of documents up to 10,000 lines with support for up to 1,000 concurrent documents
- **Error Resilient**: Graceful degradation with partial AST generation for documents with syntax errors

## Requirements

- Rust 1.75+ (edition 2021)
- Compatible with Linux/macOS platforms
- Works with editors supporting the Language Server Protocol

## Installation

The PHP LSP server is designed as a Rust crate that can be built as a standalone binary:

```bash
# Build the project
cargo build --release

# The binary can be executed with stdio mode for editor integration
./target/release/php-lsp --stdio
```

## Usage

### As a Standalone Server

```bash
# Run with stdio transport (for editor integration)
cargo run -- --stdio
```

### With Zed Editor

This LSP server is designed with Zed integration in mind. The `crates/zed-php-lsp/` directory contains the necessary extension files to enable PHP language features in Zed.

## Architecture

The implementation follows a modular architecture with distinct components:

- `lifecycle.rs`: Handles LSP initialization and shutdown protocols
- `document_sync.rs`: Manages document state and synchronization
- `parsing.rs`: PHP parsing and AST generation using tree-sitter
- `transport.rs`: Stdio communication layer
- `types.rs`: Core data structures and server state management

## Performance Metrics

- Initialization time: < 5 seconds
- Document sync operations: < 100ms for documents up to 10,000 lines
- Memory usage: < 200MB when tracking 50 documents simultaneously
- Document capacity: Supports up to 1,000 documents simultaneously

## Supported PHP Features

- Full support for PHP 7.4+ syntax
- Special attention to PHP 8+ features: attributes, named arguments, match expressions
- Accurate parsing with partial AST generation for malformed documents
- UTF-8 text handling with rope-based incremental updates

## Integration

The server implements the standard LSP protocol and communicates via stdio, making it compatible with any editor that supports LSP integration.

## License

This project is licensed under the MIT license.
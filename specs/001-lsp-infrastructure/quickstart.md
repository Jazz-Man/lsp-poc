# Quickstart: Core Infrastructure

## Setup and Installation

### Prerequisites
- Rust 1.75+ installed
- Cargo package manager
- Git for version control

### Initial Setup
1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd php-lsp
   ```

2. Checkout the feature branch:
   ```bash
   git checkout 001-lsp-infrastructure
   ```

3. Install project dependencies:
   ```bash
   cargo check
   ```

## Running the LSP Server

### Building the Project
```bash
cargo build
```

### Running in stdio Mode
```bash
cargo run -- --stdio
```

### Testing with an LSP Client
The LSP server can be tested with any LSP-compatible editor. For Zed specifically:
1. Build the WASM extension using `.scripts/build-extension.sh`
2. In Zed, open Command Palette → "zed: install dev extension"
3. Select the `crates/zed-php-lsp/` directory

## Development Workflow

### Following Constitution Principles
1. **DOCUMENTATION FIRST**: Before writing code, check `target/doc-md/` for API documentation:
   ```bash
   .scripts/regen-docs.sh  # Generate/update documentation
   cat target/doc-md/async_lsp/index.md  # Read specific crate docs
   ```

2. **ITERATIVE DEVELOPMENT**: Write maximum 20-30 lines at a time and check compilation:
   ```bash
   # After every 20-30 lines of code
   cargo check
   # Fix any errors immediately before continuing
   git add .
   git commit -m "Brief description of changes"
   ```

3. **USE EXISTING FUNCTIONALITY**: Always check async-lsp documentation before implementing LSP protocol features from scratch.

## Key Components

### Core Modules
- `server/lifecycle.rs`: Handles LSP initialize/shutdown/exit
- `server/document_sync.rs`: Manages document open/change/close
- `server/parsing.rs`: Parses PHP documents with tree-sitter
- `server/transport.rs`: Manages stdio communication

### Configuration
The server accepts the following command-line arguments:
- `--stdio`: Run in stdio mode (required for editor integration)

## Testing

### Running Tests
```bash
cargo test  # Run all tests
cargo test -- --nocapture  # Run tests with output visible
```

### Test Structure
- Unit tests in each module file
- Integration tests in `tests/` directory (to be created)

## Debugging

### Using Tracing
The server uses structured tracing as per constitution requirements:
```bash
RUST_LOG=debug cargo run -- --stdio  # Enable debug logging
```

### Common Debugging Scenarios
1. **Document not updating**: Check if `textDocument/didChange` notifications are being sent properly
2. **Slow parsing**: Large files may take more time to parse; performance should be within spec requirements
3. **Connection issues**: Ensure the server is running in stdio mode when used with editors
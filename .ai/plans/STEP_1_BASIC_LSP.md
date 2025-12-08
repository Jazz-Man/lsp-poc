# STEP 1: Basic LSP Foundation

## Critical Implementation Guidelines

### Documentation-First Development Approach
**CRITICAL:** Always verify API existence before implementation. Use ONLY documented APIs from crate documentation. Never assume API behavior.

Before writing any code:
1. Generate/verify documentation: `.scripts/regen-docs.sh`
2. Locate relevant crate documentation: `cat target/doc-md/index.md`
3. Read specific API docs: `cat target/doc-md/{crate}/index.md`
4. Confirm API signatures and usage patterns

### Incremental Development Protocol
For EACH implementation task, follow this exact sequence:

1. **RESEARCH:** Read documentation for APIs to be used
2. **IMPLEMENT:** Write MAX 20-30 lines of focused code
3. **VALIDATE:** Run `cargo check` - must pass with no errors
4. **FIX:** If errors exist, resolve IMMEDIATELY (not later)
5. **COMMIT:** Run `git add -A && git commit -m "task: description"`
6. **PROCEED:** Only then continue to next task

### Absolute Development Constraints

✗ STOP if compilation fails - fix immediately
✗ STOP if API doesn't exist in docs - research alternatives
✗ STOP after 30 lines - validate before continuing
✗ STOP if deviating from planned tasks - return to plan
✗ STOP if errors occur - resolve before proceeding

✓ VERIFY documentation before each code section
✓ RUN cargo check after every change
✓ COMMIT working code regularly
✓ FOLLOW plan task order precisely
✓ IMPLEMENT ONE feature at a time

### Development Workflow Example

# Task: Add Document struct
1. # Read ropey documentation
   cat target/doc-md/ropey/index.md
2. # Implement Document struct (~20 lines)
3. # Validate implementation
   cargo check
4. # Commit working code
   git add -A && git commit -m "feat: add Document struct with Rope"

### Error Resolution Protocol

When compilation errors occur:

1. **IDENTIFY:** First error in `cargo check` output
2. **RESEARCH:** Check relevant documentation in target/doc-md/
3. **FIX:** Address ONLY the first error
4. **VERIFY:** Run `cargo check` again
5. **REPEAT:** Until compilation succeeds
6. **COMMIT:** Run `git add -A && git commit -m "fix: {description}"`

### Documentation Research Steps

1. # Update documentation
   .scripts/regen-docs.sh
2. # List available crates
   cat target/doc-md/index.md
3. # Research specific crate
   cat target/doc-md/{crate}/index.md
4. # Search for specific functionality
   grep -r "Pattern" target/doc-md/{crate}/

## Overview
Create a general-purpose LSP framework similar to async-language-server but with enhanced features and better abstractions for building language-specific LSPs. This framework should serve as the foundation for the PHP LSP and future language implementations.

## Architecture Design

### Core Components
- **CoreLspService**: Main service trait similar to async-lsp's LanguageServer but with enhanced abstractions
- **DocumentManager**: Enhanced document management with Ropey and tree-sitter support
- **ConfigurationManager**: Support for language-specific configurations
- **TransportLayer**: Support for stdio, TCP, and WebSocket transports
- **EncodingHandler**: Automatic encoding negotiation (UTF-8, UTF-16, UTF-32)
- **RequestProcessor**: Handle LSP requests with custom middleware system

### Enhanced Features over async-language-server
- Pluggable middleware system for request processing
- Generic document type supporting multiple syntax trees
- Improved error handling and recovery mechanisms
- Better support for incremental changes
- Built-in testing framework for LSP functionality
- Enhanced diagnostics processing pipeline

## Technology Stack
- Rust 2021 edition
- async-lsp 0.2.2 (or latest stable)
- tower ecosystem for middleware
- ropey for efficient text editing
- tree-sitter for grammar parsing (optional feature)
- tokio for async runtime
- tracing for logging
- dashmap for concurrent state management

## Implementation Strategy

### Phase 1: Core Service Layer
1. Define CoreLspService trait with required methods
   - All standard LSP methods (hover, completion, diagnostics, etc.)
   - Configuration and initialization methods
   - Support for custom extension methods
2. Implement basic request routing and handling
3. Set up project structure and basic Cargo.toml
4. Add initial tests for core functionality

### Phase 2: Document Management
1. Implement DocumentManager for handling text documents
   - Support for incremental updates
   - Version tracking
   - Text encoding handling
2. Integrate ropey for efficient rope-based text storage
3. Add support for document lifecycle events (open, change, save, close)
4. Create Document trait that can be extended for language-specific features
5. Implement tests for document management

### Phase 3: Transport Layer & Encoding Handling
1. Implement Transport trait supporting stdio and TCP
2. Create EncodingHandler for automatic encoding negotiation
   - Support for UTF-8, UTF-16, and UTF-32
   - Position conversion utilities
   - Range conversion utilities
3. Integrate with async-lsp's transport mechanisms
4. Add tests for encoding conversions

### Phase 4: Middleware System
1. Implement pluggable middleware system using tower concept
2. Create built-in middleware:
   - Logging middleware
   - Error handling middleware
   - Request throttling middleware
   - Metrics collection middleware
3. Allow custom middleware registration
4. Add tests for middleware functionality

### Phase 5: Tree-sitter Integration (Optional Feature)
1. Add optional tree-sitter feature dependency
2. Implement TreeDocument trait extending Document
3. Provide utilities for tree-sitter node queries
4. Add incremental tree-sitter tree updates
5. Include tests for tree-sitter functionality

### Phase 6: Configuration Management
1. Implement ConfigurationManager for workspace settings
2. Support for language-specific configurations
3. Allow dynamic configuration updates
4. Include configuration validation
5. Add tests for configuration handling

## Documentation and Testing

### Using Documentation Generation
Before implementing each feature:
- Run `.scripts/regen-docs.sh` to update documentation
- Read relevant crate docs: `target/doc-md/async_lsp/index.md`, `target/doc-md/tree_sitter/index.md`, `target/doc-md/lsp_types/index.md`, `target/doc-md/ropey/index.md`
- Ensure implementation follows current API patterns

### Testing Strategy
- Unit tests for each component
- Integration tests for request handling
- End-to-end tests with real LSP clients
- Performance benchmarks for document updates
- Fuzzing for security-critical parsing functions

## Security Considerations
- Input validation for all LSP requests
- Resource limits to prevent DoS attacks
- Proper isolation of client connections
- Secure handling of sensitive file paths
- Sanitization of error messages that might contain sensitive data

## Documentation Standards
- Comprehensive inline documentation with examples
- README with usage examples
- API documentation for all public interfaces
- Configuration guide
- Performance tuning guide

## Deliverables
- BasicLsp crate with core LSP functionality
- Document management system with ropey integration
- Encoding handling utilities
- Middleware system
- Tree-sitter integration (optional feature)
- Comprehensive test suite
- Example implementations
- Documentation

## Next Steps Preparation
- Prepare interfaces that will be easily extendable for PHP-specific functionality
- Design hooks for language-specific analysis
- Ensure clean separation between general LSP functionality and language-specific features
- Plan for easy integration with Zed editor
# Research Summary: Core Infrastructure

## Decision: LSP Framework Choice
**Rationale**: Using async-lsp framework instead of building from scratch provides a solid foundation for LSP implementation with async/await support. This aligns with the "USE EXISTING FUNCTIONALITY" principle from the constitution.
**Alternatives considered**: 
- Building raw LSP implementation with jsonrpc
- Using other LSP frameworks like lsp-server
- Using Tower-based LSP frameworks

## Decision: Transport Layer
**Rationale**: Using stdio transport as specified in requirements allows for integration with editors like Zed. The stdio transport is standard for LSP servers and enables communication with editor clients.
**Alternatives considered**: 
- TCP socket transport
- Named pipes
- HTTP-based transport

## Decision: Document Storage
**Rationale**: Using in-memory storage with DashMap for concurrent access provides fast document retrieval and updates. For the requirements of up to 1,000 documents, this approach should be sufficient without needing persistent storage.
**Alternatives considered**:
- Persistent file storage
- Database storage (SQLite, etc.)
- Hybrid approach (memory + disk cache)

## Decision: Text Handling
**Rationale**: Using ropey crate for efficient text rope handling as specified in requirements. Rope data structures are efficient for handling large text changes and incremental updates.
**Alternatives considered**:
- Standard String handling
- Other rope implementations
- Byte-based handling

## Decision: PHP Parsing
**Rationale**: Using tree-sitter-php as specified in requirements provides robust PHP parsing capabilities with support for modern PHP syntax. Tree-sitter is well-established for language parsing in LSP implementations.
**Alternatives considered**:
- Custom PHP parser
- PHP-Parser Rust bindings
- Other tree-sitter grammars

## Decision: Async Runtime
**Rationale**: Using Tokio as the async runtime aligns with the async-lsp framework and provides robust async capabilities needed for handling multiple document updates concurrently.
**Alternatives considered**:
- async-std
- Built-in async support without runtime
- Other async executors

## Decision: Error Handling
**Rationale**: Using thiserror/anyhow as specified in the constitution for proper error handling instead of unwrap calls in production code.
**Alternatives considered**:
- Standard Result types
- Custom error types without thiserror
- Simple panic/unwrap approach (rejected per constitution)

## Decision: Logging
**Rationale**: Using tracing crate as specified in constitution instead of println statements. This provides structured logging needed for observability.
**Alternatives considered**:
- Standard println!/eprintln!
- Log crate
- Custom logging implementation
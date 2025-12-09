# Research & Design Decisions Template

---
**Purpose**: Capture discovery findings, architectural investigations, and rationale that inform the technical design.

**Usage**:
- Log research activities and outcomes during the discovery phase.
- Document design decision trade-offs that are too detailed for `design.md`.
- Provide references and evidence for future audits or reuse.
---

## Summary
- **Feature**: `document-struct`
- **Discovery Scope**: New Feature
- **Key Findings**:
  - Ropey provides efficient UTF-8 text rope implementation suitable for large documents
  - Tree-sitter has flexible text provider API that can work with Ropey via callback functions
  - Integration requires careful synchronization between Ropey text operations and Tree-sitter syntax trees
  - async-lsp provides the LSP protocol framework with document synchronization capabilities

## Research Log
Document notable investigation steps and their outcomes. Group entries by topic for readability.

### Ropey Text Handling Library
- **Context**: Need to select an efficient text handling solution for large documents in the LSP server
- **Sources Consulted**: https://cessen.github.io/ropey/, Rust documentation
- **Findings**: 
  - Ropey is a UTF-8 text rope library optimized for handling large texts (gigabyte-sized)
  - Implements B-tree rope with leaf nodes of 16 KiB for efficient operations
  - Provides O(log n) for substring and indexing operations
  - Specifically designed for text editors and similar applications
- **Implications**: Provides the performance characteristics required for an LSP server handling large PHP files

### Tree-sitter Integration with Text Storage
- **Context**: Need to understand how to integrate tree-sitter parsing with ropey-based text storage
- **Sources Consulted**: https://docs.rs/tree-sitter, tree-sitter documentation
- **Findings**:
  - Tree-sitter supports custom text providers via callback functions
  - The `parse_with` method accepts a callback that can interface with text storage
  - Allows efficient updates to syntax trees when underlying text changes
  - Does not require full text to be loaded in memory at once
- **Implications**: Architecture must support synchronization between Ropey's text representation and Tree-sitter's syntax tree

### LSP Document Synchronization Patterns
- **Context**: Need to understand how document handling works in LSP context
- **Sources Consulted**: Microsoft LSP specification, async-lsp documentation
- **Findings**:
  - LSP requires text document synchronization (open, change, close notifications)
  - Supports incremental synchronization with textDocument/didChange
  - Client and server must coordinate document state
  - LSP editors typically maintain document state and provide efficient access to text
- **Implications**: Document struct must support LSP synchronization protocol and maintain document versions

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Simple wrapper | Basic wrapper around Ropey with optional tree-sitter | Simple, straightforward | May not handle synchronization properly | Doesn't align with LSP requirements |
| State manager | Document struct that manages Ropey + Tree-sitter state | Handles sync, versioning, parsing | More complex, potential race conditions | Better for LSP server needs |
| Adapter pattern | Separate components for text, parsing, and LSP integration | Clean separation, testable | More components to coordinate | Most maintainable for long-term |

## Design Decisions

### Decision: State Manager with Synchronization
- **Context**: Need a document structure that efficiently manages text and optional parsing state while supporting LSP synchronization
- **Alternatives Considered**:
  1. Simple wrapper — minimal functionality, doesn't handle LSP sync
  2. State manager with synchronization — full featured for LSP server
- **Selected Approach**: State manager that encapsulates Ropey for text handling, optional Tree-sitter integration, and LSP state management
- **Rationale**: Aligns with LSP requirements and provides the necessary functionality for a PHP LSP server
- **Trade-offs**: More complex than a simple wrapper but provides the required functionality for reliable LSP server operation
- **Follow-up**: Verify thread safety when implementing, consider async operations for parsing

### Decision: Optional Tree-sitter Integration
- **Context**: Requirements mention optional tree-sitter integration, need to define how this works
- **Alternatives Considered**:
  1. Mandatory tree-sitter dependency — simpler but forces parsing
  2. Optional tree-sitter integration — more complex but flexible
- **Selected Approach**: Optional Tree-sitter integration that can be enabled/disabled per document
- **Rationale**: Allows efficient text operations without parsing overhead when not needed, but provides parsing when required
- **Trade-offs**: More complex implementation but provides better performance characteristics based on usage
- **Follow-up**: Define clear interfaces between text and parsing components

## Risks & Mitigations
- Text-parsing synchronization — Risk that text changes aren't properly reflected in syntax tree; Mitigation: Implement proper change propagation with InputEdit
- Performance degradation — Risk of slow operations on large files; Mitigation: Use Ropey's performance characteristics and efficient tree-sitter updates
- Memory usage — Risk of excessive memory consumption with large documents; Mitigation: Leverage Ropey's memory efficiency and tree-sitter's incremental parsing

## References
Provide canonical links and citations (official docs, standards, ADRs, internal guidelines).
- [Ropey Documentation](https://cessen.github.io/ropey/) — efficient text rope data structure for Rust
- [Tree-sitter Rust Bindings](https://docs.rs/tree-sitter) — parsing library with custom text provider support
- [Language Server Protocol Specification](https://microsoft.github.io/language-server-protocol/specifications/specification-3-14/) — document synchronization requirements
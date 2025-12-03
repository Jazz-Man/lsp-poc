# ADR-0001: LSP Framework and Runtime Selection

> **Scope**: Document decision clusters, not individual technology choices. Group related decisions that work together (e.g., "Frontend Stack" not separate ADRs for framework, styling, deployment).

- **Status:** Accepted
- **Date:** 2025-12-03
- **Feature:** 001-lsp-infrastructure
- **Context:** Need to select a robust, async-capable framework for implementing the Language Server Protocol server with appropriate runtime and error handling. The solution must support concurrent document processing and integrate well with the tree-sitter parsing infrastructure.

<!-- Significance checklist (ALL must be true to justify this ADR)
     1) Impact: Long-term consequence for architecture/platform/security?
     2) Alternatives: Multiple viable options considered with tradeoffs?
     3) Scope: Cross-cutting concern (not an isolated detail)?
     If any are false, prefer capturing as a PHR note instead of an ADR. -->

## Decision

- Framework: async-lsp 0.2.2 (LSP server framework)
- Runtime: Tokio runtime for async support
- Error Handling: thiserror/anyhow for proper error handling
- Logging: tracing crate for structured logging

## Consequences

### Positive

- async-lsp provides a solid foundation for LSP implementation with async/await support
- Tokio runtime provides robust async capabilities for handling multiple document updates concurrently
- Proper error handling with thiserror/anyhow prevents crashes from unwrap calls in production
- Structured tracing enables observability and debugging

### Negative

- Additional dependencies increase build times
- Learning curve for team unfamiliar with async-lsp framework
- Tokio runtime adds complexity compared to synchronous alternatives

## Alternatives Considered

Alternative Stack A: Raw LSP implementation with jsonrpc + standard async + basic Result types
Alternative Stack B: lsp-server framework + async-std + custom error types
Why rejected: More complex to implement, missing conveniences of async-lsp, less mature ecosystem for LSP development

## References

- Feature Spec: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/spec.md
- Implementation Plan: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/plan.md
- Related ADRs:
- Evaluator Evidence: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/research.md

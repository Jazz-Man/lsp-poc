# ADR-0003: Transport Layer Approach

> **Scope**: Document decision clusters, not individual technology choices. Group related decisions that work together (e.g., "Frontend Stack" not separate ADRs for framework, styling, deployment).

- **Status:** Accepted
- **Date:** 2025-12-03
- **Feature:** 001-lsp-infrastructure
- **Context:** Need to establish a communication protocol between the LSP server and editor clients (particularly Zed) that follows LSP standards and enables efficient data exchange for real-time language features.

<!-- Significance checklist (ALL must be true to justify this ADR)
     1) Impact: Long-term consequence for architecture/platform/security?
     2) Alternatives: Multiple viable options considered with tradeoffs?
     3) Scope: Cross-cutting concern (not an isolated detail)?
     If any are false, prefer capturing as a PHR note instead of an ADR. -->

## Decision

- Transport Protocol: Standard stdio (stdin/stdout) for LSP communication
- Message Format: JSON-RPC 2.0 for request/response handling
- Integration: Direct integration with editor clients via stdio pipe

## Consequences

### Positive

- Standard approach for LSP servers, compatible with all major editors
- Efficient communication without network overhead
- Simple implementation without complex networking code
- Direct integration path for Zed editor

### Negative

- Limited to local editor integration (no remote LSP server capability)
- Requires process management for lifecycle
- Potential for blocking I/O if not properly handled asynchronously

## Alternatives Considered

Alternative Stack A: TCP socket transport + JSON-RPC + network-based integration
Alternative Stack B: Named pipes + custom protocol + editor-specific integration
Alternative Stack C: HTTP-based transport + REST API + custom client
Why rejected: TCP sockets add network complexity and potential latency, named pipes are platform-specific, and HTTP-based approaches don't align with standard LSP protocol expectations

## References

- Feature Spec: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/spec.md
- Implementation Plan: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/plan.md
- Related ADRs:
- Evaluator Evidence: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/research.md

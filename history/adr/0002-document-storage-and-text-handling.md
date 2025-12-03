# ADR-0002: Document Storage and Text Handling

> **Scope**: Document decision clusters, not individual technology choices. Group related decisions that work together (e.g., "Frontend Stack" not separate ADRs for framework, styling, deployment).

- **Status:** Accepted
- **Date:** 2025-12-03
- **Feature:** 001-lsp-infrastructure
- **Context:** Need to efficiently store and manage PHP documents in memory with concurrent access patterns for an LSP server that supports up to 1,000 documents simultaneously. The solution must handle incremental text updates efficiently for large files.

<!-- Significance checklist (ALL must be true to justify this ADR)
     1) Impact: Long-term consequence for architecture/platform/security?
     2) Alternatives: Multiple viable options considered with tradeoffs?
     3) Scope: Cross-cutting concern (not an isolated detail)?
     If any are false, prefer capturing as a PHR note instead of an ADR. -->

## Decision

- Document Storage: In-memory storage with DashMap for concurrent access
- Text Handling: ropey crate for efficient rope-based text handling
- Data Structure: Rope data structures for handling large text changes and incremental updates

## Consequences

### Positive

- Fast document retrieval and updates with concurrent access
- Efficient handling of large text changes and incremental updates
- Rope data structures are optimized for the edit patterns typical in code editors
- Memory efficient for the requirements of up to 1,000 documents

### Negative

- No persistent storage, documents are lost on server restart
- Memory usage increases with document count and size
- Learning curve for rope-based text handling

## Alternatives Considered

Alternative Stack A: Persistent file storage + standard String handling
Alternative Stack B: Database storage (SQLite) + byte-based handling
Alternative Stack C: Hybrid approach (memory + disk cache) + custom text implementation
Why rejected: File storage would be too slow for real-time updates, database storage adds unnecessary complexity for this use case, and custom implementations would be error-prone and not optimized for editor use patterns

## References

- Feature Spec: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/spec.md
- Implementation Plan: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/plan.md
- Related ADRs:
- Evaluator Evidence: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/research.md

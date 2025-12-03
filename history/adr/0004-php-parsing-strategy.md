# ADR-0004: PHP Parsing Strategy

> **Scope**: Document decision clusters, not individual technology choices. Group related decisions that work together (e.g., "Frontend Stack" not separate ADRs for framework, styling, deployment).

- **Status:** Accepted
- **Date:** 2025-12-03
- **Feature:** 001-lsp-infrastructure
- **Context:** Need to parse PHP documents efficiently to generate Abstract Syntax Trees (ASTs) that support language features like go-to-definition, hover, and completion. The solution must handle PHP 7.4+ syntax including modern PHP 8+ features.

<!-- Significance checklist (ALL must be true to justify this ADR)
     1) Impact: Long-term consequence for architecture/platform/security?
     2) Alternatives: Multiple viable options considered with tradeoffs?
     3) Scope: Cross-cutting concern (not an isolated detail)?
     If any are false, prefer capturing as a PHR note instead of an ADR. -->

## Decision

- Parsing Engine: tree-sitter-php 0.24.2 for PHP syntax parsing
- AST Generation: Tree-sitter generated Abstract Syntax Trees
- Error Handling: Support for partial AST generation when syntax errors occur
- Syntax Support: Full PHP 7.4+ syntax with focus on PHP 8.x features

## Consequences

### Positive

- Robust parsing capabilities with support for modern PHP syntax
- Tree-sitter is well-established for language parsing in LSP implementations
- Efficient parsing with good performance characteristics
- Partial AST generation allows for partial language features even with syntax errors

### Negative

- Additional dependency with potential version compatibility issues
- Learning curve for tree-sitter query language if advanced features needed
- May not capture PHP-specific semantic information that a dedicated PHP parser might

## Alternatives Considered

Alternative Stack A: Custom PHP parser + custom AST + basic error handling
Alternative Stack B: PHP-Parser Rust bindings + custom AST + standard error handling
Alternative Stack C: Other tree-sitter grammars + standard AST + error recovery
Why rejected: Custom parsers would be extremely time-consuming and error-prone, PHP-Parser bindings may not be as well-maintained, and other grammars would likely not be as mature as tree-sitter-php

## References

- Feature Spec: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/spec.md
- Implementation Plan: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/plan.md
- Related ADRs:
- Evaluator Evidence: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/research.md

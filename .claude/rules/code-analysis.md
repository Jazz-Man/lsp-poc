# Code analysis

**LSP is mandatory for ALL code reading, searching, and navigation — always. No exceptions.** Load the `lsp-code-analysis` skill first; it defines the operations and when each applies. This rule adds only what the skill lacks: scope and discipline.

## Scope

Everywhere Rust code lives: workspace crates and dependency sources — cargo git checkouts, registry caches, any tree this project compiles. Neither ownership nor gitignore status narrows it.

## Fallback

Fall back to grep/Read only in these three cases:

1. Non-code files (Markdown, TOML, JSON config).
2. Exhaustive literal search — every textual occurrence, including comments and strings.
3. LSP unavailable or erroring — say so explicitly, then proceed.

## Subagents

Instruct every dispatched code-analysis task to use the `lsp-code-analysis` skill — it is available to subagents.

---
*Read code the way the compiler sees it — through the language server, not text matching.*

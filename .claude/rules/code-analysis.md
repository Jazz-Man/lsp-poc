# Code analysis

**LSP is mandatory for ALL Rust code reading, searching, and navigation — always.** Load the `lsp-code-analysis` skill first; it defines the operations and when to use each. This rule adds only what the skill does not: scope and discipline.

## Scope: everywhere this project's Rust lives

LSP applies not only to the workspace crates — `crates/lsp-poc/src/` and `crates/zed-php-lsp/src/` — but equally to the dependency sources outside the workspace tree:

- **Git dependencies** under `~/.cargo/git/checkouts/async-language-server-*/`. The pinned rev `v0.0.1` (commit `d7795c46…`, per `Cargo.lock`) currently lives at `~/.cargo/git/checkouts/async-language-server-38a713e99d1a3f29/d7795c4/`.
- **Registry dependencies** under `~/.cargo/registry/src/index.crates.io-*/` — `async-lsp`, `lsp-types`, `tree-sitter`, `zed_extension_api`, and everything else `Cargo.lock` pins.

The checkout directory's hash suffix is not the rev, and stale ones accumulate. To find the sources cargo actually builds, take the `#<sha>` from the `Cargo.lock` entry and match its short prefix against the revision directories inside the checkout. Do this before quoting framework code from memory — after a rev bump, a directory you memorized keeps existing while no longer being built.

Being outside the workspace tree is irrelevant: a cargo-owned checkout or registry source is still source code you can and should navigate through LSP. Never skip a directory because it "isn't ours".

## Why this project cares more than most

The pinned `async-language-server` rev is not plumbing here — it is the subject of study. Its `src/` holds `server_trait.rs` (the `Server` trait), `server_state.rs`, `document.rs`, `requests.rs`, and `tree_sitter_utils.rs`. When wiring a capability, read the real definitions there:

- Jump to definition on a `Server` trait method used in `server.rs` to read the framework's signature, doc comments, and default implementation before overriding it.
- Find references on `ServerState::document` or `node_at_position_named` to see how the framework itself calls them.
- Treat the request and response types in `requests.rs` as the contract the editor sees; verify a capability's wire types there instead of guessing.
- Hover the range helpers in `tree_sitter_utils.rs` to confirm their exact signatures before calling them from a feature module.

The tech rule treats a rev bump as a deliberate experiment: after any bump, re-check the `Server` trait surface through LSP, and never carry signatures over from the previous rev.

## Fallback: when grep and Read are correct

Fall back to grep or Read in only these three cases:

1. **Non-code files** — Markdown docs, `Cargo.toml`, `extension.toml`, `.zed/settings.json`.
2. **Exhaustive literal search** — when you need every textual occurrence, including comments and strings.
3. **LSP unavailable or returns errors** — say so explicitly, then proceed.

Anything semantic — usage, types, implementations, callers — goes through LSP.

## Keep the analyzer warm

LSP-first is cheap here because `target/` is deliberately shared with rust-analyzer and kept warm — the exact reason the tech rule forbids `RUSTFLAGS` divergence. Do not introduce flag changes or cleanups that force a full dependency rebuild; a warm analyzer is what makes navigation into the cargo checkouts instant.

## Backend

rust-analyzer is this project's analyzer — `.zed/settings.json` selects it for Rust in Zed. If the backend ever changes, this rule still stands: it is about LSP-first behavior, not one specific server.

## Subagents

The `lsp-code-analysis` skill is user-level and available to subagents. When dispatching any code-analysis task, instruct the subagent to use it — including tasks that touch only the cargo checkouts or registry sources.

---
*Read code the way the compiler sees it — through the language server, not text matching.*

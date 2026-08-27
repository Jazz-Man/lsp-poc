# Technology Stack

## Architecture

One Rust workspace producing two artifacts: a native LSP server binary (`lsp-poc`) and a wasm Zed extension that launches it. The server owns all logic; the extension is a thin launcher. The structure rule covers how the pieces connect.

## Core Technologies

- **Language**: Rust, edition 2024 workspace-wide, stable channel pinned in `rust-toolchain.toml` (components: `rustfmt`, `clippy`).
- **Framework**: `async-language-server` — a git dependency pinned to rev `v0.0.1`, with features `tracing` and `tree-sitter`. It owns the JSON-RPC message loop, the document store, and tree-sitter parsing. Use its re-exports (`lsp_types`, `tree_sitter`, `tree_sitter_utils`) instead of adding those crates directly.
- **Runtime**: tokio, `current_thread` flavor (`crates/lsp-poc/src/main.rs`).
- **CLI**: clap derive, currently the single `serve` subcommand (`--socket` is parsed but unused).
- **Grammar**: `tree-sitter-json`.
- **Editor side**: `zed_extension_api`, as the wasm `cdylib` in the extension crate.

## Development Standards

### Code Quality

Run `cargo lint` (alias for `clippy --workspace --all-targets --message-format=short`) before calling work done.

Workspace clippy gates live in the root `Cargo.toml`: `all` and `pedantic` at warn, plus `unwrap_used`, `expect_used`, and `dbg_macro` at warn. Never write `.unwrap()`, `.expect()`, or `dbg!()`. Propagate errors through the crate's typed error (`PocError`, see error-handling.md) inside server code, through `anyhow::Result` at the CLI edge; use let-else for absent values, the way `hover()` in `crates/lsp-poc/src/server.rs` does:

```rust
let Some(doc) = state.document(&url) else { return Ok(None) };
```

### Formatting

Check with `cargo fmt --all --check`; fix with `cargo fmt --all`. Do not use the `fmtcheck`/`fmtall` aliases from `.cargo/config.toml` — they pass `-p zed-php-lsp`, but the package is named `zed-lsp-poc`, so both aliases fail.

### Testing

No tests exist yet. When they appear, run a single one with `cargo test -p lsp-poc <test_name>`.

## Logging Constraints

Log with `tracing` macros only, and never print to stdout — stdout is the LSP transport, and `crates/lsp-poc/src/tracing.rs` wires all output to stderr. Control verbosity with `RUST_LOG`; defaults are DEBUG in debug builds, INFO in release.

## Common Commands

```bash
cargo build                          # rebuild after server changes — the extension launches target/debug/lsp-poc
cargo run -p lsp-poc -- serve        # run the server standalone over stdio
cargo lint                           # clippy across the workspace, short output
cargo fmt --all --check              # format gate
cargo test -p lsp-poc <test_name>    # once tests exist
```

## Key Technical Decisions

- **No hand-rolled JSON-RPC.** `serve(Transport::Stdio, server)` in `crates/lsp-poc/src/cli/serve.rs` runs the entire message loop. Implement capabilities through the `Server` trait; never touch the transport.
- **Treat a rev bump as a deliberate experiment.** After any bump, re-check the `Server` trait surface, because the pinned crate can change shape between tags.
- **Do not set `RUSTFLAGS` in `.cargo/config.toml`.** The shared `target/` is deliberately kept warm for rust-analyzer, and flag divergence forces a full rebuild of the large dependency tree. If builds OOM, uncomment `jobs = 8` there instead.
- **Ignore the tuned release profile (fat LTO, `panic = "abort"`, strip) in day-to-day work** — Zed launches the debug binary.

---
_Document the stack's fixed constraints and gates, not every dependency; new crates that fit these patterns should not require updating this rule._

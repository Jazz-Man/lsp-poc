# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

- Build: `cargo build` — the Zed extension launches `target/debug/lsp-poc` by default (`target/release/lsp-poc` via `lsp.zed-lsp-poc.settings.profile` in `.zed/settings.json`), so rebuild after server changes for the extension to pick them up
- Run the server: `cargo run -p lsp-poc -- serve` (stdio transport; the `--socket` flag is parsed but currently unused)
- Lint: `cargo lint` (alias for `clippy --workspace --all-targets --message-format=short`)
- Format: `cargo fmt --all --check` / `cargo fmt --all` — note: the `fmtcheck`/`fmtall` aliases in `.cargo/config.toml` reference package `zed-md-lsp` (the directory name) instead of `zed-lsp-poc` (the package name) and fail
- Tests: none exist yet; once added, run a single test with `cargo test -p lsp-poc <test_name>`
- Zed extension: installed as a dev extension in Zed and rebuilt through the Zed UI by the owner (no CLI); `extension.wasm` is a gitignored local artifact

## Architecture

Rust workspace (edition 2024, stable toolchain) with two crates forming one pipeline: a language-server binary and a Zed extension that launches it inside the editor.

### `crates/lsp-poc` — the LSP server binary

- Built on `async-language-server` (git dep, pinned rev `v0.0.1`), which supplies the `Server` trait, the stdio transport, and a document store with tree-sitter parsing. `serve(Transport::Stdio, server)` in `src/cli/serve.rs` runs the entire message loop — there is no hand-rolled JSON-RPC code.
- `src/server.rs` defines `PocLanguageServer`, the central place features live:
  - `server_capabilities()` declares what the server supports (currently only hover)
  - `server_document_matchers()` claims JSON documents (url glob `**/*.json`, lang string `JSON`, tree-sitter-json grammar)
  - trait methods such as `hover()` implement each feature against `ServerState` (e.g. `state.document(&url)`, `doc.node_at_position_named(pos)`)
  - to add a capability: advertise it in `server_capabilities()` and implement the corresponding `Server` trait method
- Feature modules: `completions/` holds a trigger-characters helper not yet wired into capabilities (its reference in `server.rs` is commented out); `hovers/` and `schema/` are empty placeholders
- `src/tracing.rs`: all logs go to **stderr** because stdout is the LSP transport — never print to stdout. Level controlled by `RUST_LOG`; defaults to DEBUG in debug builds, INFO in release.

### `crates/zed-md-lsp` — Zed extension

- Package name is `zed-lsp-poc`; the directory name is a leftover from the project's original PHP focus. Use the package name in `cargo -p` commands.
- wasm `cdylib` on `zed_extension_api`. `language_server_command()` launches the lsp-poc binary with the `serve` subcommand; the path is `<worktree-root>/target/<debug|release>/lsp-poc`, chosen by `lsp.zed-lsp-poc.settings.profile` in `.zed/settings.json` (default `debug`), so a build of the selected profile must exist before the extension works.
- `extension.toml` registers it for JSON/JSONC; `.zed/settings.json` selects it for JSON and disables `json-language-server`.

## Conventions

- Workspace clippy gates: `all` and `pedantic` at warn, plus `unwrap_used`, `expect_used`, and `dbg_macro` — avoid `.unwrap()`, `.expect()`, and `dbg!()` in new code
- Don't set `RUSTFLAGS` in `.cargo/config.toml`: `target/` is deliberately shared with rust-analyzer, and flag divergence forces a full dependency-tree rebuild

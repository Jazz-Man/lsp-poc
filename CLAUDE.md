# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

`make help` is the authoritative command contract; `make battery` chains the gates `fmt`, `clippy`, `doc`, `test`, `dylint`. Cargo goes through `rtk` locally, plain `cargo` under CI.

- Build: `cargo build` — the Zed extension launches `target/debug/lsp-poc` (path hardcoded in the extension), so rebuild after server changes for the extension to pick them up
- Run the server: `cargo run -p lsp-poc -- serve` (stdio transport; the `--socket` and `--stdio` flags are parsed but unused)
- Tests: `make test` (`cargo nextest run --workspace`; currently the arch-lint test in `crates/lsp-poc/tests/architecture.rs`), one test via `cargo nextest run <filter>`
- Lint: `make clippy` (or `cargo lint`, the short-output alias in `.cargo/config.toml`)
- Zed extension: installed as a dev extension in Zed and rebuilt through the Zed UI by the owner (no CLI); `extension.wasm` is a gitignored local artifact

## Architecture

Rust workspace (edition 2024, stable toolchain) with two crates forming one pipeline: a language-server binary and a Zed extension that launches it inside the editor.

### `crates/lsp-poc` — the LSP server binary

- Built on `async-language-server` (direct dep of this crate, git rev `v0.10.0`, `tree-sitter` feature), which supplies the `Server` trait, `serve(server)` over stdio, and a document store with tree-sitter parsing — there is no hand-rolled JSON-RPC code.
- `src/server.rs` defines `PocLanguageServer`, the central place where features live:
  - `server_capabilities()` declares what the server supports (currently only hover)
  - `server_document_matchers()` claims Markdown documents (url glob `**/*.md`, lang string `Markdown`, `tree-sitter-md` grammar)
- Feature modules: `hovers/` is an empty placeholder — hover logic lives in `server.rs` itself. `src/utils.rs` is an undeclared PHP-era leftover (a module file on disk is dead until declared in `main.rs`).
- `src/tracing.rs`: all logs go to **stderr** because stdout is the LSP transport — never print to stdout. Level controlled by `RUST_LOG`; defaults to DEBUG in debug builds, INFO in release.

### `crates/zed-md-lsp` — Zed extension

- wasm `cdylib` on `zed_extension_api`. `language_server_command()` in `src/lib.rs` launches `<worktree-root>/target/debug/lsp-poc serve --stdio` — the profile is hardcoded (no settings switch), so only a debug build needs to exist.
- `extension.toml` registers the server for Markdown; `.zed/settings.json` selects `zed-lsp-poc` for Markdown files.

## Conventions

- Workspace clippy gates: `all`, `cargo`, and `pedantic` at deny; `unwrap_used`/`expect_used` at deny, test-aware via `clippy.toml`; a restriction set (`dbg_macro` among them) at warn — avoid `.unwrap()`, `.expect()`, and `dbg!()` in new code
- Don't set `RUSTFLAGS` in `.cargo/config.toml`: `target/` is deliberately shared with rust-analyzer, and flag divergence forces a full dependency-tree rebuild

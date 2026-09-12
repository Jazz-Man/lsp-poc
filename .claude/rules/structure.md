# Project Structure

## Organization Philosophy

One pipeline, two crates. Everything that is an LSP capability lives in or hangs off `crates/lsp-poc`; the second crate exists only to make Zed launch the first. Keep that separation under any restructuring: server logic never goes into the extension, and editor integration never goes into the server.

## Directory Patterns

### The LSP server

`/crates/lsp-poc/` (package `lsp-poc`, binary `lsp-poc`) is the entire language server. `src/main.rs` declares the module tree and starts the CLI. `src/server.rs` holds `PocLanguageServer` — the single `Server` impl where capabilities are declared and features implemented. `src/cli/` holds the clap subcommands (`serve.rs` starts the server over stdio). Per-capability helper modules keep trait methods thin; push capability logic into the matching module (the pattern's only current instance is the empty `src/hovers/` placeholder — hover logic lives in `server.rs` itself).

Gotcha: a module file on disk is dead until declared in `main.rs`. `src/utils.rs` is currently undeclared — a PHP-era leftover. When creating a capability module, add its `mod` line to `main.rs` in the same change.

### The Zed extension

`/crates/zed-md-lsp/` (package `zed-lsp-poc`, wasm `cdylib`) is a launcher only. `language_server_command()` in `src/lib.rs` spawns `<worktree root>/target/debug/lsp-poc serve --stdio` — the profile is hardcoded — there is no settings-driven switch. Consequence: a debug build must exist before the extension works, and server changes need only `cargo build` — rebuild `extension.wasm` only when the extension crate itself changes. `extension.toml` registers the server for Markdown; `.zed/settings.json` selects `zed-lsp-poc` for Markdown files.

## The Capability Wiring Pattern

Adding an LSP capability means two paired edits in `crates/lsp-poc/src/server.rs`, then a rebuild:

1. **Advertise** it in `server_capabilities()` — e.g. `hover_provider: Some(HoverProviderCapability::Simple(true))`.
2. **Implement** the matching `Server` trait method — the trait itself enumerates the handler surface; it changes shape between tags, so re-check it on rev bumps (per the tech rule).
3. **Operate on `ServerState`** inside the method: `state.document(&url)` for the document, `doc.node_at_position_named(pos)` for the node, and the range helpers from `async_language_server::tree_sitter_utils`.

Always do both halves of the pair: a capability advertised but not implemented lies to the editor, and a method implemented but not advertised is dead code. Claim documents once, in `server_document_matchers()`, via `DocumentMatcher::new("Markdown").with_url_globs(["**/*.md"]).with_lang_strings(["Markdown"]).with_lang_grammar(tree_sitter_md::LANGUAGE.into())`.

## Naming

Crates use kebab-case for both directories and package names. Modules are lowercase, one word where possible, in `mod.rs`-style directories. The server type is `PocLanguageServer`.

The extension's directory (`crates/zed-md-lsp`) and package name (`zed-lsp-poc`) disagree — the directory is a leftover from the project's original PHP focus, as is `src/utils.rs`. Always pass the package name to cargo (`cargo -p zed-lsp-poc`), and do not propagate the PHP name into new files, commands, or docs.

---
_Describe the wiring pattern once so new capabilities and modules slot in without structural debate; the two-crate split and the capability/method pairing are the invariant parts._

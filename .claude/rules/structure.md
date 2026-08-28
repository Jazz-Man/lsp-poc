# Project Structure

## Organization Philosophy

One pipeline, two crates. Everything that is an LSP capability lives in or hangs off `crates/lsp-poc`; the second crate exists only to make Zed launch the first. Keep that separation under any restructuring: server logic never goes into the extension, and editor integration never goes into the server.

## Directory Patterns

### The LSP server

`/crates/lsp-poc/` (package `lsp-poc`, binary `lsp-poc`) is the entire language server. `src/main.rs` declares the module tree and starts the CLI. `src/server.rs` holds `PocLanguageServer` — the single `Server` impl where capabilities are declared and features implemented. `src/cli/` holds the clap subcommands (`serve.rs` starts the server over stdio). Per-capability helper modules — `completions/`, `hovers/`, `schema/` — keep trait methods thin; push capability logic into the matching module.

Gotcha: a module file on disk is dead until declared in `main.rs`. `schema/` and `src/utils.rs` are currently undeclared — `utils.rs` is a PHP-era leftover. When creating a capability module, add its `mod` line to `main.rs` in the same change.

### The Zed extension

`/crates/zed-md-lsp/` (package `zed-lsp-poc`, wasm `cdylib`) is a launcher only. `language_server_command()` in `src/lib.rs` spawns the hardcoded path `/Users/vasilsokolik/www/lsp-poc/target/debug/lsp-poc` with the argument `serve`. Consequence: a debug build must exist before the extension works, and server changes need only `cargo build` — rebuild `extension.wasm` only when the extension crate itself changes. `extension.toml` registers the server for JSON and JSONC; `.zed/settings.json` selects it and disables `json-language-server`.

## The Capability Wiring Pattern

Adding an LSP capability means two paired edits in `crates/lsp-poc/src/server.rs`, then a rebuild:

1. **Advertise** it in `server_capabilities()` — e.g. `hover_provider: Some(HoverProviderCapability::Simple(true))`.
2. **Implement** the matching `Server` trait method. The trait's handler surface: `hover`, `completion`, `completion_resolve`, `code_action`, `code_action_resolve`, `link`, `link_resolve`, `declaration`, `definition`, `references`, `rename`, `rename_prepare`, `document_format`, `document_range_format`, `document_diagnostics`.
3. **Operate on `ServerState`** inside the method: `state.document(&url)` for the document, `doc.node_at_position_named(pos)` for the node, and the range helpers from `async_language_server::tree_sitter_utils`.

Always do both halves of the pair: a capability advertised but not implemented lies to the editor, and a method implemented but not advertised is dead code. Claim documents once, in `server_document_matchers()`, via `DocumentMatcher::new("json").with_url_globs(["**/*.json"]).with_lang_strings(["JSON"]).with_lang_grammar(tree_sitter_json::LANGUAGE.into())`.

## Naming

Crates use kebab-case for both directories and package names. Modules are lowercase, one word where possible, in `mod.rs`-style directories. The server type is `PocLanguageServer`.

The extension's directory (`crates/zed-md-lsp`) and package name (`zed-lsp-poc`) disagree — the directory is a leftover from the project's original PHP focus, as are `src/utils.rs` and lib.rs's "Zed extension for PHP LSP" doc comment. Always pass the package name to cargo (`cargo -p zed-lsp-poc`), and do not propagate the PHP name into new files, commands, or docs.

---
_Describe the wiring pattern once so new capabilities and modules slot in without structural debate; the two-crate split and the capability/method pairing are the invariant parts._

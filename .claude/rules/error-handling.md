# Error Handling

Errors in this workspace are strictly typed and live in one place per crate. When code fails, it fails with its crate's error enum — never with `String`, `anyhow::Error`, or a variant invented inside a feature module. The owner asked for this explicitly; treat it as settled even where a leaner pattern would do for a POC.

## One crate, one error module

Every crate that handles errors owns exactly one error module. For `crates/lsp-poc` that is `src/error.rs`, exposing `PocError` (named after `PocLanguageServer`). Add `mod error;` to `src/main.rs` in the same change that creates the file — a module on disk is dead until declared, and `schema/` and `utils.rs` are the current cautionary examples (the structure rule's gotcha). The extension crate (`zed-lsp-poc`) gets its own error module only when it grows real fallible code, not preemptively while it is a pure launcher.

Never declare error enums inside feature modules (`hovers/`, `completions/`, `schema/`). If a feature needs a new failure mode, add the variant to `PocError` and keep the feature module returning `Result<T, PocError>`.

## The enum

Derive with `thiserror` (add it to `crates/lsp-poc/Cargo.toml`; the framework itself uses it, so nothing new enters the dependency tree):

- Declare one variant per distinct failure mode, named for the failure, not for a message.
- Preserve chains: `#[from]` when the underlying error converts without extra context; `#[source]` plus `map_err` when the variant also carries call-site context (a path, a feature name). Never stringify a source error into a message field — the chain is the debugging value.
- Start messages lowercase with no trailing punctuation so chains compose: `failed to read schema file /foo.json: permission denied`. Acronyms like JSON keep their case.
- Do not write a `String`-carrying variant whose payload is really a message. Strings are fine as data (a URL, a key); a variant that only wraps a string is `ServerError::Unknown` in disguise and must not exist — the framework's `Unknown(String)` escape hatch is exactly what this rule keeps out of our code.
- Keep fields owned (`String`, `PathBuf`, `&'static str`, source errors): no `Rc`, no borrow, no non-`Send` source. thiserror then keeps `PocError: Send + Sync + 'static` for free.

Absence is not an error. `hover()`'s `let Some(doc) = ... else { return Ok(None) }` pattern stays: a missing document or position is a normal "nothing to do", answered with `Ok(None)`, not a variant. Only true failures — what the request cannot recover from — become `PocError` variants.

## Boundaries

- **Inside the server** (`server.rs` and the feature modules): fallible functions return `Result<T, PocError>`.
- **At the handler boundary**: implement `From<PocError> for ServerError`. We own `PocError`, so the orphan rule is satisfied; `?` inside handler methods then converts automatically and the client receives a proper JSON-RPC error. Log the full chain once at this edge with `tracing::error!` — this mapping is the single place a `PocError` becomes protocol output.
- **Never let a raw string reach `?`** in a `ServerResult` context. `ServerError` implements `From<&str>` and `From<String>`, so a string error compiles and silently lands in `ServerError::Unknown`, discarding the type discipline this rule exists for.
- **`anyhow` stays at the CLI edge only**: `main() -> anyhow::Result<()>` and `cli/serve.rs`'s `.context(...)` around `serve()` keep their current shape. Server code does not import `anyhow`.

## Skeleton

The shape of `crates/lsp-poc/src/error.rs`, with plausible variants for planned capabilities:

```rust
use async_language_server::{
    server::{ServerErrorCode, ServerError},
    tree_sitter,
};
use thiserror::Error;

/// Every failure the lsp-poc server can produce.
#[derive(Debug, Error)]
pub enum PocError {
    /// A tree-sitter query backing a feature failed to compile.
    #[error("failed to compile tree-sitter query for {feature}")]
    QueryCompile {
        feature: &'static str,
        #[source]
        source: tree_sitter::QueryError,
    },

    /// A schema file backing a JSON document could not be read.
    #[error("failed to read schema file {path}")]
    SchemaRead {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

impl From<PocError> for ServerError {
    fn from(err: PocError) -> Self {
        tracing::error!(error = %err, "request handler failed");
        ServerError::rpc(ServerErrorCode::INTERNAL_ERROR_CODE, err.to_string())
    }
}
```

At a call site, `map_err` supplies the context a blanket `From` cannot know:

```rust
let schema = read_schema(&path)
    .map_err(|source| PocError::SchemaRead { path: path.clone(), source })?;
```

## Restraint

Type only errors that can actually occur in this POC. No speculative variants for impossible scenarios and no catch-all "future-proofing" variant — the owner asked for typed errors, not an exhaustive taxonomy, so the product rule's escape clause applies. If a variant has no realistic producer in this workspace, delete it rather than advertise it.

The unwrap/expect/dbg gates belong to the tech rule; this rule adds nothing there and inherits them.

---
_One typed error per crate, in one module: `PocError` for lsp-poc, chains preserved, mapped to a JSON-RPC error at the handler edge, `anyhow` only where the process talks to a human._

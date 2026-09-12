# Error Handling

## One crate, one error module

`crates/lsp-poc` owns exactly one error module, `src/error.rs`, exposing `PocError`
(named after `PocLanguageServer`). The owner asked for typed errors explicitly — treat
this as settled even for a POC. Declare `mod error;` in `src/main.rs` in the same
change, per the structure rule's module gotcha. The extension crate earns its own
error module only when it grows real fallible code.

- Derive with `thiserror`; add it to `crates/lsp-poc/Cargo.toml` when `error.rs` lands
  — the framework already depends on it, so nothing new enters the dependency tree.
- Never declare error enums inside feature modules: a new failure mode is a new
  `PocError` variant, and feature code returns `Result<T, PocError>`.
- A leaf utility without protocol semantics may carry its own narrow error type in the
  same `error.rs` — the framework's `RangeError` works this way. Do not fold it into
  `PocError` for the name's sake, and do not scatter error types across modules.

## The enum

- Preserve `source()` chains; never stringify a source error into a message field —
  the chain is the debugging value.
- Use `#[from]` only where the conversion loses no context: it implies `#[source]` and
  permits nothing in the variant beyond the source. When context rides along (a path,
  a key), keep `#[source]` and build the variant through `map_err` at the call site.
- No `#[error(transparent)]` in a catch-all: it forwards `source()` into the inner
  error, so the variant itself vanishes from the chain. Keep the wrapped error as both
  the `Display` message and the chain node (`#[error("{0}")]`).
- Keep fields owned (`String`, `PathBuf`, source errors) so `PocError` stays
  `Send + Sync + 'static` — the shape the boundary boxing requires.
- Display messages start lowercase with no trailing punctuation; acronyms like JSON
  keep their case.
- Absence is not an error: a missing document or node is a normal "nothing to do",
  answered with `Ok(None)` via let-else, as `hover()` does.
- Restraint: type only failures that can actually occur — no speculative variants, no
  catch-all future-proofing variant, no variant that only wraps a string message.

## One boundary

Trait methods return `ServerResult`; the code below them returns `Result<T, PocError>`.
Exactly one conversion — `From<PocError> for ServerError` (we own `PocError`, so the
orphan rule holds) — is the single place a `PocError` becomes protocol output:

```rust
impl From<PocError> for ServerError {
    fn from(err: PocError) -> Self {
        tracing::error!(error = %err, "request handler failed");
        ServerError::Other(Box::new(err))
    }
}
```

Boxing keeps the chain, and the framework maps `ServerError` onto the wire error from
there. Log once at this edge; no second conversion site anywhere. `ServerError::Other`
accepts any boxed error — a bare string reaches it through std's
`From<&str> for Box<dyn Error>` — so never build one by hand from a stringified
failure.

`anyhow` stays at the CLI edge only: `main() -> anyhow::Result<()>` and the
`.context(...)` around `serve()` in `cli/serve.rs`. Server code never imports
`anyhow`.

## No swallowed failures, no panics on input

- Never drop a fallible call — propagate it with `?` or trace it, including the error
  value (`tracing::warn!("failed: {error}")`). A bare `let _ =` on a fallible call is
  a bug unless the failure is impossible by construction.
- A stream of fallible entries is not one failure: report the unreadable entry and
  continue — one bad directory must not abort the whole scan.
- Client-supplied values — positions, URIs, document text — are untrusted: convert
  them through fallible paths, never `panic!` or `unreachable!` on them. `expect` is
  only for invariants whose violation means a bug, its message stating the contract;
  a caller-supplied value is never an invariant. The `expect_used`/`unwrap_used`
  gates are test-aware via `clippy.toml`.

## Docs

`# Errors` on every public fallible item, `# Panics` on every public panicking item —
enforced workspace-wide by clippy pedantic (`missing_errors_doc`/`missing_panics_doc`).

---
_No failure is stringified, swallowed, or panicked on: `PocError` keeps the chain
until the one boundary boxes it into a `ServerError`._

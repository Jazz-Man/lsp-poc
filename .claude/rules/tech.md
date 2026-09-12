# Technology

## Toolchain

Keep the workspace on Rust edition 2024, pinned to the `stable` channel by
`rust-toolchain.toml` (components: `rustfmt`, `clippy`) — do not bypass the pin.
Run tokio in `current_thread` flavor (`crates/lsp-poc/src/main.rs`); upgrade
dated pins (miri's `MIRI_TOOLCHAIN` in the Makefile) deliberately, never casually.

## Stack

`async-language-server` — a direct dependency of `crates/lsp-poc`, git rev
`v0.10.0`, with the `tree-sitter` feature — owns the JSON-RPC loop, the
document store, and tree-sitter parsing. Use its re-exports (`lsp_types`,
`tree_sitter`, `tree_sitter_utils`) instead of adding those crates directly.
`serve(server)` serves over stdio; the single `serve` subcommand parses its
flags but never reads them. The Markdown document matcher's grammar is
`tree-sitter-md`. The workspace declares no `[features]` section yet — add a
no-default-features test leg together with the first feature. The Zed
extension (`zed_extension_api`, wasm `cdylib`) is a launcher only: it spawns
`target/debug/lsp-poc serve --stdio`; the tuned `[profile.release]` at the
workspace root is for release builds — ignore it day-to-day.

## Verification battery

The Makefile is the command contract (`make help` is authoritative); it routes
cargo through `rtk` locally and plain `cargo` when `CI` is set. `make battery`
chains the gates `fmt`, `clippy`, `doc`, `test`, `dylint`:

- `fmt` checks formatting (`fmt-fix` applies); `clippy` lints every target
  with warnings as errors.
- `doc` builds `+nightly cargo doc --workspace --no-deps` with warnings as
  errors (the index-page flags need nightly).
- `test` runs `cargo nextest run --workspace` — today, the single arch-lint
  test (`crates/lsp-poc/tests/architecture.rs`, declarative layers in
  `crates/lsp-poc/arch-lint.toml`); `cargo nextest run <filter>` runs one test.
  No doctest leg: no doctestable `[lib]` target exists.
- `dylint` runs the external lint suites (see Lints).

On demand: `make dupes` (`dupes.toml` tolerates zero duplicate groups;
deliberate ones get a reasoned `.dupes-ignore.toml` entry) and `make deny`
(`deny.toml`: deny-level findings gate, warnings advise). Never gates, never
in the battery: `make mutants` (a diagnostic sweep — `FILE=<path>` scopes it,
exit 2 means survivors, run it alone) and `make miri` (one-time `make
miri-setup`; pinned toolchain, `x86_64-apple-darwin` target, arch-lint test
excluded by name, `--no-tests=warn` — rationale and canary live in the
Makefile). A failing check is a signal about the code: investigate it per the
global `no-workarounds` process — never make a check pass by suppressing it.

## Lints

Set lint levels in the root `Cargo.toml`, not source attributes: clippy `all`,
`cargo`, `pedantic` at deny, with five inherited allow entries — do not add
new ones; `unwrap_used`/`expect_used` at deny, test-aware via `clippy.toml`
(which also holds the complexity thresholds); the restriction set, `dbg_macro`
among them, at warn; `missing_docs` and `unsafe_code` at deny;
`unexpected_cfgs` whitelists dylint's `cfg(dylint_lib)`. The dylint suites
(tag-pinned Trail of Bits examples, rc-pinned `perfectionist`) are pinned in
`dylint.toml` alongside the reasoned disables; after any tag bump, re-verify
the `[perfectionist]` entries still name real rules — unknown names silently
vanish.

## Logging

Log with `tracing` macros only; never print to stdout — it is the LSP transport.
`crates/lsp-poc/src/tracing.rs` writes everything to stderr; `RUST_LOG` wins
when set, otherwise `debug` in debug builds and `info` in release.

## Key technical decisions

- Implement capabilities through the `Server` trait; never touch the transport.
- Treat an `async-language-server` rev bump as a deliberate experiment:
  re-check the `Server` trait surface — the pinned crate changes shape
  between tags.
- Do not set `RUSTFLAGS` in `.cargo/config.toml`: `target/` is shared with
  rust-analyzer, and flag divergence forces a full dep-tree rebuild. If builds
  OOM, uncomment `jobs = 8` there; its `[alias] lint` gives short clippy output.

---
_Use the pinned toolchain and let the Makefile verify the tree — failures are
investigated, never suppressed._

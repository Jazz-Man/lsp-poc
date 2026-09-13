# Remove the CLI vestige — stdio-only entry

Date: 2026-09-13 · Status: approved by owner (design + docs-in-scope decision) · Branch: `feature/markdown` · Owner commits (git is read-only for agents)

## Context

Upstream `async-language-server` removed the socket transport — commit `6c62469`
("Remove socket transport, serve over stdio only"), verified as an ancestor of the
pinned rev `v0.10.0` (`935d564`). The upstream crate is a library with no binary and
no CLI; its entire transport story is `serve(server)` over stdio, re-exported from
`async_language_server::server` (already the path `src/cli/serve.rs:5` imports).

Everything left in this tree that remembers a socket or a subcommand is therefore
vestigial:

- `crates/lsp-poc/src/cli/mod.rs` — clap `Parser`/`Subcommand` wrapper whose only
  subcommand is `serve`.
- `crates/lsp-poc/src/cli/serve.rs` — declares `--socket` (alias `port`) and
  `--stdio` (lines 10–15); the flags are parsed and never read, because the pinned
  `serve()` takes no arguments.
- `clap = { version = "4.6.6", features = ["derive","env"] }` at
  `crates/lsp-poc/Cargo.toml:19` — exists only for the module above.
- `crates/zed-md-lsp/src/lib.rs:27` — passes `["serve", "--stdio"]` to a binary that
  ignores both.
- `crates/lsp-poc/arch-lint.toml` — a `cli` scope (80–82), `"cli"` in all 12
  `deny-scope-dep` `to`-lists, the `server-below-cli` rule (156–160), and the header
  comment "cli owns clap and the serve() call" (21–22) — all guarding a module about
  to not exist.
- Doc references that become false: `CLAUDE.md:10` (run `-- serve`, "flags are parsed
  but unused"), `CLAUDE.md:30`, `structure.md:11` ("starts the CLI", "`src/cli/`
  holds the clap subcommands"), `structure.md:17`, `tech.md:16` ("the single `serve`
  subcommand parses its flags"), `tech.md:21`.

## Decisions

- **Entry shape**: approach A — `main.rs` calls `serve()` directly. No shim module
  (YAGNI), `anyhow` stays at the CLI edge per the error-handling rule (main *is*
  that edge).
- **Docs fold into the same plan** (owner decision): the six stale statements are
  direct consequences of the code change and get corrected in it, not in a later
  steering round.
- **Pin stays `v0.10.0`.** Upstream HEAD (`a5d3b83`) is one CI-only commit ahead; a
  rev bump is a separate deliberate experiment per the tech rule, not part of this.
- **Launcher**: `zed::Command::new(bin).envs(worktree.shell_env())` with no args.
  The hardcoded `target/debug` profile path stays (accepted POC trait).

## Changes

1. **Delete `crates/lsp-poc/src/cli/`** (`mod.rs`, `serve.rs`) and the `clap`
   dependency from `crates/lsp-poc/Cargo.toml`.
2. **`crates/lsp-poc/src/main.rs`** — drop `mod cli;`; the entry becomes
   `setup_tracing()` then `serve(PocLanguageServer::new())`, keeping the moved
   `.context("encountered fatal error - language server shutting down")` (from
   `cli/serve.rs:23`, unchanged wording — the fatal-shutdown signal is preserved).
   The crate doc comment, tokio `current_thread` flavor, and `anyhow::Result`
   return type stay.
3. **`crates/zed-md-lsp/src/lib.rs`** — remove the `.args(["serve", "--stdio"])`
   call; everything else untouched.
4. **`crates/lsp-poc/arch-lint.toml`** — remove the `cli` scope, strip `"cli"` from
   the 12 `to`-lists, delete the `server-below-cli` rule, and reword the header
   comment to "main.rs owns the serve() call at the process edge" (senses preserved:
   `server.rs` is still the single `Server` impl). `main.rs` remains outside the
   scope stack, as before. The arch-lint test validates this file, so any dangling
   `"cli"` reference fails `make test` — the gate catches an incomplete edit.
5. **Docs, six spots** — see the list in Context; each corrected to state the new
   truth: no subcommand, no flags, no `cli/` module, arg-less launcher command.

## Out of scope

- `src/utils.rs` fate (parked owner decision), `src/hovers/` placeholder, release
  profile, rev bump, any Zed settings or `extension.toml` change (they name no
  arguments today).

## Verification

- `make battery` green (fmt, clippy, doc, test — the arch-lint leg validates the
  edited toml —, dylint).
- **`make deny` + `make dupes` with the delta recorded**: clap leaves the dep tree,
  so some ratified duplicate groups may dissolve (e.g. `syn` via `clap_derive`).
  Stale skip entries are surfaced and reported, never silently kept.
- End-to-end (owner): `cargo build`, rebuild `extension.wasm` through the Zed UI
  (no CLI exists for that), open a `.md` file, confirm hover still fires — proving
  the arg-less launch path works inside Zed.

## Constraints

- Surgical: no drive-by edits beyond the files and spots named above.
- The owner commits; agents never run git-write commands.

# Configurable binary path in the Zed extension — design

**Date:** 2026-08-28
**Status:** Approved by owner
**Scope:** `crates/zed-md-lsp` (package `zed-lsp-poc`), `.zed/settings.json`, one CLAUDE.md line

## Problem

`language_server_command()` in `crates/zed-md-lsp/src/lib.rs` hardcodes the absolute path `/Users/vasilsokolik/www/lsp-poc/target/debug/lsp-poc`. The owner wants:

1. the path composed relative to the project instead of a machine-specific absolute string, and
2. a per-project switch in `.zed/settings.json` choosing the `debug` or `release` binary.

The POC is live-tested only inside this repository (owner's statement), which makes the open worktree's root the natural path base.

## Decision

- **Path base:** `worktree.root_path()` → `<root>/target/<profile>/lsp-poc`. A TODO comment in the code marks this as a POC decision to revisit once the real work is done.
- **Switch:** custom key `lsp.zed-lsp-poc.settings.profile` in `.zed/settings.json`, read via `zed::settings::LspSettings::for_worktree("zed-lsp-poc", worktree)` (zed_extension_api 0.7.0 — surface verified against the registry sources).
- **Rejected alternatives:** `$HOME`-based path via `shell_env()` (hardcodes the `~/www/lsp-poc` layout; its only advantage — foreign worktrees — is out of scope); Zed-standard `binary.path` override (no profile semantics, manual full paths); combined profile + path override (most flexible, most code, unneeded).

## Data flow

Zed starts a server for a worktree → calls `language_server_command(id, worktree)` → the extension reads its `lsp.zed-lsp-poc` section via `LspSettings::for_worktree` → extracts `settings.profile` → composes `<root_path>/target/<profile>/lsp-poc` → returns `Command { command, args: ["serve"], env: worktree.shell_env() }`. A changed `profile` applies after the language server restart in Zed (standard behavior).

## Profile rules

| Input | Result |
|---|---|
| key absent / `null` | `debug` |
| `"debug"` / `"release"` | matching `target/` subdirectory |
| any other string or non-string | start error: `invalid profile <value>: expected "debug" or "release"` |
| `settings` present but not an object | start error: `settings must be an object` |

Errors from `for_worktree` itself propagate as start errors too. No silent fallback: an invalid setting is a signal, not noise.

## Code changes

All in `crates/zed-md-lsp/src/lib.rs` (~20 lines), no new modules:

- rewrite `language_server_command()`: read `LspSettings`, resolve the profile, compose the path;
- add a TODO comment next to path composition (POC: base is this repo's worktree root; replace with a real installation mechanism — packaged binary or a `binary.path` override — after the experimental phase);
- delete dead weight in the same file: the unused `cached_binary_path` field and the commented-out `worktree.which()` block.

## Error handling

The `Extension` trait's contract is `Result<T, String>` (host API), so short descriptive String errors are the boundary here — not a violation of the typed-error rule. The launcher crate gets no error module; the error-handling rule grants one only when the crate grows real fallible code.

## Settings

`.zed/settings.json` gets an explicit, self-documenting default:

```json
"lsp": {
  "zed-lsp-poc": {
    "settings": { "profile": "debug" }
  }
}
```

## Verification

1. Static (agent): `cargo lint`, `cargo fmt --all --check`, `cargo check -p zed-lsp-poc --target wasm32-wasip2` (target installed — verified via `rustup target list --installed`).
2. Live (owner, via the Zed UI): rebuild the dev extension; hover over a JSON file (debug path works); flip `profile` to `"release"` and restart the server (release path; a missing `target/release/lsp-poc` shows as a start error — a correct signal); an invalid value shows a start error.

## Out of scope

- Testing in worktrees other than this repo.
- Honoring `binary.path` or any absolute-path override.
- A packaging/installation mechanism for the binary.

## Doc fix (same change)

Replace the incorrect `zed extension build` line in CLAUDE.md (leftover from `/init`; the command does not exist in the owner's environment) with: the extension is installed as a dev extension in Zed; the owner rebuilds it through the Zed UI (no CLI); `extension.wasm` is a gitignored local artifact.

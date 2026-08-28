# Configurable binary path in the Zed extension — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hardcoded absolute binary path in the Zed extension with a worktree-relative path and a `debug`/`release` switch read from `.zed/settings.json`.

**Architecture:** `language_server_command()` forwards its `language_server_id` to `LspSettings::for_worktree`, reads the extension's `settings` JSON (`Value` via the `zed_extension_api::serde_json` re-export — no new dependencies), validates it once through `Profile::from_settings` (parse-don't-validate, errors on anything but the documented shapes), composes the path with `std::path::Path::join` from `worktree.root_path()`, and returns the `Command` via the ready-made builder from the API's `process` module. Invalid settings surface as server-start errors in Zed — never as silent fallbacks.

**Tech Stack:** Rust edition 2024, `zed_extension_api` 0.7.0 (wasm32-wasip2 cdylib). **Zero new dependencies** — everything needed is already exported by `zed_extension_api`.

**Spec:** `docs/superpowers/specs/2026-08-28-zed-extension-binary-path-design.md`

**Reference idiom:** `biomejs/biome-zed` `src/biome.rs` (same API version 0.7.0, verified via repomix): it depends on no serde crates, imports `serde_json` through the `zed_extension_api` re-export, forwards `language_server_id.as_ref()` to `LspSettings::for_worktree`, and builds paths with `Path::new(worktree.root_path().as_str()).join(...)`. This plan follows all four. Additionally, the crate's `src/process.rs` provides a builder (`Command::new(...).arg(...).envs(...)`) on the very `Command` type `language_server_command` returns — verified one type via the WIT: `extension.wit` does `use process.{command};`, so the extension interface imports the record from the `process` interface.

**Tests: deliberately none in this crate** (owner's decision, 2026-08-28): the Zed-integration glue is temporary POC scaffolding — the `profile` option included — and the real work happens in the server crate `crates/lsp-poc`, where tests belong. Verification here is compile + lint + the owner's live test in Zed.

## Global Constraints

- Always `cargo -p zed-lsp-poc` (package name); the directory is `crates/zed-md-lsp`.
- **Zero new dependencies.** Never add `serde`/`serde_json` (or anything else) to `Cargo.toml` `[dependencies]` — use the `zed_extension_api::serde_json` re-export for `Value`. The only manifest change in this plan is `[lints]`.
- Workspace clippy gates are warn-level: never write `.unwrap()`, `.expect()`, `dbg!()`.
- Error messages: lowercase start, no trailing punctuation, JSON keeps its case (`err-lowercase-msg`). `String` errors here are the `Extension` trait's host-API contract (`pub type Result<T, E = String>`), not a violation of the typed-error rule — the launcher crate gets no error module (project rule `error-handling.md`).
- `.zed/settings.json` is **JSONC with tab indentation** — preserve both.
- The server crate `crates/lsp-poc` is untouched by this plan.
- **Git is strictly read-only for every agent** — no agent runs any git write command (`add`, `commit`, `push`, …), ever. When a task's verification passes, report the changed files and the verification output to the owner and move on; the owner commits.
- No workarounds: every failure (invalid profile value, non-object settings, unknown settings key, unreadable settings) is a start error shown by Zed. No silent fallback except the documented `debug` default for absent/null `profile`.

### rust-skills rules this plan follows (mandated by the owner)

| Rule | How it shows up in the code |
|---|---|
| `type-no-stringly` | `Profile` enum instead of a string profile |
| `api-builder-pattern` | `zed::Command::new(...).arg(...).envs(...)` — the builder the API's `process` module already ships for the return type |
| `api-parse-dont-validate` | `Profile::from_settings` is the single validating constructor; the command only ever holds a validated `Profile` |
| `serde-deny-unknown-fields` (its intent, hand-rolled) | unknown settings keys are hard errors, not silently dropped — manual because no serde derive is used |
| `type-path-not-string` | binary path built with `Path::join`, converted to `String` only at the `Command` boundary |
| `pat-exhaustive-enum` | every `match` enumerates its variants explicitly; catch-alls only where the type is open (`Value`) |
| `err-lowercase-msg` | all error strings lowercase, no trailing punctuation |
| `name-as-free` | `Profile::as_str()` for the cheap borrow conversion |

---

### Task 1: `Profile` type + settings-driven `language_server_command()`

**Files:**
- Modify: `crates/zed-md-lsp/Cargo.toml` (lint inheritance only)
- Modify: `crates/zed-md-lsp/src/lib.rs`

**Interfaces:**
- Consumes: `zed::settings::LspSettings::for_worktree`, `worktree.root_path()`, `worktree.shell_env()`, `LanguageServerId::as_ref`, `zed_extension_api::serde_json::Value` (re-export; no new dependency), the `Command` builder from the API's `process` module.
- Produces: the final launcher behavior — `enum Profile { Debug, Release }` with `as_str()` and `from_settings(Option<&Value>)`, and the command built from worktree root + profile. Task 2's settings key and docs consume this behavior.

- [ ] **Step 1: Add lint inheritance to `crates/zed-md-lsp/Cargo.toml`**

Append at the end of the file (this brings the crate under the workspace clippy gates, matching `crates/lsp-poc`; one deliberate line beyond the spec, flagged to the owner):

```toml

[lints]
workspace = true
```

The `[dependencies]` section stays exactly as it is (`zed_extension_api = "0.7.0"` only).

- [ ] **Step 2: Rewrite `crates/zed-md-lsp/src/lib.rs`**

Replace the entire file content with (the current file's first line says "PHP", which is false; the owner's two exploratory imports and the dead `cached_binary_path` field are consolidated away):

```rust
//! Zed extension launcher for the lsp-poc language server.

use std::path::Path;

use zed_extension_api::{self as zed, settings::LspSettings, serde_json::Value, Result};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct LspPocExtension;

/// Cargo build profile the extension launches.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Profile {
    #[default]
    Debug,
    Release,
}

impl Profile {
    /// The profile's directory name under `target/`.
    fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }

    /// Extracts the profile from the extension's `settings` JSON.
    ///
    /// An absent or `null` `profile` falls back to `Debug`. An unknown
    /// settings key, an invalid profile value, or a non-object `settings`
    /// is an error, so a misconfiguration surfaces at server start.
    fn from_settings(settings: Option<&Value>) -> Result<Self> {
        let settings = match settings {
            None => return Ok(Self::default()),
            Some(Value::Object(map)) => map,
            Some(_) => return Err("settings must be an object".to_string()),
        };
        if let Some(unknown) = settings.keys().find(|key| key.as_str() != "profile") {
            return Err(format!("unknown setting {unknown:?}: expected \"profile\""));
        }
        match settings.get("profile") {
            None | Some(Value::Null) => Ok(Self::default()),
            Some(Value::String(profile)) => match profile.as_str() {
                "debug" => Ok(Self::Debug),
                "release" => Ok(Self::Release),
                other => Err(format!(
                    "invalid profile {other:?}: expected \"debug\" or \"release\""
                )),
            },
            Some(other) => Err(format!(
                "invalid profile {other}: expected \"debug\" or \"release\""
            )),
        }
    }
}

impl zed::Extension for LspPocExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // TODO(POC): the binary path is built from this repo's worktree root,
        // because the POC is live-tested only inside this project. Replace
        // with a real installation mechanism (packaged binary or a
        // `binary.path` settings override) once the experimental phase is
        // over.
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        let profile = Profile::from_settings(settings.settings.as_ref())?;
        let bin_file = Path::new(worktree.root_path().as_str())
            .join("target")
            .join(profile.as_str())
            .join("lsp-poc")
            .to_string_lossy()
            .to_string();

        Ok(zed::Command::new(bin_file)
            .arg("serve")
            .envs(worktree.shell_env()))
    }
}

zed::register_extension!(LspPocExtension);
```

Deleted along the way: the unused `cached_binary_path` field, the commented-out `worktree.which()` block, the false "PHP" header, and the owner's exploratory `use zed::http_client;` line (`LspSettings` returns via the merged import).

- [ ] **Step 3: Verify it compiles for the wasm target**

Run: `cargo check -p zed-lsp-poc --target wasm32-wasip2`
Expected: `Finished` with no errors (the target is installed — verified during planning).

- [ ] **Step 4: Format and lint**

Run: `cargo fmt --all && cargo lint`
Expected: rustfmt may reflow some arms — let it; lint reports no new warnings for this crate (now gated by the workspace `[lints]`).

---

### Task 2: Default `profile` in settings, doc truth fixes, final gate

**Files:**
- Modify: `.zed/settings.json`
- Modify: `CLAUDE.md`
- Modify: `.claude/rules/structure.md`

**Interfaces:**
- Consumes: the behavior shipped in Task 1 (the settings key it reads: `lsp.zed-lsp-poc.settings.profile`).
- Produces: documentation that matches the shipped behavior; the owner handoff.

- [ ] **Step 1: Set the explicit default in `.zed/settings.json`**

Replace the `"lsp"` block (the file is JSONC — keep the existing tab indentation):

```jsonc
	"lsp": {
		"zed-lsp-poc": {
			"settings": {
				"profile": "debug"
			}
		}
	}
```

- [ ] **Step 2: Fix the three invalidated statements in `CLAUDE.md`**

2a. Build bullet (first bullet under Commands) — replace:

```markdown
- Build: `cargo build` — the Zed extension launches the debug binary at `target/debug/lsp-poc`, so rebuild after server changes for the extension to pick them up
```

with:

```markdown
- Build: `cargo build` — the Zed extension launches `target/debug/lsp-poc` by default (`target/release/lsp-poc` via `lsp.zed-lsp-poc.settings.profile` in `.zed/settings.json`), so rebuild after server changes for the extension to pick them up
```

2b. The `zed extension build` line (spec's doc fix) — replace:

```markdown
- Zed extension wasm: rebuilt with Zed's extension CLI (`zed extension build` from `crates/zed-md-lsp/`); `extension.wasm` is a gitignored local artifact
```

with:

```markdown
- Zed extension: installed as a dev extension in Zed and rebuilt through the Zed UI by the owner (no CLI); `extension.wasm` is a gitignored local artifact
```

2c. Architecture statement about the hardcoded path — replace the sentence:

```markdown
wasm `cdylib` on `zed_extension_api`. `language_server_command()` launches the lsp-poc binary with the `serve` subcommand; the binary path is hardcoded to the absolute debug path `/Users/vasilsokolik/www/lsp-poc/target/debug/lsp-poc`, so a debug build must exist before the extension works.
```

with:

```markdown
wasm `cdylib` on `zed_extension_api`. `language_server_command()` launches the lsp-poc binary with the `serve` subcommand; the path is `<worktree-root>/target/<debug|release>/lsp-poc`, chosen by `lsp.zed-lsp-poc.settings.profile` in `.zed/settings.json` (default `debug`), so a build of the selected profile must exist before the extension works.
```

- [ ] **Step 3: Fix the two invalidated statements in `.claude/rules/structure.md`**

3a. Replace:

```markdown
`language_server_command()` in `src/lib.rs` spawns the hardcoded path `/Users/vasilsokolik/www/lsp-poc/target/debug/lsp-poc` with the argument `serve`.
```

with:

```markdown
`language_server_command()` in `src/lib.rs` spawns `<worktree root>/target/<profile>/lsp-poc` — `profile` comes from `lsp.zed-lsp-poc.settings.profile` (default `debug`) — with the argument `serve`.
```

3a-2. The next sentence's first clause is stale for the same reason — replace:

```markdown
Consequence: a debug build must exist before the extension works, and server changes need only `cargo build`
```

with:

```markdown
Consequence: a build of the selected profile must exist before the extension works, and server changes need only `cargo build`
```

3b. The Naming section lists the PHP doc comment as a leftover; Task 1 removed it. Replace:

```markdown
the directory is a leftover from the project's original PHP focus, as are `src/utils.rs` and lib.rs's "Zed extension for PHP LSP" doc comment.
```

with:

```markdown
the directory is a leftover from the project's original PHP focus, as is `src/utils.rs`.
```

- [ ] **Step 4: Final gate**

Run each; all must pass:

```bash
cargo fmt --all --check
cargo lint
cargo check -p zed-lsp-poc --target wasm32-wasip2
```

Expected: fmt clean, lint with no new warnings, wasm check `Finished`.

- [ ] **Step 5: Hand off to the owner for the live test and commits**

All changes from Tasks 1–2 sit in the working tree uncommitted; the owner reviews and commits them (a natural split is one commit per task's file set). The agent's work ends here. Report to the owner:
1. Rebuild the dev extension via the Zed UI and restart the language server.
2. Hover over a JSON file — server starts from `target/debug/lsp-poc`.
3. Flip `profile` to `"release"`, restart the server — starts from `target/release/lsp-poc` (if that binary is absent, Zed shows a start error — correct signal).
4. Set `profile` to an invalid value, restart — Zed shows the `invalid profile ...` start error.

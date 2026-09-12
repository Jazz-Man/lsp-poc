# Quality-Gate Tooling Port — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring this workspace's quality gates to parity with the owner's
async-language-server fork (v0.10.0): fix the three live errors, port the remaining
config files with per-file triage, and leave `make battery` + `make deny` + `make dupes`
+ wasm check green.

**Architecture:** Round two of a parity port (spec:
`docs/superpowers/specs/2026-09-12-quality-gates-design.md`, approach A). Code fixes
first so the tree compiles, then configs in dependency order (deny last-but-one because
its skips depend on the manifest change), then the Makefile/dylint layer, then the
verification battery.

**Tech Stack:** cargo (workspace, edition 2024, stable pinned), cargo-deny 0.20.2,
cargo-nextest 0.9.143, cargo-mutants 27.1.0, cargo-dylint (drivers build on first run),
miri on `nightly-2026-09-08`, GNU make, rtk (local output filter).

## Global Constraints

- Branch: `feature/markdown` only. **No git write commands from agents — the owner
  commits.** Every task ends at its verification gate; the owner commits reviewed work.
  (No "Commit" steps appear in this plan on purpose.)
- No workarounds: a finding is fixed at its cause. Suppressions exist only as reasoned,
  owner-visible entries (deny `skip` reasons, perfectionist `disable` reasons,
  arch-lint comment-form allows). If a gate surfaces an unpredicted fire, STOP and
  triage with the owner — never silence silently.
- **Never set `RUSTFLAGS` in `.cargo/config.toml`** — `target/` is shared with
  rust-analyzer. Miri needs only `MIRIFLAGS` + `+toolchain` selection.
- Makefile **recipe lines use hard tabs** — spaces break make with
  "missing separator".
- Tests belong in `crates/lsp-poc`, never in `crates/zed-md-lsp`.
- `vendors/md-lsp/` is read-only reference material, excluded from dupes (already
  configured); both workspace crates stay analyzed.
- Clippy gates are deny-level (`all`/`cargo`/`pedantic`, `expect_used`/`unwrap_used`,
  `missing_docs`, `unsafe_code`); any code change must keep `cargo lint` clean.
- Source of truth for the fork's file contents:
  `/Users/vasilsokolik/www/async-language-server/` (read freely; never modify).

---

### Task 1: Restore the `lib.rs` extension-registration wrapper (fixes the build)

**Files:**
- Modify: `crates/zed-md-lsp/src/lib.rs:32`

**Interfaces:**
- Consumes: nothing (standalone fix).
- Produces: a compiling workspace — every later task's verification depends on this.

- [ ] **Step 1: Replace the bare macro invocation with the private-module wrapper**

Current line 32 is exactly:

```rust
zed::register_extension!(LspPocExtension);
```

Replace it with:

```rust
// zed's register_extension! expands to a pub extern "C" `init-extension`
// guest export (plus wasi-gated glue). The private module keeps that
// macro-generated item out of this crate's public API surface — the docs
// gate then has nothing to fire on — while the linker-level export_name is
// unaffected by the Rust-side module path.
mod register {
    use super::LspPocExtension;

    super::zed::register_extension!(LspPocExtension);
}
```

This is a root fix, not a suppression: the macro-generated `pub` item stops being
reachable from the crate's public API, so `missing_docs` has nothing to fire on.
Attributes on the invocation were already proven not to propagate into the macro
expansion — do not try `#[allow]` here.

- [ ] **Step 2: Verify the host build**

Run: `cargo build 2>&1 | tail -3`
Expected: `Finished` — no `missing documentation for a function` error.

- [ ] **Step 3: Verify the wasm build**

Run: `cargo check -p zed-lsp-poc --target wasm32-wasip2 2>&1 | tail -3`
Expected: `Finished` — clean; the `unsafe_code` deny must NOT fire on the macro's
wasi-gated glue (verified previously, confirm again).

---

### Task 2: Consolidate manifests (release profile + direct git dep + unexpected_cfgs)

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Modify: `crates/lsp-poc/Cargo.toml`

**Interfaces:**
- Consumes: Task 1 (tree compiles).
- Produces: root `[profile.release]` carries the fat values; `async-language-server`
  is declared directly in `crates/lsp-poc/Cargo.toml` (removed from
  `[workspace.dependencies]`); `unexpected_cfgs` check-cfg line present. Task 3's deny
  run relies on the direct declaration to eliminate the `wildcard` finding.

- [ ] **Step 1: Root `Cargo.toml` — replace the minimal release profile with the fat one**

Current block:

```toml
[profile.release]
lto = true
```

Replace with (values moved verbatim from the deleted crate-level profile):

```toml
# Consolidated from crates/lsp-poc (owner call 2026-09-12): cargo applies
# profiles only from the workspace root. Zed launches the debug binary; this
# profile exists for release builds of the server.
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
overflow-checks = false
```

- [ ] **Step 2: Root `Cargo.toml` — add the dylint check-cfg line**

In `[workspace.lints.rust]`, after the `unsafe_code = "deny"` line, add:

```toml
unexpected_cfgs = { level = "warn", check-cfg = ["cfg(dylint_lib, values(any()))"] }
```

(dylint runs set `cfg(dylint_lib)`; without this the compiler warns on every dylint
pass. Fork-identical.)

- [ ] **Step 3: Root `Cargo.toml` — remove the git dep from `[workspace.dependencies]`**

Delete this line (single consumer; the pin moves to the consumer in Step 4):

```toml
async-language-server = { git = "https://github.com/Jazz-Man/async-language-server", rev = "v0.10.0", features = ["tree-sitter"] }
```

Keep `tokio` and `arch-lint` in `[workspace.dependencies]` — they carry versions and
deny resolves them fine.

- [ ] **Step 4: `crates/lsp-poc/Cargo.toml` — declare the git dep directly**

Replace:

```toml
async-language-server.workspace = true
```

with:

```toml
async-language-server = { git = "https://github.com/Jazz-Man/async-language-server", rev = "v0.10.0", features = ["tree-sitter"] }
```

- [ ] **Step 5: `crates/lsp-poc/Cargo.toml` — delete the ignored crate-level profile**

Delete the entire block at the bottom of the file:

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
overflow-checks = false
```

- [ ] **Step 6: Verify — build is warning-free and the dep graph resolves**

Run: `cargo build 2>&1 | tail -3`
Expected: `Finished`; the `profiles for the non root package will be ignored` warning
is GONE. `Cargo.lock` is rewritten for the moved dep — fine.

Run: `cargo deny check bans 2>&1 | grep -c "wildcard"`
Expected: `0` — the `wildcard`/`unresolved-workspace-dependency` error disappeared.
(bans still FAILs on the four duplicate groups; that is Task 3's job — expected, not a
regression.)

---

### Task 3: `deny.toml` — final shape with this tree's triage

**Files:**
- Modify: `deny.toml` (full rewrite; current content is the cargo-deny template + edits)

**Interfaces:**
- Consumes: Task 2 (direct git dep — no wildcard finding).
- Produces: `cargo deny check` fully green. `make deny` (Task 6) calls this.

- [ ] **Step 1: Replace the whole file with**

```toml
# Dependency-hygiene policies. Run: make deny. Deny-level findings gate;
# warnings advise. Every exception carries a reason: what fires, why it is
# accepted, and the condition that removes it.

[advisories]
version = 2
# cargo-deny 0.18+ schema: vulnerabilities are unconditionally error-level (no
# key); unmaintained/unsound are scopes, and anything in scope is error-level.
unmaintained = "all"
yanked = "warn"

[bans]
multiple-versions = "deny"
wildcards = "deny"
# Structural duplicates (first-run triage 2026-09-12): every entry states what
# fires, why it is accepted, and the condition that removes it. Fix was
# exhausted first — each pinned side is the newest release of its dependent,
# so no lock-compatible `cargo update` can merge them today.
skip = [
    { name = "bitflags", version = "1", reason = "duplicate of 2.13.1: lsp-types 0.95.1 (via async-lsp 0.2.4) requires bitflags ^1.0.1 (every lsp-types release through the newest 0.97.0 still does), so no lock-compatible upgrade can merge them; removed when an async-lsp upgrade adopts an lsp-types on bitflags 2" },
    { name = "hashbrown", version = "0.14.5", reason = "duplicate of 0.17.1: dashmap 6.2.1 (the newest dashmap release) requires hashbrown ^0.14.5, so no lock-compatible upgrade can merge them; removed when a dashmap release using hashbrown 0.15+ lands" },
    { name = "hashbrown", version = "0.15.5", reason = "duplicate of 0.17.1: wasmparser 0.227 (via wit-bindgen 0.41 through zed_extension_api 0.7.0 — the wasm side the async-language-server fork does not have) pins 0.15.5; removed when the wit-bindgen chain moves past 0.15" },
    { name = "syn", version = "2", reason = "duplicate of 3.0.4: the syn 2 side is upstream proc-macro crates (arch-lint tooling, miette-derive, tracing-attributes, wit-bindgen, the url/icu chain), the syn 3 side is serde_derive, thiserror-impl, tokio-macros, clap_derive, futures-macro and lsp_macros; a major-version split cannot be merged by cargo update; removed when the syn 2 dependents release syn 3-compatible versions" },
    { name = "windows-sys", version = "0.59", reason = "duplicate of 0.61.2: waitpid-any 0.3.0 (the newest release, via async-lsp 0.2.4) requires windows-sys ^0.59 while rustix 1.1.4 / errno 0.3.14 (the same waitpid-any chain) and clap's anstream chain already sit on 0.61.2; removed when a waitpid-any release allowing windows-sys 0.60+ lands, then cargo update -p errno -p rustix and delete this skip" },
]

[licenses]
version = 2
# Minimal allow-list: trimmed to the licenses actually encountered in the tree
# (first-run triage + owner edits 2026-09-12). A dependency needing anything
# else deny-gates here and the list grows consciously, entry by entry.
allow = [
    "MIT",
    "Apache-2.0",
    "Unicode-3.0",
    "Zlib",
]
unused-allowed-license = "warn"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]

[sources.allow-org]
# allow-org matches ORGANIZATION names, not owner/repo — the starter file's
# "Jazz-Man/async-language-server" entry matched nothing (cargo-deny warned
# unmatched-organization).
github = ["Jazz-Man"]
```

Notes: the `[graph]`/`[output]` template sections are dropped (defaults are what the
fork runs with, too); `Zlib` stays in the allow-list — the triage run confirmed it is
exercised (zero `unused-allowed-license` warnings).

- [ ] **Step 2: Verify the full deny check**

Run: `cargo deny check 2>&1 | tail -1`
Expected: `advisories ok, bans ok, licenses ok, sources ok` and exit code 0.

---

### Task 4: Re-sync `tests/architecture.rs` comments + create `nextest.toml` / `mutants.toml`

**Files:**
- Modify: `crates/lsp-poc/tests/architecture.rs`
- Create: `.config/nextest.toml`
- Create: `.cargo/mutants.toml`

**Interfaces:**
- Consumes: nothing new.
- Produces: `.config/nextest.toml` `[profile.ci]` used by `make test` (Task 6) and
  `.cargo/mutants.toml` `test_tool` used by `make mutants` (Task 6); the architecture
  test still passes (Task 8's battery).

- [ ] **Step 1: Rewrite `crates/lsp-poc/tests/architecture.rs` to**

(Fork's expanded doc comment, adapted to this repo; code unchanged — note the
`result` → `analysis` rename to match the fork's current file.)

```rust
//! Architecture checks via `arch-lint`, wired programmatically.
//!
//! The `check!()` macro path applies preset defaults only: behavior knobs
//! (complexity thresholds, `allow_in_tests`, ...) configured in `[rules.*]`
//! TOML sections are parsed but never consulted. Building the analyzer here
//! makes the wiring explicit and every knob real, keeps `arch-lint` a
//! dev-dependency, and still loads the declarative layer rules from
//! `arch-lint.toml` (scopes and `deny-scope-dep`).
//!
//! Rule choices:
//! - `no-unwrap-expect` stays off: clippy's `expect_used`/`unwrap_used`
//!   (deny, test-aware via clippy.toml) own that axis, with self-verifying
//!   `#[expect(..., reason)]` suppressions.
//! - `handler-complexity` is excluded: it only measures functions named
//!   `handle_*`/`process_*`/`on_*`/`update*`, while clippy's
//!   `cognitive_complexity` and `too_many_lines` thresholds (clippy.toml)
//!   already cover every function in the crate.
//! - Per-site suppressions use the comment form with a mandatory reason.

use std::path::Path;

use arch_lint::{
    Analyzer, RuleBox, Severity,
    declarative::load_rules_from_toml,
    rules::{
        NoErrorSwallowing, NoSilentResultDrop, NoSyncIo, RequireThiserror, RequireTracing,
        TracingEnvInit,
    },
};

#[test]
fn architecture_rules_hold() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let rules: Vec<RuleBox> = vec![
        Box::new(NoSyncIo::new()),
        Box::new(NoErrorSwallowing::new()),
        Box::new(NoSilentResultDrop::new()),
        Box::new(RequireThiserror::new()),
        Box::new(RequireTracing::new()),
        Box::new(TracingEnvInit::new()),
    ];

    // Only build output is excluded: arch-lint scans `src/` and `tests/` of
    // this crate; `vendors/` lies outside the crate root, and the lint suites
    // in `dylint.toml` are external git dependencies — there is no nested
    // workspace in this tree for the scan to fence off.
    let mut builder = Analyzer::builder().root(root).exclude("**/target/**");
    for rule in rules {
        builder = builder.rule_box(rule);
    }

    // arch-lint: allow(no-sync-io) reason="the analyzer setup reads its own config synchronously; this is test code"
    let config = std::fs::read_to_string(root.join("arch-lint.toml"))
        .expect("arch-lint.toml is committed at the crate root");
    for rule in load_rules_from_toml(&config).expect("arch-lint.toml parses") {
        builder = builder.rule_box(rule);
    }

    let analyzer = builder.build().expect("analyzer builds");
    let analysis = analyzer.analyze().expect("analysis completes");

    assert!(
        !analysis.has_violations_at(Severity::Error),
        "{}",
        analysis.format_test_report(Severity::Error),
    );
}
```

- [ ] **Step 2: Create `.config/nextest.toml`**

```toml
[profile.ci]
fail-fast = false
```

- [ ] **Step 3: Create `.cargo/mutants.toml`**

```toml
test_tool = "nextest"

all_features = true
```

(`all_features` is a no-op until this crate grows `[features]`; it then becomes
correct without maintenance — fork parity.)

- [ ] **Step 4: Verify the architecture test**

Run: `cargo test -p lsp-poc architecture_rules_hold 2>&1 | tail -3`
Expected: `test architecture_rules_hold ... ok`

---

### Task 5: `.cargo/config.toml` — MIRIFLAGS in, broken aliases out

**Files:**
- Modify: `.cargo/config.toml`

**Interfaces:**
- Consumes: nothing new.
- Produces: the `MIRIFLAGS` env var consumed by `make miri` (Task 6); the `lint` alias
  stays (used across all verification steps).

- [ ] **Step 1: Replace the whole file with**

```toml
# Project-local Cargo settings.
#
# IMPORTANT: do NOT set RUSTFLAGS here. We deliberately share `target/` with
# rust-analyzer (the project is run often via `cargo run`, so the dir stays warm
# and rust-analyzer reuses the already-built dependency tree instead of
# recompiling it a second time). Any RUSTFLAGS divergence between this file and a
# plain `cargo run` would change the metadata hash and force a full recompile of
# the whole dep tree — the single biggest responsiveness killer on a workspace
# this size.
#
# If you genuinely need RUSTFLAGS (or a custom linker), also set
#   cargo.targetDir = true
# in .lsp.json and .zed/settings.json so rust-analyzer builds in its own target
# dir and stops fighting the CLI.

[build]
# The dep tree is memory-heavy to compile in parallel. If `cargo build`
# OOM-kills or makes the machine unresponsive, cap parallelism:
# jobs = 8

[alias]
# Lean-output lint: short machine-parseable diagnostics, no color codes —
# keeps agent/CI terminal output (and token burn) minimal.
lint = "clippy --workspace --all-targets --message-format=short"

[env]
# Uncomment to inspect exactly which config rust-analyzer applies on startup:
RA_DISABLE_INCREMENTAL_SYNC = "0"

# Miri interpreter flags for `make miri`. MIRIFLAGS is the interpreter's env
# var, not RUSTFLAGS — the shared-target invariant above is untouched.
MIRIFLAGS = "-Zmiri-backtrace=full -Zmiri-disable-isolation"
```

(The `fmtcheck`/`fmtall` aliases are deleted, not fixed: they pass `-p zed-md-lsp`, a
package that does not exist — the package is `zed-lsp-poc` — and their original
reason, vendored path deps with nightly-only rustfmt.toml in the zap project, does not
exist here. `make fmt` / `make fmt-fix` (Task 6) replace them; `cargo fmt --all`
touches only the two workspace members.)

- [ ] **Step 2: Verify the surviving alias and the fmt gate**

Run: `cargo lint 2>&1 | tail -2`
Expected: `Finished` — zero warnings.

Run: `cargo fmt --all --check 2>&1 | tail -2`
Expected: silent / `Diff in ...` absent, exit 0.

---

### Task 6: `Makefile` — the single source of verification commands

**Files:**
- Create: `Makefile` (repo root)

**Interfaces:**
- Consumes: `deny.toml` (Task 3), `dylint.toml` (Task 7's target consumes it — the
  Makefile itself only calls `cargo dylint --all`), `.config/nextest.toml` (Task 4),
  `.cargo/mutants.toml` (Task 4), `.cargo/config.toml` `MIRIFLAGS` (Task 5).
- Produces: `make fmt / fmt-fix / clippy / doc / test / dylint / battery / dupes /
  deny / mutants / miri / miri-setup / help` — Task 8's battery is `make battery`.
  Fork parity: `battery` excludes `deny`/`dupes`/`mutants`/`miri` (on-demand).

- [ ] **Step 1: Create `Makefile` with exactly this content**

**Recipe lines start with a hard TAB.** The plan file renders them below with tabs —
verify after writing (e.g. `grep -P '^\t' Makefile | wc -l` > 0).

```makefile
# Single source of verification commands — the contract referenced by
# CLAUDE.md and .claude/rules/tech.md. Edit freely; keep targets .PHONY and
# bodies plain.

# Route cargo through rtk locally (token-optimized output, transparent
# pass-through for commands rtk does not filter); plain cargo under CI, where
# CI=true is set by the runner and rtk is not installed.
ifeq ($(CI),)
CARGO_BIN := rtk cargo
else
CARGO_BIN := cargo
endif

# Toolchains (dated pins; upgrades are deliberate, never casual).
MIRI_TOOLCHAIN ?= nightly-2026-09-08

# Miri cross-interprets for x86_64: the document-parsing tests this repo is
# heading toward exercise ropey/str_indices NEON paths the interpreter cannot
# run on an aarch64 host (fork-inherited rationale).
TARGET := x86_64-apple-darwin

.DEFAULT_GOAL := help
.PHONY: help fmt fmt-fix clippy doc test dylint battery dupes deny mutants miri miri-setup

## help: list available targets
help:
	@rtk grep -E '^## ' $(MAKEFILE_LIST) | sed 's/^## //'

## fmt: check formatting (gate)
fmt:
	@$(CARGO_BIN) fmt --all --check

## fmt-fix: apply formatting
fmt-fix:
	@$(CARGO_BIN) fmt --all

## clippy: lint every target, warnings are errors (gate)
clippy:
	@$(CARGO_BIN) clippy --workspace --all-targets -- -D warnings

## doc: build docs, doc warnings are errors (gate)
doc:
	@RUSTDOCFLAGS="--enable-index-page -Zunstable-options -D warnings" $(CARGO_BIN) +nightly doc --workspace --no-deps

## test: nextest leg, then doctests (gate)
test:
	@$(CARGO_BIN) nextest run --workspace
	@$(CARGO_BIN) test --doc --workspace

## dylint: external lint suites (gate)
dylint:
	@$(CARGO_BIN) dylint --all -- --all-targets

## battery: the full pre-done gate (CI parity)
battery: fmt clippy doc test dylint

## dupes: duplication gate (on demand)
dupes:
	@$(CARGO_BIN) dupes check

## deny: dependency policies — advisories, duplicate bans, licenses, sources (on demand; deny-level findings gate)
deny:
	@$(CARGO_BIN) deny check

## mutants: mutation-testing sweep (on demand, heavy; run it alone — a concurrent build poisons its auto-derived per-scenario timeout). Optional FILE=crates/lsp-poc/src/foo.rs scopes the sweep to one file. Exit code 2 means survivors were found: this is a diagnostic sweep, not a gate — the battery never runs it.
mutants:
	@$(CARGO_BIN) mutants $(if $(FILE),-f $(FILE))

## miri: UB interpreter, cross-interpreted for x86_64 (ropey/str_indices NEON
## paths are not interpretable on an aarch64 host); on demand, slow — never in
## the battery. No --no-default-features leg: this crate has no features yet.
# architecture_rules_hold is excluded by name: its workspace-wide fs walk does
# not terminate under the interpreter (fork observed >1h) — infeasible, not failing.
miri:
	@$(CARGO_BIN) +$(MIRI_TOOLCHAIN) miri nextest run --target $(TARGET) -E 'not(test(architecture_rules_hold))'

## miri-setup: one-time toolchain preparation for make miri
miri-setup:
	@rustup component add miri --toolchain $(MIRI_TOOLCHAIN)
	@rustup target add $(TARGET) --toolchain $(MIRI_TOOLCHAIN)
	@$(CARGO_BIN) +$(MIRI_TOOLCHAIN) miri setup --target $(TARGET)
```

(No `bench` target: no benches exist. No `test-no-default-features`: no `[features]`
yet — added together with the first feature.)

- [ ] **Step 2: Verify the targets on the existing gate set (dylint arrives in Task 7)**

Run: `make help`
Expected: the target list above, one line each.

Run: `make fmt && make clippy && make doc && make test`
Expected: all four green. `make doc` needs nightly (installed ✓) and must finish
without doc warnings. `make test` runs the architecture test via nextest, then doctests
(none yet — trivially ok).

---

### Task 7: `dylint.toml` — external lint suites

**Files:**
- Create: `dylint.toml` (repo root)

**Interfaces:**
- Consumes: nothing.
- Produces: library discovery for `make dylint` (Task 6's target; first run builds the
  drivers). The `unexpected_cfgs` line from Task 2 keeps dylint passes quiet.

- [ ] **Step 1: Create `dylint.toml` with exactly this content**

(Suites and the ToB `pattern` list copied verbatim from the fork's `dylint.toml`; the
two comment adaptations are marked below.)

```toml
# Dylint configuration: library discovery + per-library config.
# Docs: https://github.com/trailofbits/dylint#workspace-metadata

[workspace.metadata.dylint]
libraries = [
    # Third-party style suite (owner decision 2026-09-08: adopt instead of
    # maintaining custom lints). Pinned rc tag — master-HEAD would move under
    # us; bump deliberately, like the ToB tag below.
    { git = "https://github.com/KSXGitHub/perfectionist", tag = "0.0.0-rc.22" },
    { git = "https://github.com/trailofbits/dylint", tag = "v6.0.4", pattern = [
        # `non_thread_safe_call_in_test` (dropped from v6.0.4 `general`): it
        # flags every `std::fs` call in tests, but real temp workspaces on disk
        # are the test convention this repo is adopting — the fires are by
        # design, so the lint is dropped, not silenced.
        "examples/general/abs_home_path",
        "examples/general/await_holding_span_guard",
        "examples/general/basic_dead_store",
        "examples/general/crate_wide_allow",
        "examples/general/incorrect_matches_operation",
        "examples/general/non_local_effect_before_unhandled_error",
        "examples/general/wrong_serialize_struct_arg",
        "examples/supplementary",
        # `assert_eq_arg_misordering` (dropped, v6.0.4 restriction): it
        # demands const-first `assert_eq!` arguments, against the actual-first
        # convention this codebase and the wider Rust ecosystem follow.
        "examples/restriction/collapsible_unwrap",
        "examples/restriction/misleading_variable_name",
    ] },
]

# Per-library lint configuration: keyed by library name, then lint name.
# Perfectionist rules that fight this repo's own conventions are disabled
# with reasons below — see CONFIGURATION.md in the perfectionist repo for
# the knob names. Unknown rule names are silently ignored by the library;
# all entries here are verified rule names.

[perfectionist]
disable = [
    # Prescribes `derive_more::{Display, Error}` and bans thiserror;
    # `.claude/rules/error-handling.md` mandates thiserror for this crate
    # (PocError). The two suites prescribe opposite error-derive policies,
    # and ours wins here.
    "thiserror_usage",
    # Wants inline test code extracted to separate files; the testing
    # convention this repo is adopting mandates inline `#[cfg(test)] mod
    # tests` (or sibling `tests.rs`) per module — the steering round will
    # codify it.
    "excessive_inline_tests",
    # Flags `T`/`E`/`F`-style generic parameters. Single-letter generics are
    # the Rust ecosystem idiom (`Result<T, E>`, `HashMap<K, V>`); renaming
    # them degrades the code. Distinct from the single-letter closure/const
    # rules, which we keep and fix: local bindings gain clarity from
    # descriptive names, canonical generics do not.
    "single_letter_generic",
]

# `impure_macro_arguments` stays active — the one genuine fire
# (`info!` in src/server/with_state/initialize.rs) is fixed in code.
# The exemption below covers only the quote-family token DSL: the rule's
# own docs put curly-brace DSL bodies (`quote! { ... }`) out of scope —
# "the evaluation contract is the macro's, not the call site's" — and
# `parse_quote!`/`quote!` with parentheses hit the same DSL through the
# paren form; their arguments are quoted tokens, never evaluated.
["perfectionist::impure_macro_arguments"]
ignore = ["quote", "parse_quote"]
```

(The `impure_macro_arguments` block is fork-verbatim on purpose: the exemption costs
nothing here and documents the DSL contract for when macros appear.)

- [ ] **Step 2: Verify discovery (cheap, no builds)**

Run: `cargo dylint list 2>&1 | wc -l`
Expected: `11` (perfectionist + 10 ToB example lints).

- [ ] **Step 3: First full dylint pass**

Run: `make dylint` — **expect a long first run**: dylint builds all drivers for the
active stable toolchain (tens of minutes on first run, cached after).

Expected outcome A (likely): the pass completes; with a ~90-line codebase the suites
are quiet.

Expected outcome B: fires appear. **STOP — do not suppress.** Per the no-workarounds
constraint: triage each fire with the owner — fix in code at the root, or add a
reasoned `disable` entry stating what fires and why it is by-design.

Expected outcome C: a driver fails to build under stable. STOP and surface (the fork's
list showed drivers built under `nightly-2026-05-28`; if the ToB examples need nightly,
the owner decides between a `DYLINT_TOOLCHAIN` pin and dropping a suite).

---

### Task 8: The verification battery

**Files:**
- none (verification only)

**Interfaces:**
- Consumes: Tasks 1–7 complete.
- Produces: the green-gate evidence the owner reviews before committing.

- [ ] **Step 1: Run the battery**

Run: `make battery`
Expected: `fmt`, `clippy`, `doc`, `test`, `dylint` all green (dylint cached from
Task 7).

- [ ] **Step 2: On-demand gates**

Run: `make deny`
Expected: `advisories ok, bans ok, licenses ok, sources ok`.

Run: `make dupes`
Expected: `Check passed.` (11 units / ~189 lines, 0 groups — vendors excluded).

- [ ] **Step 3: The extension target**

Run: `cargo check -p zed-lsp-poc --target wasm32-wasip2 2>&1 | tail -2`
Expected: `Finished` — clean.

- [ ] **Step 4: Optional miri mechanism check (owner's signal — heavy)**

`make miri-setup` then `make miri`: builds the whole tree for x86_64 under
`nightly-2026-09-08`; today it executes zero tests (the only test is excluded by name)
— the point is the mechanism, not coverage. Run only if the owner asks.

- [ ] **Step 5: Report and owner checkpoint**

Report per file: disposition, deviations + reasons, gate result. Hand the diff to the
owner — they review and commit (agents never write git). Out of scope, next round:
steering adaptation (`CLAUDE.md`, `.claude/rules/*` — tech.md's command section must
switch to `make` targets; testing.md does not exist yet; product/structure rules still
describe the JSON era; the `str_to_string`/`cognitive_complexity` "Fires today" comment
in root Cargo.toml needs a fire-count check — both lints measured zero fires in the
pre-port battery).

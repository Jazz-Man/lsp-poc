# Perf Remediation — derived MdIndex Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace every per-request full Markdown re-parse with one memoized per-generation derive — all nine `self.parse(...)` call sites in `server.rs` route through a single `index_of(&self, doc: &Document)` helper over `doc.derived::<Option<Arc<MdIndex>>>` — per `docs/superpowers/specs/2026-09-20-md-perf-derived-index-design.md`.

**Architecture:** The memoization is upstream's (`Document::derived`, keyed by `TypeId`, invalidated by any document write that starts a fresh generation). Our side adds one helper and reroutes nine call sites; the `parse(&self, text)` helper and its per-request cost retire. Outputs are byte-identical per request — this is a cost change, not a behavior change (the one semantic nuance, a pinned `None` derive, is near-unreachable and spec-ratified).

**Tech Stack:** unchanged — Rust 2024 stable, `async-language-server` git rev `v0.19.0`, `tree-sitter-md` 0.5.3 (`parser` feature), `cargo nextest`, Makefile battery. **Zero new dependencies.**

## Global Constraints

- Git is read-only for every agent: NO git-write commands. The owner commits. (No commit steps; each task ends STOP for owner commit.)
- Zero new dependencies; zero new `PocError` variants; no stdout in server code; no panics in hooks/handlers; no `.unwrap()`/`.expect()` outside tests.
- **Surgical: `crates/lsp-poc/src/server.rs` is the ONLY touched file.** Every other file byte-identical (the spec's Changes list is exhaustive).
- Battery discipline: `make battery` must finish with **zero `^warning` lines**.
- Absence stays `Ok(None)`/skip everywhere; trace messages change wording only ("markdown parse produced no tree" → "markdown index derived as None"; the per-site tails stay verbatim).
- **No new tests is the spec-ratified position** (spec, Verification): `Document` is constructible only via upstream `pub(crate) from_parts` and upstream's `testing` module is `#[cfg(test)] pub(crate)` — the seam admits no unit test. The 46-test suite staying green IS the regression gate; the live ritual covers the wiring (cycles 1–4 precedent). Do not invent implementation-shaped cache-peeking tests.
- Dupes: run `cargo dupes cleanup --dry-run` first if `make dupes` goes red; the existing `f22e948e` (Arc parse helpers) group may shift shape because the closures' bodies change — prune stale fingerprints, add a reasoned `.dupes-ignore.toml` entry only for a genuinely new deliberate group.
- LESSON (cycles 1–4): brief code is untested by construction — minimal mechanical fixes documented, never BLOCK for brief compile errors; BLOCK only for genuine ambiguity or a mandated anchor not matching the tree.
- Dispatch rules (owner): workers `sonnet`, reviewers `opus` who run `make battery`/`make dupes` themselves; findings = actionable fix-tasks; `rust-skills` + LSP verification mandatory; no workarounds. Model names carry no suffixes — long context is decided by global configuration.
- Every implementation task ends with `make fmt-fix && make fmt && make clippy && make test` green ("the task gates").

### Behavioral facts relied on (LSP-verified 2026-09-20 at the pinned v0.19.0 checkout `c8c6ec1`)

- `pub fn derived<T>(&self, compute: impl FnOnce(&Document) -> T) -> Arc<T> where T: Send + Sync + 'static` — hover-verified verbatim. Memoized per document **generation** (any write = fresh generation = fresh derived data); `compute` must not re-enter `derived` for the same `T` on the same document (ours calls only `links::build` — safe).
- `async_language_server::server::Document` is the public re-export (hover resolves to `documents::document::Document`; doc: "cloning bumps a refcount… the store installs a fresh generation… every outstanding clone keeps the content it was created with").
- `links::build(parser: &mut MarkdownParser, text: &str) -> Option<MdIndex>` (`links/mod.rs:203`, current tree).
- `MarkdownParser` is not `Sync` — that is why the field is `Arc<Mutex<MarkdownParser>>` (established by current `server.rs`). The lock is taken inside the derive closure, held only across the synchronous build; no handler in this server awaits anything (`ready(...)` eager compute), so the anti-lock-across-await rule holds by construction.
- `rename_prepare_at` is the ONE site whose `let text = doc.text_contents();` binding survives — the prepare placeholder slices `text` afterwards (`server.rs:183`). At every other site the binding exists only to feed `parse` and is deleted with it.
- Watchpoint: `(*memoized).clone()` may draw a perfectionist lint (`explicit_auto_deref` class). If a gate fires, apply the minimal mechanical fix and document it — never suppress.

---

### Task 1: `index_of` on `derived` — the nine-site swap

**Files:**
- Modify: `crates/lsp-poc/src/server.rs` (helper, import, nine call sites, one doc comment, six trace messages)

**Interfaces:**
- Consumes: `Document::derived` (facts above), `links::build`, the existing `Arc<Mutex<MarkdownParser>>` field.
- Produces (Task 2 relies on): `fn index_of(&self, doc: &Document) -> Option<Arc<links::MdIndex>>` on `PocLanguageServer`; `parse(&self, text)` retired.

- [ ] **Step 1: Import `Document`**

`server.rs:16` — add `Document` to the server import list:

old:
```rust
use async_language_server::server::{DocumentMatcher, Server, ServerResult, ServerState};
```
new:
```rust
use async_language_server::server::{
    Document, DocumentMatcher, Server, ServerResult, ServerState,
};
```
(rustfmt owns the final wrapping; `make fmt-fix` in the gates settles it.)

- [ ] **Step 2: Replace the `parse` helper with `index_of`**

`server.rs:38-41` — old:
```rust
    fn parse(&self, text: &str) -> Option<Arc<links::MdIndex>> {
        let mut parser = self.parser.lock().unwrap_or_else(PoisonError::into_inner);
        links::build(&mut parser, text).map(Arc::new)
    }
```
new (same position — after `new()`, before `resolver`):
```rust
    /// The document's link index, memoized on the document's current
    /// generation: the first caller builds it, every later caller within
    /// the same generation shares the `Arc`, and any write (edit, save,
    /// disk refresh) starts a fresh generation. A build that answers
    /// `None` (a tree-sitter timeout or cancellation — the server sets
    /// no timeout, so near-impossible) stays `None` until the document
    /// changes: deterministic input reproduces the result, so the pinned
    /// answer is the consistent one.
    fn index_of(&self, doc: &Document) -> Option<Arc<links::MdIndex>> {
        let parser = Arc::clone(&self.parser);
        (*doc.derived::<Option<Arc<links::MdIndex>>>(|document| {
            let mut parser = parser.lock().unwrap_or_else(PoisonError::into_inner);
            links::build(&mut parser, &document.text_contents()).map(Arc::new)
        }))
        .clone()
    }
```
Implementer notes: the closure moves the `Arc` clone (`closure-move-capture`) and locks inside its body — the guard lives only across the synchronous `links::build`. `(*...).clone()` clones the `Option` (an `Arc` bump when `Some`) — if perfectionist flags the deref shape, apply the minimal mechanical fix and document.

- [ ] **Step 3: Reroute the nine call sites**

3a. `resolver()` `open` closure (`server.rs:54`) — old:
```rust
                state
                    .document(open_url)
                    .and_then(|open_doc| self.parse(&open_doc.text_contents()))
```
new:
```rust
                state
                    .document(open_url)
                    .and_then(|open_doc| self.index_of(&open_doc))
```

3b. `document_snapshots()` (`server.rs:66`) — old:
```rust
            let Some(index) = self.parse(&document.text_contents()) else {
                tracing::debug!(
                    "markdown parse produced no tree; skipping {url} in reference scans",
                    url = document.url(),
                );
                continue;
            };
```
new:
```rust
            let Some(index) = self.index_of(&document) else {
                tracing::debug!(
                    "markdown index derived as None; skipping {url} in reference scans",
                    url = document.url(),
                );
                continue;
            };
```
and its doc comment (`server.rs:60-62`) — old:
```rust
    /// Candidate documents for reference scans: open documents first,
    /// then the workspace index's cached files (own parse per open doc —
    /// the established POC cost).
```
new:
```rust
    /// Candidate documents for reference scans: open documents first,
    /// then the workspace index's cached files (own derived index per
    /// open doc, memoized on its generation).
```

3c. `compute_diagnostics()` (`server.rs:90-94`) — old:
```rust
        let text = doc.text_contents();
        let Some(index) = self.parse(&text) else {
            tracing::warn!("markdown parse produced no tree; skipping diagnostics for {url}");
            return Vec::new();
        };
```
new (the `text` binding dies with the parse — it has no other reader here):
```rust
        let Some(index) = self.index_of(&doc) else {
            tracing::warn!("markdown index derived as None; skipping diagnostics for {url}");
            return Vec::new();
        };
```

3d. `definition_at()` (`server.rs:113-117`) — old:
```rust
        let text = doc.text_contents();
        let Some(index) = self.parse(&text) else {
            tracing::debug!("markdown parse produced no tree; no definition for {url}");
            return Ok(None);
        };
```
new:
```rust
        let Some(index) = self.index_of(&doc) else {
            tracing::debug!("markdown index derived as None; no definition for {url}");
            return Ok(None);
        };
```

3e. `references_at()` own site (`server.rs:136-140`) — old:
```rust
        let text = doc.text_contents();
        let Some(index) = self.parse(&text) else {
            tracing::debug!("markdown parse produced no tree; no references for {url}");
            return Ok(None);
        };
```
new:
```rust
        let Some(index) = self.index_of(&doc) else {
            tracing::debug!("markdown index derived as None; no references for {url}");
            return Ok(None);
        };
```

3f. `references_at()` `resolve_from` closure (`server.rs:148`) — old:
```rust
                state
                    .document(open_url)
                    .and_then(|open_doc| self.parse(&open_doc.text_contents()))
```
new:
```rust
                state
                    .document(open_url)
                    .and_then(|open_doc| self.index_of(&open_doc))
```

3g. `rename_prepare_at()` (`server.rs:175-179`) — **the `text` binding STAYS** (the placeholder slice at `server.rs:183` reads it). Old:
```rust
        let text = doc.text_contents();
        let Some(index) = self.parse(&text) else {
            tracing::debug!("markdown parse produced no tree; no rename for {url}");
            return Ok(None);
        };
```
new:
```rust
        let text = doc.text_contents();
        let Some(index) = self.index_of(&doc) else {
            tracing::debug!("markdown index derived as None; no rename for {url}");
            return Ok(None);
        };
```

3h. `rename_at()` own site (`server.rs:207-211`) — old:
```rust
        let text = doc.text_contents();
        let Some(index) = self.parse(&text) else {
            tracing::debug!("markdown parse produced no tree; no rename for {url}");
            return Ok(None);
        };
```
new (no other reader of `text` here — the binding dies):
```rust
        let Some(index) = self.index_of(&doc) else {
            tracing::debug!("markdown index derived as None; no rename for {url}");
            return Ok(None);
        };
```

3i. `rename_at()` `resolve_from` closure (`server.rs:226`) — old:
```rust
                state
                    .document(open_url)
                    .and_then(|open_doc| self.parse(&open_doc.text_contents()))
```
new:
```rust
                state
                    .document(open_url)
                    .and_then(|open_doc| self.index_of(&open_doc))
```

After 3a–3i a project-wide search for `self.parse(` in `crates/lsp-poc/src` must return zero hits.

- [ ] **Step 4: Compile and run the suite**

Run: `cargo nextest run -p lsp-poc`
Expected: 46/46 green (no test changed — this is the regression gate).

- [ ] **Step 5: Task gates**

Run: `make fmt-fix && make fmt && make clippy && make test && make dupes`
Expected: exit 0 across all five. `make dupes`: zero unrationed groups (see Global Constraints for the `f22e948e` shift).

STOP for owner commit.

---

### Task 2: battery + live verification (both clients) + whole-cycle final review

**Files:**
- No file changes (verification; controller-run).

- [ ] **Step 1: Gates + fresh binary**

Run: `make battery` → exit 0 AND **zero `^warning` lines**.
Run: `make deny` → exit 2 with exactly the one ratified wildcard.
Run: `make dupes` → green.
Run: `make build-poc` → fresh debug binary for both live clients.

- [ ] **Step 2: Live verification**

Owner, in Zed: edit an md file — per-keystroke diagnostics stay correct and responsive; goToDefinition and findReferences intact; F2 on the `rename-demo-a/b.md` pair still rewrites heading + inbound fragments (latency verdict: the original complaint was «досить великі затримки» — this cycle's observable).
Claude Code dogfood: `/reload-plugins`, then definition on `[summary](#top)` and findReferences on `# Top` (regression probes), «метод вже так».

- [ ] **Step 3: Whole-cycle final review**

Dispatch the final reviewer (`opus`, rust-skills + LSP, runs battery/dupes itself) with the whole-cycle package (BASE = the cycle start commit) and the ledger's open-minors list; triage findings; close the cycle. Ledger closes perf items 1+2 of the RESUME re-triage.

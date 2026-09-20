# Perf remediation — derived MdIndex (per-generation memoization)

Date: 2026-09-20 · Status: approved by owner (schema + rust-skills consultation) · Branch: `feature/markdown` · Owner commits (git is read-only for agents)

## Context

Cycles 1–4 shipped hover, push+pull diagnostics, definition, references, and
rename. Every link-model consumer except hover **re-parses the full document on
every request** through the `parse(&self, text)` helper (`server.rs:38`) — the
cost the SDD ledger carried as "documented POC cost" until the owner's upstream
agent escalated it to must-fix (2026-09-15 pause), now the resume point.

Nine `self.parse(...)` call sites (verified 2026-09-20 on the v0.19.0 tree;
the table groups them by function, the shared `parse()` helper is the base):

| Site | Where | Cost today |
|---|---|---|
| `parse()` helper | `server.rs:38` | full `links::build` per call, shared `Arc<Mutex<MarkdownParser>>` |
| `resolver()` `open` closure | `server.rs:54` | parse per target resolution (feeds diagnostics, definition, references, rename) |
| `document_snapshots()` | `server.rs:66` | parse of **every open document** per references/rename request |
| `compute_diagnostics()` | `server.rs:91` | parse per keystroke (`did_change` → publish) + per pull |
| `definition_at()` | `server.rs:114` | parse per request |
| `references_at()` ×2 direct | `server.rs:137,148` | own parse + `resolve_from` closure parse (+ shared snapshots) |
| `rename_prepare_at()` | `server.rs:176` | parse per request |
| `rename_at()` ×2 direct | `server.rs:208,226` | own parse + `resolve_from` closure parse (+ shared snapshots) |

Hover does not parse (framework tree via `node_at_position_named`).

Upstream v0.19.0 ships the remedy: `Document::derived::<T>(compute)`
(`documents/document.rs`, v0.18.0+). Contract facts, read at the pinned
checkout: the value is keyed by `TypeId` and lives for the document's
**generation** — any write (edit, save, disk refresh) starts a fresh generation
and fresh derived data; `compute` runs at most once per generation; the return
is `Arc<T>` (`#[must_use]`); `compute` must not re-enter `derived` for the same
`T` on the same document; fallible derives express themselves through `T`
(`Option<Foo>`). Memoization lives on the document's inner state, shared by
every snapshot of the same generation — repeated `state.document(url)` calls
within one generation hit the same cache.

## Decisions

- **Approach A (approved): one helper, all sites behind it.**
  `index_of(&self, doc: &Document) -> Option<Arc<links::MdIndex>>` on
  `PocLanguageServer`, implemented over
  `doc.derived::<Option<Arc<MdIndex>>>(...)`; every site's
  `self.parse(&doc.text_contents())` becomes `self.index_of(&doc)`;
  `parse(&self, text)` retires. Cache ownership stays upstream — this repo adds
  zero cache code and zero invalidation logic.
  Rejected: **B** per-capability derived types (the `TypeId` key means separate
  parses of the same text per capability); **C** an own
  `HashMap<Url, (version, Arc<MdIndex>)>` on the server (reimplements what
  upstream just shipped and takes on version-tracking bugs).
- **`T = Option<Arc<MdIndex>>`.** `derived` returns `Arc<T>`; the helper hands
  out `(*arc).clone()` — an `Option::clone`, an `Arc` bump when `Some`.
- **A `None` build is pinned for its generation.** `links::build` answers
  `None` only on a tree-sitter timeout/cancellation; the server never calls
  `set_timeout`, so the path is near-impossible, and on deterministic input a
  retry would reproduce the result. Memoizing `None` makes behavior consistent
  within a generation (today: per-request retry of the same failing parse).
  Stated in the helper's doc comment.
- **The parser stays `Arc<Mutex<MarkdownParser>>`**, captured by the derive
  closure. The lock is held only during the synchronous compute — never across
  an `.await` — and the runtime is `current_thread`, so it is uncontended by
  construction. Cost model: once per document generation instead of once per
  request. Rejected: a fresh parser per derive (language reload per
  generation); a `thread_local` parser (no benefit on a single-threaded
  runtime).
- **Seam shapes unchanged.** The `resolver()`/`resolve_from` `open` closures
  swap their inner parse for `index_of`; `document_snapshots()` derives per
  document and keeps its skip (plus debug trace) for a `None` derive. Absence
  semantics everywhere are unchanged: no document, no derive, nothing
  link-worthy → `Ok(None)` / skip.
- **No behavior change is the contract.** Outputs are identical per request
  for identical document states; tracing sites stay (wording updated where
  "parse" becomes "derive"). Zero new `PocError` variants, zero new modules —
  the seam lives in `server.rs`; `arch-lint.toml` untouched.
- **Out of scope:** the multi-root consumer chunk (unblocked by v0.19.0's
  public `workspace_roots()` — the next chunk, its own spec); the hover upgrade
  (cycle 8, will consume this same derived index); log volume (a
  `RUST_LOG=debug` default in debug builds, unrelated to parsing); any
  benchmark harness (POC — the owner's latency observation is the verdict).
- rust-skills consultation (2026-09-20): `own-arc-shared` (the `Arc<MdIndex>`
  handout), `own-mutex-interior` + `conc-lock-reentry` +
  `anti-lock-across-await` (lock discipline around the shared parser),
  `perf-hoist-loop-invariant` (the memoization rationale),
  `type-option-nullable` (`Option` as the fallible-`T` shape),
  `test-observable-coverage` (the behavioral contract is "unchanged output" —
  covered by the existing suite + live ritual, not implementation-shaped
  cache-peeking tests).

## Changes

- `crates/lsp-poc/src/server.rs` — only file touched:
  - `parse()` removed; `index_of(&self, doc: &Document) -> Option<Arc<links::MdIndex>>`
    added (derive closure locks the shared parser, builds via `links::build`
    from `doc.text_contents()`).
  - The nine call sites above route through `index_of` (the two `resolve_from`
    closures included).
  - Import `Document` (`async_language_server::server::Document`, verified
    re-export).
  - `document_snapshots()`'s doc comment drops the "own parse per open doc —
    the established POC cost" sentence.
- Unchanged: `links/`, `workspace/`, `diagnostics/`, `definitions/`,
  `references/`, `renames/`, `info.rs`, `main.rs`, fixtures,
  `arch-lint.toml`, `Cargo.toml`.

## Verification

- Unit: the existing suite (46 tests) green and untouched — the pure modules
  the seam feeds do not change.
- The seam itself admits no unit test: `Document` is constructible only via
  upstream `pub(crate) from_parts`, and upstream's `testing` module is
  `#[cfg(test)] pub(crate)`. Server wiring stays covered the way cycles 1–4
  covered it: the live ritual.
- Gates: `make battery` exit 0 with **zero `^warning` lines**; `make dupes`
  green; `make build-poc` fresh binary.
- Live, both clients: Zed — per-keystroke diagnostics stay correct and
  responsive, definition / references / F2 rename intact on the demo pair;
  Claude Code dogfood — `/reload-plugins`, definition + references regression
  probes, "метод вже так".
- Perf verdict: the owner's latency observation before/after (the original
  complaint: «досить великі затримки»); no benchmark harness.

## Constraints

- Git is read-only for every agent; the owner commits.
- Zero new dependencies; zero new `PocError` variants; no stdout in server
  code; no panics in hooks/handlers; no `.unwrap()`/`.expect()` outside tests.
- Surgical: every file except `crates/lsp-poc/src/server.rs` byte-identical.
- `rust-skills` applies to planning and implementation; LSP-first analysis;
  dispatches carry the standing rules (workers `sonnet`, reviewers
  `opus` with their own battery/dupes runs, findings as actionable
  fix-tasks, no workarounds; model names carry no suffixes — long context
  is decided by global configuration).

# Doc-Fixes Design

Date: 2026-09-12. Branch: `feature/markdown`. Scope: four comment/doc-text
corrections from the steering backlog; no code changes. Execution: one
subagent, atomic batch; the owner commits.

## Context

Each fix removes a statement that is false in this tree, verified 2026-09-12:

1. Root `Cargo.toml` claims `str_to_string`/`cognitive_complexity` "fire
   today" — a full `cargo clippy --workspace --all-targets` run fires zero
   warnings (exit 0). The pair was carried from the fork, where it did fire.
2. `clippy.toml`'s "Current maxima" comment cites
   `requests/conversion.rs` — a fork-only file that does not exist here.
3. `.cargo/config.toml`'s "Uncomment to inspect…" comment sits above an
   active line — the text (including the contradiction) was carried verbatim
   from the fork (git provenance: pre-dates commit 45ca0e7).
4. Spec `2026-09-12-quality-gates-design.md` §"Current-state survey" claims
   the deny `wildcard` came from workspace-dependency inheritance, and
   §Changes 1.3 predicts the direct git declaration "removes the deny
   wildcard at its root". Both falsified during execution: cargo-deny's
   `wildcards` lint flags ANY version-less dependency — a git dep carries no
   version, so the finding is permanent by design. Only the sibling
   `bug[unresolved-workspace-dependency]` came from the inheritance and
   disappeared with the direct declaration.

## Owner decisions (2026-09-12, one per fix)

1. Lint pair: KEEP both entries as `warn` tripwires; fix the comment only.
2. RA line: KEEP active at `"0"`; reword the comment only.
3. `clippy.toml`: DELETE the comment outright (threshold keys stay, bare).
4. Spec: rewrite both spots to the verified truth WITH a dated
   "(Corrected 2026-09-12)" marker — history stays readable, the edit is
   visible.

## Changes

### 1. Root `Cargo.toml` — comment above the lint pair

Replace:

```toml
# Fires today, adopted as explicit work items (see clippy.toml thresholds).
```

with:

```toml
# Carried from the fork as explicit work items; zero fires in this tree so
# far — tripwires only. Promote or drop at a lint review.
```

`str_to_string = "warn"` and `cognitive_complexity = "warn"` unchanged.

### 2. `clippy.toml` — delete the maxima comment

Delete lines 8-10 (the three-line "Complexity thresholds… Current maxima…"
comment, including the `requests/conversion.rs` citation). The keys
`cognitive-complexity-threshold = 15`, `too-many-lines-threshold = 75`,
`too-many-arguments-threshold = 6` stay, with no comment above them.

### 3. `.cargo/config.toml` — reword the RA comment

Replace:

```toml
# Uncomment to inspect exactly which config rust-analyzer applies on startup:
RA_DISABLE_INCREMENTAL_SYNC = "0"
```

with:

```toml
# RA sync debug knob: "0" leaves incremental sync on; flip to "1" to freeze it
# while inspecting which config rust-analyzer applies on startup.
RA_DISABLE_INCREMENTAL_SYNC = "0"
```

### 4. Spec — two truth corrections, both dated

`docs/superpowers/specs/2026-09-12-quality-gates-design.md`:

- Survey paragraph (the `wildcard` bullet): state the real mechanism — the
  `wildcards` lint flags any dependency resolving without a version; a git
  dep carries none, so the finding is permanent by design, not an
  inheritance artifact. The sibling `bug[unresolved-workspace-dependency]`
  fired on the inheritance and disappears with the direct declaration; the
  `wildcard` does not. End with `(Corrected 2026-09-12: the original text
  blamed workspace inheritance.)`
- §Changes 1.3: replace "This removes the deny `wildcard` at its root." with
  the removal of `bug[unresolved-workspace-dependency]` only, plus the
  ratified-red state of the remaining `wildcard` (owner decision 2026-09-12,
  documented in `deny.toml`). Same `(Corrected 2026-09-12)` marker.

## Verification

- `make clippy` green after the edits (comment-only TOML changes must not
  alter lint behavior; the TOMLs must still parse).
- Spec diff limited to the two passages plus markers.
- Success gate: all four texts state only what was verified this cycle; zero
  other changed lines.

## Constraints

- Git is read-only for agents — the owner commits.
- Surgical: no line changed outside the four spots.
- Execution runs in a subagent; the controller verifies by diff.
- Out of scope: retuning thresholds, measuring this tree's maxima, any lint
  set change, `utils.rs` fate, dylint enforcement, `vendors/` gitignore.

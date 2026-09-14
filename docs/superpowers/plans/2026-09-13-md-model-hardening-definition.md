# Cycle 2 — Model Hardening + Definition Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop scanning code regions (fences + code spans), collect setext headings, fix `scan_range` columns on continuation lines, and serve the `definition` capability over the un-trimmed model — per `docs/superpowers/specs/2026-09-13-md-model-hardening-definition-design.md`.

**Architecture:** `src/links/` gains block-level code-range collection with inline-tree filtering and `code_span`-complement scanning, a setext arm, the column fix, and struct-shaped definitions/footnote-definitions with ranges. `src/workspace/`'s `Resolved::Found` starts carrying the target `Url`. New `src/definitions/` maps a cursor position to the model item containing it and resolves it to a `Location`. `server.rs` wires the capability pair and lifts one shared resolver closure.

**Tech Stack:** unchanged from cycle 1 — Rust 2024 stable, `async-language-server` v0.10.0, `tree-sitter-md` 0.5.3, `cargo nextest`, Makefile battery. **Zero new dependencies.**

## Global Constraints

- Git is read-only for every agent: NO `git add`/`commit`/`rm`/`stash` or any git-write command. The owner commits. (No commit steps in this plan.)
- Zero new dependencies; zero new `PocError` variants; no `error.rs`.
- Never print to stdout (LSP transport); `tracing` only, error values carried into warnings.
- Mutex poisoning recovers via `lock().unwrap_or_else(PoisonError::into_inner)`; no `.ok()?` on locks.
- Sync `std::fs`/`Path::exists` sites carry `// arch-lint: allow(no-sync-io) reason="..."`.
- No `.unwrap()`/`.expect()` outside `#[cfg(test)]`; hooks and request handlers must not panic.
- Positions are UTF-8 at the trait boundary; tree-sitter → LSP ranges only via `ts_range_to_lsp_range`.
- Surgical: hover stays byte-identical; `arch-lint.toml` scopes untouched; references/rename/completion untouched (later cycles).
- LESSON FROM CYCLE 1: brief code is untested by construction — if mandated code does not compile or trips a deny, apply the minimal mechanical fix (prefer clippy's own suggestion, behavior identical) and document the deviation; BLOCK only for genuine ambiguity or a mandated anchor not matching the tree.
- Dupes ledger: `.dupes-ignore.toml` entries are fingerprint-keyed; refactoring a flagged unit stales its entry — prune with `cargo dupes cleanup` (dry-run first), never leave rotting entries.
- `rust-skills` rules apply to all Rust work (err-\*, own-\*, pat-let-else, api-must-use, num-cast-try-from, test-cfg-test-module).
- Every task ends with `make fmt-fix && make fmt && make clippy && make test` green ("the task gates").

### Model facts relied on (verified during cycle-1 planning/review)

- Inline trees parse with **absolute document coordinates**; `MarkdownTree` does not expose the inline-tree → parent mapping, so code exclusion must be range-based.
- `setext_heading` nodes carry the same `heading_content` field; underline children are `setext_h1_underline`/`setext_h2_underline`.
- Code spans (`code_span` typed nodes) take precedence over links, so typed link collection needs no span handling — only the off-tree scans do.

---

### Task 1: Code exclusion — fences and code spans stop producing shapes

**Files:**
- Modify: `crates/lsp-poc/src/links/mod.rs` (module doc, `build`, `collect_block`, `collect_inline`, `scan_offtree`, tests)
- Modify: `crates/lsp-poc/tests/fixtures/links-fixture.md` (append decoys)

**Interfaces:**
- Consumes: the current `links::build` pipeline (unchanged signatures).
- Produces: same public surface as today (`build`, accessors, `Target`, parsers) — no consumer changes. Internally: `build` filters inline trees against block code ranges; `scan_offtree` skips `code_span` windows.

- [ ] **Step 1: Append decoys to `crates/lsp-poc/tests/fixtures/links-fixture.md`**

Append at the end of the file (currently ends with `[ref]: https://example.com` + newline) exactly this block — a blank line, a code-span decoy line, a blank line, and a fenced block:

```markdown

Span `[[SpanDecoy]]` and `[^77]` here.

```
[a](decoy.md) and [[WikiDecoy]] and [^99]
```
```

The last four lines above are: a line with inline code spans, a blank line, an opening ` ``` ` fence, the decoy content line, a closing ` ``` ` fence. Byte-exact matters for later range assertions only for lines ABOVE this block — the existing tests' indices stay valid because nothing before this point changes.

- [ ] **Step 2: Write the failing test**

In `crates/lsp-poc/src/links/mod.rs` `mod tests`, add:

```rust
    #[test]
    fn code_regions_produce_no_shapes() {
        let index = fixture();
        // Only the real links/wikilinks/footnotes — no decoy.md, WikiDecoy,
        // SpanDecoy, ^99, or ^77.
        assert_eq!(index.links.len(), 2);
        assert!(!index
            .links
            .iter()
            .any(|link| link.destination == "decoy.md"));
        let wikis: Vec<&str> = index.wikilinks.iter().map(|w| w.target.as_str()).collect();
        assert!(!wikis.contains(&"WikiDecoy"));
        assert!(!wikis.contains(&"SpanDecoy"));
        let footnotes: Vec<&str> = index
            .footnote_references
            .iter()
            .map(|f| f.id.as_str())
            .collect();
        assert!(!footnotes.contains(&"99"));
        assert!(!footnotes.contains(&"77"));
    }
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo nextest run -p lsp-poc links::` (the `tests::` path segment makes bare `links::code_regions` match nothing)
Expected: FAIL — **only the code-span decoys leak on the pinned rev**: `SpanDecoy` among the wikilinks and `77` among the footnotes. The fence decoys (`decoy.md`, `WikiDecoy`, `^99`) do NOT reproduce — tree-sitter-md already emits no inline trees for fenced blocks, so the fence filter this task adds is no-op defense for rev bumps. (Corrected 2026-09-13 during execution: the brief's original red expectation assumed fence leakage; the live cycle-1 dogfood noise on the plan document traced to nested-fence misparsing — inner ``` closing the outer markdown fence — not to fence inline trees.)

- [ ] **Step 4: Implement the exclusion**

4a. Replace the module-doc limitation sentence (keep the "ATX only" sentence — Task 2 replaces it when setext lands):

old:
```rust
//! link-reference-definitions (with a paragraph fallback), references and
//! wikilinks are byte scans over inline text. Known limitation: those
//! scans also match shapes inside code spans — the grammars give no
//! cheaper boundary; refine when a cycle needs it. Headings are ATX
//! (`#`) only; setext underlines are not collected — cycle 2.
```
new:
```rust
//! link-reference-definitions (with a paragraph fallback), references and
//! wikilinks are byte scans over inline text. Code regions are excluded:
//! fenced and indented code blocks suppress their inline trees entirely,
//! and off-tree scans skip `code_span` windows — example links in
//! documentation produce no shapes. Headings are ATX
//! (`#`) only; setext underlines are not collected — cycle 2.
```

4b. `build` — collect block code ranges, filter inline trees, thread span windows:

old:
```rust
pub fn build(parser: &mut MarkdownParser, text: &str) -> Option<MdIndex> {
    let tree = parser.parse(text.as_bytes(), None)?;
    let mut index = MdIndex::default();
    collect_block(tree.block_tree().root_node(), text, &mut index);
    for inline in tree.inline_trees() {
        collect_inline(inline.root_node(), text, &mut index);
        scan_offtree(inline.root_node(), text, &mut index);
    }
    Some(index)
}
```
new:
```rust
pub fn build(parser: &mut MarkdownParser, text: &str) -> Option<MdIndex> {
    let tree = parser.parse(text.as_bytes(), None)?;
    let mut index = MdIndex::default();
    let mut code = Vec::new();
    collect_block(tree.block_tree().root_node(), text, &mut index, &mut code);
    for inline in tree.inline_trees() {
        let root = inline.root_node();
        if code.iter().any(|range| covers(*range, root.range())) {
            continue;
        }
        let mut spans = Vec::new();
        collect_inline(root, text, &mut index, &mut spans);
        scan_offtree(root, text, &mut index, &spans);
    }
    Some(index)
}

/// `true` when `inner` lies fully inside `outer` (byte comparison — both
/// ranges come from the same parse of the same text).
fn covers(outer: Range, inner: Range) -> bool {
    inner.start_byte >= outer.start_byte && inner.end_byte <= outer.end_byte
}
```

4c. `collect_block` — new parameter, collect the two code-block kinds:

old:
```rust
fn collect_block(node: Node, text: &str, index: &mut MdIndex) {
    match node.kind() {
        "atx_heading" => collect_heading(node, text, index),
        "link_reference_definition" => collect_definition(node, text, index),
        "paragraph" => collect_footnote_definition_from_paragraph(node, text, index),
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_block(child, text, index);
    }
}
```
new:
```rust
fn collect_block(node: Node, text: &str, index: &mut MdIndex, code: &mut Vec<Range>) {
    match node.kind() {
        "atx_heading" => collect_heading(node, text, index),
        "link_reference_definition" => collect_definition(node, text, index),
        "paragraph" => collect_footnote_definition_from_paragraph(node, text, index),
        "fenced_code_block" | "indented_code_block" => code.push(node.range()),
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_block(child, text, index, code);
    }
}
```

4d. `collect_inline` — new parameter, collect `code_span` windows:

old: `fn collect_inline(root: Node, text: &str, index: &mut MdIndex) {`
new: `fn collect_inline(root: Node, text: &str, index: &mut MdIndex, spans: &mut Vec<Range>) {`

and inside the walk's match, add one arm before the catch-all (replacing the old catch-all comment block):

old:
```rust
        // Images are not diagnosed in cycle 1 (the reference skips them too),
        // and nothing else in the inline grammar is collected either.
        _ => {}
```
new:
```rust
        "code_span" => spans.push(node.range()),
        // Images are not diagnosed in cycle 1 (the reference skips them too),
        // and nothing else in the inline grammar is collected either.
        _ => {}
```

4e. `scan_offtree` — run the scans only over the complement of the span windows.
**(Corrected 2026-09-13 during execution: as written below, `complement` mixed
document-absolute span bytes with root-relative `source` indexing and panicked. The
landed fix converts spans to root-relative pairs before the complement;
`complement(&[(usize, usize)], len)`; the shape below otherwise holds.)**

old:
```rust
fn scan_offtree(root: Node, text: &str, index: &mut MdIndex) {
    let Ok(source) = root.utf8_text(text.as_bytes()) else {
        return;
    };
    let base = (root.start_byte(), root.start_position());
    scan_footnote_references(source, base, index);
    scan_wikilinks(source, base, index);
}
```
new:
```rust
fn scan_offtree(root: Node, text: &str, index: &mut MdIndex, spans: &[Range]) {
    let Ok(source) = root.utf8_text(text.as_bytes()) else {
        return;
    };
    let root_base = (root.start_byte(), root.start_position());
    for (start, end) in complement(spans, source.len()) {
        let slice = &source[start..end];
        let before = &source[..start];
        let rows = before.matches('\n').count();
        let line_start = before.rfind('\n').map_or(0, |i| i + 1);
        let column = if rows == 0 {
            root_base.1.column + start
        } else {
            start - line_start
        };
        let base = (root_base.0 + start, Point { row: root_base.1.row + rows, column });
        scan_footnote_references(slice, base, index);
        scan_wikilinks(slice, base, index);
    }
}

/// Maximal source windows not covered by any span, in document order.
fn complement(spans: &[Range], len: usize) -> Vec<(usize, usize)> {
    let mut edges: Vec<(usize, usize)> = spans
        .iter()
        .map(|range| (range.start_byte, range.end_byte))
        .collect();
    edges.sort_unstable();
    let mut segments = Vec::new();
    let mut at = 0;
    for (start, end) in edges {
        if start > at {
            segments.push((at, start));
        }
        at = at.max(end);
    }
    if at < len {
        segments.push((at, len));
    }
    segments
}
```

(The segment `column` formula already uses the fixed continuation-line rule; Task 2
applies the same rule inside `scan_range` itself.)

- [ ] **Step 5: Run the links tests**

Run: `cargo nextest run -p lsp-poc links::`
Expected: 7 passed (6 existing + the new one). If the fenced decoys still leak, check that the fence's inline tree root is inside the collected `fenced_code_block` range (byte containment) before touching anything else.

- [ ] **Step 6: Task gates**

Run: `make fmt-fix && make fmt && make clippy && make test`
Expected: exit 0 across all four.

---

### Task 2: Setext headings + scan_range continuation-line columns

**Files:**
- Modify: `crates/lsp-poc/src/links/mod.rs` (`collect_block`, `collect_heading`, new `collect_setext_heading` + `setext_level`, `scan_range`, tests)
- Modify: `crates/lsp-poc/tests/fixtures/links-fixture.md` (append setext + lazy-continuation block)

**Interfaces:**
- Consumes: Task 1's pipeline (signatures unchanged).
- Produces: setext headings collected into `headings` (slug + level, same struct shape as ATX for now — ranges land in Task 3); `scan_range` with correct columns on any line of the inline node.

- [ ] **Step 1: Append the setext + lazy-continuation block to the fixture**

Append at the end (after the fenced decoy block from Task 1) exactly:

```markdown

Setext Title
============

> quoted first line
lazy continuation [[LazyWiki]] here
```

(Blank line, `Setext Title`, `============` underline, blank line, `> quoted first line`, `lazy continuation [[LazyWiki]] here`. The blockquote's second line is a lazy continuation — no `> ` prefix — which is the exact case the old column formula got wrong.)

- [ ] **Step 2: Write the failing tests**

In `mod tests`, add:

```rust
    #[test]
    fn setext_headings_are_collected() {
        let index = fixture();
        assert_eq!(index.headings.len(), 2);
        let setext = index
            .headings
            .iter()
            .find(|heading| heading.slug == "setext-title")
            .expect("setext heading collected");
        assert_eq!(setext.level, 1);
        assert!(index.has_heading_slug("setext-title"));
    }

    #[test]
    fn scanned_points_match_their_source_lines() {
        let text = fixture_text();
        let index = fixture();
        let lazy = index
            .wikilinks
            .iter()
            .find(|wikilink| wikilink.target == "LazyWiki")
            .expect("lazy-continuation wikilink collected");
        let slice = &text[lazy.range.start_byte..lazy.range.end_byte];
        assert_eq!(slice, "[[LazyWiki]]");
        let line_start = text[..lazy.range.start_byte]
            .rfind('\n')
            .map_or(0, |i| i + 1);
        assert_eq!(
            lazy.range.start_point.row,
            text[..lazy.range.start_byte].matches('\n').count(),
        );
        assert_eq!(
            lazy.range.start_point.column,
            lazy.range.start_byte - line_start,
            "continuation-line column is the offset within its own line",
        );
    }
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo nextest run -p lsp-poc links::setext + cargo nextest run -p lsp-poc links::scanned_points`
Run both filters: `cargo nextest run -p lsp-poc links::` and inspect.
Expected: `setext_headings_are_collected` FAILS (headings.len() == 1, no setext slug). `scanned_points_match_their_source_lines` FAILS on the column assertion (the current formula adds the node's base column to a continuation-line offset).

- [ ] **Step 4: Implement**

4a. Setext collection — `collect_block` match gains one arm and the two collectors:

old (the `atx_heading` arm):
```rust
        "atx_heading" => collect_heading(node, text, index),
```
new:
```rust
        "atx_heading" => collect_heading(node, text, index),
        "setext_heading" => collect_setext_heading(node, text, index),
```

and below `collect_heading` + `heading_level` add:

```rust
/// A setext heading: paragraph text underlined by `===` or `---`. Same
/// `heading_content` field as ATX; the level comes from the underline.
fn collect_setext_heading(heading: Node, text: &str, index: &mut MdIndex) {
    let Some(content) = heading.child_by_field_name("heading_content") else {
        return;
    };
    let mut cursor = heading.walk();
    let Some(underline) = heading
        .children(&mut cursor)
        .find(|child| child.kind().starts_with("setext_h"))
    else {
        return;
    };
    let Some(level) = setext_level(&underline) else {
        return;
    };
    let Ok(raw) = content.utf8_text(text.as_bytes()) else {
        return;
    };
    index.headings.push(Heading {
        slug: slugify(raw),
        level,
    });
}

fn setext_level(underline: &Node) -> Option<u8> {
    underline
        .kind()
        .strip_prefix("setext_h")?
        .strip_suffix("_underline")?
        .parse()
        .ok()
}
```

4b. `scan_range` — the continuation-line column drops the base column:

old:
```rust
    let before = &source[..start];
    let rows = before.matches('\n').count();
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    let start_point = Point {
        row: base.1.row + rows,
        column: base.1.column + (start - line_start),
    };
```
new:
```rust
    let before = &source[..start];
    let rows = before.matches('\n').count();
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    let column = if rows == 0 {
        base.1.column + start
    } else {
        // A continuation line's prefix width is not the node's start
        // column (lazy continuations carry none) — the column is the
        // offset within its own line.
        start - line_start
    };
    let start_point = Point {
        row: base.1.row + rows,
        column,
    };
```

4c. Module doc — the "ATX only" sentence (kept by Task 1) is now false:

old:
```rust
//! documentation produce no shapes. Headings are ATX
//! (`#`) only; setext underlines are not collected — cycle 2.
```
new:
```rust
//! documentation produce no shapes. Headings come in both ATX (`#`) and
//! setext (underlined) forms.
```

- [ ] **Step 5: Run the links tests**

Run: `cargo nextest run -p lsp-poc links::`
Expected: 9 passed (7 + 2 new; `fixture_collects_headings_definitions_and_links` still asserts `headings.len() == 1` — UPDATE it to `2` in this step; its `headings[0].slug == "top"` assertion stays valid because the ATX heading precedes the setext one in document order).

- [ ] **Step 6: Task gates**

Run: `make fmt-fix && make fmt && make clippy && make test`
Expected: exit 0 across all four.

---

### Task 3: Un-trimmed model + `Resolved` carries the target URL + `src/definitions/`

**Files:**
- Modify: `crates/lsp-poc/src/links/mod.rs` (struct shapes, collectors, accessors, `has_*` bodies)
- Modify: `crates/lsp-poc/src/workspace/mod.rs` (`Resolved::Found` gains `url`; `load` threads it)
- Modify: `crates/lsp-poc/src/diagnostics/mod.rs` (match arms on the new `Found` shape)
- Create: `crates/lsp-poc/src/definitions/mod.rs`
- Modify: `crates/lsp-poc/src/main.rs` (`mod definitions;` after `mod diagnostics;`)

**Interfaces:**
- Consumes: Tasks 1–2 pipeline; `ts_range_contains_lsp_position` + `ts_range_to_lsp_range` from `async_language_server::tree_sitter_utils`.
- Produces (Task 4 relies on these exact names):
  - links: `Heading { slug: String, level: u8, range: Range }`, `Definition { label: String, destination: String, range: Range }`, `FootnoteDefinition { id: String, range: Range }`; accessors `headings(&self) -> &[Heading]`, `definitions(&self) -> &[Definition]`, `footnote_definitions(&self) -> &[FootnoteDefinition]` (new), plus the existing `links/references/footnote_references/wikilinks`; `has_heading_slug/has_definition/has_footnote_definition` keep their signatures.
  - workspace: `Resolved::Found { url: Url, index: Arc<links::MdIndex> }` (fields public), `Resolved::Missing` unchanged.
  - definitions: `pub fn at_position(index: &links::MdIndex, self_url: &Url, position: LspPosition, resolve: &dyn Fn(&Target) -> Option<Resolved>) -> Option<GotoDefinitionResponse>` — `None` when nothing definition-worthy is under the position or the target does not resolve.

- [ ] **Step 1: Un-trim the model shapes in `src/links/mod.rs`**

Structs:

old:
```rust
/// A heading, reduced to what matching needs: its anchor slug and depth.
#[derive(Debug)]
pub struct Heading {
    pub slug: String,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "heading depth is consumed from cycle 2 (symbols/ToC) onward"
        )
    )]
    pub level: u8,
}
```
new:
```rust
/// A heading: its anchor slug, depth, and document range (first match by
/// slug wins for definition routing).
#[derive(Debug)]
pub struct Heading {
    pub slug: String,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "heading depth is consumed from cycle 2 (symbols/ToC) onward"
        )
    )]
    pub level: u8,
    pub range: Range,
}

/// A link reference definition: `[label]: destination`.
#[derive(Debug)]
pub struct Definition {
    pub label: String,
    pub destination: String,
    pub range: Range,
}

/// A footnote definition `[^id]: text`.
#[derive(Debug)]
pub struct FootnoteDefinition {
    pub id: String,
    pub range: Range,
}
```

`MdIndex` fields change type:

old:
```rust
    headings: Vec<Heading>,
    definitions: Vec<String>,
```
new:
```rust
    headings: Vec<Heading>,
    definitions: Vec<Definition>,
```
old:
```rust
    footnote_references: Vec<FootnoteReference>,
    footnote_definitions: Vec<String>,
```
new:
```rust
    footnote_references: Vec<FootnoteReference>,
    footnote_definitions: Vec<FootnoteDefinition>,
```

Accessors — extend the `impl MdIndex` block (keep `has_*` signatures):

```rust
    #[must_use]
    pub fn headings(&self) -> &[Heading] {
        &self.headings
    }

    #[must_use]
    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    #[must_use]
    pub fn footnote_definitions(&self) -> &[FootnoteDefinition] {
        &self.footnote_definitions
    }
```

`has_*` bodies (bodies diverge — the `.dupes-ignore.toml` group dissolves):

old:
```rust
    #[must_use]
    pub fn has_definition(&self, label: &str) -> bool {
        self.definitions.iter().any(|known| known == label)
    }

    #[must_use]
    pub fn has_footnote_definition(&self, id: &str) -> bool {
        self.footnote_definitions.iter().any(|known| known == id)
    }
```
new:
```rust
    #[must_use]
    pub fn has_definition(&self, label: &str) -> bool {
        self.definitions.iter().any(|known| known.label == label)
    }

    #[must_use]
    pub fn has_footnote_definition(&self, id: &str) -> bool {
        self.footnote_definitions.iter().any(|known| known.id == id)
    }
```

Collectors — push the new fields. `collect_heading`'s push becomes:

old:
```rust
    index.headings.push(Heading {
        slug: slugify(raw),
        level,
    });
```
new:
```rust
    index.headings.push(Heading {
        slug: slugify(raw),
        level,
        range: heading.range(),
    });
```

`collect_setext_heading`'s push becomes:

old:
```rust
    index.headings.push(Heading {
        slug: slugify(raw),
        level,
    });
```
(inside `collect_setext_heading`)
new:
```rust
    index.headings.push(Heading {
        slug: slugify(raw),
        level,
        range: heading.range(),
    });
```

`collect_definition` — find the destination child and push the struct:

old:
```rust
fn collect_definition(definition: Node, text: &str, index: &mut MdIndex) {
    let mut cursor = definition.walk();
    let Some(label_node) = definition
        .children(&mut cursor)
        .find(|child| child.kind() == "link_label")
    else {
        return;
    };
    let Some(raw) = bracketed_text(label_node, text) else {
        return;
    };
    let label = normalize_label(&raw);
    if let Some(id) = label.strip_prefix('^') {
        if !id.is_empty() {
            index.footnote_definitions.push(id.to_owned());
        }
        return;
    }
    index.definitions.push(label);
}
```
new:
```rust
fn collect_definition(definition: Node, text: &str, index: &mut MdIndex) {
    let mut cursor = definition.walk();
    let Some(label_node) = definition
        .children(&mut cursor)
        .find(|child| child.kind() == "link_label")
    else {
        return;
    };
    let Some(raw) = bracketed_text(label_node, text) else {
        return;
    };
    let label = normalize_label(&raw);
    let destination = definition
        .children(&mut cursor)
        .find(|child| child.kind() == "link_destination")
        .and_then(|child| clean_destination(child, text));
    if let Some(id) = label.strip_prefix('^') {
        if !id.is_empty() {
            index.footnote_definitions.push(FootnoteDefinition {
                id: id.to_owned(),
                range: definition.range(),
            });
        }
        return;
    }
    index.definitions.push(Definition {
        label,
        destination: destination.unwrap_or_default(),
        range: definition.range(),
    });
}
```

`collect_footnote_definition_from_paragraph`'s push becomes:

old:
```rust
    index.footnote_definitions.push(normalize_label(id));
```
new:
```rust
    index.footnote_definitions.push(FootnoteDefinition {
        id: normalize_label(id),
        range: paragraph.range(),
    });
```

- [ ] **Step 2: `Resolved::Found` carries the target URL (`src/workspace/mod.rs`)**

old:
```rust
/// What a resolved target turned out to be.
#[derive(Debug)]
pub enum Resolved {
    /// The target file exists (or is open); its index is attached.
    Found(Arc<links::MdIndex>),
    /// No candidate path exists.
    Missing,
}
```
new:
```rust
/// What a resolved target turned out to be.
#[derive(Debug)]
pub enum Resolved {
    /// The target file exists (or is open); its URL and index are attached.
    Found {
        url: Url,
        index: Arc<links::MdIndex>,
    },
    /// No candidate path exists.
    Missing,
}
```

`load`'s two hit arms become:

old:
```rust
            if let Some(index) = open(&url) {
                return Resolved::Found(index);
            }
            if let Some(index) = self.load_from_disk(&url, path) {
                return Resolved::Found(index);
            }
```
new:
```rust
            if let Some(index) = open(&url) {
                return Resolved::Found {
                    url: url.clone(),
                    index,
                };
            }
            if let Some(index) = self.load_from_disk(&url, path) {
                return Resolved::Found {
                    url: url.clone(),
                    index,
                };
            }
```

- [ ] **Step 3: Update the diagnostics match arms (`src/diagnostics/mod.rs`)**

In the four `check_*` helpers (or wherever the code stands after the cycle-1 split),
every `Some(Resolved::Found(target_index))` pattern becomes
`Some(Resolved::Found { index: target_index, .. })` — three sites: the `Doc` arm and
the `Wiki` arm in the links loop, and the single arm in the wikilinks loop. The
`&target_index` argument at the `check_target_heading` call sites becomes `index`
(passing the struct field directly):

old (each site, modulo the loop's range/message):
```rust
                Some(Resolved::Found(target_index)) => {
```
new:
```rust
                Some(Resolved::Found { index: target_index, .. }) => {
```

The `check_target_heading` signature stays `target_index: &links::MdIndex` — the
destructured `index` field is already an `Arc<links::MdIndex>`, which derefs to
`&links::MdIndex` at the call site as `&target_index` today; after destructuring pass
`&target_index` unchanged (deref through the Arc is unchanged).

- [ ] **Step 4: Write the failing definition tests (`src/definitions/mod.rs`)**

Create `crates/lsp-poc/src/definitions/mod.rs` with module doc, imports, and this test
module (implementation in Step 6):

```rust
//! The `textDocument/definition` capability: what is under the cursor,
//! and where does it point.
//!
//! The position is matched against the requesting document's model items
//! (all ranges are document-absolute), then the item's target resolves to
//! a `Location`. External or unresolved targets answer `None` — absence
//! is not an error.

use async_language_server::lsp_types::{
    GotoDefinitionResponse, Location, Position as LspPosition, Url,
};
use async_language_server::tree_sitter_utils::{ts_range_contains_lsp_position, ts_range_to_lsp_range};

use crate::links::{self, Target};
use crate::workspace::Resolved;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tree_sitter_md::MarkdownParser;

    use crate::links::MdIndex;

    fn parse(text: &str) -> MdIndex {
        let mut parser = MarkdownParser::default();
        links::build(&mut parser, text).expect("input parses")
    }

    fn location_of(index: &MdIndex, url: &Url, line: u32, character: u32) -> Option<Location> {
        let resolve = |_: &Target| {
            Some(Resolved::Found {
                url: Url::parse("file:///target.md").expect("url parses"),
                index: Arc::new(parse("# Target\n")),
            })
        };
        match at_position(index, url, LspPosition { line, character }, &resolve) {
            Some(GotoDefinitionResponse::Scalar(location)) => Some(location),
            Some(other) => panic!("unexpected response shape: {other:?}"),
            None => None,
        }
    }

    #[test]
    fn fragment_link_routes_to_the_own_document_heading() {
        let url = Url::parse("file:///doc.md").expect("url parses");
        let index = parse("# Top\n\n[summary](#top)\n");
        // Cursor inside `[summary](#top)` — the link's range covers it.
        let location = location_of(&index, &url, 2, 3).expect("definition resolves");
        assert_eq!(location.uri, url);
        // The `# Top` heading's range starts at the document's first byte.
        assert_eq!(location.range.start.line, 0);
        assert_eq!(location.range.start.character, 0);
    }

    #[test]
    fn cross_file_link_routes_to_the_target_document() {
        let url = Url::parse("file:///doc.md").expect("url parses");
        let index = parse("file [f](other.md#section)\n");
        let location = location_of(&index, &url, 0, 8).expect("definition resolves");
        assert_eq!(
            location.uri,
            Url::parse("file:///target.md").expect("url parses")
        );
        assert_eq!(location.range.start.line, 0);
    }

    #[test]
    fn reference_and_footnote_routes_land_on_their_definitions() {
        let url = Url::parse("file:///doc.md").expect("url parses");
        let index = parse("[label][ref]\n\n[ref]: https://example.com\n\nfoot[^1]\n\n[^1]: text\n");
        let reference = location_of(&index, &url, 0, 2).expect("reference resolves");
        assert_eq!(reference.uri, url);
        let footnote = location_of(&index, &url, 4, 2).expect("footnote resolves");
        assert_eq!(footnote.uri, url);
        // Both land on their definitions (ranges non-degenerate).
        assert!(reference.range.end.character > reference.range.start.character);
        assert!(footnote.range.end.character > footnote.range.start.character);
    }

    #[test]
    fn unresolved_targets_answer_none() {
        let url = Url::parse("file:///doc.md").expect("url parses");
        let index = parse("[f](nowhere.md)\n");
        let resolve = |_: &Target| Some(Resolved::Missing);
        assert!(
            at_position(
                &index,
                &url,
                LspPosition { line: 0, character: 3 },
                &resolve,
            )
            .is_none(),
        );
        // Plain text under the cursor is not definition-worthy either.
        let no_resolve = |_: &Target| -> Option<Resolved> { None };
        assert!(
            at_position(
                &index,
                &url,
                LspPosition { line: 5, character: 0 },
                &no_resolve,
            )
            .is_none(),
        );
    }
}
```

- [ ] **Step 5: Add `mod definitions;` and run the tests to verify they fail**

In `crates/lsp-poc/src/main.rs`, after `mod diagnostics;` add `mod definitions;`
(alphabetical). Run: `cargo nextest run -p lsp-poc definitions::`
Expected: compile error — `at_position` not defined.

- [ ] **Step 6: Implement `at_position`**

Add below the imports (before `#[cfg(test)]`):

```rust
/// Answers the definition target for the item under `position`, or `None`
/// when nothing definition-worthy is under it or the target does not
/// resolve. First matching item in collection order wins.
pub fn at_position(
    index: &links::MdIndex,
    self_url: &Url,
    position: LspPosition,
    resolve: &dyn Fn(&Target) -> Option<Resolved>,
) -> Option<GotoDefinitionResponse> {
    let at = |range: async_language_server::tree_sitter::Range| {
        ts_range_contains_lsp_position(range, position)
    };

    if let Some(link) = index.links().iter().find(|link| at(link.range)) {
        let target = parse_destination(&link.destination)?;
        return route(index, self_url, &target, resolve);
    }
    if let Some(reference) = index.references().iter().find(|r| at(r.range)) {
        let definition = index
            .definitions()
            .iter()
            .find(|definition| definition.label == reference.label)?;
        return Some(scalar(self_url.clone(), definition.range));
    }
    if let Some(footnote) = index
        .footnote_references()
        .iter()
        .find(|footnote| at(footnote.range))
    {
        let definition = index
            .footnote_definitions()
            .iter()
            .find(|definition| definition.id == footnote.id)?;
        return Some(scalar(self_url.clone(), definition.range));
    }
    if let Some(wikilink) = index.wikilinks().iter().find(|w| at(w.range)) {
        let target = parse_wiki_target(&wikilink.target)?;
        let Target::Wiki { .. } = &target else {
            return None;
        };
        return route(index, self_url, &target, resolve);
    }
    None
}

/// Routes a parsed target to its `Location`: fragments land on the own
/// document's first matching heading, file targets on the resolved
/// document's heading (or the file start when no fragment is given).
fn route(
    index: &links::MdIndex,
    self_url: &Url,
    target: &Target,
    resolve: &dyn Fn(&Target) -> Option<Resolved>,
) -> Option<GotoDefinitionResponse> {
    match target {
        Target::Fragment(fragment) => {
            let heading = heading_by_slug(index, &slugify(fragment))?;
            Some(scalar(self_url, heading.range))
        }
        Target::Doc { fragment, .. } | Target::Wiki { fragment, .. } => {
            let Some(Resolved::Found { url, index: target_index }) = resolve(target) else {
                return None;
            };
            let range = match fragment {
                Some(fragment) => {
                    heading_by_slug(&target_index, &slugify(fragment))?.range
                }
                None => target_index.headings().first().map_or_else(
                    || ts_range_to_lsp_range(async_language_server::tree_sitter::Range {
                        start_byte: 0,
                        end_byte: 0,
                        start_point: async_language_server::tree_sitter::Point { row: 0, column: 0 },
                        end_point: async_language_server::tree_sitter::Point { row: 0, column: 0 },
                    }),
                    |heading| ts_range_to_lsp_range(heading.range),
                ),
            };
            Some(scalar(&url, range))
        }
    }
}

/// First heading whose slug matches, in document order.
fn heading_by_slug(index: &links::MdIndex, slug: &str) -> Option<&links::Heading> {
    index.headings().iter().find(|heading| heading.slug == slug)
}

fn scalar(url: &Url, range: async_language_server::tree_sitter::Range) -> GotoDefinitionResponse {
    GotoDefinitionResponse::Scalar(Location {
        uri: url.clone(),
        range: ts_range_to_lsp_range(range),
    })
}
```

Imports must also bring in `slugify`/`parse_destination`/`parse_wiki_target`: use
`use crate::links::{self, parse_destination, parse_wiki_target, slugify, Target};`.

**Note on the fragment-less file target:** a zero range is built inline; if the
`Range { .. }` literal reads heavy, a module-level `const ZERO: …` is not possible
(non-const types) — instead extract a tiny `fn zero_range() ->
async_language_server::tree_sitter::Range` next to `scalar` and use it in the
`None` arm of the `match fragment`. Either form is acceptable; keep one.

- [ ] **Step 7: Run the definition tests**

Run: `cargo nextest run -p lsp-poc definitions::`
Expected: 4 passed.

- [ ] **Step 8: Run the whole crate and the task gates**

Run: `cargo nextest run -p lsp-poc && make fmt-fix && make fmt && make clippy && make test`
Expected: all tests pass (diagnostics suite included — the `Found { .. }` destructuring
compiles), gates exit 0.

---

### Task 4: Server wiring — `definition_provider` + shared resolver + dupes prune

**Files:**
- Modify: `crates/lsp-poc/src/server.rs` (capability, `definition` method, resolver lift)
- Modify: `crates/lsp-poc/.dupes-ignore.toml` (prune the dissolved fingerprint via `cargo dupes cleanup`)

**Interfaces:**
- Consumes: `definitions::at_position`, `workspace::Resolved { url, .. }`, existing `parse`/`compute_diagnostics`.
- Produces: `textDocument/definition` served; one resolver construction shared by diagnostics and definitions.

- [ ] **Step 1: Lift the resolver into a private helper**

In `crates/lsp-poc/src/server.rs`, add below `parse`:

```rust
    /// The cross-file resolver shared by diagnostics and definitions:
    /// open documents win over the disk, exactly as in cycle 1.
    fn resolver<'a>(
        &'a self,
        state: &'a ServerState,
        url: &'a Url,
    ) -> impl Fn(&Target) -> Option<Resolved> + 'a {
        let open = move |open_url: &Url| {
            state
                .document(open_url)
                .and_then(|open_doc| self.parse(&open_doc.text_contents()))
        };
        move |target: &Target| self.files.resolve(&open, url, target)
    }
```

and rewrite `compute_diagnostics`'s tail to use it:

old:
```rust
        let open = |open_url: &Url| {
            state
                .document(open_url)
                .and_then(|open_doc| self.parse(&open_doc.text_contents()))
        };
        diagnostics::compute(&index, &|target: &Target| {
            self.files.resolve(&open, url, target)
        })
```
new:
```rust
        diagnostics::compute(&index, &self.resolver(state, url))
```

The `Target` import may become unused in `compute_diagnostics`'s sight but is still
used by `resolver` — keep `use crate::links::{self, Target};` as is. The
`use crate::workspace::Index;` import gains `Resolved`:
`use crate::workspace::{Index, Resolved};`.

- [ ] **Step 2: Capability + method**

In `server_capabilities`, after the `diagnostic_provider` field add:

```rust
            definition_provider: Some(OneOf::Left(true)),
```

and extend the `lsp_types` import list with `GotoDefinitionParams`,
`GotoDefinitionResponse`, `OneOf` (keep alphabetical order rustfmt enforces).

In the `impl Server for PocLanguageServer` block, next to `document_diagnostics`, add:

```rust
    fn definition(
        &self,
        state: ServerState,
        params: GotoDefinitionParams,
    ) -> impl Future<Output = ServerResult<Option<GotoDefinitionResponse>>> + Send {
        let result = definition_for(self, &state, &params);
        ready(result)
    }
```

and below the trait impl (next to `hover`), the free function:

```rust
fn definition_for(
    server: &PocLanguageServer,
    state: &ServerState,
    params: &GotoDefinitionParams,
) -> ServerResult<Option<GotoDefinitionResponse>> {
    let url = params.text_document_position_params.text_document.uri;
    let position = params.text_document_position_params.position;

    let Some(doc) = state.document(&url) else {
        return Ok(None);
    };
    let text = doc.text_contents();
    let Some(index) = server.parse(&text) else {
        tracing::warn!("markdown parse produced no tree; skipping definition for {url}");
        return Ok(None);
    };

    Ok(definitions::at_position(
        &index,
        &url,
        position,
        &server.resolver(state, &url),
    ))
}
```

with `use crate::definitions;` added to the imports.

- [ ] **Step 3: Prune the dissolved dupes entry**

Run: `cargo dupes cleanup --dry-run`
Expected: reports the `4bdf42868ea34aef` entry (has_definition/has_footnote_definition
group) as stale — the bodies now differ. Then run `cargo dupes cleanup` and show
`.dupes-ignore.toml` diff: only that entry removed. (The hook-pair and test-closure
entries must survive.)

- [ ] **Step 4: Task gates**

Run: `make fmt-fix && make fmt && make clippy && make test && make dupes`
Expected: exit 0 across all five (dupes green with two remaining entries). There is no
unit-test leg for the wiring (ServerState is framework-`#[cfg(test)]` only) — live
verification is Task 5.

---

### Task 5: Battery, gates, live verification on both clients

**Files:**
- No file changes (verification; controller-run like cycle 1's Task 6).

- [ ] **Step 1: Battery + deny**

Run: `make battery`
Expected: exit 0 (fmt, clippy, doc, test, dylint — dylint findings stay advisory).
Run: `make deny`
Expected: exit 2, exactly the one ratified `wildcard` error, no new findings.

- [ ] **Step 2: Changed-set report**

Run: `git status --porcelain && git diff --stat`
Expected: modified `crates/lsp-poc/src/links/mod.rs`, `crates/lsp-poc/src/workspace/mod.rs`,
`crates/lsp-poc/src/diagnostics/mod.rs`, `crates/lsp-poc/src/server.rs`,
`crates/lsp-poc/src/main.rs`, `crates/lsp-poc/.dupes-ignore.toml`,
`crates/lsp-poc/tests/fixtures/links-fixture.md`; created
`crates/lsp-poc/src/definitions/mod.rs` — 8 paths, no `Cargo.lock` delta.

- [ ] **Step 3: Hand off to the owner — live verification on both clients**

Owner: `cargo build`, commit; in Zed open a `.md`, put the cursor on a link, press F12 —
the editor jumps to the target heading/file. Then `/reload-plugins` and Claude Code
dogfood: `goToDefinition` on the fixture link must return the target location
("метод вже так"), and the cycle-1 plan document's fenced-example false positives
(codes 5/6 on lines with `[[Other]]`/`[^1]` inside code) must be gone on the next
diagnostics pass.

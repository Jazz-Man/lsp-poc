# Product

## What This Is

lsp-poc is a personal, experimental JSON Language Server Protocol server for the Zed editor — a proof of concept. The owner's own framing: «Цей проект це мій тестовий JSON LSP для Zed IDE» ("this project is my test JSON LSP for Zed IDE"). Read every file and every decision through that lens.

The project is a learning vehicle for three subjects at once:

- the LSP protocol itself — how a server declares capabilities and serves them over stdio
- the `async-language-server` crate — its `Server` trait, `ServerState`, document store, and tree-sitter integration
- Zed's extension mechanism — a wasm extension launching a native Rust binary

## What It Is Not

Not production software. Hardcoded local paths, launching the debug binary, and having no test suite are accepted traits of a POC here — leave them alone unless the owner asks for hardening.

Not a replacement for the built-in JSON tooling. The project deliberately claims JSON documents away from `json-language-server` so the POC receives real input (the structure rule owns the settings); it is not trying to win users from the real server.

Not a PHP project anymore, despite PHP-era leftovers still in the tree (the structure rule catalogs them). Target all new work at JSON and `tree-sitter-json`.

## How to Make Decisions

Optimize for learning value over robustness. Prefer the smallest change that demonstrates a mechanism end-to-end, and skip enterprise machinery — telemetry, publishing, semver discipline, exhaustive error taxonomies, CI — unless the owner asks for it.

Grow the server capability-by-capability. Treat each LSP capability as one self-contained experiment: advertise it, implement it, verify it live in Zed, then move to the next. Do not start three capabilities in parallel — the point of a POC is to see one mechanism clearly.

Keep JSON as the boundary. A capability that needs another language's grammar or cross-file analysis is out of scope for this project.

Judge results by "does the mechanism work," not by user-facing polish. A hover that returns the raw tree-sitter node in a fenced `json` block is a successful hover here.

When a change is ambiguous, ask the owner — the single user of this software is the person writing it.

## Target Use Cases

- Live experiments against real JSON files open in Zed (currently: hover).
- A sandbox for the `async-language-server` API under a pinned rev.
- A harness for the Zed-extension ↔ native-binary interplay.

---
_This rule exists to keep the project a fast, forgiving experiment: learn one LSP capability at a time, and do not harden, generalize, or productionize anything unasked._

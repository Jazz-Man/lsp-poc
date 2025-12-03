---
id: 1
title: update-php-lsp-constitution
stage: constitution
date: 2025-12-03
surface: agent
model: Qwen
feature: none
branch: main
user: vasilsokolik
command: /sp.constitution
labels: ["constitution", "php-lsp", "project-setup"]
links:
  spec: null
  ticket: null
  adr: null
  pr: null
files:
 - .specify/memory/constitution.md
tests:
 - none
---

## Prompt

```
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Outline

You are updating the project constitution at `.specify/memory/constitution.md`. This file is a TEMPLATE containing placeholder tokens in square brackets (e.g. `[PROJECT_NAME]`, `[PRINCIPLE_1_NAME]`). Your job is to (a) collect/derive concrete values, (b) fill the template precisely, and (c) propagate any amendments across dependent artifacts.

Follow this execution flow:

1. Load the existing constitution template at `.specify/memory/constitution.md`.
   - Identify every placeholder token of the form `[ALL_CAPS_IDENTIFIER]`.
   **IMPORTANT**: The user might require less or more principles than the ones used in the template. If a number is specified, respect that - follow the general template. You will update the doc accordingly.

2. Collect/derive values for placeholders:
   - If user input (conversation) supplies a value, use it.
   - Otherwise infer from existing repo context (README, docs, prior constitution versions if embedded).
   - For governance dates: `RATIFICATION_DATE` is the original adoption date (if unknown ask or mark TODO), `LAST_AMENDED_DATE` is today if changes are made, otherwise keep previous.
   - `CONSTITUTION_VERSION` must increment according to semantic versioning rules:
     - MAJOR: Backward incompatible governance/principle removals or redefinitions.
     - MINOR: New principle/section added or materially expanded guidance.
     - PATCH: Clarifications, wording, typo fixes, non-semantic refinements.
   - If version bump type ambiguous, propose reasoning before finalizing.

3. Draft the updated constitution content:
   - Replace every placeholder with concrete text (no bracketed tokens left except intentionally retained template slots that the project has chosen not to define yet—explicitly justify any left).
   - Preserve heading hierarchy and comments can be removed once replaced unless they still add clarifying guidance.
   - Ensure each Principle section: succinct name line, paragraph (or bullet list) capturing non‑negotiable rules, explicit rationale if not obvious.
   - Ensure Governance section lists amendment procedure, versioning policy, and compliance review expectations.

4. Consistency propagation checklist (convert prior checklist into active validations):
   - Read `.specify/templates/plan-template.md` and ensure any "Constitution Check" or rules align with updated principles.
   - Read `.specify/templates/spec-template.md` for scope/requirements alignment—update if constitution adds/removes mandatory sections or constraints.
   - Read `.specify/templates/tasks-template.md` and ensure task categorization reflects new or removed principle-driven task types (e.g., observability, versioning, testing discipline).
   - Read each command file in `.specify/templates/commands/*.md` (including this one) to verify no outdated references (agent-specific names like CLAUDE only) remain when generic guidance is required.
   - Read any runtime guidance docs (e.g., `README.md`, `docs/quickstart.md`, or agent-specific guidance files if present). Update references to principles changed.

5. Produce a Sync Impact Report (prepend as an HTML comment at top of the constitution file after update):
   - Version change: old → new
   - List of modified principles (old title → new title if renamed)
   - Added sections
   - Removed sections
   - Templates requiring updates (✅ updated / ⚠ pending) with file paths
   - Follow-up TODOs if any placeholders intentionally deferred.

6. Validation before final output:
   - No remaining unexplained bracket tokens.
   - Version line matches report.
   - Dates ISO format YYYY-MM-DD.
   - Principles are declarative, testable, and free of vague language ("should" → replace with MUST/SHOULD rationale where appropriate).

7. Write the completed constitution back to `.specify/memory/constitution.md` (overwrite).

8. Output a final summary to the user with:
   - New version and bump rationale.
   - Any files flagged for manual follow-up.
   - Suggested commit message (e.g., `docs: amend constitution to vX.Y.Z (principle additions + governance update)`).

Formatting & Style Requirements:

- Use Markdown headings exactly as in the template (do not demote/promote levels).
- Wrap long rationale lines to keep readability (<100 chars ideally) but do not hard enforce with awkward breaks.
- Keep a single blank line between sections.
- Avoid trailing whitespace.

If the user supplies partial updates (e.g., only one principle revision), still perform validation and version decision steps.

If critical info missing (e.g., ratification date truly unknown), insert `TODO(<FIELD_NAME>): explanation` and include in the Sync Impact Report under deferred items.

Do not create a new template; always operate on the existing `.specify/memory/constitution.md` file.

---

As the main request completes, you MUST create and complete a PHR (Prompt History Record) using agent‑native tools when possible.

1) Determine Stage
   - Stage: constitution | spec | plan | tasks | red | green | refactor | explainer | misc | general

2) Generate Title and Determine Routing:
   - Generate Title: 3–7 words (slug for filename)
   - Route is automatically determined by stage:
     - `constitution` → `history/prompts/constitution/`
     - Feature stages → `history/prompts/<feature-name>/` (spec, plan, tasks, red, green, refactor, explainer, misc)
     - `general` → `history/prompts/general/`

3) Create and Fill PHR (Shell first; fallback agent‑native)
   - Run: `.specify/scripts/bash/create-phr.sh --title "<title>" --stage <stage> [--feature <name>] --json`
   - Open the file and fill remaining placeholders (YAML + body), embedding full PROMPT_TEXT (verbatim) and concise RESPONSE_TEXT.
   - If the script fails:
     - Read `.specify/templates/phr-template.prompt.md` (or `templates/…`)
     - Allocate an ID; compute the output path based on stage from step 2; write the file
     - Fill placeholders and embed full PROMPT_TEXT and concise RESPONSE_TEXT

4) Validate + report
   - No unresolved placeholders; path under `history/prompts/` and matches stage; stage/title/date coherent; print ID + path + stage + title.
   - On failure: warn, don't block. Skip only for `/sp.phr`.

## Response snapshot

# PHP LSP Server Constitution

## Core Principles

### DOCUMENTATION FIRST
Before writing ANY code: Read documentation from `target/doc-md/` for available crates and APIs; Use `cargo info <crate>` to check versions and features; DO NOT invent APIs — use ONLY what exists in documentation; If documentation is missing, run `.scripts/regen-docs.sh`

### ITERATIVE DEVELOPMENT
Write MAX 20-30 lines at a time; After EVERY change: `cargo check`; If errors: FIX IMMEDIATELY before continuing; After fix confirmed: `git commit -m "..."; NEVER proceed with broken code; NEVER say "errors exist but out of scope"

### USE EXISTING FUNCTIONALITY
Check if feature exists in dependencies before implementing; async-lsp likely has what you need — READ ITS DOCS; Don't reinvent wheels; Prefer composition over custom implementations

### CODE QUALITY STANDARDS
All handlers must be async and non-blocking; Error handling with thiserror/anyhow (no unwrap in production code); Tracing for all logging (not println!); Tests for each module; Documentation comments for public APIs

### PHP & WORDPRESS SPECIFICS
Support PHP 8+ syntax fully; Parse PHPDoc annotations: @param, @return, @var, @template, @psalm-*, @phpstan-*; WordPress Hook API: all 18 functions with go-to-definition; composer.json integration: PHP version detection, ext-* warnings, PSR-4/PSR-0 autoload

### SPECIFICATION-DRIVEN DEVELOPMENT (SDD)
Follow Specification-Driven Development (SDD) principles with spec-kit-plus; Create detailed specs before implementation; Break down features into testable tasks; Ensure all outputs strictly follow user intent

## Technology Stack Requirements
Language: Rust (edition 2021); LSP Framework: async-lsp 0.2.2 with tokio runtime; PHP Parser: tree-sitter-php 0.24.2; Text Handling: ropey for rope-based incremental text; LSP Types: lsp-types 0.97; Target Editor: Zed (via WebAssembly extension using zed_extension_api)

## Development Workflow
Follow feature phases in order: 1) Core Infrastructure (LSP lifecycle, document sync, PHP parsing), 2) Symbol Navigation (Document symbols, go-to-definition, references), 3) Code Completion (Variables, members, classes, signature help), 4) WordPress Hooks (Hook navigation, completion, hover), 5) Composer Integration (PHP version, autoload, vendor navigation); Each task follows implementation contract: Read docs → Write code (max 30 lines) → cargo check → Fix errors → Commit → Next task

## Governance
Constitution supersedes all other practices; Amendments require documentation, approval, migration plan; All PRs/reviews must verify compliance; Complexity must be justified; Use development principles for runtime guidance

**Version**: 1.0.0 | **Ratified**: TODO(RATIFICATION_DATE): Original adoption date unknown | **Last Amended**: 2025-12-03

## Outcome

- ✅ Impact: Created initial project constitution for PHP LSP Server with 6 core principles, technology stack requirements, development workflow, and governance section
- 🧪 Tests: none
- 📁 Files: .specify/memory/constitution.md updated
- 🔁 Next prompts: None needed
- 🧠 Reflection: The constitution now properly reflects the project's development principles and guidelines as specified in the input

## Evaluation notes (flywheel)

- Failure modes observed: none
- Graders run and results (PASS/FAIL): not applicable
- Prompt variant (if applicable): not applicable
- Next experiment (smallest change to try): not applicable
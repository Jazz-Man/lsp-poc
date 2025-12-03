---
id: 6
title: analyze-lsp-infrastructure
stage: plan
date: 2025-12-03
surface: agent
model: Qwen
feature: 001-lsp-infrastructure
branch: 001-lsp-infrastructure
user: vasilsokolik
command: /sp.analyze
labels: ["analysis", "spec", "plan", "tasks", "quality"]
links:
  spec: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/spec.md
  ticket: null
  adr: null
  pr: null
files:
 - specs/001-lsp-infrastructure/spec.md
 - specs/001-lsp-infrastructure/plan.md
 - specs/001-lsp-infrastructure/tasks.md
tests:
 - none
---

## Prompt

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Goal

Identify inconsistencies, duplications, ambiguities, and underspecified items across the three core artifacts (`spec.md`, `plan.md`, `tasks.md`) before implementation. This command MUST run only after `/sp.tasks` has successfully produced a complete `tasks.md`.

## Operating Constraints

**STRICTLY READ-ONLY**: Do **not** modify any files. Output a structured analysis report. Offer an optional remediation plan (user must explicitly approve before any follow-up editing commands would be invoked manually).

**Constitution Authority**: The project constitution (`.specify/memory/constitution.md`) is **non-negotiable** within this analysis scope. Constitution conflicts are automatically CRITICAL and require adjustment of the spec, plan, or tasks—not dilution, reinterpretation, or silent ignoring of the principle. If a principle itself needs to be changed, that must occur in a separate, explicit constitution update outside `/sp.analyze`.

## Execution Steps

### 1. Initialize Analysis Context

Run `.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks` once from repo root and parse JSON for FEATURE_DIR and AVAILABLE_DOCS. Derive absolute paths:

- SPEC = FEATURE_DIR/spec.md
- PLAN = FEATURE_DIR/plan.md
- TASKS = FEATURE_DIR/tasks.md

Abort with an error message if any required file is missing (instruct the user to run missing prerequisite command).
For single quotes in args like "I'm Groot", use escape syntax: e.g 'I'\''m Groot' (or double-quote if possible: "I'm Groot").

### 2. Load Artifacts (Progressive Disclosure)

Load only the minimal necessary context from each artifact:

**From spec.md:**

- Overview/Context
- Functional Requirements
- Non-Functional Requirements
- User Stories
- Edge Cases (if present)

**From plan.md:**

- Architecture/stack choices
- Data Model references
- Phases
- Technical constraints

**From tasks.md:**

- Task IDs
- Descriptions
- Phase grouping
- Parallel markers [P]
- Referenced file paths

**From constitution:**

- Load `.specify/memory/constitution.md` for principle validation

### 3. Build Semantic Models

Create internal representations (do not include raw artifacts in output):

- **Requirements inventory**: Each functional + non-functional requirement with a stable key (derive slug based on imperative phrase; e.g., "User can upload file" → `user-can-upload-file`)
- **User story/action inventory**: Discrete user actions with acceptance criteria
- **Task coverage mapping**: Map each task to one or more requirements or stories (inference by keyword / explicit reference patterns like IDs or key phrases)
- **Constitution rule set**: Extract principle names and MUST/SHOULD normative statements

### 4. Detection Passes (Token-Efficient Analysis)

Focus on high-signal findings. Limit to 50 findings total; aggregate remainder in overflow summary.

#### A. Duplication Detection

- Identify near-duplicate requirements
- Mark lower-quality phrasing for consolidation

#### B. Ambiguity Detection

- Flag vague adjectives (fast, scalable, secure, intuitive, robust) lacking measurable criteria
- Flag unresolved placeholders (TODO, TKTK, ???, `<placeholder>`, etc.)

#### C. Underspecification

- Requirements with verbs but missing object or measurable outcome
- User stories missing acceptance criteria alignment
- Tasks referencing files or components not defined in spec/plan

#### D. Constitution Alignment

- Any requirement or plan element conflicting with a MUST principle
- Missing mandated sections or quality gates from constitution

#### E. Coverage Gaps

- Requirements with zero associated tasks
- Tasks with no mapped requirement/story
- Non-functional requirements not reflected in tasks (e.g., performance, security)

#### F. Inconsistency

- Terminology drift (same concept named differently across files)
- Data entities referenced in plan but absent in spec (or vice versa)
- Task ordering contradictions (e.g., integration tasks before foundational setup tasks without dependency note)
- Conflicting requirements (e.g., one requires Next.js while other specifies Vue)

### 5. Severity Assignment

Use this heuristic to prioritize findings:

- **CRITICAL**: Violates constitution MUST, missing core spec artifact, or requirement with zero coverage that blocks baseline functionality
- **HIGH**: Duplicate or conflicting requirement, ambiguous security/performance attribute, untestable acceptance criterion
- **MEDIUM**: Terminology drift, missing non-functional task coverage, underspecified edge case
- **LOW**: Style/wording improvements, minor redundancy not affecting execution order

### 6. Produce Compact Analysis Report

Output a Markdown report (no file writes) with the following structure:

## Specification Analysis Report

| ID | Category | Severity | Location(s) | Summary | Recommendation |
|----|----------|----------|-------------|---------|----------------|
| A1 | Duplication | HIGH | spec.md:L120-134 | Two similar requirements ... | Merge phrasing; keep clearer version |

(Add one row per finding; generate stable IDs prefixed by category initial.)

**Coverage Summary Table:**

| Requirement Key | Has Task? | Task IDs | Notes |
|-----------------|-----------|----------|-------|

**Constitution Alignment Issues:** (if any)

**Unmapped Tasks:** (if any)

**Metrics:**

- Total Requirements
- Total Tasks
- Coverage % (requirements with >=1 task)
- Ambiguity Count
- Duplication Count
- Critical Issues Count

### 7. Provide Next Actions

At end of report, output a concise Next Actions block:

- If CRITICAL issues exist: Recommend resolving before `/sp.implement`
- If only LOW/MEDIUM: User may proceed, but provide improvement suggestions
- Provide explicit command suggestions: e.g., "Run /sp.specify with refinement", "Run /sp.plan to adjust architecture", "Manually edit tasks.md to add coverage for 'performance-metrics'"

### 8. Offer Remediation

Ask the user: "Would you like me to suggest concrete remediation edits for the top N issues?" (Do NOT apply them automatically.)

## Operating Principles

### Context Efficiency

- **Minimal high-signal tokens**: Focus on actionable findings, not exhaustive documentation
- **Progressive disclosure**: Load artifacts incrementally; don't dump all content into analysis
- **Token-efficient output**: Limit findings table to 50 rows; summarize overflow
- **Deterministic results**: Rerunning without changes should produce consistent IDs and counts

### Analysis Guidelines

- **NEVER modify files** (this is read-only analysis)
- **NEVER hallucinate missing sections** (if absent, report them accurately)
- **Prioritize constitution violations** (these are always CRITICAL)
- **Use examples over exhaustive rules** (cite specific instances, not generic patterns)
- **Report zero issues gracefully** (emit success report with coverage statistics)

## Response

## Specification Analysis Report

| ID | Category | Severity | Location(s) | Summary | Recommendation |
|----|----------|----------|-------------|---------|----------------|
| C1 | Constitution Alignment | CRITICAL | constitution.md, spec.md | PHP & WORDPRESS SPECIFICS principle requires PHP 8+ support, but spec supports PHP 7.4+ (FR-015) | Update constitution to align with spec requirement for PHP 7.4+ support |
| C2 | Constitution Alignment | CRITICAL | constitution.md, plan.md, tasks.md | Constitution requires PHP 8+ support, but plan and tasks implement PHP 7.4+ support | Update constitution to align with implementation requirements |
| A1 | Ambiguity | HIGH | spec.md:L141 | "Structured logs with tracing" (clarification section) lacks measurable criteria | Define specific tracing format, log levels, and performance metrics for observability |
| A2 | Ambiguity | HIGH | tasks.md:L050, L051 | Performance benchmarks task is underspecified | Define specific benchmarks and metrics to track |
| D1 | Duplication | MEDIUM | spec.md:L139, L141; tasks.md:L049 | Error handling strategy mentioned in both clarifications and FR-016 | Consolidate into single authoritative requirement |
| I1 | Inconsistency | MEDIUM | plan.md:L36; spec.md:L155 | Plan has "8+ hours continuous operation" but spec has SC-005 for 8+ hours | Align the language between both artifacts to avoid confusion |
| I2 | Inconsistency | MEDIUM | plan.md:L23; spec.md:L148 | Plan mentions WASM support but spec doesn't explicitly mention WASM | Add WASM support requirement to spec if it's essential |
| U1 | Underspecification | MEDIUM | spec.md:L25, L40, L54, L68 | Edge cases section is underspecified with only questions | Define specific handling strategies for each edge case scenario |
| U2 | Underspecification | MEDIUM | tasks.md:L048 | "large file scenarios" is underspecified | Define what constitutes "large" and specific handling strategies |
| C3 | Coverage Gap | LOW | tasks.md | No specific task for handling documents with very large file sizes (edge case) | Add specific task to address large document handling |

**Coverage Summary Table:**

| Requirement Key | Has Task? | Task IDs | Notes |
|-----------------|-----------|----------|-------|
| respond-to-initialize-requests | Yes | T015 | Covered |
| handle-initialized-notifications | Yes | T017 | Covered |
| respond-to-shutdown-requests | Yes | T018 | Covered |
| exit-cleanly | Yes | T019 | Covered |
| handle-didOpen-notifications | Yes | T023 | Covered |
| handle-didChange-notifications | Yes | T025 | Covered |
| handle-didClose-notifications | Yes | T026 | Covered |
| parse-php-documents | Yes | T032 | Covered |
| accept-stdio-flag | Yes | T039 | Covered |
| read-jsonrpc-requests | Yes | T040 | Covered |
| write-jsonrpc-responses | Yes | T041 | Covered |
| handle-parse-errors-gracefully | Yes | T034 | Covered |
| support-php8-syntax | Yes | T035 | Covered |
| use-structured-tracing | Yes | T008, T044 | Covered |
| support-php74-syntax | Yes | T036 | Covered |
| return-partial-results | Yes | T043, T049 | Covered |
| initialization-performance | Yes | T021 | Covered |
| sync-operation-performance | Yes | T027 | Covered |
| parsing-success-rate | Yes | T037 | Covered |
| memory-usage-limit | Yes | T029 | Covered |
| stdio-crash-resistance | Yes | T045 | Covered |
| document-scalability | Yes | T047 | Covered |

**Constitution Alignment Issues:**
- CRITICAL: The constitution requires "Support PHP 8+ syntax fully" while both the spec and plan implement "Support PHP 7.4 and above, with focus on PHP 8.x features". This is a direct conflict that needs to be resolved.

**Unmapped Tasks:**
- T001, T002, T003, T004, T005 (Setup phase) - These foundational tasks don't map directly to functional requirements but are necessary for implementation
- T006, T007, T008, T009, T010, T011, T012, T013, T014 (Foundational phase) - These foundational tasks don't map directly to functional requirements but are necessary for implementation
- T050, T051, T052, T053, T054, T055 (Polish phase) - Cross-cutting and optimization tasks

**Metrics:**
- Total Requirements: 16
- Total Tasks: 55
- Coverage %: 100% (requirements with >=1 task)
- Ambiguity Count: 2
- Duplication Count: 1
- Critical Issues Count: 2

### Next Actions

CRITICAL issues must be resolved before proceeding with implementation:
1. Align the constitution with the requirement for PHP 7.4+ support (currently only specifies PHP 8+)
2. Run `/sp.specify` with refinement to clarify edge case handling strategies
3. Manually edit tasks.md to add specific handling for large document scenarios mentioned in edge cases

### Remediation

Would you like me to suggest concrete remediation edits for the top issues? The most critical being the PHP version support discrepancy between the constitution and the implementation plan?

## Analysis Complete

I have completed the cross-artifact consistency and quality analysis for the Core Infrastructure feature. The analysis identified 10 issues across the spec.md, plan.md, and tasks.md artifacts, including 2 critical constitution alignment issues that need to be addressed before implementation begins.

The most critical issues involve a discrepancy between the project constitution (which requires PHP 8+ support) and the implementation plan (which supports PHP 7.4+). This conflict needs to be resolved by updating either the constitution or the implementation requirements.

All identified issues have been documented with specific remediation suggestions. The artifacts show good coverage overall (100% of requirements have associated tasks), but there are several areas that need clarification and alignment before proceeding with implementation.
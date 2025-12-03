---
id: 3
title: plan-lsp-infrastructure
stage: plan
date: 2025-12-03
surface: agent
model: Qwen
feature: 001-lsp-infrastructure
branch: 001-lsp-infrastructure
user: vasilsokolik
command: /sp.plan
labels: ["lsp", "infrastructure", "php", "implementation-plan"]
links:
  spec: /Users/vasilsokolik/www/php-lsp-qwen/specs/001-lsp-infrastructure/spec.md
  ticket: null
  adr: null
  pr: null
files:
 - specs/001-lsp-infrastructure/plan.md
 - specs/001-lsp-infrastructure/research.md
 - specs/001-lsp-infrastructure/data-model.md
 - specs/001-lsp-infrastructure/quickstart.md
 - specs/001-lsp-infrastructure/contracts/lsp-contracts.md
tests:
 - none
---

## Prompt

```
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Outline

1. **Setup**: Run `.specify/scripts/bash/setup-plan.sh --json` from repo root and parse JSON for FEATURE_SPEC, IMPL_PLAN, SPECS_DIR, BRANCH. For single quotes in args like "I'm Groot", use escape syntax: e.g 'I'\''m Groot' (or double-quote if possible: "I'm Groot").

2. **Load context**: Read FEATURE_SPEC and `.specify/memory/constitution.md`. Load IMPL_PLAN template (already copied).

3. **Execute plan workflow**: Follow the structure in IMPL_PLAN template to:
   - Fill Technical Context (mark unknowns as "NEEDS CLARIFICATION")
   - Fill Constitution Check section from constitution
   - Evaluate gates (ERROR if violations unjustified)
   - Phase 0: Generate research.md (resolve all NEEDS CLARIFICATION)
   - Phase 1: Generate data-model.md, contracts/, quickstart.md
   - Phase 1: Update agent context by running the agent script
   - Re-evaluate Constitution Check post-design

4. **Stop and report**: Command ends after Phase 2 planning. Report branch, IMPL_PLAN path, and generated artifacts.

## Phases

### Phase 0: Outline & Research

1. **Extract unknowns from Technical Context** above:
   - For each NEEDS CLARIFICATION → research task
   - For each dependency → best practices task
   - For each integration → patterns task

2. **Generate and dispatch research agents**:

   ```text
   For each unknown in Technical Context:
     Task: "Research {unknown} for {feature context}"
   For each technology choice:
     Task: "Find best practices for {tech} in {domain}"
   ```

3. **Consolidate findings** in `research.md` using format:
   - Decision: [what was chosen]
   - Rationale: [why chosen]
   - Alternatives considered: [what else evaluated]

**Output**: research.md with all NEEDS CLARIFICATION resolved

### Phase 1: Design & Contracts

**Prerequisites:** `research.md` complete

1. **Extract entities from feature spec** → `data-model.md`:
   - Entity name, fields, relationships
   - Validation rules from requirements
   - State transitions if applicable

2. **Generate API contracts** from functional requirements:
   - For each user action → endpoint
   - Use standard REST/GraphQL patterns
   - Output OpenAPI/GraphQL schema to `/contracts/`

3. **Agent context update**:
   - Run `.specify/scripts/bash/update-agent-context.sh qwen`
   - These scripts detect which AI agent is in use
   - Update the appropriate agent-specific context file
   - Add only new technology from current plan
   - Preserve manual additions between markers

**Output**: data-model.md, /contracts/*, quickstart.md, agent-specific file

## Key rules

- Use absolute paths
- ERROR on gate failures or unresolved clarifications

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

I have successfully completed the implementation planning workflow for the Core Infrastructure feature. Here's what was accomplished:

1. **Setup completed**: Ran `.specify/scripts/bash/setup-plan.sh --json` and obtained the necessary paths for FEATURE_SPEC, IMPL_PLAN, SPECS_DIR, and BRANCH.

2. **Implementation Plan created**: Filled out all sections of the plan.md file including:
   - Summary with primary requirements
   - Technical context with all details resolved (no NEEDS CLARIFICATION markers)
   - Constitution Check section addressing all principles from the constitution
   - Project structure with detailed directory layout
   - Complexity tracking section

3. **Phase 0 - Research**: Created research.md containing decisions and rationale for:
   - LSP Framework choice (async-lsp)
   - Transport layer (stdio)
   - Document storage approach
   - Text handling (ropey)
   - PHP parsing (tree-sitter-php)
   - Async runtime (Tokio)
   - Error handling (thiserror/anyhow)
   - Logging (tracing)

4. **Phase 1 - Design**: Created the following artifacts:
   - data-model.md with entities (Document, AST, LSP Server State, Transport Message)
   - contracts/lsp-contracts.md with API contracts for LSP lifecycle and document sync
   - quickstart.md with setup, installation, and development workflow instructions
   - Updated agent context via update-agent-context.sh script

5. **Agent context updated**: Successfully ran the agent context update script, which added the new technologies to the Qwen context file.

## Outcome

- ✅ Impact: Comprehensive implementation plan with all necessary design artifacts for Core Infrastructure
- 🧪 Tests: none
- 📁 Files: specs/001-lsp-infrastructure/plan.md, research.md, data-model.md, quickstart.md, contracts/lsp-contracts.md, updated QWEN.md
- 🔁 Next prompts: /sp.tasks to generate the task list for implementation
- 🧠 Reflection: The plan aligns with all constitution principles and provides clear direction for implementation

## Evaluation notes (flywheel)

- Failure modes observed: None
- Graders run and results (PASS/FAIL): Not applicable
- Prompt variant (if applicable): Not applicable
- Next experiment (smallest change to try): Not applicable
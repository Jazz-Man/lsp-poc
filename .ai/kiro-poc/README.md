# Kiro Proof of Concept Testing Plan

## Overview

This document provides step-by-step instructions to test whether Kiro properly integrates steering constraints into all phases of specification development. Based on previous experience with github/spec-kit losing context, we need to verify that Kiro consistently applies steering rules.

## Objective

Test if Kiro maintains steering context (especially strict development constraints) throughout the entire workflow:
- Requirements generation
- Design generation  
- Task generation
- Implementation execution

## Test Scenario

We'll create a simple "Document Struct" specification to test Kiro's constraint integration without the complexity of the full PHP LSP project.

---

## Phase 0: Setup Steering Documents

### Step 1: Initialize Base Steering

**Command:**
```
/kiro:steering
```

**Expected Result:**
- Creates `.kiro/steering/` directory
- Generates `product.md`, `tech.md`, `structure.md`

### Step 2: Add Custom Development Constraints

**Command:**
```
/kiro:steering-custom
```

**When prompted for custom steering content, use:**

```markdown
# Development Constraints

## Critical Implementation Guidelines

### Documentation-First Development Approach
**CRITICAL:** Always verify API existence before implementation. Use ONLY documented APIs from crate documentation. Never assume API behavior.

Before writing any code:
1. Generate/verify documentation: `.scripts/regen-docs.sh`
2. Locate relevant crate documentation: `cat target/doc-md/index.md`
3. Read specific API docs: `cat target/doc-md/{crate}/index.md`
4. Confirm API signatures and usage patterns

### Incremental Development Protocol
For EACH implementation task, follow this exact sequence:

1. **RESEARCH:** Read documentation for APIs to be used
2. **IMPLEMENT:** Write MAX 20-30 lines of focused code
3. **VALIDATE:** Run `cargo check` - must pass with no errors
4. **FIX:** If errors exist, resolve IMMEDIATELY (not later)
5. **COMMIT:** Run `git add -A && git commit -m "task: description"`
6. **PROCEED:** Only then continue to next task

### Absolute Development Constraints

✗ STOP if compilation fails - fix immediately
✗ STOP if API doesn't exist in docs - research alternatives
✗ STOP after 30 lines - validate before continuing
✗ STOP if deviating from planned tasks - return to plan
✗ STOP if errors occur - resolve before proceeding

✓ VERIFY documentation before each code section
✓ RUN cargo check after every change
✓ COMMIT working code regularly
✓ FOLLOW plan task order precisely
✓ IMPLEMENT ONE feature at a time

### Error Resolution Protocol

When compilation errors occur:

1. **IDENTIFY:** First error in `cargo check` output
2. **RESEARCH:** Check relevant documentation in target/doc-md/
3. **FIX:** Address ONLY the first error
4. **VERIFY:** Run `cargo check` again
5. **REPEAT:** Until compilation succeeds
6. **COMMIT:** Run `git add -A && git commit -m "fix: {description}"`

## Task Generation Requirements

All generated tasks MUST include:
- Research step with specific documentation to read
- Implementation step limited to 20-30 lines
- Validation step with `cargo check`
- Fix step if needed
- Commit step with descriptive message

## Implementation Validation

Each task implementation must:
- Include explicit research of API documentation
- Never exceed 30 lines of code changes
- Pass `cargo check` before proceeding
- Include git commit after successful validation
```

**Expected Result:**
- Creates additional steering file with development constraints

---

## Phase 1: Test Specification Creation

### Step 3: Initialize Test Specification

**Command:**
```
/kiro:spec-init "Simple Document Struct - Basic document representation using Ropey for text handling and optional tree-sitter integration"
```

**Expected Result:**
- Creates `.kiro/specs/simple-document-struct/` (or similar name)
- Generates `spec.json` and `requirements.md`

### Step 4: Generate Requirements

**Command:**
```
/kiro:spec-requirements simple-document-struct
```

**What to Check:**
- Look for references to development constraints in generated requirements
- Check if incremental development is mentioned
- Verify research-first approach is included

**Expected Result:**
- Requirements should include references to documentation-first approach
- Should mention 20-30 line constraints

### Step 5: Generate Design

**Command:**
```
/kiro:spec-design simple-document-struct
```

**What to Check:**
- Design should reference the development constraints
- Architecture should be broken into small, validatable pieces
- Should include documentation verification steps

**Expected Result:**
- Technical design incorporating development methodology
- References to error resolution protocol

### Step 6: Generate Tasks

**Command:**
```
/kiro:spec-tasks simple-document-struct
```

**Critical Validation:**
This is the key test! Check if generated tasks include:

**Each task should have structure like:**
```markdown
- [ ] Research: Read ropey documentation (`cat target/doc-md/ropey/index.md`)
- [ ] Implement: Create Document struct (MAX 20-30 lines)
- [ ] Validate: Run `cargo check` - must pass with no errors  
- [ ] Fix: Resolve any compilation errors immediately
- [ ] Commit: `git add -A && git commit -m "feat: add Document struct"`
```

**If tasks DON'T include these steps = Kiro FAILED the constraint integration test**

---

## Phase 2: Test Implementation

### Step 7: Test Implementation Phase

**Command:**
```
/kiro:spec-impl simple-document-struct 1.1
```

**What to Monitor:**
- Does the implementation agent follow the research-first protocol?
- Does it limit code to 20-30 lines?
- Does it run `cargo check` after changes?
- Does it commit after successful validation?

**Expected Behavior:**
- Agent should read documentation before implementing
- Should implement in small increments
- Should validate with `cargo check`
- Should make git commits after each successful step

---

## Validation Criteria

### ✅ SUCCESS Indicators:

1. **Steering Integration:**
   - Requirements mention development constraints
   - Design incorporates methodology
   - Tasks explicitly include research/validate/commit steps

2. **Task Structure:**
   - Each task has research step with specific documentation paths
   - Implementation steps limited to 20-30 lines
   - Validation steps with `cargo check`
   - Commit steps with descriptive messages

3. **Implementation Behavior:**
   - Agent reads documentation before coding
   - Implements in small increments
   - Validates with cargo check
   - Makes commits after validation

### ❌ FAILURE Indicators:

1. **Missing Constraints:**
   - Tasks don't mention documentation verification
   - No 20-30 line limits in task descriptions
   - Missing validation steps
   - No commit instructions

2. **Wrong Implementation Behavior:**
   - Agent implements without reading docs
   - Writes more than 30 lines at once
   - Ignores compilation errors
   - Doesn't make incremental commits

---

## Commands for Both Agents

### For Claude Code:
Copy-paste these commands one by one, checking results after each:

```
/kiro:steering
/kiro:steering-custom
/kiro:spec-init "Simple Document Struct - Basic document representation using Ropey for text handling and optional tree-sitter integration"
/kiro:spec-requirements simple-document-struct
/kiro:spec-design simple-document-struct
/kiro:spec-tasks simple-document-struct
/kiro:spec-impl simple-document-struct 1.1
```

### For Qwen Code:
Same commands (Kiro commands are identical for both agents):

```
/kiro:steering
/kiro:steering-custom
/kiro:spec-init "Simple Document Struct - Basic document representation using Ropey for text handling and optional tree-sitter integration"
/kiro:spec-requirements simple-document-struct
/kiro:spec-design simple-document-struct
/kiro:spec-tasks simple-document-struct
/kiro:spec-impl simple-document-struct 1.1
```

---

## Next Steps Based on Results

### If Test PASSES ✅:
- Kiro properly maintains steering context
- Safe to proceed with full PHP LSP project using Kiro
- Can trust that development constraints will be enforced

### If Test FAILS ❌:
- Kiro has the same context-loss issues as github/spec-kit
- Need to either:
  - Use Kiro only for planning phases, implement manually
  - Find ways to reinforce constraints in each phase
  - Consider alternative SDD tools

---

## Monitoring Checklist

Use this checklist while running the test:

**After /kiro:spec-requirements:**
- [ ] Requirements mention documentation-first approach
- [ ] Requirements reference 20-30 line constraints
- [ ] Requirements include validation protocols

**After /kiro:spec-design:**
- [ ] Design incorporates development methodology  
- [ ] Design breaks work into small pieces
- [ ] Design references error resolution protocol

**After /kiro:spec-tasks:**
- [ ] Each task has research step with specific docs
- [ ] Each task limits implementation to 20-30 lines
- [ ] Each task includes validation with `cargo check`
- [ ] Each task includes commit step

**During /kiro:spec-impl:**
- [ ] Agent reads documentation before implementing
- [ ] Agent limits code changes to ~20-30 lines
- [ ] Agent runs `cargo check` after changes
- [ ] Agent makes git commits after validation
- [ ] Agent fixes compilation errors immediately

---

## File Locations to Check

After running the test, examine these files:

- `.kiro/steering/` - All steering documents
- `.kiro/specs/simple-document-struct/requirements.md` - Should reference constraints
- `.kiro/specs/simple-document-struct/design.md` - Should incorporate methodology  
- `.kiro/specs/simple-document-struct/tasks.md` - Critical validation point
- Git log - Should show incremental commits if implementation ran

---

## Expected Timeline

- Phase 0 (Steering setup): 5-10 minutes
- Phase 1 (Spec creation): 15-20 minutes  
- Phase 2 (Implementation test): 10-15 minutes
- Total: ~30-45 minutes per agent

Test both agents separately to compare results and ensure consistency.

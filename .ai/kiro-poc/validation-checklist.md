# Kiro PoC Validation Checklist

## Pre-Test Setup

- [ ] Both agents (Claude Code + Qwen Code) are available
- [ ] Project is in clean git state
- [ ] Ready to run commands sequentially

---

## Phase 0: Steering Setup Validation

### After `/kiro:steering`
- [ ] `.kiro/steering/` directory created
- [ ] `product.md` exists and contains project context
- [ ] `tech.md` exists and mentions Rust/async-lsp
- [ ] `structure.md` exists with architectural guidelines

### After `/kiro:steering-custom`
- [ ] Custom steering file created with development constraints
- [ ] File contains "Documentation-First Development Approach" section
- [ ] File contains "Incremental Development Protocol" with 20-30 line limit
- [ ] File contains "Absolute Development Constraints" with ✗/✓ rules
- [ ] File contains "Error Resolution Protocol"
- [ ] File contains "Task Generation Requirements"

---

## Phase 1: Specification Validation

### After `/kiro:spec-init`
- [ ] `.kiro/specs/simple-document-struct/` directory created (or similar name)
- [ ] `spec.json` exists with correct metadata
- [ ] `requirements.md` exists (initial template)
- [ ] Feature name is reasonable and clear

### After `/kiro:spec-requirements` ⚠️ CRITICAL TEST
**Check `requirements.md` for steering integration:**
- [ ] Requirements mention documentation-first approach
- [ ] Requirements reference incremental development methodology
- [ ] Requirements include validation protocols
- [ ] Requirements mention cargo check validation
- [ ] Requirements reference commit requirements

**If ANY of these are missing = STEERING INTEGRATION FAILED**

### After `/kiro:spec-design` ⚠️ CRITICAL TEST  
**Check `design.md` for constraint integration:**
- [ ] Design incorporates development methodology constraints
- [ ] Design breaks implementation into small, validatable pieces
- [ ] Design references error resolution protocol
- [ ] Design mentions documentation verification steps
- [ ] Architecture considers 20-30 line implementation limits

**If design ignores constraints = STEERING INTEGRATION FAILED**

### After `/kiro:spec-tasks` 🚨 MOST CRITICAL TEST
**Check `tasks.md` structure - Each task should include:**

**Example Expected Task Structure:**
```
- [ ] Research: Read ropey documentation (cat target/doc-md/ropey/index.md)
- [ ] Implement: Create Document struct (MAX 20-30 lines)  
- [ ] Validate: Run cargo check - must pass with no errors
- [ ] Fix: Resolve any compilation errors immediately
- [ ] Commit: git add -A && git commit -m "feat: add Document struct"
```

**Validation Checklist for tasks.md:**
- [ ] Each major task has research sub-task with specific documentation paths
- [ ] Implementation sub-tasks explicitly mention 20-30 line limits  
- [ ] Each task includes validation step with `cargo check`
- [ ] Each task includes fix step for error resolution
- [ ] Each task includes commit step with message format
- [ ] Tasks follow incremental development protocol order
- [ ] No task tries to implement too much at once

**🚨 IF TASKS DON'T INCLUDE THESE STEPS = KIRO FAILED THE TEST**
**Stop here and document the failure before proceeding**

---

## Phase 2: Implementation Validation

### During `/kiro:spec-impl simple-document-struct 1.1`

**Monitor implementation agent behavior:**

**Research Phase:**
- [ ] Agent attempts to read documentation before coding
- [ ] Agent mentions `.scripts/regen-docs.sh` or similar doc generation
- [ ] Agent checks for API existence in documentation
- [ ] Agent reads specific crate documentation files

**Implementation Phase:**
- [ ] Agent implements in small increments (≤30 lines per change)
- [ ] Agent focuses on single task, not multiple features
- [ ] Agent follows the planned task structure
- [ ] Agent doesn't deviate from approved tasks

**Validation Phase:**
- [ ] Agent runs `cargo check` after implementing code
- [ ] Agent addresses compilation errors immediately if they occur
- [ ] Agent doesn't proceed with broken compilation
- [ ] Agent follows error resolution protocol (first error only)

**Commit Phase:**
- [ ] Agent makes git commits after successful validation
- [ ] Commit messages follow specified format
- [ ] Agent commits working code regularly
- [ ] Agent doesn't skip commit steps

**Overall Behavior:**
- [ ] Agent follows the exact sequence: RESEARCH → IMPLEMENT → VALIDATE → FIX → COMMIT
- [ ] Agent respects the constraints throughout implementation
- [ ] Agent doesn't try to shortcut the process
- [ ] Agent maintains focus on single task

---

## Success/Failure Determination

### ✅ SUCCESS Criteria (ALL must be true):
- [ ] Steering constraints appear in requirements.md
- [ ] Design incorporates development methodology  
- [ ] Tasks.md includes research/validate/commit steps for each task
- [ ] Implementation agent follows the constraint protocol
- [ ] Agent respects 20-30 line limits
- [ ] Agent validates with cargo check
- [ ] Agent makes incremental commits

### ❌ FAILURE Indicators (ANY means failure):
- [ ] Requirements ignore steering constraints
- [ ] Design doesn't mention development methodology
- [ ] Tasks.md lacks research/validate/commit structure
- [ ] Implementation agent ignores documentation-first approach
- [ ] Agent writes >30 lines without validation
- [ ] Agent ignores compilation errors
- [ ] Agent skips commit steps
- [ ] Agent deviates from approved task structure

---

## Post-Test Actions

### If Test PASSES ✅:
- [ ] Document successful steering integration
- [ ] Proceed with confidence to full PHP LSP project
- [ ] Use same steering setup for actual project
- [ ] Trust Kiro workflow for complex implementations

### If Test FAILS ❌:
- [ ] Document specific failure points
- [ ] Note which phase lost steering context
- [ ] Consider alternative approaches:
  - [ ] Use Kiro only for planning (requirements/design/tasks)
  - [ ] Implement manually with strict constraint adherence
  - [ ] Try reinforcing constraints in each phase manually
  - [ ] Evaluate other SDD tools

---

## Agent Comparison

### Claude Code Results:
- [ ] Steering setup: ✅/❌
- [ ] Requirements integration: ✅/❌  
- [ ] Design integration: ✅/❌
- [ ] Tasks structure: ✅/❌
- [ ] Implementation behavior: ✅/❌

### Qwen Code Results:
- [ ] Steering setup: ✅/❌
- [ ] Requirements integration: ✅/❌
- [ ] Design integration: ✅/❌  
- [ ] Tasks structure: ✅/❌
- [ ] Implementation behavior: ✅/❌

### Consistency Check:
- [ ] Both agents produce similar steering documents
- [ ] Both agents integrate constraints similarly
- [ ] Both agents generate consistent task structures
- [ ] Both agents follow implementation constraints similarly

**Notes:**
- If agents behave differently, Kiro may have consistency issues
- If both fail similarly, it's a systematic Kiro limitation
- If both succeed, Kiro is reliable for constraint integration

---

## Emergency Stop Conditions

**STOP TESTING IMMEDIATELY IF:**
- [ ] Tasks.md generated without research/validate/commit steps
- [ ] Implementation agent ignores all constraints
- [ ] Agent produces broken code and continues anyway
- [ ] Agent completely ignores steering documents

**When stopping early:**
1. Document the exact failure point
2. Save all generated files for analysis
3. Note which specific constraints were ignored
4. Prepare feedback for improving the process

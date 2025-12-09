# Kiro PoC Commands - Quick Reference

## Copy-Paste Commands for Testing

### Phase 0: Setup Steering

```
/kiro:steering
```

```
/kiro:steering-custom
```

**When prompted for custom content, paste:**

```
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

### Phase 1: Test Specification

```
/kiro:spec-init "Simple Document Struct - Basic document representation using Ropey for text handling and optional tree-sitter integration"
```

```
/kiro:spec-requirements simple-document-struct
```

```
/kiro:spec-design simple-document-struct
```

```
/kiro:spec-tasks simple-document-struct
```

### Phase 2: Test Implementation

```
/kiro:spec-impl simple-document-struct 1.1
```

## Status Check Command

```
/kiro:spec-status simple-document-struct
```

## Alternative Commands for Testing

If the feature name is different than expected, use:

```
/kiro:spec-status
```

Then use the actual feature name shown in the status.

## Validation Commands (Optional)

```
/kiro:validate-gap simple-document-struct
```

```
/kiro:validate-design simple-document-struct
```

```
/kiro:validate-impl simple-document-struct
```

---

## Critical Checkpoints

**After each command, check:**

1. **After steering setup:** Verify `.kiro/steering/` contains your constraints
2. **After spec-requirements:** Check if requirements mention 20-30 line limits  
3. **After spec-tasks:** **CRITICAL** - Verify tasks include research/validate/commit steps
4. **During spec-impl:** Watch if agent actually follows the constraints

**STOP testing if tasks.md doesn't include the constraint steps - that means Kiro failed the test.**

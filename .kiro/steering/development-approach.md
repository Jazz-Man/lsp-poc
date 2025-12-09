# Development Approach

## Core Philosophy

A documentation-first, incremental development methodology that emphasizes verifying API existence before implementation and maintaining code quality through continuous validation.

## Critical Implementation Guidelines

### Documentation-First Development
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

## Development Constraints

### Absolute Development Rules

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

---
_Document patterns, not file trees. New files following patterns shouldn't require updates_
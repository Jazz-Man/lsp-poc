# Kiro Instructions for STEP 1: Basic LSP Foundation

## Overview
This document provides step-by-step instructions for using Kiro commands to implement the Basic LSP Foundation according to your implementation guidelines.

## Phase 1: Specification Initialization
```bash
/kiro:spec-init "Basic LSP Foundation - General-purpose LSP framework with enhanced features similar to async-language-server"
```

## Phase 2: Requirements Generation
```bash
/kiro:spec-requirements basic-lsp
```
**Note**: The requirements should capture your core components (CoreLspService, DocumentManager, etc.) and enhanced features over async-language-server.

## Phase 3: Design Generation
```bash
/kiro:spec-design basic-lsp -y
```
**Important**: During the design phase, ensure to incorporate your Critical Implementation Guidelines:
- Documentation-First Development Approach: Verify API existence before design decisions
- Reference your technology stack: async-lsp, tower, ropey, tree-sitter, tokio, tracing, dashmap
- Include your architecture design: CoreLspService trait, DocumentManager, etc.
- Consider your documentation research steps in the design decisions

## Phase 4: Tasks Generation
```bash
/kiro:spec-tasks basic-lsp -y
```
**Guidance**: Tasks should follow your Incremental Development Protocol:
- Each task should be small (align with your 20-30 lines guideline)
- Tasks should follow the sequence: Research -> Implement -> Validate -> Fix -> Commit
- Map all requirements to specific implementation tasks
- Include validation tasks for cargo check after each implementation

## Phase 5: Implementation Execution
```bash
/kiro:spec-impl basic-lsp
```
**Critical**: During implementation, follow your exact development protocol:
1. **RESEARCH**: Read documentation using your research steps before each task
2. **IMPLEMENT**: Write MAX 20-30 lines of focused code per task
3. **VALIDATE**: Run `cargo check` - must pass with no errors
4. **FIX**: If errors exist, resolve IMMEDIATELY
5. **COMMIT**: Run `git add -A && git commit` after each task
6. **PROCEED**: Only then continue to next task

## Dependency Verification Integration
For each dependency verification task in your plan, use WebSearch/WebFetch through Kiro to verify:
- Crate information
- Available features
- Latest version stability
- Security advisories
- Source/repo maintenance status

## Error Resolution Protocol
When you encounter compilation errors during Kiro's TDD cycle, follow your protocol:
1. **IDENTIFY**: First error in `cargo check` output
2. **RESEARCH**: Check relevant documentation in target/doc-md/
3. **FIX**: Address ONLY the first error
4. **VERIFY**: Run `cargo check` again
5. **REPEAT**: Until compilation succeeds
6. **COMMIT**: Complete the task in Kiro system

## Documentation Research Integration
Your documentation research process should be integrated into each Kiro implementation task:
1. Update documentation: `.scripts/regen-docs.sh`
2. List available crates: `cat target/doc-md/index.md`
3. Research specific crate: `cat target/doc-md/{crate}/index.md`
4. Search for functionality: `grep -r "Pattern" target/doc-md/{crate}/`

## Validation and Status Check
Monitor progress with:
```bash
/kiro:spec-status basic-lsp
```

## Optional Validations
You can also run additional validations during the process:
- `/kiro:validate-gap basic-lsp` - to analyze gaps with existing codebase
- `/kiro:validate-design basic-lsp` - for design review
- `/kiro:validate-impl basic-lsp` - to validate completed implementations

## Next Phase Preparation
Ensure interfaces are prepared for the PHP-specific implementation by following your guidelines:
- Design hooks for language-specific analysis
- Ensure clean separation between general LSP functionality and language-specific features
- Plan for easy integration with Zed editor
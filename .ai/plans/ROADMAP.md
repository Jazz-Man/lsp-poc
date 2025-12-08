# Implementation Roadmap

This roadmap outlines the chronological implementation of the PHP LSP project across the three main steps, highlighting dependencies and integration points.

## Critical Implementation Guidelines for ALL Phases

### Documentation-First Development Approach
**CRITICAL:** Always verify API existence before implementation. Use ONLY documented APIs from crate documentation or proven examples. Never assume API behavior.

Before writing any code:
1. Generate/verify documentation: `.scripts/regen-docs.sh` (for Rust crates)
2. Study Deputy example implementation patterns (for Zed integration)
3. Locate relevant crate documentation: `cat target/doc-md/index.md`
4. Read specific API docs: `cat target/doc-md/{crate}/index.md`
5. Confirm API signatures and usage patterns

### Incremental Development Protocol
For EACH implementation task in the entire project, follow this exact sequence:

1. **RESEARCH:** Read documentation/examples for APIs to be used
2. **IMPLEMENT:** Write MAX 20-30 lines of focused code
3. **VALIDATE:** Run `cargo check` - must pass with no errors
4. **FIX:** If errors exist, resolve IMMEDIATELY (not later)
5. **COMMIT:** Run `git add -A && git commit -m "task: description"`
6. **PROCEED:** Only then continue to next task

### Absolute Development Constraints

✗ STOP if compilation fails - fix immediately
✗ STOP if API doesn't exist in docs/examples - research alternatives
✗ STOP after 30 lines - validate before continuing
✗ STOP if deviating from planned tasks - return to plan
✗ STOP if errors occur - resolve before proceeding

✓ VERIFY documentation before each code section
✓ RUN cargo check after every change
✓ COMMIT working code regularly
✓ FOLLOW plan task order precisely
✓ IMPLEMENT ONE feature at a time
✓ VERIFY dependencies before adding: `cargo info {crate}` and `cargo tree --prune={crate}`
✓ ENABLE only required features to minimize dependencies
✓ CHECK security with `cargo audit` (install if needed)

## Phase 1: Basic LSP Foundation
Timeline: Weeks 1-4

### Week 1: Core Service Layer
- Define CoreLspService trait
- Set up project structure
- Implement basic request routing

### Week 2: Document Management
- Implement DocumentManager with ropey
- Add incremental update support
- Create document lifecycle handlers

### Week 3: Transport & Encoding
- Implement Transport trait
- Create EncodingHandler
- Integrate with async-lsp mechanisms

### Week 4: Middleware System
- Build pluggable middleware system
- Add built-in middleware components
- Complete basic LSP functionality

## Phase 2: PHP LSP Implementation
Timeline: Weeks 5-10
Prerequisite: Completion of Phase 1

### Week 5: PHP Parser Integration
- Integrate tree-sitter-php
- Create PhpDocument implementation
- Add PHP-specific node queries

### Week 6: PHPDoc Analysis
- Integrate phpstan/phpdoc-parser
- Create PhpDocAnalyzer
- Connect to completion/hover functionality

### Week 7: Symbol Indexing
- Implement PhpSymbolIndexer
- Create indexing for classes/functions
- Implement go-to-definition

### Week 8: Diagnostics & Composer Integration
- Implement syntax error diagnostics
- Parse composer.json for PHP version
- Add extension requirement validation

### Week 9: WordPress Hook System
- Analyze WordPress hook patterns
- Implement WordPressHookProcessor
- Add go-to-definition for hooks

### Week 10: Completion & Hover
- Implement context-aware completion
- Enhance hover functionality
- Complete all PHP LSP features

## Phase 3: Zed Editor Integration
Timeline: Weeks 11-13
Prerequisites: Completion of Phases 1 & 2

### Week 11: Extension Structure
- Create Zed extension project
- Implement basic Extension trait
- Set up binary management

### Week 12: Configuration & Project Detection
- Integrate with Zed configuration
- Implement project type detection
- Add platform-specific handling

### Week 13: Advanced Features & Testing
- Implement slash commands
- Add authentication support if needed
- Complete testing and documentation

## Integration Points

### Phase 1 → Phase 2
- PHP LSP will depend on BasicLsp crate
- DocumentManager from Phase 1 will be extended for PHP-specific functionality
- Middleware system will be used for PHP analysis pipelines

### Phase 2 → Phase 3
- PHP LSP binary will be packaged for Zed distribution
- Configuration system from Phase 2 will integrate with Zed settings
- Project detection will inform Zed extension behavior

## Validation Criteria

### After Phase 1
- Basic LSP functionality with document management works
- Pluggable middleware system functions correctly
- Encoding handling works properly
- Ready for language-specific extensions

### After Phase 2
- Complete PHP LSP functionality implemented
- WordPress hook system working
- Composer integration functional
- Performance meets requirements

### After Phase 3
- Zed extension installs and runs properly
- Automatic binary management works across platforms
- User experience is smooth and intuitive
- All features accessible through Zed interface

## Risk Mitigation
- Regular testing at each week's end
- Early detection of integration issues
- Maintaining backward compatibility
- Performance monitoring throughout development
- Security reviews at each phase
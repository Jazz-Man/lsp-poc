# Implementation Roadmap

This roadmap outlines the chronological implementation of the PHP LSP project across the three main steps, highlighting dependencies and integration points.

## Critical Implementation Guidelines for ALL Phases
**DO NOT invent APIs. Use ONLY what exists in the documentation.**

### Implementation Contract (CRITICAL!)

For EACH task in the entire project:

**Read docs** for the APIs you'll use
**Write MAX 20-30 lines** of code
**Run:** cargo check
**If error → FIX IMMEDIATELY** (not "out of scope"!)
**Run:** cargo check (must pass)
**Run:** git add -A && git commit -m "task: description"
**Only then → next task**

### ABSOLUTE RULES

✗ NEVER write more than 30 lines without cargo check
✗ NEVER proceed with compilation errors
✗ NEVER invent APIs not in documentation
✗ NEVER skip commits between tasks
✗ NEVER say "errors exist but out of scope"
✗ NEVER implement features not in the current plan

✓ ALWAYS read docs before writing code
✓ ALWAYS cargo check after every change
✓ ALWAYS fix errors immediately
✓ ALWAYS commit working code
✓ ALWAYS follow the task order from the plans

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
# STEP 2: PHP LSP Implementation

## Critical Implementation Guidelines
**DO NOT invent APIs. Use ONLY what exists in the documentation.**

### Implementation Contract (CRITICAL!)

For EACH task in this plan:

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
✓ ALWAYS follow the task order from this plan

### Example Workflow

# Task 1: Add PHP parser integration
# 1. Read tree-sitter-php docs
cat target/doc-md/tree_sitter/index.md

# 2. Write ~20 lines for parser integration
# 3. Check
cargo check

# 4. If OK, commit
git add -A && git commit -m "feat: add PHP parser integration"

# Task 2: Add PHPDoc analyzer
# Repeat process...

### Research API Before Implementation

Before implementing, you must understand the available APIs.

### Step 1: List available crates
cat target/doc-md/index.md

### Step 2: Read the main crate you'll use
cat target/doc-md/{crate}/index.md

### Step 3: Find specific types/functions
grep -r "TypeName" target/doc-md/{crate}/

### Step 4: Check crate info
cargo info {crate}

### Step 5: Look at examples in the crate repo

### Fix Compilation Error

There's a compilation error. Let me fix it properly.

### Step 1: Understand the error
cargo check 2>&1 | head -50

### Step 2: Read relevant documentation
The error mentions a specific type/function. Let me check the docs:
```bash
# Find the crate
cat target/doc-md/index.md

# Read the specific crate docs
cat target/doc-md/{crate}/index.md
```

### Step 3: Fix ONE error at a time
I will:
1. Fix only the FIRST error
2. Run cargo check
3. If more errors, repeat

### Step 4: Commit the fix
git add -A && git commit -m "fix: {description of what was fixed}"

DO NOT try to fix everything at once. One error at a time.

## Overview
Build a comprehensive PHP Language Server on top of the basic LSP framework created in STEP 1. This implementation should include all requested features including WordPress hook support, PHPDoc parsing, and composer integration.

## Architecture Design

### Core Components
- **PhpLspService**: Implementation of the basic LSP framework for PHP
- **PhpParser**: Integration with tree-sitter-php for syntax analysis
- **PhpDocAnalyzer**: PHPDoc parsing using phpstan/phpdoc-parser
- **WordPressHookProcessor**: Specialized module for WordPress hook system analysis
- **ComposerIntegration**: Composer.json parsing and PHP version detection
- **PhpSymbolIndexer**: Symbol indexing and cross-references for PHP code

### PHP-Specific Features
- Complete LSP protocol support (diagnostics, completions, hover, go-to-definition, etc.)
- PHPDoc parsing and analysis for type inference
- Support for PHP 8+ features (attributes, union types, etc.)
- WordPress hook system support (go-to-definition, find usages)
- Composer integration (PHP version detection, extension warnings)
- PHPStan/PSALM diagnostic integration

## Technology Stack
- Rust 2021 edition
- BasicLsp framework from STEP_1
- tree-sitter-php for PHP parsing
- phpstan/phpdoc-parser for PHPDoc parsing
- serde_json for JSON parsing (composer.json)
- globset for file pattern matching
- reqwest for HTTP requests (package information)

## Implementation Strategy

### Phase 1: PHP Parser Integration
1. Integrate tree-sitter-php with the BasicLsp framework
   - Create PhpDocument implementing TreeDocument trait
   - Implement PHP-specific node queries
   - Add utilities for PHP syntax tree analysis
2. Create PHP-specific node query functions
   - Find class/function definitions
   - Parse variable scopes
   - Identify PHPDoc comments
3. Implement incremental tree updates for PHP code changes
4. Add tests for PHP parsing functionality

### Phase 2: PHPDoc Analysis
1. Integrate phpstan/phpdoc-parser for PHPDoc analysis
2. Create PhpDocAnalyzer for type inference
3. Implement type resolution for generics, unions, etc.
4. Connect PHPDoc analysis to completion and hover functionality
5. Add tests for PHPDoc parsing and type inference

### Phase 3: Symbol Indexing
1. Implement PhpSymbolIndexer for symbol cross-references
2. Create index for classes, functions, methods, properties
3. Implement go-to-definition functionality
4. Implement find-references functionality
5. Add support for workspace-wide symbol search
6. Add tests for symbol indexing and navigation

### Phase 4: Diagnostics
1. Implement basic syntax error diagnostics
2. Create integration points for external tools (phpstan, psalm)
3. Implement PHP version-specific diagnostics
4. Add checks for PHP extension usage vs composer.json requirements
5. Add tests for diagnostic functionality

### Phase 5: WordPress Hook System Support
1. Analyze WordPress hook patterns (add_action, add_filter, etc.)
2. Create WordPressHookProcessor to identify and index hooks
3. Implement go-to-definition for WordPress hooks
4. Implement find usages for WordPress hooks
5. Handle custom hooks defined in plugins/themes
6. Add tests for WordPress hook functionality

### Phase 6: Composer Integration
1. Parse composer.json files to detect PHP version requirements
2. Adjust code completion and diagnostics based on detected PHP version
3. Warn about PHP extension usage not declared in composer.json
4. Support for specifying different PHP binary versions
5. Add tests for composer integration

### Phase 7: Completion and Hover
1. Implement context-aware PHP code completion
2. Integrate PHPDoc information into hover functionality
3. Add completion for PHP built-ins based on detected PHP version
4. Implement WordPress-specific completion (hook names, etc.)
5. Add tests for completion and hover functionality

## Documentation and Testing

### Using Documentation Generation
Before implementing each feature:
- Run `.scripts/regen-docs.sh` to update documentation
- Read relevant crate docs: `target/doc-md/async_lsp/index.md`, `target/doc-md/tree_sitter/index.md`, `target/doc-md/lsp_types/index.md`, `target/doc-md/ropey/index.md`, `target/doc-md/basic_lsp/index.md`
- Ensure implementation follows current API patterns

### Testing Strategy
- Unit tests for PHP parsing functions
- Integration tests for symbol indexing
- End-to-end tests with real PHP code
- WordPress-specific functionality tests
- Composer integration tests
- Performance benchmarks for large PHP projects

## Security Considerations
- Safe parsing of potentially malicious PHP code
- Validate all file paths to prevent directory traversal
- Sanitize all user inputs to prevent injection attacks
- Proper isolation when executing external tools (phpstan, psalm, etc.)

## Documentation Standards
- Complete API documentation
- Configuration guide for PHP-specific settings
- WordPress hook system usage documentation
- Integration guide with external tools
- Performance tuning guide for large codebases

## Deliverables
- PhpLspService implementation
- PHP-specific document handling with tree-sitter integration
- PHPDoc analysis and type inference
- WordPress hook system support
- Composer integration
- Complete LSP functionality (diagnostics, completions, hover, etc.)
- Comprehensive test suite
- Performance benchmarks

## Next Steps Preparation
- Ensure clean interfaces for Zed integration
- Prepare for packaging and distribution
- Plan for updating and maintaining PHP language support
- Consider support for additional frameworks beyond WordPress
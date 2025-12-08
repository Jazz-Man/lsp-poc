# STEP 2: PHP LSP Implementation

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

### Development Workflow Example

# Task: Add PHP parser integration
1. # Read tree-sitter-php documentation
   cat target/doc-md/tree_sitter/index.md
2. # Implement PHP parser (~20 lines)
3. # Validate implementation
   cargo check
4. # Commit working code
   git add -A && git commit -m "feat: add PHP parser integration"

### Error Resolution Protocol

When compilation errors occur:

1. **IDENTIFY:** First error in `cargo check` output
2. **RESEARCH:** Check relevant documentation in target/doc-md/
3. **FIX:** Address ONLY the first error
4. **VERIFY:** Run `cargo check` again
5. **REPEAT:** Until compilation succeeds
6. **COMMIT:** Run `git add -A && git commit -m "fix: {description}"`

### Documentation Research Steps

1. # Update documentation
   .scripts/regen-docs.sh
2. # List available crates
   cat target/doc-md/index.md
3. # Research specific crate
   cat target/doc-md/{crate}/index.md
4. # Search for specific functionality
   grep -r "Pattern" target/doc-md/{crate}/

### Dependency Verification Protocol

Before adding ANY new dependency:

1. # Check crate information
   cargo info {crate_name}
2. # Review available features
   cargo info {crate_name} --features
3. # Verify latest version stability
   # Ensure using stable version (not beta/alpha)
4. # Examine dependency tree impact
   cargo tree --prune={crate_name}
5. # Check security advisories
   cargo audit || echo "Install cargo-audit for security checking"
6. # Review source/repo if possible
   # Check last update, maintenance status, etc.

### Feature Selection Best Practices

- Enable ONLY required features to minimize dependencies
- Prefer default features when they match project needs
- Document rationale for each enabled feature
- Avoid feature combinations that introduce conflicts

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
# Kiro Instructions for STEP 2: PHP LSP Implementation

## Overview
This document provides step-by-step instructions for using Kiro commands to implement the PHP LSP according to your implementation guidelines and building on the Basic LSP Foundation.

## Phase 1: Specification Initialization
```bash
/kiro:spec-init "PHP LSP Implementation - PHP-specific language server built on Basic LSP Foundation with WordPress hook system support, PHPDoc analysis, and Composer integration"
```

## Phase 2: Requirements Generation
```bash
/kiro:spec-requirements php-lsp
```
**Note**: The requirements should capture your PHP-specific needs:
- Integration with BasicLsp framework
- PHP parsing with tree-sitter-php
- PHPDoc analysis via phpstan/phpdoc-parser
- WordPress hook system support (add_action, add_filter)
- Composer integration for dependency management
- PHP version detection and feature support

## Phase 3: Design Generation
```bash
/kiro:spec-design php-lsp -y
```
**Important**: During the design phase:
- Reference your Basic LSP Foundation's CoreLspService trait
- Design PHP-specific extensions to DocumentManager
- Plan integration with tree-sitter-php parser
- Design WordPress hook analysis system
- Plan PHPDoc parsing integration
- Design Composer integration
- Include your documentation research steps in the design decisions
- Consider dependency on phpstan/phpdoc-parser and tree-sitter-php

## Phase 4: Tasks Generation
```bash
/kiro:spec-tasks php-lsp -y
```
**Guidance**: Tasks should follow your Incremental Development Protocol:
- Each task should be small (align with your 20-30 lines guideline)
- Tasks should follow the sequence: Research -> Implement -> Validate -> Fix -> Commit
- Map all requirements to specific implementation tasks
- Include validation tasks for cargo check after each implementation
- Prioritize core PHP parsing functionality first
- Include tasks for WordPress hook system implementation
- Include tasks for PHPDoc analysis integration
- Include tasks for Composer integration

## Phase 5: Implementation Execution
```bash
/kiro:spec-impl php-lsp
```
**Critical**: During implementation, follow your exact development protocol:
1. **RESEARCH**: Read documentation using your research steps before each task
2. **IMPLEMENT**: Write MAX 20-30 lines of focused code per task
3. **VALIDATE**: Run `cargo check` - must pass with no errors
4. **FIX**: If errors exist, resolve IMMEDIATELY
5. **COMMIT**: Run `git add -A && git commit` after each task
6. **PROCEED**: Only then continue to next task

## PHP-Specific Integration Tasks
During implementation, ensure to:
- Verify tree-sitter-php API usage through documentation
- Integrate with BasicLsp framework properly
- Implement WordPress hook detection and linking
- Support PHPDoc parsing for type information
- Integrate with composer.json for project configuration
- Ensure proper PHP version feature support

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
- Research tree-sitter-php API documentation
- Study phpstan/phpdoc-parser usage patterns
- Review WordPress hook system documentation
- Check composer.json schema and parsing

## Validation and Status Check
Monitor progress with:
```bash
/kiro:spec-status php-lsp
```

## Optional Validations
You can also run additional validations during the process:
- `/kiro:validate-gap php-lsp` - to analyze gaps with existing codebase
- `/kiro:validate-design php-lsp` - for design review
- `/kiro:validate-impl php-lsp` - to validate completed implementations

## Preparation for Next Phase
Ensure the PHP LSP implementation is structured to allow easy integration with Zed editor by:
- Providing clean, well-documented APIs
- Following standard LSP protocols
- Including proper configuration options
- Planning for editor-specific extensions
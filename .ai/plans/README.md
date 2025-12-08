# PHP LSP Development Plan Overview

This document provides an overview of the three-step development plan for creating a comprehensive PHP Language Server with Zed editor integration.

## Step 1: Basic LSP Foundation (STEP_1_BASIC_LSP.md)
Create a general-purpose LSP framework with enhanced features compared to async-language-server:
- Core service layer with enhanced abstractions
- Document management with Ropey and optional tree-sitter support
- Transport layer with stdio and TCP support
- Pluggable middleware system
- Encoding handling utilities
- Configuration management system

## Step 2: PHP LSP Implementation (STEP_2_PHP_LSP.md)
Build a comprehensive PHP Language Server on top of the basic LSP framework:
- PHP parser integration with tree-sitter-php
- PHPDoc analysis using phpstan/phpdoc-parser
- Symbol indexing and cross-references
- WordPress hook system support
- Composer integration
- Complete LSP functionality (diagnostics, completions, hover, etc.)

## Step 3: Zed Editor Integration (STEP_3_ZED_INTEGRATION.md)
Create a comprehensive Zed extension for the PHP LSP:
- Automatic binary download and management
- Configuration integration with Zed
- Project type detection (Composer, WordPress)
- Slash commands for special functionality
- Cross-platform support

## Development Approach

### Critical Implementation Guidelines
**CRITICAL:** Always verify API existence before implementation. Use ONLY documented APIs from crate documentation or proven examples. Never assume API behavior.

- Use incremental development: MAX 20-30 lines followed by `cargo check`
- Leverage `.scripts/regen-docs.sh` to keep documentation up-to-date before implementing each feature
- Study Deputy example implementation patterns for Zed integration
- Follow the code style and quality standards from `.ai/code-stye-prompt.md`
- Fix compilation errors immediately - never proceed with errors
- Commit working code regularly with descriptive commit messages
- Ensure comprehensive testing at each step
- Maintain security best practices throughout development

## Success Criteria
- All three components work seamlessly together
- PHP LSP provides full functionality including WordPress hook support
- Zed integration provides a smooth user experience
- Codebase follows Rust best practices and security guidelines
- Performance benchmarks meet expectations for large PHP projects
# PHP Language Server Protocol (LSP) Server - Specification-Driven Development (SDD) Prompt

## Project Overview

This is a custom Language Server Protocol (LSP) server for PHP, implemented in Rust using `async-lsp` and following Specification-Driven Development (SDD) principles with `spec-kit-plus`. The goal is to create a lightweight, modular, and hackable LSP that provides full PHP language support comparable to PHPStorm's built-in capabilities.

## Personal Motivation & Business Case

### Personal Motivation
One of my main motivations for this project is that I've long dreamed of writing an LSP server in Rust using modern async tools. I'm also not satisfied with the current state of PHP support in existing tools. I've used [Intelephense](https://intelephense.com/) extensively — while it's quite powerful and feature-rich, it often feels heavy, opaque, and hard to extend or debug. Similarly, JetBrains IDEs, although popular, tend to be resource-intensive and somewhat rigid for modern PHP workflows. What I really want is something lightweight, modular, and hackable — a language server that's truly optimized for my workflow and that I fully control. That's what pushed me toward building a custom solution from scratch.

### Business Value & Time Savings
This custom LSP is not just a technical exercise - it's a strategic investment that will save me significant time and money:

- **License Cost Savings**: Avoiding annual subscriptions to PhpStorm (~$100-200/year) or Intelephense (~$40-100/year) by having a custom, open-source solution
- **Time Efficiency**: Having full control over the LSP means I can optimize it specifically for my workflow and the specific PHP projects I work on
- **Customization**: Tailor the LSP to my exact needs rather than paying for features I don't use in commercial solutions
- **WordPress-Specific Features**: The advanced WordPress hooks system support that only PHPStorm provides adequately (but at a high cost) will be available in my custom Zed-based setup
- **Long-term Maintainability**: Having source code control means I can maintain and extend the LSP as my needs evolve without being dependent on third-party vendors

## Technical Stack

- **Language**: Rust
- **LSP Framework**: `async-lsp` (v0.2.2)
- **Syntax Parsing**: `tree-sitter` (v0.25.10) and `tree-sitter-php` (v0.24.2)
- **SDD Framework**: `spec-kit-plus` (for specification-driven development)
- **Target Editor**: Zed.dev integration

## SDD Constitutional Principles

### Article I: Library-First Principle
- Every feature must begin as a standalone library—no exceptions
- The LSP server must be modular with clear boundaries and minimal dependencies

### Article II: CLI Interface Mandate
- All components must expose functionality through a command-line interface
- Support text input/output and JSON format for structured data exchange
- Ensure observability and testability of all functionality

### Article III: Test-First Imperative
- All implementation MUST follow strict Test-Driven Development
- No implementation code shall be written before:
  1. Unit tests are written and validated
  2. Tests are approved by the user
  3. Tests are confirmed to FAIL (Red phase)

### Articles VII & VIII: Simplicity and Anti-Abstraction
- Maximum 3 projects for initial implementation
- Use `async-lsp` framework features directly rather than wrapping them
- Single model representation for each concept

### Article IX: Integration-First Testing
- Prioritize real-world testing over isolated unit tests
- Use real LSP clients (Zed) for testing rather than mocks
- Contract tests mandatory before implementation

## Core LSP Features Required

### 1. Initialization & Configuration
- Support LSP initialization protocol with proper capability negotiation
- Configure based on project settings (composer.json PHP version, extensions, etc.)
- Support workspace configurations specific to PHP projects

### 2. Diagnostics
- Real-time syntax error detection using tree-sitter PHP parser
- Integration with static analysis tools (PHPStan, Psalm, PHPMD) for enhanced diagnostics
- Support for detecting PHP version compatibility issues based on composer.json
- Warning for extensions used but not declared in composer.json
- Custom diagnostic rules for PHP best practices and security issues

### 3. Code Completion (IntelliSense)
- Context-aware completions for variables, functions, classes, methods, and constants
- Support for PHP 8+ features (attributes, named parameters, union types, etc.)
- PHPDoc-based completions incorporating type information from documentation
- Framework-specific completions (WordPress hooks, actions, filters)
- Composer dependency-based completions

### 4. Hover Information
- Function/method signature information with parameter types and return types
- PHPDoc content display for functions, classes, methods, and properties
- Inline documentation for built-in PHP functions
- Type inference display for variables and expressions

### 5. Go-to-Definition
- Navigate to function, method, class, property, and constant definitions
- Support for WordPress hooks and filters (actions/filters declared with add_action/add_filter)
- Navigate to custom hooks defined in the codebase
- Cross-file navigation within the workspace

### 6. Document Symbols
- Hierarchical symbol outline for PHP files (classes, methods, properties, constants, functions)
- Support for namespace organization
- Ability to search and navigate to symbols within the document

### 7. Workspace Symbols
- Global symbol search across the entire workspace
- Indexing of all project symbols for fast lookup
- Support for searching classes, functions, interfaces, traits, etc.

### 8. Find References
- Locate all references to functions, methods, classes, variables, etc.
- Support for finding hook/function usage in WordPress context

### 9. Signature Help
- Parameter information during function/method calls
- Support for overloaded functions and methods
- Display of PHPDoc for parameters during signature help

### 10. Document Formatting (Future)
- Integration with code formatters (php-cs-fixer, rector)
- Support for project-specific formatting rules

## PHP-Specific Requirements

### PHP Version Support
- Automatic detection of PHP version from composer.json
- Adapt completions, syntax checking, and diagnostics based on specified PHP version
- Support for PHP 8+ features when version is 8.0 or higher

### PHPDoc Integration
- Comprehensive parsing of PHPDoc comments
- Type information extraction from `@param`, `@return`, `@var`, `@property`, etc.
- Support for complex type expressions and generics
- Integration with static analysis tools (PHPStan, Psalm)

### Framework Integration

#### WordPress Support
- Advanced completion and navigation for WordPress hooks system
- Recognition and completion of core WordPress functions, hooks, and constants
- Navigate from hook usage to hook definition (add_action/add_filter)
- Support for custom action and filter definitions
- Context-aware completions based on hook type and parameters

#### Future Framework Support
- Modular architecture to support other frameworks (Laravel, Symfony, etc.)
- Pluggable framework detection and feature enhancement

### Extension Detection
- Detect PHP extensions in use but not declared in composer.json
- Provide warnings and suggestions to update composer.json
- Support for extension-specific completions and documentation

## Architecture & Design

### Async Architecture
- Leverage `async-lsp`'s async-native design for optimal performance
- Handle multiple concurrent requests efficiently
- Maintain responsive user experience during analysis

### Modularity
- Clean, router-style handler architecture
- Pluggable analysis components
- Extensible framework support
- Configurable feature sets per project

## Zed Editor Integration

### Direct LSP Integration
- Binary executable that Zed can invoke as a language server
- Proper handling of Zed-specific LSP extensions
- Optimal performance for Zed's editing experience

### Potential Zed Extensions
- Configuration options via Zed's language settings
- Custom keybindings for PHP-specific actions
- Integration with Zed's project management features

## SDD Implementation Workflow

### Phase 1: Feature Specification
- Use `/sp.specify` command to create structured feature specifications
- Focus on WHAT users need and WHY, avoiding HOW to implement
- Include user stories and acceptance criteria
- Mark unclear requirements with [NEEDS CLARIFICATION] tags

### Phase 2: Implementation Planning
- Use `/sp.plan` command to generate comprehensive implementation plans
- Ensure constitutional compliance (Library-first, Test-first, etc.)
- Convert business requirements to technical architecture
- Generate API contracts and data models
- Create research documents for technical decisions

### Phase 3: Task Generation
- Use `/sp.tasks` command to create executable task lists
- Convert contracts and scenarios into specific tasks
- Mark independent tasks for parallelization
- Generate `tasks.md` in feature directories

### Phase 4: Implementation with SDD Principles
- Follow test-first development (Article III)
- Create libraries first, then applications (Article I)
- Ensure CLI interfaces for all components (Article II)
- Use integration testing with real LSP client (Article IX)

## Implementation Roadmap

### Phase 1: Basic LSP Protocol Support
- Initialize and shutdown
- Basic diagnostics for syntax errors
- Simple completions for local variables
- Tree-sitter based syntax highlighting

### Phase 2: Enhanced Analysis
- Go-to-definition for basic symbols
- Hover information
- Document symbols
- PHPDoc parsing for enhanced completions

### Phase 3: Framework Integration
- WordPress hooks system support
- Advanced completion with type inference
- Cross-file navigation
- PHP version-specific features

### Phase 4: Advanced Features
- Workspace symbols
- Find references
- Signature help
- Enhanced diagnostics integration

### Phase 5: Performance & Polish
- Performance optimization
- Caching mechanisms
- Advanced formatting support
- Extensive framework support

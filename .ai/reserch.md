# PHP LSP Project - Comprehensive Research Summary

## 1. Project Overview and Requirements

Your goal is to create a fully functional PHP LSP server in Rust using async-lsp. This is an ambitious project with comprehensive requirements:

- Complete LSP functionality (diagnostics, completions, hover, go-to-definition, etc.)
- PHPDoc parsing support for tools like phpstan and psalm
- Support for modern PHP 8+ features
- PHP version compatibility detection via composer.json
- Framework-specific support, particularly for WordPress hooks system
- Integration with Zed editor
- Potential for code formatting, though not a primary concern initially

## 2. Technical Foundation

### 2.1 async-lsp Crate
The async-lsp crate is an excellent choice for your implementation. It's built on the tower ecosystem and offers:
- Modern async design with built-in middleware system
- LspService trait for both servers and clients
- Built-in concurrency handling and panic catching
- Tracing integration for debugging
- Support for both stdio and TCP transports

It stands out from alternatives like tower-lsp and lsp-server with its modular architecture and better async support, making it ideal for your custom solution.

### 2.2 async-language-server Reference Project
The async-language-server project provides a valuable abstraction layer on top of async-lsp that would be very beneficial for your PHP LSP:
- Handles incremental document updates using Ropey library
- Automatically manages encoding negotiation (UTF-8, UTF-16, UTF-32)
- Optional Tree-sitter integration for syntax tree management
- Reduces boilerplate code significantly
- Supports multi-language architecture

Note: The author explicitly states this crate is not intended for public consumption and won't be published to crates.io, so you'd need to use it as a git dependency or fork it.

## 3. Architecture Based on Deputy Example

Based on the Deputy project analysis, a recommended architecture for the PHP LSP would be:

### 3.1 Multi-crate Structure
```
php-lsp/
├── crates/
│   ├── php-lsp/                 # Main LSP binary
│   ├── php-parser/              # Tree-sitter based PHP parsing
│   ├── php-tools/               # PHP analysis logic
│   ├── php-clients/             # API clients for external services
│   └── php-versioning/          # PHP versioning logic
└── editors/
    ├── zed/
    └── vscode/
```

### 3.2 Server Implementation
Using async-language-server's Server trait:
- Implement core LSP methods (hover, completion, diagnostics, etc.)
- Use DocumentMatcher for identifying PHP files
- Leverage tree-sitter-php for syntax parsing

### 3.3 Document and Syntax Handling
- Use async-language-server's `Document` struct
- Leverage tree-sitter for PHP syntax analysis
- Implement custom functions for parsing PHP constructs

## 4. PHP Parsing and Analysis

### 4.1 Tree-sitter-php Parser
For parsing PHP code, tree-sitter-php provides:
- Incremental parsing capabilities (efficient updates as users type)
- Integration with the async-language-server's Tree-sitter support
- AST generation for robust code analysis
- Good performance characteristics for language server operations

### 4.2 PHPDoc Support
PHPDoc parsing is critical for your requirements, especially for tools like phpstan and psalm. Key libraries to consider:
- phpstan/phpdoc-parser: Well-maintained library that represents PHPDocs with an AST
- This will be essential for type inference and intelligent code completion

## 5. Existing PHP LSP Implementations Analysis

### 5.1 PHPActor
- General-purpose PHP development tool with refactoring and introspection
- Provides LSP capabilities, code completion, and navigation
- **Limitation**: No specific WordPress hook system support mentioned
- Not framework-specific, which fits your general approach but doesn't solve the WordPress hook challenge

### 5.2 FelixFBEcker PHP Language Server
- Pure PHP implementation with comprehensive LSP features
- Uses Tolerant PHP Parser and phpDocumentor's DocBlock reflection
- Supports indexing of project files and vendor dependencies
- **Relevance**: Good reference for general PHP LSP capabilities but may not address WordPress-specific hook challenges

## 6. WordPress Hook System Challenge

This is a critical point for your requirements. The main challenge is:
- Existing LSPs treat WordPress functions like `add_action` as generic functions taking string parameters
- They don't understand the semantic relationship between action names, filter names, and function callbacks
- No ability to navigate from hook calls to their definitions
- No recognition of custom hooks defined in plugins/themes

**Potential Solution Approach**: You'll need to implement custom semantic analysis that:
1. Identifies WordPress hook patterns in code
2. Creates virtual symbols for action/filter names
3. Establishes relationships between hook declarations and usage
4. Indexes custom hooks defined throughout the WordPress project

### 6.1 Implementation Strategy for WordPress Hooks
Based on tree-sitter parsing:
- Create custom functions to identify hook patterns in PHP AST
- Build an internal index of hook definitions and usages
- Implement custom go-to-definition for hook-related functions
- Track callback functions and their relationship to hooks

## 7. PHP Version and Composer Integration

Your requirements for PHP version compatibility are well-defined:
- Parse composer.json to detect required PHP version
- Adapt code completion and diagnostics based on target PHP version
- Warning when using PHP extensions not declared in composer.json
- Support for specifying different PHP binary versions

This could be implemented by:
- Parsing composer.json upon project initialization
- Maintaining configuration per workspace based on detected PHP version
- Using PHP reflection or a PHP info utility to determine available extensions vs. composer requirements

## 8. Zed Editor Integration

Following the Deputy example for Zed integration:
- Create a `editors/zed/` directory with extension implementation
- Implement the Zed extension API to download and manage the LSP binary
- Use `extension.toml` to define the language server configuration
- Support slash commands for configuration (if needed)

### 8.1 Zed Extension Structure
- `extension.toml`: Configuration file defining the language server
- `Cargo.toml`: Dependencies for the extension
- `src/extension.rs`: Main extension logic including binary download and management
- Platform-specific binary downloading and management

## 9. Implementation Strategy

Based on the research, here's a recommended approach for your PHP LSP:

1. **Foundation Layer**:
   - Use async-lsp as the core LSP implementation
   - Leverage async-language-server as a git dependency for document management
   - Integrate tree-sitter-php for parsing and syntax tree management

2. **Analysis Layer**:
   - Implement PHPDoc parsing using phpstan/phpdoc-parser
   - Build a symbol indexer for PHP code understanding
   - Add WordPress-specific semantic analysis for hook system

3. **Multi-crate Architecture**:
   - Separate parsing, analysis, and tooling logic into different crates
   - Follow Deputy's pattern for organized code structure
   - Enable easier testing and maintenance

4. **Integration Layer**:
   - Parse composer.json for PHP version and extension requirements
   - Integrate with standard PHP tooling for diagnostics (phpstan, psalm)
   - Implement Zed editor integration based on Deputy example

5. **Specialized Features**:
   - WordPress hook system navigation and completion
   - PHP version-specific feature awareness
   - Extension requirement validation

## 10. Key Challenges and Considerations

- **WordPress Hook System**: This will be the most complex part of your implementation, requiring custom semantic analysis
- **Performance**: Indexing large PHP codebases (especially with WordPress) requires careful optimization
- **PHP Version Support**: Need to account for different PHP versions and available functions/classes
- **Dependency Management**: Handling vendor dependencies and project-specific configurations appropriately
- **Zed Integration**: Implementing automatic binary management and configuration

This research provides a solid foundation for beginning your PHP LSP implementation. The async-lsp ecosystem combined with tree-sitter parsing and the architectural patterns demonstrated by the Deputy project give you the tools and structure needed for a robust implementation that can handle your specific requirements, especially the WordPress hook challenges that existing solutions struggle with.
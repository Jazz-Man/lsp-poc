# Development Constraints

## Language & Runtime
- **Rust Edition 2021**: All development must use Rust 2021 edition
- **MSRV (Minimum Supported Rust Version)**: Must maintain compatibility with specified MSRV
- **Async Runtime**: Use tokio for async operations with appropriate executor configuration

## Architecture & Dependencies
- **Workspace Structure**: Follow multi-crate workspace organization (crates/*)
- **Dependency Management**: Use workspace Cargo.toml for shared dependencies
- **External Dependencies**: Minimize dependencies; prefer pure Rust implementations where possible
- **Tree-sitter Integration**: Maintain optional tree-sitter functionality with feature flags

## Code Quality & Style
- **Documentation-First**: Always implement with comprehensive documentation
- **Incremental Development**: Write 20-30 lines maximum per commit, validate with cargo check
- **API Verification**: Always verify API existence in documentation before implementation
- **Error Handling**: Use appropriate Result and Option types consistently

## Performance Requirements
- **Efficiency**: Use Ropey for efficient text handling in document operations
- **Memory Management**: Follow Rust's ownership model strictly to avoid memory leaks
- **LSP Performance**: Maintain fast response times for LSP operations (under 100ms for most operations)

## Testing & Validation
- **Cargo Check**: Validate each incremental change with cargo check
- **Test Coverage**: Maintain comprehensive test coverage for all critical paths
- **API Compatibility**: Ensure compatibility with LSP specification and zed_extension_api
- **Cross-Platform**: Maintain compatibility across supported platforms

## Security & Safety
- **Safe Rust**: Use safe Rust practices; unsafe code requires explicit justification
- **Input Validation**: Validate all external inputs (file contents, client requests, etc.)
- **No Secrets**: Never hardcode secrets or credentials in the codebase
- **Sandbox Safety**: Maintain safe execution environment for parser operations

## Build & Distribution
- **Cargo Standards**: Follow standard Cargo practices for builds and distribution
- **Feature Flags**: Use appropriate feature flags for optional functionality (tree-sitter integration)
- **Binary Size**: Optimize for reasonable binary size while maintaining functionality
- **Build Times**: Maintain reasonable build times for development workflow
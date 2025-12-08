# STEP 3: Zed Editor Integration

## Critical Implementation Guidelines

### Documentation-First Development Approach
**CRITICAL:** Always verify API existence before implementation. Use ONLY documented APIs from crate documentation. Never assume API behavior.

Before writing any code:
1. Study Deputy example implementation patterns
2. Locate Zed Extension API documentation and patterns
3. Read zed_extension_api documentation: `cat target/doc-md/zed_extension_api/index.md` (if exists)
4. Confirm API signatures and usage patterns

### Incremental Development Protocol
For EACH implementation task, follow this exact sequence:

1. **RESEARCH:** Review Deputy example and Zed Extension API patterns
2. **IMPLEMENT:** Write MAX 20-30 lines of focused code
3. **VALIDATE:** Run `cargo check` - must pass with no errors
4. **FIX:** If errors exist, resolve IMMEDIATELY (not later)
5. **COMMIT:** Run `git add -A && git commit -m "task: description"`
6. **PROCEED:** Only then continue to next task

### Absolute Development Constraints

✗ STOP if compilation fails - fix immediately
✗ STOP if API doesn't exist in examples - research alternatives
✗ STOP after 30 lines - validate before continuing
✗ STOP if deviating from planned tasks - return to plan
✗ STOP if errors occur - resolve before proceeding

✓ VERIFY patterns in Deputy example before each code section
✓ RUN cargo check after every change
✓ COMMIT working code regularly
✓ FOLLOW plan task order precisely
✓ IMPLEMENT ONE feature at a time

### Development Workflow Example

# Task: Add Zed extension structure
1. # Study Deputy extension implementation
   # Review src/extension.rs pattern
2. # Implement basic extension structure (~20 lines)
3. # Validate implementation
   cargo check
4. # Commit working code
   git add -A && git commit -m "feat: add basic Zed extension structure"

### Error Resolution Protocol

When compilation errors occur:

1. **IDENTIFY:** First error in `cargo check` output
2. **RESEARCH:** Check Deputy example implementation patterns
3. **FIX:** Address ONLY the first error
4. **VERIFY:** Run `cargo check` again
5. **REPEAT:** Until compilation succeeds
6. **COMMIT:** Run `git add -A && git commit -m "fix: {description}"`

### Documentation Research Steps

1. # Study Deputy extension structure
   cat /Users/vasilsokolik/www/php-lsp-qwen/tmp/deputy/editors/zed/src/extension.rs
2. # Review Zed Extension API documentation
   # Check zed_extension_api patterns
3. # Identify relevant API usage patterns
   grep -r "LanguageServer" /Users/vasilsokolik/www/php-lsp-qwen/tmp/deputy/editors/zed/src/

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
Create a comprehensive Zed extension for the PHP LSP implementation. This extension should handle automatic binary management, configuration, and provide a seamless user experience within the Zed editor.

## Architecture Design

### Core Components
- **ZedExtension**: Main extension implementation using Zed Extension API
- **BinaryManager**: Automatic download, caching, and management of PHP LSP binary
- **ConfigurationManager**: Integration with Zed's configuration system
- **PlatformManager**: Cross-platform binary handling (Linux, macOS, Windows)
- **AuthProvider**: Optional authentication for external services
- **CommandHandler**: Handle Zed-specific commands and slash commands

### Extension Features
- Automatic PHP LSP binary download and management
- PHP binary detection and configuration
- Composer project detection and integration
- WordPress project detection and configuration
- Zed-specific settings for PHP LSP
- Slash commands for special functionality

## Technology Stack
- Rust 2021 edition
- Zed Extension API (zed_extension_api 0.6 or latest)
- reqwest for binary downloading
- zip for extraction of binaries
- serde for configuration handling

## Implementation Strategy

### Phase 1: Basic Extension Structure
1. Create the Zed extension project structure
   - extension.toml configuration file
   - Cargo.toml with zed_extension_api dependency
   - Main extension.rs implementation
2. Implement basic Extension trait methods
   - new() constructor
   - language_server_command() for starting the LSP
3. Create basic binary download and caching mechanism
4. Add tests for extension functionality

### Phase 2: Binary Management
1. Implement PlatformDescriptor for cross-platform support
   - Detect OS (Windows, macOS, Linux)
   - Detect architecture (x86_64, aarch64)
   - Handle executable extensions (.exe for Windows)
2. Create BinaryManager for automatic binary handling
   - Check for local binary first
   - Download latest release from GitHub if needed
   - Cache downloaded binaries
   - Handle binary permissions
3. Implement update checking functionality
4. Add tests for binary management

### Phase 3: Configuration Integration
1. Integrate with Zed's configuration system
   - Support for PHP binary path configuration
   - Composer project settings
   - WordPress-specific settings
   - PHP version detection settings
2. Implement configuration validation
3. Handle dynamic configuration updates
4. Add tests for configuration handling

### Phase 4: Project Detection
1. Implement project type detection
   - Detect Composer projects (composer.json)
   - Detect WordPress installations
   - Configure PHP LSP accordingly based on project type
2. Create project-specific initialization
3. Add project type detection caching
4. Add tests for project detection

### Phase 5: Zed-Specific Features
1. Implement slash commands for special functionality
   - `/php-set-php-binary` - Set PHP binary path
   - `/php-set-composer-config` - Configure composer settings
   - `/php-refresh-index` - Refresh symbol index
2. Implement Zed-specific settings handling
3. Add status reporting during LSP startup
4. Add tests for slash commands and settings

### Phase 6: Advanced Integration
1. Implement authentication support if needed
   - For external services (packages, documentation, etc.)
   - Secure token storage
   - Token validation
2. Add error handling for network failures
3. Create proper user feedback during binary download
4. Add tests for authentication and error handling

## Documentation and Testing

### Using Documentation Generation
Before implementing each feature:
- Review Zed Extension API documentation
- Understand current patterns from Deputy example
- Ensure implementation follows Zed's best practices

### Testing Strategy
- Unit tests for binary management
- Mock tests for network operations (binary downloads)
- Integration tests with Zed's test infrastructure (if available)
- Manual testing across different platforms
- Performance tests for binary download and startup

## Security Considerations
- Verify binary integrity after download (checksums)
- Secure storage of authentication tokens
- Validate all configuration inputs
- Proper isolation of binary execution
- Secure handling of file paths

## Documentation Standards
- Extension configuration guide
- Troubleshooting documentation
- Platform-specific installation notes
- Integration guide with the PHP LSP
- Slash commands reference

## Deliverables
- Complete Zed extension implementation
- Automatic binary management
- Configuration integration
- Project type detection
- Slash commands
- Comprehensive test suite
- Installation and usage documentation

## Distribution Preparation
- Package extension for Zed marketplace
- Prepare release notes for different versions
- Create update mechanism for the extension itself
- Prepare documentation for users
- Plan for ongoing maintenance and updates

## Next Steps Preparation
- Ensure backward compatibility for future PHP LSP updates
- Plan for potential integration with other editors
- Prepare for community contributions and feedback
- Set up proper release and versioning strategy
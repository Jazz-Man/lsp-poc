# Documentation-First Development Approach

## Core Principle
Always verify API existence before implementation. Use ONLY documented APIs from crate documentation. Never assume API behavior.

## Pre-Implementation Workflow

### 1. Generate/Verify Documentation
- Run `.scripts/regen-docs.sh` before starting any implementation
- Ensure documentation is up-to-date with current codebase
- Verify that the documentation generation completes successfully

### 2. Locate Relevant Crate Documentation
- Check overall documentation index: `cat target/doc-md/index.md`
- Identify relevant crates for the task at hand
- Understand the scope and capabilities of available crates

### 3. Read Specific API Documentation
- Access detailed API docs: `cat target/doc-md/{crate}/index.md`
- Review specific module and function documentation
- Understand the complete API surface before implementation

### 4. Confirm API Signatures and Usage Patterns
- Verify function signatures, parameters, and return types
- Check example usage in the documentation
- Understand error handling patterns for the APIs
- Confirm feature flags if applicable

## Documentation Verification Checklist
- [ ] Documentation generation completed successfully
- [ ] Target API exists in documentation
- [ ] API signature matches implementation needs
- [ ] Error handling approach understood
- [ ] Example usage available and clear
- [ ] Feature flags understood (if applicable)

## Implementation Rules
- Never implement using undocumented APIs
- If documentation is unclear, clarify before implementing
- When APIs are missing, create an issue rather than assuming behavior
- Always reference documentation in code comments when using external APIs
- Update documentation if implementation changes public interfaces

## Tool Integration
- Integrate documentation verification into development workflow
- Consider adding documentation checks to CI pipeline
- Maintain up-to-date documentation as part of each feature implementation
- Use documentation generation as part of the testing process

## Error Prevention
- If uncertain about an API, research further rather than guessing
- Consult multiple sources when documentation is ambiguous
- Test API behavior in a small prototype before full implementation
- Collaborate with team members to validate API understanding

This approach ensures that all implementation is based on well-documented and confirmed APIs, reducing errors and compatibility issues during development.
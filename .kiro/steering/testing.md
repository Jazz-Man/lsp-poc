# Testing Approach

## Philosophy
- **Test-Driven Development**: Write tests first when possible, especially for critical functionality
- **Comprehensive Coverage**: Aim for high test coverage with focus on critical paths and edge cases
- **Fast Feedback**: Tests should run quickly to provide immediate feedback during development
- **Reliable Tests**: Tests should be deterministic and not depend on external factors

## Test Organization

### Directory Structure
- Place unit tests in the same file as the module being tested
- Integration tests in `tests/` directory at the crate level
- Use feature-specific test directories for complex functionality (e.g., `tests/document_struct/`)
- Separate benchmark tests in `benches/` directory

### Test Categories
- **Unit Tests**: Test individual functions and modules in isolation
- **Integration Tests**: Test interactions between multiple components
- **LSP Protocol Tests**: Verify LSP request/response compliance
- **Performance Tests**: Benchmark critical operations (document parsing, text operations)

## Testing Patterns

### Unit Testing
- Use `#[cfg(test)]` module for unit tests
- Test both success cases and error conditions
- Use descriptive test names that explain expected behavior
- Isolate external dependencies using mocking when necessary

### Integration Testing
- Test public APIs of each crate
- Verify proper integration between crates
- Test real-world usage scenarios
- Use dedicated test binaries in integration tests

### Property-Based Testing
- Use `proptest` for property-based testing of complex data structures
- Test invariants that should hold across various inputs
- Particularly useful for document structure operations and text handling

## Test Utilities
- Create test helpers in `tests/common/` for shared functionality
- Use test fixtures to set up common test scenarios
- Create mock implementations for external dependencies
- Implement test builders for complex object creation

## LSP-Specific Testing
- Test LSP message serialization/deserialization
- Verify correct handling of LSP protocol requests and responses
- Test error handling in LSP operations
- Validate document synchronization behavior

## Performance Testing
- Benchmark document parsing and text operations
- Monitor performance regressions with benchmarks
- Test performance with various document sizes
- Profile critical paths in the LSP server

## Testing Tools & Libraries
- Use `tokio::test` for async tests
- Utilize `assert_matches` for pattern matching assertions
- Use `serial_test` for tests that need to run sequentially
- Implement custom matchers when appropriate

## Continuous Testing
- Run unit tests with `cargo test` on every commit
- Execute integration tests in CI pipeline
- Run benchmarks periodically to catch performance regressions
- Use `cargo nextest` for parallel test execution when available
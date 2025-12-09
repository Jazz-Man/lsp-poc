# Kiro Instructions for STEP 3: Zed Editor Integration

## Overview
This document provides step-by-step instructions for using Kiro commands to implement the Zed Editor Integration according to your implementation guidelines and building on the PHP LSP implementation.

## Phase 1: Specification Initialization
```bash
/kiro:spec-init "Zed Editor Integration - Zed extension for PHP LSP server with automatic binary management and seamless user experience"
```

## Phase 2: Requirements Generation
```bash
/kiro:spec-requirements zed-integration
```
**Note**: The requirements should capture your Zed integration needs:
- Zed extension for PHP LSP server
- Automatic binary management (download, update, versioning)
- Configuration options for PHP LSP features
- Integration with Zed's LSP system
- Support for PHP project initialization
- User-friendly installation and update process

## Phase 3: Design Generation
```bash
/kiro:spec-design zed-integration -y
```
**Important**: During the design phase:
- Design the Zed extension architecture
- Plan automatic binary management system
- Design configuration schema for PHP LSP features
- Plan integration with Zed's extension API
- Consider user experience for installation/update process
- Include your documentation research steps in the design decisions
- Plan for handling different PHP versions and configurations

## Phase 4: Tasks Generation
```bash
/kiro:spec-tasks zed-integration -y
```
**Guidance**: Tasks should follow your Incremental Development Protocol:
- Each task should be small (align with your 20-30 lines guideline)
- Tasks should follow the sequence: Research -> Implement -> Validate -> Fix -> Commit
- Map all requirements to specific implementation tasks
- Include validation tasks for cargo check after each implementation
- Prioritize core extension architecture first
- Include tasks for binary management system
- Include tasks for configuration handling
- Include tasks for Zed API integration

## Phase 5: Implementation Execution
```bash
/kiro:spec-impl zed-integration
```
**Critical**: During implementation, follow your exact development protocol:
1. **RESEARCH**: Read documentation using your research steps before each task
2. **IMPLEMENT**: Write MAX 20-30 lines of focused code per task
3. **VALIDATE**: Run `cargo check` - must pass with no errors
4. **FIX**: If errors exist, resolve IMMEDIATELY
5. **COMMIT**: Run `git add -A && git commit` after each task
6. **PROCEED**: Only then continue to next task

## Zed-Specific Integration Tasks
During implementation, ensure to:
- Research Zed extension API documentation
- Implement automatic binary download and management
- Design clean configuration schema
- Integrate with Zed's LSP client system
- Handle different PHP versions appropriately
- Provide good user experience for installation/updates
- Implement proper error handling and feedback

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
- Research Zed extension documentation and API
- Study Zed's LSP integration patterns
- Review existing Zed extensions for best practices
- Check Zed's binary management approaches
- Study Zed's configuration system

## Validation and Status Check
Monitor progress with:
```bash
/kiro:spec-status zed-integration
```

## Optional Validations
You can also run additional validations during the process:
- `/kiro:validate-gap zed-integration` - to analyze gaps with existing codebase
- `/kiro:validate-design zed-integration` - for design review
- `/kiro:validate-impl zed-integration` - to validate completed implementations

## Integration with Previous Steps
Ensure tight integration with:
- Basic LSP Foundation (clean API usage)
- PHP LSP Implementation (proper configuration and communication)
- Verify compatibility between all three components
- Test end-to-end functionality with Zed editor

## Project Completion Considerations
- Plan for final integration testing
- Consider documentation and user guides
- Plan for distribution and publishing to Zed extension marketplace
- Consider ongoing maintenance and version compatibility
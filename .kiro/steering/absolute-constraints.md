# Absolute Development Constraints

## Stop Rules (Do Not Proceed)
- **✗ STOP if compilation fails** - Fix immediately before continuing
- **✗ STOP if API doesn't exist in docs** - Research alternatives before implementation
- **✗ STOP after 30 lines** - Validate with cargo check before continuing
- **✗ STOP if deviating from planned tasks** - Return to plan and complete current task
- **✗ STOP if errors occur** - Resolve before proceeding to next change

## Verification Rules (Always Execute)
- **✓ VERIFY documentation** before each code section - Ensure API exists and behavior is understood
- **✓ RUN cargo check** after every change - Validate compilation without running tests
- **✓ COMMIT working code** regularly - Use `git add -A && git commit -m "<TASK_NAME>: <DESCRIPTION>"`
- **✓ FOLLOW plan task order** precisely - Complete tasks in specified sequence
- **✓ IMPLEMENT ONE feature** at a time - Focus on single functionality before moving to next

## Enforcement
These constraints are absolute and non-negotiable. Every development session must adhere to these rules without exception. Any deviation from these constraints must be documented and justified before proceeding.

## Workflow Integration
- Place these constraints at the top of your development workflow
- Review before starting any implementation task
- Use as a checklist during development
- Refer to when making decisions about code changes

## Purpose
These constraints ensure code quality, maintainability, and adherence to the documentation-first approach while preventing common development pitfalls that can lead to technical debt or unstable code.
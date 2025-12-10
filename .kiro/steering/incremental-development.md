# Incremental Development

## Core Principle
For each implementation task, follow this exact sequence to ensure code quality and maintainability.

## Required Sequence

### 1. Read Documentation
- Read documentation for all APIs to be used in the task
- Verify API signatures, parameters, and expected behavior
- Understand error handling requirements for the APIs

### 2. Write MAX 20-30 Lines of Focused Code
- Implement only the specific functionality needed for the current task
- Keep changes small and focused on one concern at a time
- Write clear, well-documented code following project standards

### 3. Run `cargo check` - Must Pass with No Errors
- Validate code compilation without running tests
- Address all compilation errors immediately
- Do not continue if `cargo check` fails

### 4. Resolve Errors IMMEDIATELY
- If errors exist, resolve them immediately before proceeding
- Do not delay error resolution for "later"
- Ensure code compiles cleanly before moving forward

### 5. Commit Changes
- Run `git add -A && git commit -m "<TASK_NAME>: <DESCRIPTION>"`
- Use descriptive commit messages that explain the change
- Include the task name in the commit for traceability

### 6. Proceed to Next Task
- Only continue to the next task or additional implementation after successful commit
- Maintain a clean, working codebase at each commit point

## Benefits
- Ensures code quality at every step
- Makes debugging easier with smaller changes
- Maintains a working codebase throughout development
- Facilitates easier code reviews and rollbacks
- Reduces complexity by focusing on small, achievable goals

## Enforcement
- This sequence is mandatory for all implementation tasks
- Team members should follow this pattern consistently
- Use in both feature development and bug fixes
- Adapt sequence as needed for specific project requirements while maintaining core principles
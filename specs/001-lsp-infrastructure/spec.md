# Feature Specification: Core Infrastructure

**Feature Branch**: `001-lsp-infrastructure`
**Created**: 2025-12-03
**Status**: Draft
**Input**: User description: "Implement the foundational LSP server infrastructure that handles lifecycle management, document synchronization, and PHP parsing with tree-sitter."

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.

  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - LSP Lifecycle (Priority: P1)

As a Zed user, I want the PHP LSP to properly initialize and shutdown so that my editor integrates smoothly.

**Why this priority**: This is the foundational requirement for the LSP to work - without proper initialization and shutdown, the entire system cannot function.

**Independent Test**: The LSP server can be started, responds to initialization requests with its capabilities, and shuts down cleanly when requested.

**Acceptance Scenarios**:

1. **Given** an LSP client wants to connect to the PHP LSP, **When** the client sends an `initialize` request, **Then** the server responds with capabilities including textDocumentSync, hoverProvider, definitionProvider, referencesProvider, documentSymbolProvider, and completionProvider.
2. **Given** the LSP server is initialized, **When** the client sends a `shutdown` request, **Then** the server responds to the request and prepares for clean exit.
3. **Given** the LSP server has shutdown, **When** the client sends an `exit` notification, **Then** the server terminates its process cleanly.
4. **Given** the LSP server is running, **When** the client sends an `initialized` notification, **Then** the server handles it appropriately and is ready for document operations.

---

### User Story 2 - Document Synchronization (Priority: P2)

As a developer, I want my PHP files to be tracked by the LSP so that I get real-time feedback.

**Why this priority**: Core functionality for the LSP once initialized - it needs to track document state to provide language features.

**Independent Test**: The LSP can open, track changes to, and close PHP documents correctly, maintaining internal state that reflects the current content.

**Acceptance Scenarios**:

1. **Given** a PHP document is not yet open in the LSP, **When** the client sends a `textDocument/didOpen` notification, **Then** the LSP stores the document content and is ready to provide language services.
2. **Given** a PHP document is open in the LSP, **When** the client sends a `textDocument/didChange` notification, **Then** the LSP updates the content using incremental sync.
3. **Given** a PHP document is open in the LSP, **When** the client sends a `textDocument/didClose` notification, **Then** the LSP cleans up the document from its internal state.

---

### User Story 3 - PHP Parsing (Priority: P3)

As a developer, I want PHP files to be parsed correctly so that all language features work.

**Why this priority**: Essential for providing intelligent language features - the LSP needs to understand the structure of the PHP code.

**Independent Test**: The LSP can parse a PHP document and create an AST representation that other features can use for analysis.

**Acceptance Scenarios**:

1. **Given** a PHP document is opened or changed, **When** the document content is received, **Then** the LSP parses the content with tree-sitter and caches the AST with the document version.
2. **Given** a PHP document contains syntax errors, **When** the document is parsed, **Then** the LSP handles parse errors gracefully by creating a partial AST.
3. **Given** a PHP document uses PHP 8+ syntax, **When** the document is parsed, **Then** the LSP supports parsing features like attributes, named arguments, and match expressions.

---

### User Story 4 - stdio Transport (Priority: P4)

As a Zed extension, I want the LSP to communicate via stdio so that it integrates with Zed.

**Why this priority**: Required for integration with Zed editor - without proper communication transport, the LSP cannot function in the target environment.

**Independent Test**: The LSP can start with `--stdio` flag and properly read requests from stdin and write responses to stdout.

**Acceptance Scenarios**:

1. **Given** the LSP binary is executed, **When** started with `--stdio` command line flag, **Then** it uses stdio as its communication transport.
2. **Given** the LSP is running in stdio mode, **When** a JSON-RPC message is received on stdin, **Then** the LSP processes the request appropriately.
3. **Given** the LSP processes a request in stdio mode, **When** generating a response, **Then** the LSP writes the JSON-RPC response to stdout.

### Edge Cases

- What happens when the LSP receives malformed JSON-RPC messages?
- How does the system handle documents with very large file sizes?
- What happens when the LSP encounters PHP code with severe syntax errors?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: System MUST respond to `initialize` requests with server capabilities that include textDocumentSync, hoverProvider, definitionProvider, referencesProvider, documentSymbolProvider, and completionProvider.
- **FR-002**: System MUST handle `initialized` notifications appropriately after initialization.
- **FR-003**: System MUST respond to `shutdown` requests with a valid response before preparing to exit.
- **FR-004**: System MUST exit cleanly when receiving an `exit` notification after shutdown.
- **FR-005**: System MUST handle `textDocument/didOpen` notifications by storing document content in memory.
- **FR-006**: System MUST handle `textDocument/didChange` notifications by updating document content using incremental synchronization.
- **FR-007**: System MUST handle `textDocument/didClose` notifications by cleaning up document state from memory.
- **FR-008**: System MUST parse PHP documents using tree-sitter when opened or changed and cache the resulting AST.
- **FR-009**: System MUST accept a `--stdio` command line flag to enable stdio transport mode.
- **FR-010**: System MUST read JSON-RPC requests from stdin when running in stdio mode.
- **FR-011**: System MUST write JSON-RPC responses to stdout when running in stdio mode.
- **FR-012**: System MUST handle parse errors gracefully by creating a partial AST representation.
- **FR-013**: System MUST support PHP 8+ syntax including attributes, named arguments, and match expressions.
- **FR-014**: System MUST log events using structured tracing instead of basic print statements.
- **FR-015**: System MUST support PHP 7.4 and above, with focus on PHP 8.x features.
- **FR-016**: System MUST return partial results with error notifications when possible, with graceful degradation.

### Key Entities *(include if feature involves data)*

- **Document**: Represents a PHP file being tracked by the LSP, containing: URI, version number, content (as a rope data structure), and the parsed AST.
- **AST (Abstract Syntax Tree)**: Tree-sitter generated representation of the parsed PHP document structure.
- **LSP Server**: The main server process that handles LSP protocol requests and responses.
- **Transport Layer**: Manages communication between the LSP server and the client (e.g. Zed editor) via stdio.

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: LSP server initializes and responds to `initialize` requests in under 5 seconds.
- **SC-002**: Document synchronization operations (open, change, close) complete in under 100ms for documents up to 10,000 lines.
- **SC-003**: 95% of valid PHP 8+ documents parse successfully without errors.
- **SC-004**: Memory usage remains under 200MB when tracking 50 documents simultaneously.
- **SC-005**: LSP server can be launched with `--stdio` flag and maintains communication with the editor without crashes for 8+ hours of continuous use.
- **SC-006**: LSP server can handle up to 1,000 documents simultaneously with minimal performance degradation.

## Clarifications

### Session 2025-12-03

- Q: Security & authentication requirements → A: Authentication not required for local LSP communication - focus on communication security
- Q: Performance and scalability targets → A: Support up to 1,000 documents simultaneously with minimal performance degradation
- Q: Error handling strategy → A: Return partial results with error notifications when possible, with graceful degradation
- Q: Supported PHP versions → A: Full support for PHP 7.4 and above, with focus on PHP 8.x features
- Q: Logging and observability → A: Structured logs with tracing for performance metrics, error rates, and usage patterns

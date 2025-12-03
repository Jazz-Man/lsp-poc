# Data Model: Core Infrastructure

## Entities

### Document
- **uri**: String - Unique identifier for the document (typically file path)
- **version**: i32 - Version number for tracking document changes
- **content**: Rope - The actual document content using rope data structure
- **ast**: Option<Tree> - The parsed Abstract Syntax Tree, if parsing was successful

**Relationships**: None directly

**Validation Rules**:
- URI must be a valid document identifier
- Version must be non-negative
- Content must be valid UTF-8

**State Transitions**:
- Created when document is opened via `textDocument/didOpen`
- Updated when changes received via `textDocument/didChange`
- Removed when document is closed via `textDocument/didClose`

### AST (Abstract Syntax Tree)
- **tree**: Tree - The parsed tree-sitter tree structure
- **version**: i32 - Version number corresponding to the document version
- **errors**: Vec<SyntaxError> - List of syntax errors encountered during parsing

**Relationships**: Related to one Document

**Validation Rules**:
- Tree must have valid root node
- Version must match the document version it was created from
- Errors list should be empty for successfully parsed documents

### LSP Server State
- **documents**: DashMap<Url, Document> - Concurrent map of open documents
- **capabilities**: ServerCapabilities - Server capabilities to report to the client
- **is_initialized**: bool - Flag indicating if server is initialized
- **is_shutting_down**: bool - Flag indicating if server is shutting down

**Relationships**: Contains multiple Documents

**Validation Rules**:
- Server must be initialized before handling most requests
- Server must not accept new requests after shutdown initiated

### Transport Message
- **jsonrpc**: String - JSON-RPC version (should be "2.0")
- **id**: Option<RequestId> - Request ID for matching responses to requests
- **method**: String - LSP method name (initialize, textDocument/didChange, etc.)
- **params**: Option<Value> - Request parameters

**Relationships**: None

**Validation Rules**:
- JSON-RPC version must be "2.0"
- Method names must be valid LSP methods
- Parameters must match the expected schema for each method
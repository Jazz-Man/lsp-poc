# LSP API Contracts: Core Infrastructure

## LSP Lifecycle Contracts

### initialize Request
- **Method**: `initialize`
- **Direction**: Client → Server
- **Params**: InitializeParams (includes client capabilities, root path, etc.)
- **Response**: InitializeResult with server capabilities
- **Contract**: Server must respond with capabilities including textDocumentSync, hoverProvider, definitionProvider, referencesProvider, documentSymbolProvider, and completionProvider

### initialized Notification
- **Method**: `initialized`
- **Direction**: Client → Server
- **Params**: None
- **Response**: None (notification)
- **Contract**: Server should prepare for document operations after receiving this

### shutdown Request
- **Method**: `shutdown`
- **Direction**: Client → Server
- **Params**: None
- **Response**: null
- **Contract**: Server must respond to indicate readiness for exit, but not exit yet

### exit Notification
- **Method**: `exit`
- **Direction**: Client → Server
- **Params**: None
- **Response**: None (notification)
- **Contract**: Server must terminate its process cleanly after receiving this

## Document Synchronization Contracts

### textDocument/didOpen Notification
- **Method**: `textDocument/didOpen`
- **Direction**: Client → Server
- **Params**: DidOpenTextDocumentParams with document URI, languageId, version, and text
- **Response**: None (notification)
- **Contract**: Server must store document content and be ready to provide language services

### textDocument/didChange Notification
- **Method**: `textDocument/didChange`
- **Direction**: Client → Server
- **Params**: DidChangeTextDocumentParams with document URI, version, and changes
- **Response**: None (notification)
- **Contract**: Server must update content using incremental sync

### textDocument/didClose Notification
- **Method**: `textDocument/didClose`
- **Direction**: Client → Server
- **Params**: DidCloseTextDocumentParams with document URI
- **Response**: None (notification)
- **Contract**: Server must clean up document from internal state

## Error Handling Contracts

### Response Format for Errors
- **Structure**: { "jsonrpc": "2.0", "id": <request_id>, "error": { "code": <error_code>, "message": <error_message> } }
- **Contract**: When errors occur, server must respond with proper error format according to LSP specification

### Partial Result Handling
- **Contract**: Server must return partial results with error notifications when possible, with graceful degradation rather than complete failure
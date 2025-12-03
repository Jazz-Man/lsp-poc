//! Document Synchronization Handlers
//! 
//! This module handles document synchronization notifications:
//! - textDocument/didOpen
//! - textDocument/didChange
//! - textDocument/didClose

use async_lsp::ResponseError;
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    TextDocumentContentChangeEvent,
};
use url::Url;

use crate::server::types::{Document, LspServerState};

/// Handles the `textDocument/didOpen` notification
pub async fn handle_did_open(
    state: &LspServerState,
    params: DidOpenTextDocumentParams,
) -> Result<(), ResponseError> {
    tracing::info!("Processing textDocument/didOpen for URI: {}", params.text_document.uri);

    let uri = params.text_document.uri;
    let version = params.text_document.version;
    let content = params.text_document.text;

    // Check document size and log if it's large
    let content_len = content.len();
    if content_len > 10000 {
        tracing::info!("Opening large document ({} chars): {}", content_len, uri);
    }

    // Create a new Document instance
    let document = Document {
        uri: uri.clone(),
        version,
        content: ropey::Rope::from(content),
        ast: None, // AST will be generated when needed
    };

    // Store the document in the server state
    {
        let mut server_data = state.write().await;

        // Check if we're approaching the document limit
        if server_data.documents.len() >= 1000 {
            tracing::warn!("Server is approaching document limit (1000 documents)");
        }

        server_data.documents.insert(uri.clone(), document);
    }

    tracing::info!("Document opened successfully: {}", uri);
    Ok(())
}

/// Handles the `textDocument/didChange` notification
pub async fn handle_did_change(
    state: &LspServerState,
    params: DidChangeTextDocumentParams,
) -> Result<(), ResponseError> {
    tracing::info!("Processing textDocument/didChange for URI: {}", params.text_document.uri);
    
    let uri = params.text_document.uri;
    let changes = params.content_changes;
    
    // Get the current document
    let mut server_data = state.write().await;
    if let Some(mut document) = server_data.documents.get_mut(&uri) {
        // Apply each change incrementally
        for change in changes {
            apply_text_change(document, change)?;
        }
        
        // Update the document version
        document.version = params.text_document.version;
        
        // Clear the AST since the document content has changed
        document.ast = None;
        
        tracing::info!("Document changes applied successfully: {}", uri);
    } else {
        tracing::warn!("Attempted to change non-existent document: {}", uri);
        return Err(ResponseError::new(
            async_lsp::ErrorCode::InvalidParams,
            format!("Document not found: {}", uri),
        ));
    }
    
    Ok(())
}

/// Applies a text change to a document
fn apply_text_change(
    document: &mut Document,
    change: TextDocumentContentChangeEvent,
) -> Result<(), ResponseError> {
    match change.range {
        Some(range) => {
            // Convert LSP range to rope indices and apply replacement
            let start_idx = lsp_pos_to_rope_idx(&document.content, range.start);
            let end_idx = lsp_pos_to_rope_idx(&document.content, range.end);
            
            if start_idx <= end_idx && start_idx <= document.content.len_bytes() {
                document.content.remove(start_idx..end_idx);
                document.content.insert(start_idx, &change.text);
            } else {
                tracing::warn!("Invalid range for text change, replacing entire document");
                document.content = ropey::Rope::from(change.text);
            }
        }
        None => {
            // Full document replacement
            document.content = ropey::Rope::from(change.text);
        }
    }
    
    Ok(())
}

/// Converts LSP position to rope index
fn lsp_pos_to_rope_idx(content: &ropey::Rope, pos: lsp_types::Position) -> usize {
    let line_idx = pos.line as usize;
    let col_idx = pos.character as usize;
    
    // Get the line
    if line_idx < content.len_lines() {
        let line = content.line(line_idx);
        let line_len = line.len_chars();
        if col_idx <= line_len {
            // Calculate the absolute character index
            let mut char_idx = 0;
            for i in 0..line_idx {
                char_idx += content.line(i).len_chars();
            }
            char_idx += col_idx;
            char_idx
        } else {
            // If column is beyond line length, return end of line
            let mut char_idx = 0;
            for i in 0..line_idx {
                char_idx += content.line(i).len_chars();
            }
            char_idx + line_len
        }
    } else {
        // If line is beyond document length, return document length
        content.len_chars()
    }
}

/// Handles the `textDocument/didClose` notification
pub async fn handle_did_close(
    state: &LspServerState,
    params: DidCloseTextDocumentParams,
) -> Result<(), ResponseError> {
    tracing::info!("Processing textDocument/didClose for URI: {}", params.text_document.uri);
    
    let uri = params.text_document.uri;
    
    // Remove the document from the server state
    {
        let mut server_data = state.write().await;
        if server_data.documents.remove(&uri).is_some() {
            tracing::info!("Document closed successfully: {}", uri);
        } else {
            tracing::warn!("Attempted to close non-existent document: {}", uri);
        }
    }
    
    Ok(())
}
//! PHP Parsing Module
//!
//! This module handles PHP document parsing using tree-sitter-php,
//! creating Abstract Syntax Trees (ASTs) for further language features.

use tree_sitter::Parser;

use crate::server::types::{LspServerState, AstWrapper};

/// Parse a PHP document and return an AST
pub fn parse_php_document(content: &str, version: i32) -> Result<AstWrapper, Box<dyn std::error::Error>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_php::LANGUAGE_PHP.into())
        .map_err(|e| format!("Error setting PHP language: {}", e))?;

    let tree = parser
        .parse(content, None)
        .ok_or("Parsing failed")?;

    // Check for syntax errors in the tree
    let has_errors = has_syntax_errors(tree.root_node());

    if has_errors {
        tracing::warn!("Document contains syntax errors, but partial AST generated");
    }

    Ok(AstWrapper {
        tree,
        version,
    })
}

/// Check if the AST contains syntax errors
fn has_syntax_errors(node: tree_sitter::Node) -> bool {
    if node.is_error() || node.is_missing() {
        return true;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if has_syntax_errors(child) {
            return true;
        }
    }

    false
}

/// Parse a document in the server state and cache the resulting AST
pub async fn parse_and_cache_document(
    state: &LspServerState,
    uri: &async_lsp::lsp_types::Url,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    tracing::info!("Parsing document: {:?}", uri);

    // Get the document from state
    let mut server_data = state.write().await;
    if let Some(mut doc) = server_data.documents.get_mut(uri) {
        let content_str = doc.content.to_string();
        let version = doc.version;
        let content_len = content_str.len();

        // Parse the document
        match parse_php_document(&content_str, version) {
            Ok(ast_wrapper) => {
                // Update the document with the new AST
                doc.ast = Some(ast_wrapper);

                let duration = start_time.elapsed();
                tracing::info!("Successfully parsed and cached AST for: {:?} ({} chars in {:?})", uri, content_len, duration);

                // Performance metric: Log parsing time
                if content_len > 10000 && duration.as_millis() > 100 {
                    tracing::warn!("Parsing of large document ({} chars) took longer than expected: {:?}", content_len, duration);
                }
            }
            Err(e) => {
                tracing::error!("Error parsing document {:?}: {}", uri, e);
                // Even if parsing fails completely, continue processing
                // The document will be stored without an AST
            }
        }
    } else {
        tracing::warn!("Document not found in state: {:?}", uri);
    }

    Ok(())
}

/// Get the AST for a document, parsing it if necessary
pub async fn get_or_parse_document_ast(
    state: &LspServerState,
    uri: &async_lsp::lsp_types::Url,
) -> Option<AstWrapper> {
    // First check if the document exists and has an up-to-date AST
    // Get a clone of the document to avoid borrowing issues
    let doc_clone = {
        let server_data = state.read().await;
        server_data.documents.get(uri).map(|doc| (doc.value().clone()))
    }; // server_data is dropped here

    // If document doesn't exist, return early
    let doc = match doc_clone {
        Some(doc) => doc,
        None => return None,
    };

    // Check if we already have a parsed AST that's up to date
    let has_up_to_date_ast = if let Some(ref ast) = doc.ast {
        // Check if AST is up-to-date with document version
        ast.version == doc.version
    } else {
        false // No AST exists
    };

    // If AST is not up-to-date, parse the document
    if !has_up_to_date_ast {
        let _ = parse_and_cache_document(state, uri).await;
    }

    // Now get the AST (either it was already up-to-date or we just parsed it)
    let server_data = state.read().await;
    if let Some(doc) = server_data.documents.get(uri) {
        if let Some(ref ast) = doc.ast {
            return Some(ast.clone());
        }
    }

    None
}
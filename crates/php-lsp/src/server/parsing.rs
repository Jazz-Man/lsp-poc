//! PHP Parsing Module
//! 
//! This module handles PHP document parsing using tree-sitter-php,
//! creating Abstract Syntax Trees (ASTs) for further language features.

use tree_sitter::{Language, Parser, Tree};
use lsp_types::Url;

use crate::server::types::{Document, LspServerState};

/// Wrapper for tree-sitter's Tree to make it more manageable
pub struct AstWrapper {
    pub tree: Tree,
    pub version: i32,
}

/// Parse a PHP document and return an AST
pub fn parse_php_document(content: &str) -> Result<AstWrapper, Box<dyn std::error::Error>> {
    let mut parser = Parser::new();
    let php_language = tree_sitter_php::language();
    parser
        .set_language(php_language)
        .map_err(|e| format!("Error setting PHP language: {}", e))?;
    
    let tree = parser
        .parse(content, None)
        .ok_or("Parsing failed")?;
    
    Ok(AstWrapper {
        tree,
        version: 0, // This would be set based on document version
    })
}

/// Parse a document in the server state and cache the resulting AST
pub async fn parse_and_cache_document(
    state: &LspServerState,
    uri: &Url,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Parsing document: {}", uri);
    
    // Get the document from state
    let mut server_data = state.write().await;
    if let Some(doc) = server_data.documents.get_mut(uri) {
        let content_str = doc.content.to_string();
        
        // Parse the document
        match parse_php_document(&content_str) {
            Ok(ast_wrapper) => {
                // Update the document with the new AST
                doc.ast = Some(ast_wrapper);
                tracing::info!("Successfully parsed and cached AST for: {}", uri);
            }
            Err(e) => {
                tracing::error!("Error parsing document {}: {}", uri, e);
                // Even if parsing fails, we might want to store partial results
                // For now, we'll just log the error
            }
        }
    } else {
        tracing::warn!("Document not found in state: {}", uri);
    }
    
    Ok(())
}

/// Get the AST for a document, parsing it if necessary
pub async fn get_or_parse_document_ast(
    state: &LspServerState,
    uri: &Url,
) -> Option<AstWrapper> {
    // First check if we already have a parsed AST that's up to date
    {
        let server_data = state.read().await;
        if let Some(doc) = server_data.documents.get(uri) {
            if let Some(ref ast) = doc.ast {
                if ast.version == doc.version {
                    return Some(ast.clone());
                }
            }
        }
    }
    
    // If we don't have a current AST, parse the document
    if parse_and_cache_document(state, uri).await.is_ok() {
        // Try again to get the newly cached AST
        let server_data = state.read().await;
        if let Some(doc) = server_data.documents.get(uri) {
            if let Some(ref ast) = doc.ast {
                return Some(ast.clone());
            }
        }
    }
    
    None
}
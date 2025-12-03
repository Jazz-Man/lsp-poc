#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use lsp_types::{
        DidOpenTextDocumentParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
        TextDocumentItem, VersionedTextDocumentIdentifier, TextDocumentContentChangeEvent,
        OptionalVersionedTextDocumentIdentifier,
    };
    use ropey::Rope;
    use serde_json::Value;
    use tokio::sync::RwLock;
    use url::Url;
    
    use crate::server::{
        types::{create_server_state, Document},
        document_sync::{handle_did_open, handle_did_change, handle_did_close},
    };

    #[tokio::test]
    async fn test_document_sync_open_change_close() {
        let state = create_server_state();
        let test_uri = Url::parse("file:///test.php").unwrap();
        
        // Test document open
        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: test_uri.clone(),
                language_id: "php".to_string(),
                version: 1,
                text: "<?php echo 'Hello, World!'; ?>".to_string(),
            },
        };
        
        let result = handle_did_open(&state, open_params).await;
        assert!(result.is_ok());
        
        // Verify document was added to state
        {
            let server_data = state.read().await;
            assert!(server_data.documents.contains_key(&test_uri));
            
            let doc = server_data.documents.get(&test_uri).unwrap();
            assert_eq!(doc.version, 1);
            assert_eq!(doc.content.to_string(), "<?php echo 'Hello, World!'; ?>");
        }
        
        // Test document change
        let change_params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: test_uri.clone(),
                version: 2,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None, // Full document replacement
                range_length: None,
                text: "<?php echo 'Updated content'; ?>".to_string(),
            }],
        };
        
        let result = handle_did_change(&state, change_params).await;
        assert!(result.is_ok());
        
        // Verify document was updated
        {
            let server_data = state.read().await;
            let doc = server_data.documents.get(&test_uri).unwrap();
            assert_eq!(doc.version, 2);
            assert_eq!(doc.content.to_string(), "<?php echo 'Updated content'; ?>");
        }
        
        // Test document close
        let close_params = DidCloseTextDocumentParams {
            text_document: OptionalVersionedTextDocumentIdentifier {
                uri: test_uri.clone(),
                version: None,
            },
        };
        
        let result = handle_did_close(&state, close_params).await;
        assert!(result.is_ok());
        
        // Verify document was removed from state
        {
            let server_data = state.read().await;
            assert!(!server_data.documents.contains_key(&test_uri));
        }
        
        println!("Document sync tests passed");
    }

    #[tokio::test]
    async fn test_document_sync_incremental_change() {
        let state = create_server_state();
        let test_uri = Url::parse("file:///test_incremental.php").unwrap();
        
        // Open document
        let open_params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: test_uri.clone(),
                language_id: "php".to_string(),
                version: 1,
                text: "<?php\necho 'Hello';\necho 'World';\n?>".to_string(),
            },
        };
        
        let result = handle_did_open(&state, open_params).await;
        assert!(result.is_ok());
        
        // Make incremental change (replace 'World' with 'Rust')
        let range = lsp_types::Range {
            start: lsp_types::Position { line: 2, character: 5 }, // Position of 'World'
            end: lsp_types::Position { line: 2, character: 10 },   // End of 'World'
        };
        
        let change_params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: test_uri.clone(),
                version: 2,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(range),
                range_length: Some(5), // Length of 'World'
                text: "Rust".to_string(),
            }],
        };
        
        let result = handle_did_change(&state, change_params).await;
        assert!(result.is_ok());
        
        // Verify document was updated with incremental changes
        {
            let server_data = state.read().await;
            let doc = server_data.documents.get(&test_uri).unwrap();
            assert_eq!(doc.version, 2);
            assert_eq!(doc.content.to_string(), "<?php\necho 'Hello';\necho 'Rust';\n?>");
        }
        
        // Close document
        let close_params = DidCloseTextDocumentParams {
            text_document: OptionalVersionedTextDocumentIdentifier {
                uri: test_uri.clone(),
                version: None,
            },
        };
        
        let result = handle_did_close(&state, close_params).await;
        assert!(result.is_ok());
        
        println!("Incremental document sync test passed");
    }
}
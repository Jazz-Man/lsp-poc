use std::sync::Arc;
use tokio::sync::RwLock;
use lsp_types::{
    DidOpenTextDocumentParams, TextDocumentItem, InitializeParams, ClientCapabilities,
    InitializedParams, DidChangeTextDocumentParams, VersionedTextDocumentIdentifier,
    TextDocumentContentChangeEvent, DidCloseTextDocumentParams, OptionalVersionedTextDocumentIdentifier,
};
use url::Url;

use php_lsp::server::{LspServer, create_server_state};

#[tokio::test]
async fn test_full_lsp_workflow() {
    // Create server instance
    let server = LspServer::new();
    
    // Test 1: Initialize the server
    let init_params = InitializeParams {
        process_id: None,
        root_path: None,
        root_uri: None,
        initialization_options: None,
        capabilities: ClientCapabilities::default(),
        trace: None,
        workspace_folders: None,
        client_info: None,
        locale: None,
    };
    
    let init_result = server.initialize(init_params).await;
    assert!(init_result.is_ok());
    println!("✓ Server initialization successful");
    
    // Test 2: Send initialized notification
    let initialized_params = InitializedParams {};
    let initialized_result = server.initialized(initialized_params).await;
    assert!(initialized_result.is_ok());
    println!("✓ Server initialized notification successful");
    
    // Test 3: Open a document
    let test_uri = Url::parse("file:///integration_test.php").unwrap();
    let open_params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: test_uri.clone(),
            language_id: "php".to_string(),
            version: 1,
            text: "<?php echo 'Hello from integration test'; ?>".to_string(),
        },
    };
    
    let open_result = server.did_open(open_params).await;
    assert!(open_result.is_ok());
    println!("✓ Document open successful");
    
    // Verify document exists in state
    {
        let state = &server.state;
        let server_data = state.read().await;
        assert!(server_data.documents.contains_key(&test_uri));
        let doc = server_data.documents.get(&test_uri).unwrap();
        assert_eq!(doc.version, 1);
        assert_eq!(doc.content.to_string(), "<?php echo 'Hello from integration test'; ?>");
    }
    
    // Test 4: Change the document
    let change_params = DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: test_uri.clone(),
            version: 2,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None, // Full document replacement
            range_length: None,
            text: "<?php echo 'Updated content from integration test'; ?>".to_string(),
        }],
    };
    
    let change_result = server.did_change(change_params).await;
    assert!(change_result.is_ok());
    println!("✓ Document change successful");
    
    // Verify document was updated
    {
        let state = &server.state;
        let server_data = state.read().await;
        let doc = server_data.documents.get(&test_uri).unwrap();
        assert_eq!(doc.version, 2);
        assert_eq!(doc.content.to_string(), "<?php echo 'Updated content from integration test'; ?>");
    }
    
    // Test 5: Close the document
    let close_params = DidCloseTextDocumentParams {
        text_document: OptionalVersionedTextDocumentIdentifier {
            uri: test_uri.clone(),
            version: None,
        },
    };
    
    let close_result = server.did_close(close_params).await;
    assert!(close_result.is_ok());
    println!("✓ Document close successful");
    
    // Verify document was removed
    {
        let state = &server.state;
        let server_data = state.read().await;
        assert!(!server_data.documents.contains_key(&test_uri));
    }
    
    // Test 6: Shutdown the server
    let shutdown_result = server.shutdown().await;
    assert!(shutdown_result.is_ok());
    println!("✓ Server shutdown successful");
    
    // Test 7: Exit the server
    let exit_result = server.exit().await;
    assert!(exit_result.is_ok());
    println!("✓ Server exit successful");
    
    println!("All integration tests passed!");
}
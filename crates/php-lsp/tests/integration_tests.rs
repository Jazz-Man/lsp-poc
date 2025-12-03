use std::sync::Arc;
use tokio::sync::RwLock;
use lsp_types::{
    DidOpenTextDocumentParams, TextDocumentItem, InitializeParams, ClientCapabilities,
    InitializedParams, DidChangeTextDocumentParams, VersionedTextDocumentIdentifier,
    TextDocumentContentChangeEvent, DidCloseTextDocumentParams, TextDocumentIdentifier,
};
use url::Url;

use php_lsp::server::types::{create_server_state, LspServerState};
use php_lsp::server::{lifecycle, document_sync, parsing};

#[tokio::test]
async fn test_full_lsp_workflow() {
    // Create server state
    let state = create_server_state();

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
        work_done_progress_params: Default::default(),
    };

    let init_result = lifecycle::handle_initialize(&state, init_params).await;
    assert!(init_result.is_ok());
    println!("✓ Server initialization successful");

    // Test 2: Send initialized notification
    let initialized_params = InitializedParams {};
    let initialized_result = lifecycle::handle_initialized(&state, initialized_params).await;
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

    let open_result = document_sync::handle_did_open(&state, open_params).await;
    assert!(open_result.is_ok());
    println!("✓ Document open successful");

    // Verify document exists in state
    {
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

    let change_result = document_sync::handle_did_change(&state, change_params).await;
    assert!(change_result.is_ok());
    println!("✓ Document change successful");

    // Verify document was updated
    {
        let server_data = state.read().await;
        let doc = server_data.documents.get(&test_uri).unwrap();
        assert_eq!(doc.version, 2);
        assert_eq!(doc.content.to_string(), "<?php echo 'Updated content from integration test'; ?>");
    }

    // Test 5: Parse the document
    let parse_result = parsing::parse_and_cache_document(&state, &test_uri).await;
    assert!(parse_result.is_ok(), "Parsing should succeed: {:?}", parse_result.err());
    println!("✓ Document parsing successful");

    // Verify AST was created
    {
        let server_data = state.read().await;
        let doc = server_data.documents.get(&test_uri).unwrap();
        assert!(doc.ast.is_some(), "AST should be created after parsing, but doc.ast is None. Content: '{}'", doc.content.to_string());
        println!("✓ AST successfully created for document");
    }

    // Test 6: Close the document
    let close_params = DidCloseTextDocumentParams {
        text_document: TextDocumentIdentifier {
            uri: test_uri.clone(),
        },
    };

    let close_result = document_sync::handle_did_close(&state, close_params).await;
    assert!(close_result.is_ok());
    println!("✓ Document close successful");

    // Verify document was removed
    {
        let server_data = state.read().await;
        assert!(!server_data.documents.contains_key(&test_uri));
    }

    // Test 7: Shutdown the server
    let shutdown_result = lifecycle::handle_shutdown(&state).await;
    assert!(shutdown_result.is_ok());
    println!("✓ Server shutdown successful");

    // Test 8: Exit the server
    let exit_result = lifecycle::handle_exit(&state).await;
    assert!(exit_result.is_ok());
    println!("✓ Server exit successful");

    println!("All integration tests passed!");
}
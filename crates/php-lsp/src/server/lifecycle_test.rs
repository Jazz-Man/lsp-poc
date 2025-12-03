#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use lsp_types::{
        InitializeParams, ClientCapabilities, TextDocumentSyncOptions, TextDocumentSyncKind,
        InitializedParams,
    };
    use serde_json::json;
    
    use crate::server::{
        types::{create_server_state, LspServerStateData, LspServerState},
        lifecycle::{handle_initialize, handle_initialized, handle_shutdown, handle_exit},
    };

    #[tokio::test]
    async fn test_lifecycle_initialize() {
        let state = create_server_state();
        
        let params = InitializeParams {
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

        let result = handle_initialize(&state, params).await;
        assert!(result.is_ok());
        
        // Check that the server state is updated
        {
            let server_data = state.read().await;
            assert!(server_data.is_initialized);
            assert!(!server_data.is_initializing);
        }
        
        println!("Initialize test passed");
    }

    #[tokio::test]
    async fn test_lifecycle_initialized() {
        let state = create_server_state();
        
        // First initialize the server
        {
            let mut server_data = state.write().await;
            server_data.is_initialized = false;
        }
        
        let params = InitializedParams {};
        
        let result = handle_initialized(&state, params).await;
        assert!(result.is_ok());
        
        // Check that the server state is updated
        {
            let server_data = state.read().await;
            assert!(server_data.is_initialized);
        }
        
        println!("Initialized test passed");
    }

    #[tokio::test]
    async fn test_lifecycle_shutdown() {
        let state = create_server_state();
        
        // Set up initial state
        {
            let mut server_data = state.write().await;
            server_data.is_shutting_down = false;
        }
        
        let result = handle_shutdown(&state).await;
        assert!(result.is_ok());
        
        // Check that the server state is updated
        {
            let server_data = state.read().await;
            assert!(server_data.is_shutting_down);
        }
        
        println!("Shutdown test passed");
    }

    #[tokio::test]
    async fn test_lifecycle_exit() {
        let state = create_server_state();
        
        // Set up initial state
        {
            let mut server_data = state.write().await;
            server_data.should_exit = false;
        }
        
        let result = handle_exit(&state).await;
        assert!(result.is_ok());
        
        // Check that the server state is updated
        {
            let server_data = state.read().await;
            assert!(server_data.should_exit);
        }
        
        println!("Exit test passed");
    }
}
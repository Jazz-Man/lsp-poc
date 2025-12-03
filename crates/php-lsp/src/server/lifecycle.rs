//! LSP Lifecycle Handlers
//! 
//! This module handles the core LSP lifecycle requests: initialize, initialized, shutdown, and exit.

use async_lsp::{ResponseError};
use lsp_types::{
    InitializeParams, InitializeResult, InitializedParams, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind, ServerCapabilities,
};
use serde_json::Value;

use crate::server::types::LspServerState;

/// Handles the `initialize` request from the LSP client
pub async fn handle_initialize(
    state: &LspServerState,
    params: InitializeParams,
) -> Result<InitializeResult, ResponseError> {
    let start_time = std::time::Instant::now();
    tracing::info!("Processing initialize request");

    // Update server state to indicate initialization is in progress
    {
        let mut server_data = state.write().await;
        server_data.is_initializing = true;
    }

    // Define the server capabilities
    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::INCREMENTAL,
        )),
        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),
        definition_provider: Some(lsp_types::OneOf::Left(true)),
        references_provider: Some(lsp_types::OneOf::Left(true)),
        document_symbol_provider: Some(lsp_types::OneOf::Left(true)),
        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec!["$".to_string(), ">".to_string(), "::".to_string()]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let server_info = Some(ServerInfo {
        name: "php-lsp".to_string(),
        version: Some("0.1.0".to_string()),
    });

    let result = InitializeResult {
        capabilities,
        server_info,
    };

    // Update server state to indicate initialization is complete
    {
        let mut server_data = state.write().await;
        server_data.is_initialized = true;
        server_data.is_initializing = false;
    }

    let duration = start_time.elapsed();
    tracing::info!("Initialize request completed successfully in {:?}", duration);

    // Log performance metric for initialization time
    if duration.as_secs() >= 5 {
        tracing::warn!("Initialization took longer than 5 seconds: {:?}", duration);
    } else {
        tracing::info!("Initialization completed within performance target: {:?}", duration);
    }

    Ok(result)
}

/// Handles the `initialized` notification from the LSP client
pub async fn handle_initialized(
    state: &LspServerState,
    _params: InitializedParams,
) -> Result<(), ResponseError> {
    tracing::info!("Processing initialized notification");
    
    // Update server state to indicate server is ready for document operations
    {
        let mut server_data = state.write().await;
        server_data.is_initialized = true;
    }
    
    Ok(())
}

/// Handles the `shutdown` request from the LSP client
pub async fn handle_shutdown(
    state: &LspServerState,
) -> Result<Value, ResponseError> {
    tracing::info!("Processing shutdown request");
    
    // Update server state to indicate shutdown is in progress
    {
        let mut server_data = state.write().await;
        server_data.is_shutting_down = true;
    }
    
    // Return null as per LSP specification for shutdown response
    Ok(Value::Null)
}

/// Handles the `exit` notification from the LSP client
pub async fn handle_exit(state: &LspServerState) -> Result<(), ResponseError> {
    tracing::info!("Processing exit notification");
    
    // Update server state to indicate server should exit
    {
        let mut server_data = state.write().await;
        server_data.should_exit = true;
    }
    
    // In a real implementation, we might want to do cleanup here
    // For now, we'll just log the event
    
    Ok(())
}
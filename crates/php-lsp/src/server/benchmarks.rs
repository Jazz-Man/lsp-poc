//! Performance Benchmarking Module
//! 
//! Contains functions for benchmarking key performance metrics of the LSP server.

use std::time::Instant;
use lsp_types::{DidOpenTextDocumentParams, TextDocumentItem};
use url::Url;

use crate::server::types::LspServerState;

/// Benchmark document operations performance
pub async fn benchmark_document_operations(state: &LspServerState) {
    let start_time = Instant::now();
    
    // Create a test document
    let test_uri = Url::parse("file:///benchmark_test.php").unwrap();
    let test_content = "<?php\nfor ($i = 0; $i < 100; $i++) {\n    echo \"Item $i\\n\";\n}".repeat(50); // Create a larger document
    
    let params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: test_uri.clone(),
            language_id: "php".to_string(),
            version: 1,
            text: test_content,
        },
    };
    
    // Measure document opening performance
    let open_start = Instant::now();
    crate::server::document_sync::handle_did_open(state, params).await.unwrap();
    let open_duration = open_start.elapsed();
    
    // Measure parsing performance
    let parse_start = Instant::now();
    let _ = crate::server::parsing::parse_and_cache_document(state, &test_uri).await;
    let parse_duration = parse_start.elapsed();
    
    let total_duration = start_time.elapsed();
    
    tracing::info!(
        "Document operation benchmark - Open: {:?}, Parse: {:?}, Total: {:?}",
        open_duration,
        parse_duration,
        total_duration
    );
}

/// Benchmark server initialization performance
pub async fn benchmark_initialization() {
    let start_time = Instant::now();
    
    // Create a new server instance
    let server = crate::server::LspServer::new();
    
    let init_duration = start_time.elapsed();
    tracing::info!("Server initialization benchmark: {:?}", init_duration);
    
    // Verify initialization was fast enough
    if init_duration.as_millis() > 5000 {
        tracing::warn!("Server initialization took longer than 5 seconds: {:?}", init_duration);
    } else {
        tracing::info!("Server initialization completed within performance target: {:?}", init_duration);
    }
}

/// Run all performance benchmarks
pub async fn run_benchmarks(state: &LspServerState) {
    tracing::info!("Starting performance benchmarks...");
    
    benchmark_initialization().await;
    benchmark_document_operations(state).await;
    
    tracing::info!("Performance benchmarks completed");
}
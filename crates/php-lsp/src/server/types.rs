//! Type Definitions
//! 
//! This module contains the core data types used throughout the LSP server.

use dashmap::DashMap;
use lsp_types::{Uri, Position, Range};
use ropey::Rope;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tree_sitter::Tree;
use url::Url;

/// Represents a PHP document being tracked by the LSP
#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Url,  // Using url::Url
    pub version: i32,
    pub content: Rope,
    pub ast: Option<AstWrapper>,
}

/// Wrapper for tree-sitter's Tree to make it more manageable
#[derive(Debug, Clone)]
pub struct AstWrapper {
    pub tree: Tree,
    pub version: i32,
}

/// Represents the overall state of the LSP server
#[derive(Debug)]
pub struct LspServerStateData {
    pub documents: DashMap<Url, Document>,
    pub is_initialized: bool,
    pub is_initializing: bool,
    pub is_shutting_down: bool,
    pub should_exit: bool,
}

/// Thread-safe wrapper for the LSP server state
pub type LspServerState = Arc<RwLock<LspServerStateData>>;

/// Create a new instance of the LSP server state
pub fn create_server_state() -> LspServerState {
    Arc::new(RwLock::new(LspServerStateData {
        documents: DashMap::new(),
        is_initialized: false,
        is_initializing: false,
        is_shutting_down: false,
        should_exit: false,
    }))
}
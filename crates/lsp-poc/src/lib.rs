//! PHP Language Server Protocol implementation
//!
//! A custom LSP server for PHP with WordPress hooks support.

pub mod server;

pub use server::run_server;

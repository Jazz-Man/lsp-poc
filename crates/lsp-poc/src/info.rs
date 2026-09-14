//! Server identity: one source of truth for the name and version the
//! server reports, reused by `ServerInfo` and `Diagnostic::source`.

/// The server name: the crate's package name at compile time.
#[must_use]
pub fn server_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

/// The server version: the crate's package version at compile time.
#[must_use]
pub fn server_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

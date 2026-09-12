use std::io::{IsTerminal, stderr};

use tracing_subscriber::filter::EnvFilter;

#[cfg(debug_assertions)]
const IS_DEBUG: bool = true;
#[cfg(not(debug_assertions))]
const IS_DEBUG: bool = false;

/// Installs the global tracing subscriber writing to stderr — stdout is the
/// LSP transport.
///
/// `RUST_LOG` wins when set; otherwise the profile default applies
/// (`debug`/`info`, with tower capped at `info`). Invalid directives are
/// dropped with a warning by the lossy parser instead of panicking.
pub fn setup_tracing() {
    // tower_lsp=warn from the zap-era template is gone on purpose: this
    // stack runs async-lsp + tower, so the target never existed here.
    let default_spec = if IS_DEBUG {
        "debug,tower=info"
    } else {
        "info,tower=info"
    };

    let filter = std::env::var("RUST_LOG").map_or_else(
        |_| EnvFilter::builder().parse_lossy(default_spec),
        |spec| EnvFilter::builder().parse_lossy(&spec),
    );

    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(filter)
        .without_time()
        .with_target(IS_DEBUG)
        .with_level(true)
        .with_ansi(stderr().is_terminal())
        .with_writer(stderr)
        .init();
}

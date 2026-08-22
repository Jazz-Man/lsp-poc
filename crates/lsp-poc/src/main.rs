mod tracing;

use self::tracing::setup_tracing;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    setup_tracing();

    println!("PHP LSP Server - Work in Progress TEST");
    println!("Run with --stdio for LSP mode");
}

use std::process::{Command, Stdio};
use std::io::{Write, BufRead, BufReader};
use std::time::Duration;

#[test]
#[ignore] // Ignore this test by default as it requires external communication
fn test_stdio_communication() -> Result<(), Box<dyn std::error::Error>> {
    // Start the server process in stdio mode
    let mut server = Command::new("cargo")
        .args(&["run", "--", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Get stdin and stdout handles
    let mut stdin = server.stdin.take().expect("Failed to get stdin");
    let stdout = server.stdout.take().expect("Failed to get stdout");

    // Create a reader for the output
    let mut reader = BufReader::new(stdout);
    let mut output = String::new();

    // Send an initialize request
    let initialize_request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":123,"rootPath":"/test","capabilities":{},"trace":"off"}}"#;

    // Write the request followed by the LSP header
    let request_with_header = format!("Content-Length: {}\r\n\r\n{}", initialize_request.len(), initialize_request);
    stdin.write_all(request_with_header.as_bytes())?;

    // Give the server some time to process
    std::thread::sleep(Duration::from_millis(500));

    // Try to read the response
    // Note: This is a simplified version - proper reading would need to parse Content-Length headers
    reader.read_line(&mut output)?;

    // Kill the server process
    server.kill()?;

    // Just ensure that we can start the process without errors
    // Actual response parsing would be more complex
    println!("Server process started and sent initialize request");

    Ok(())
}

#[tokio::test]
async fn test_server_startup() {
    // This test just verifies that the server state can be instantiated
    // without errors, which is a basic health check
    use php_lsp::server::types::create_server_state;

    let state = create_server_state();
    assert!(!state.read().await.is_initialized);

    println!("Server state can be instantiated");
}
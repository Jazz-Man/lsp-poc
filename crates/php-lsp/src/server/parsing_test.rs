#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use lsp_types::Url;
    use tokio::sync::RwLock;
    
    use crate::server::{
        types::{create_server_state, Document},
        parsing::{parse_php_document, parse_and_cache_document},
    };

    #[tokio::test]
    async fn test_php_parsing_valid_syntax() {
        let state = create_server_state();
        let test_uri = Url::parse("file:///test_valid.php").unwrap();
        
        // Add a document to the state
        {
            let mut server_data = state.write().await;
            server_data.documents.insert(
                test_uri.clone(),
                Document {
                    uri: test_uri.clone(),
                    version: 1,
                    content: ropey::Rope::from("<?php echo 'Hello, World!'; ?>"),
                    ast: None,
                },
            );
        }
        
        // Parse the document
        let result = parse_and_cache_document(&state, &test_uri).await;
        assert!(result.is_ok());
        
        // Verify the AST was created
        {
            let server_data = state.read().await;
            let doc = server_data.documents.get(&test_uri).unwrap();
            assert!(doc.ast.is_some());
        }
        
        println!("Valid PHP syntax parsing test passed");
    }

    #[tokio::test]
    async fn test_php_parsing_php8_syntax() {
        let state = create_server_state();
        let test_uri = Url::parse("file:///test_php8.php").unwrap();
        
        // Add a document with PHP 8+ syntax (attributes, named arguments, match expression)
        {
            let content = r#"<?php
#[Attribute]
function testFunction() {
    $arr = [1, 2, 3];
    $result = match ($arr[0]) {
        1 => 'one',
        2 => 'two',
        default => 'other'
    };
    echo $result;
}
?>"#;
            
            let mut server_data = state.write().await;
            server_data.documents.insert(
                test_uri.clone(),
                Document {
                    uri: test_uri.clone(),
                    version: 1,
                    content: ropey::Rope::from(content),
                    ast: None,
                },
            );
        }
        
        // Parse the document
        let result = parse_and_cache_document(&state, &test_uri).await;
        assert!(result.is_ok());
        
        // Verify the AST was created
        {
            let server_data = state.read().await;
            let doc = server_data.documents.get(&test_uri).unwrap();
            assert!(doc.ast.is_some());
        }
        
        println!("PHP 8+ syntax parsing test passed");
    }

    #[tokio::test]
    async fn test_php_parsing_with_errors() {
        let state = create_server_state();
        let test_uri = Url::parse("file:///test_error.php").unwrap();
        
        // Add a document with syntax errors
        {
            let mut server_data = state.write().await;
            server_data.documents.insert(
                test_uri.clone(),
                Document {
                    uri: test_uri.clone(),
                    version: 1,
                    content: ropey::Rope::from("<?php echo 'Hello, World!'; /* unclosed comment "), // Missing closing comment
                    ast: None,
                },
            );
        }
        
        // Parse the document - this should handle errors gracefully
        let result = parse_and_cache_document(&state, &test_uri).await;
        assert!(result.is_ok()); // Should not fail even with syntax errors
        
        println!("PHP parsing with errors test passed");
    }

    #[tokio::test]
    async fn test_php_parsing_php74_syntax() {
        let state = create_server_state();
        let test_uri = Url::parse("file:///test_php74.php").unwrap();
        
        // Add a document with PHP 7.4 syntax (typed properties, null coalescing assignment)
        {
            let content = r#"<?php
class TestClass {
    public string $name;
    public int $count;
    
    public function __construct() {
        $this->data ??= [];
    }
}
?>"#;
            
            let mut server_data = state.write().await;
            server_data.documents.insert(
                test_uri.clone(),
                Document {
                    uri: test_uri.clone(),
                    version: 1,
                    content: ropey::Rope::from(content),
                    ast: None,
                },
            );
        }
        
        // Parse the document
        let result = parse_and_cache_document(&state, &test_uri).await;
        assert!(result.is_ok());
        
        // Verify the AST was created
        {
            let server_data = state.read().await;
            let doc = server_data.documents.get(&test_uri).unwrap();
            assert!(doc.ast.is_some());
        }
        
        println!("PHP 7.4 syntax parsing test passed");
    }
}
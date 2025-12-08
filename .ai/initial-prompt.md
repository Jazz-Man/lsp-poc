Hi! 

I’m working on creating a custom Language Server Protocol (LSP) server for PHP, and I want to implement it in Rust. Ideally, I’d like to use the [`async-lsp`](https://docs.rs/async-lsp/latest/async_lsp/) crate, as I’m more excited about its architecture and async-native design. 

### 🧠 Motivation 
To be honest, one of my main motivations is that I’ve long dreamed of writing an LSP server in Rust using modern async tools. 
I’m also not satisfied with the current state of PHP support in existing tools. I’ve used [Intelephense](https://intelephense.com/) extensively — while it’s quite powerful and feature-rich, it often feels heavy, opaque, and hard to extend or debug. Similarly, JetBrains IDEs, although popular, tend to be resource-intensive and somewhat rigid for modern PHP workflows. What I really want is something lightweight, modular, and hackable — a language server that’s truly optimized for my workflow and that I fully control. That’s what pushed me toward building a custom solution from scratch. ### 🧱 LSP Server Requirements - The server should support basic LSP features: initialization, diagnostics, completions, hover, go-to-definition. - I want to use `tree-sitter-php` for PHP syntax parsing. - The server should analyze open documents and provide diagnostics (e.g., detect TODOs). - Full support for async handling using `tokio`. 

### 💻 Technical Goals - Use `async-lsp` and Rust for the implementation. 

- Architect it cleanly with a router-style handler setup. 
- Add snippet completions, variable completions, hover messages, and symbol extraction. 
- Possibly support workspace-wide features like document symbols and go-to-definition. 

### 🧪 Editor Integration 

I also want to integrate this LSP server with the [Zed.dev](https://zed.dev) code editor. So far, I understand that: 
- Zed is written in Rust and supports LSPs through a language extension system. 
- Extensions are configured with TOML files and placed in `~/.config/zed/extensions/`. 
- There’s a way to link an LSP executable to a language (like PHP) by specifying `language_server.command`. 

But I’d like more clarity on this: 
- Is there an official or recommended way to integrate *custom* LSP servers with Zed? 
- Are there any Zed APIs or native Rust crates I can use to build LSPs or language extensions directly for Zed? 
- Does Zed provide any richer language service integration beyond the standard LSP (e.g., internal APIs or plugin hooks)? 

### 🌐 What I’d Appreciate Help With 

- Step-by-step instructions or links on integrating my Rust-based async-lsp server into Zed 
- Code examples for both the server and the Zed extension (if available) 
- Any guidance on best practices or community-supported approaches 

Thanks so much!

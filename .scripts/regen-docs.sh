#!/bin/bash
set -e
echo "📚 Generating documentation..."
cargo doc --workspace --no-deps 2>/dev/null || cargo doc --workspace
cargo doc-md --workspace 2>/dev/null || cargo +nightly doc-md --workspace 2>/dev/null || echo "⚠️ MD docs failed"
[ -d "target/doc-md" ] && echo "✅ Done: target/doc-md/index.md"

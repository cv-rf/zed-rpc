#!/usr/bin/env bash
set -e

echo "==> Building Zed Discord RPC Plugin..."

# 1. Build the native companion LSP binary
echo "==> Building native LSP companion..."
cargo build --release -p discord-rpc-companion

# Copy binary to extension root for local resolution
cp target/release/discord-rpc-companion ./discord-rpc-companion
chmod +x ./discord-rpc-companion

# 2. Build the WebAssembly extension module for Zed
echo "==> Building WebAssembly extension (wasm32-wasip2)..."
cargo build --release --target wasm32-wasip2

echo ""
echo "=========================================================="
echo "✓ Build complete!"
echo "  - Native Companion: ./discord-rpc-companion"
echo "  - WASM Component:   ./target/wasm32-wasip2/release/zed_discord_rpc.wasm"
echo "=========================================================="
echo "To install companion binary into ~/.local/bin run: ./install.sh"

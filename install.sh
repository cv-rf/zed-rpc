#!/usr/bin/env bash
set -e

# Make sure companion is built
if [ ! -f "./target/release/discord-rpc-companion" ]; then
    echo "==> Building companion binary first..."
    cargo build --release -p discord-rpc-companion
fi

INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "${INSTALL_DIR}"

echo "==> Installing discord-rpc-companion to ${INSTALL_DIR}..."
cp ./target/release/discord-rpc-companion "${INSTALL_DIR}/discord-rpc-companion"
chmod +x "${INSTALL_DIR}/discord-rpc-companion"

echo "✓ Companion installed to ${INSTALL_DIR}/discord-rpc-companion"
echo ""
echo "Next step in Zed:"
echo "1. Press Ctrl+Shift+P (or Cmd+Shift+P on macOS)"
echo "2. Type: 'zed: install dev extension'"
echo "3. Select this folder: $(pwd)"
echo "4. Open any code file and watch your Discord profile update!"

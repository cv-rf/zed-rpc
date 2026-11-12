# Zed Discord RPC Plugin

A high-performance **Discord Rich Presence (RPC)** plugin for the [Zed](https://zed.dev) code editor.

Displays what you're working on in real-time on your Discord profile:
- 📂 **Workspace & Project Name**
- 🌿 **Git Branch** (e.g. `main`, `feat/auth`)
- 📄 **Active File & Line Count**
- 🎨 **Programming Language Icon & Tooltip**
- ⏱️ **Elapsed Time**
- 💤 **Idle Detection** (automatically shows "Idling" when you step away)

---

## Architecture

Zed extensions run inside a WebAssembly sandbox (`wasm32-wasip2`) which does not have direct access to local system IPC pipes. To overcome this limitation safely and cleanly, this extension uses a **Language Server Protocol (LSP)** companion architecture:

```
┌────────────────────────────────────────────────────────┐
│                      Zed Editor                        │
│                                                        │
│  ┌───────────────────────────┐                         │
│  │ WebAssembly Extension     │                         │
│  │ (zed_extension_api)       │                         │
│  └─────────────┬─────────────┘                         │
└────────────────┼───────────────────────────────────────┘
                 │ Launches & communicates via LSP (stdio)
                 ▼
┌────────────────────────────────────────────────────────┐
│  Native Companion Binary (discord-rpc-companion)       │
│  - Tracks active document, line count, workspace       │
│  - Detects Git repository and current branch           │
│  - Detects idle state after configurable timeout       │
└────────────────┬───────────────────────────────────────┘
                 │ Discord IPC Socket (/run/user/1000/discord-ipc-0)
                 ▼
┌────────────────────────────────────────────────────────┐
│                    Discord Client                      │
│                  Rich Presence Card                    │
└────────────────────────────────────────────────────────┘
```

---

## Quick Start

### 1. Build and Install Companion

From this directory:

```bash
# Build both the Wasm extension and native companion binary
./build.sh

# Install companion binary to ~/.local/bin (in your PATH)
./install.sh
```

### 2. Verify Discord Connection (Test Mode)

You can test that your Discord client connects and shows Rich Presence immediately:

```bash
discord-rpc-companion --test
```

### 3. Load into Zed Editor

1. Open Zed.
2. Open the Command Palette (`Ctrl+Shift+P` on Linux/Windows, `Cmd+Shift+P` on macOS).
3. Type and select: **`zed: install dev extension`**.
4. Choose this directory: `/home/atticl/Documents/projects/zed-rpc`.
5. Open any code file — your Discord presence will immediately update!

---

## Configuration

You can customize the presence format and behavior in your Zed `settings.json` (accessible via `Ctrl+,` or `Cmd+,` or Command Palette -> `zed: open settings`):

```json
{
  "lsp": {
    "discord_rpc": {
      "initialization_options": {
        "idle_timeout_seconds": 300,
        "show_git": true,
        "show_line_count": true,
        "show_timestamps": true,
        "details_format": "Editing {filename}",
        "state_format": "{workspace}"
      }
    }
  }
}
```

### Available Placeholders

| Variable | Description |
| :--- | :--- |
| `{filename}` | Current file name (e.g. `main.rs`, `app.py`) |
| `{language}` | Language display name (e.g. `Rust`, `TypeScript`) |
| `{workspace}` | Current workspace or git repository name |
| `{branch}` | Current Git branch (e.g. `main`, `develop`) |

---

## Project Structure

```
zed-rpc/
├── extension.toml         # Zed extension manifest declaring LSP integration
├── Cargo.toml             # Root workspace Cargo manifest
├── src/
│   └── lib.rs             # WebAssembly extension (launches companion LSP)
├── companion/             # Native companion crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs        # Entry point: LSP server / test mode / standalone mode
│       ├── discord_ipc.rs # Low-level async Discord IPC socket client
│       ├── lsp.rs         # LSP server implementation (tracks didOpen, didChange, etc.)
│       ├── git.rs         # Git repository & branch detection
│       ├── languages.rs   # Language mappings & icon resolution
│       └── config.rs      # User configuration parser
├── build.sh               # Build script for Wasm + native companion
├── install.sh             # Installs companion binary to ~/.local/bin
└── README.md
```

---

## Standalone Mode

You can also run the companion independently outside of Zed to show presence for any directory:

```bash
discord-rpc-companion --standalone
```

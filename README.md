# zed-n8n-as-code

Native Zed extension scaffold for `n8n-as-code`.

## What it does today
- Adds Zed slash commands:
  - `/n8n-list`
  - `/n8n-pull <workflowId>`
  - `/n8n-push <filename.workflow.ts>`
  - `/n8n-verify <workflowId>`
  - `/n8n-validate <filename.workflow.ts>`
- Exposes an MCP server entry named `n8nac`
- Executes the existing `n8nac` CLI as backend from the extension runtime
- Returns real command output into the Zed assistant slash-command result
- Provides workflow-id and local filename completions by parsing `n8nac list`
- Surfaces command exit status in a cleaner, more daily-usable output format

## Architecture
- **Zed extension layer**: Rust + WebAssembly, registered through `extension.toml`
- **Command execution layer**: delegates workflow operations to `n8nac` on PATH
- **MCP bridge**: returns a command for launching `n8nac skills mcp`

This is a native Zed extension, but the workflow engine is still backed by the mature `n8nac` CLI.
That is the fastest path to parity without rewriting the full n8n-as-code stack in Rust.

## Current limitations
- Depends on `n8nac` being installed and reachable on PATH
- Slash commands return text output, not a custom visual panel yet
- No embedded workflow canvas yet
- No direct native HTTP client to n8n yet
- Argument completion is intentionally minimal in this MVP

## Local development in Zed
1. Install Rust via `rustup`
2. Clone this repo
3. In Zed, run **Install Dev Extension**
4. Choose this folder
5. Open Zed logs with `zed: open log` if needed
6. For verbose extension logging, start Zed with `zed --foreground`

## Expected environment
Optional environment variables used by your local shell / CLI setup:
- `N8NAC_BIN` to override the executable path, default: `n8nac`
- `N8NAC_WORKSPACE` to force the workspace directory used by the CLI

## Roadmap
### Phase 1
- Stable slash commands
- MCP bridge
- Better argument completions from `n8nac list`

### Phase 2
- Native settings UI
- Better command result formatting
- Workflow-aware completions

### Phase 3
- Native n8n API client in Rust
- Remote workflow metadata cache
- Custom Zed panels / richer workflow browsing

### Phase 4
- Full native parity ambitions:
  - local/remote diffing
  - push/pull UX inside Zed
  - workflow inspector
  - validation diagnostics surfaced directly in editor tooling

## File layout
- `extension.toml` — Zed manifest
- `Cargo.toml` — Rust crate config
- `src/lib.rs` — extension implementation
- `LICENSE` — MIT
- `.gitignore`

## Notes
The implementation is designed to be realistic for Zed's extension model while staying conservative about API assumptions. If Zed's extension API changes, small signature updates may be needed.

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

## Install in Zed (dev extension)
1. Clone this repo locally
2. Make sure `n8nac` works in your shell
   - `n8nac list`
3. Optional but recommended: export workspace override
   - `export N8NAC_WORKSPACE=/home/motului/.openclaw/n8nac`
4. Open Zed
5. Run command palette: **zed: install dev extension**
6. Pick this repository folder
7. Open Zed logs with `zed: open log` if needed
8. For verbose extension logging, start Zed with `zed --foreground`

## Quick test plan in Zed
After the dev extension is installed, open the Assistant / slash-command surface in Zed and test these in order:

1. `/n8n-list`
   - Expected: workflow list output from `n8nac list`
2. `/n8n-verify <workflow-id>`
   - Expected: verify output for a real workflow id
3. `/n8n-pull <workflow-id>`
   - Expected: remote workflow is pulled into the local n8nac workspace
4. `/n8n-validate <filename.workflow.ts>`
   - Expected: validation output for a local workflow file
5. `/n8n-push <filename.workflow.ts>`
   - Expected: workflow is pushed back to n8n

## Minimal smoke test
Use one real remote workflow id and run:
- `/n8n-list`
- `/n8n-pull <id>`
- `/n8n-verify <id>`

Then use the local filename created by pull and run:
- `/n8n-validate <filename.workflow.ts>`

Only test `/n8n-push` after you intentionally change or confirm the local file you want to upload.

## Troubleshooting
- If commands fail immediately, verify `n8nac` is on PATH in the environment Zed inherits.
- If completions are empty, run `n8nac list --remote` and `n8nac list --local` in a shell first.
- If Zed loads the extension but commands fail, inspect `zed: open log`.
- If MCP does not appear, restart Zed after installing the dev extension.

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
- `TESTING.md` — practical install and smoke-test guide

## Notes
The implementation is designed to be realistic for Zed's extension model while staying conservative about API assumptions. If Zed's extension API changes, small signature updates may be needed.

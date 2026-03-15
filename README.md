# zed-n8n-as-code

Native Zed extension scaffold for `n8n-as-code`.

## What it does today

### CLI-Backed Commands (stable)
- `/n8n-list` - List all workflows via n8nac CLI
- `/n8n-pull <workflowId>` - Pull remote workflow by ID
- `/n8n-push <filename.workflow.ts>` - Push local workflow to n8n
- `/n8n-verify <workflowId>` - Verify workflow by ID
- `/n8n-validate <filename.workflow.ts>` - Validate local workflow file

### NEW: Native Commands (Phase 2)
- `/n8n-browse` - Rich workflow browser with native HTTP client
- `/n8n-status` - Local/remote workflow diff status (hybrid CLI + native)
- `/n8n-config` - Show extension configuration and settings

### Core Features
- MCP server integration (`n8nac` context server)
- Native Rust HTTP client to n8n API (health checks + workflow metadata)
- Environment-based configuration system
- Automatic fallback from native to CLI if API key not configured
- Workflow ID and filename completions from parsed CLI output
- Modular codebase (service, commands, completions, settings)

## Architecture

### Three-Layer Design
1. **Zed extension layer**: Rust + WebAssembly (wasm32-wasip1), registered via `extension.toml`
2. **Native service layer**: Direct HTTP client using Zed's http_client API
   - Health checks (`/healthz`)
   - Workflow listing (`/api/v1/workflows`)
   - Lightweight metadata queries
3. **CLI delegation layer**: Subprocess execution of `n8nac` for complex operations
   - Push/pull workflows (file I/O + validation)
   - Local validation and verification
   - Completions via table parsing

### Hybrid Approach
Commands use **native HTTP** where appropriate (`/n8n-browse`, `/n8n-status`) and **CLI delegation** for file operations and complex workflows. This keeps existing stable commands working while introducing native capabilities incrementally.

## Current limitations

### What is NOT native yet
- Push/pull operations still use CLI (file I/O is complex)
- Validation logic delegated to CLI (full TypeScript AST parsing)
- Completions still parse CLI text output (no caching)

### Zed Extension API constraints
- No custom UI panels - Zed extensions cannot render custom visual components beyond text
- No settings.json schema - Extensions cannot define custom settings (using env vars instead)
- HttpResponse lacks status codes - Must rely on fetch() error handling
- Stateless Extension trait - run_slash_command() receives &self, limiting init logic

### What works around these limits
- Rich markdown output with sections (emojis, headers, structured text)
- Environment variable configuration (documented in `/n8n-config`)
- HTTP errors surfaced as Result::Err, successful responses assumed 2xx
- Settings loaded from env on every command (stateless but functional)

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

## Configuration (Environment Variables)

Since Zed extensions cannot define custom settings.json schemas yet, configure via environment:

### CLI Configuration
- `N8NAC_BIN` - Path to n8nac executable (default: `n8nac`)
- `N8NAC_WORKSPACE` - Override workspace directory for CLI operations

### Native Service Configuration
- `N8N_URL` or `N8N_HOST` - n8n instance URL (default: `http://localhost:5678`)
- `N8N_API_KEY` - n8n API key for authentication (required for native commands)
- `N8NAC_NATIVE` - Enable/disable native service (default: `true`)

### How to set in Zed
Add to your `~/.zshrc` or `~/.bashrc`:
```bash
export N8N_URL="https://your-n8n.example.com"
export N8N_API_KEY="your-api-key-here"
export N8NAC_WORKSPACE="/path/to/your/workflows"
```

Or set in Zed's terminal settings (check `/n8n-config` output for current values).

## Roadmap

### Phase 1 ✅ COMPLETE
- Stable slash commands via CLI delegation
- MCP bridge (`n8nac` context server)
- Workflow ID and filename completions

### Phase 2 ✅ COMPLETE (this release)
- ✅ Native HTTP client using Zed's http_client API
- ✅ Environment-based configuration system
- ✅ `/n8n-browse` - native rich workflow browser
- ✅ `/n8n-status` - local/remote diff UX (hybrid)
- ✅ `/n8n-config` - show settings and environment
- ✅ Modular codebase (service, commands, completions, settings)
- ✅ Bootstrap/init with health checks
- ⚠️  Native settings UI - NOT POSSIBLE (Zed API limitation, using env vars)
- ⚠️  Custom visual panels - NOT POSSIBLE (Zed API limitation, using rich markdown)

### Phase 3 (Future)
- Workflow metadata caching (KeyValueStore)
- Streaming workflow updates
- Rich diff output with side-by-side comparison
- Native push/pull if Zed adds filesystem API

### Phase 4 (Blocked on Zed API)
- Custom UI panels (waiting for Zed extension API)
- Settings schema in settings.json (waiting for Zed extension API)
- Workflow canvas rendering (waiting for Zed extension API)
- LSP-style diagnostics for workflow validation

## File layout
- `extension.toml` - Zed manifest (8 slash commands + context server)
- `Cargo.toml` - Rust crate config (cdylib for WASM)
- `src/lib.rs` - Extension trait implementation + command routing
- `src/service.rs` - Native n8n HTTP client (health, list, get)
- `src/settings.rs` - Environment-based configuration
- `src/commands.rs` - Slash command handlers (CLI + native)
- `src/completions.rs` - Argument completion logic
- `LICENSE` - MIT
- `README.md` - This file
- `TESTING.md` - Install and testing guide

## Implementation Notes

### What is truly native now
- HTTP client using `zed_extension_api::http_client`
- Direct API calls to n8n REST endpoints
- Health checks on bootstrap
- Workflow metadata listing without subprocess
- Settings from environment (no external config files)

### What remains CLI-backed (and why)
- Push/pull - require filesystem access beyond extension sandbox
- Validation - complex TypeScript parsing better left to mature CLI
- Completions - already fast enough via table parsing

### API Discoveries
- `HttpResponse` has no status field - rely on `fetch()` returning `Err` for non-2xx
- `Extension::run_slash_command` takes `&self` - cannot store mutable state
- Settings must be environment-based - no extension settings schema yet
- Text output only - no custom panels or visual components

This implementation honestly documents what Zed's extension API can and cannot do as of zed_extension_api 0.7.

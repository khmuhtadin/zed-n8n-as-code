# Zed Testing Guide

## Prerequisites
- Zed installed
- `n8nac` installed and working in your shell (for CLI-backed commands)
- Access to n8n instance with API key (for native commands)
- Initialized workspace for `n8nac`

## Environment Setup

### Required for CLI commands
```bash
export N8NAC_WORKSPACE=/home/motului/.openclaw/n8nac
export N8NAC_BIN=n8nac  # optional, defaults to 'n8nac'
```

### Required for native commands (NEW in Phase 2)
```bash
export N8N_URL=http://localhost:5678  # or your n8n instance URL
export N8N_API_KEY=your-api-key-here  # get from n8n Settings > API
export N8NAC_NATIVE=true  # optional, defaults to true
```

### Sanity checks before opening Zed
```bash
# CLI sanity check
n8nac list
n8nac list --remote
n8nac list --local

# Native API sanity check
curl -H "X-N8N-API-KEY: $N8N_API_KEY" $N8N_URL/api/v1/workflows
```

## Install as a dev extension
1. Open Zed
2. Run: `zed: install dev extension`
3. Select the `zed-n8n-as-code` repository folder
4. Restart Zed if the slash commands do not appear immediately

## Slash command tests

### Configuration Check (NEW)
```text
/n8n-config
```
Expected:
- Shows current extension settings
- Displays all environment variables and their values
- Confirms native service enabled/disabled status

### CLI-Backed Commands (Stable)

#### 1. List workflows (CLI)
```text
/n8n-list
```
Expected:
- Executes `n8nac list`
- Shows table output with Status, ID, Name, Local Path columns
- Exit status displayed

#### 2. Verify remote workflow
```text
/n8n-verify <workflow-id>
```
Expected:
- Runs `n8nac verify <id>`
- Shows verification output

#### 3. Pull remote workflow
```text
/n8n-pull <workflow-id>
```
Expected:
- Workflow downloaded to local workspace
- File appears in subsequent `/n8n-list --local` output
- Filename available in completions

#### 4. Validate local workflow
```text
/n8n-validate <filename.workflow.ts>
```
Expected:
- Validation output from CLI
- TypeScript parse errors shown if invalid

#### 5. Push local workflow
```text
/n8n-push <filename.workflow.ts>
```
Expected:
- Workflow uploaded to n8n
- Success/failure status clearly indicated

### Native Commands (NEW in Phase 2)

#### 6. Browse workflows (Native)
```text
/n8n-browse
```
Expected (if N8N_API_KEY set):
- Rich markdown output with workflow list
- Shows workflow ID, name, active status (🟢/⚫)
- Displays tags and last updated time
- Fetched directly from n8n API (no subprocess)

Expected (if N8N_API_KEY not set):
- Falls back to `/n8n-list` CLI output

#### 7. Workflow status (Native + CLI hybrid)
```text
/n8n-status
```
Expected (if N8N_API_KEY set):
- Remote workflow count from native API
- Local file count from CLI parsing
- "Remote Only" section showing unpulled workflows
- "Local Files" section listing local .workflow.ts files

Expected (if N8N_API_KEY not set):
- Falls back to `/n8n-list` CLI output

## Daily-use workflow recommendations

### With native service enabled (API key configured)
1. `/n8n-config` - verify settings
2. `/n8n-browse` - see all workflows with rich output
3. `/n8n-status` - check what's remote vs local
4. `/n8n-pull <id>` - pull workflow you want to edit
5. Edit the .workflow.ts file locally
6. `/n8n-validate <filename>` - validate before pushing
7. `/n8n-push <filename>` - push changes to n8n
8. `/n8n-browse` - confirm changes reflected

### Without API key (CLI-only mode)
1. `/n8n-list` - see all workflows
2. `/n8n-pull <id>` - pull workflow
3. Edit locally
4. `/n8n-validate <filename>`
5. `/n8n-push <filename>`
6. `/n8n-verify <id>`

## Troubleshooting

### General Issues
- Open Zed logs: `zed: open log`
- Launch with verbose output: `zed --foreground`
- Confirm PATH inheritance: Zed must find `n8nac` binary

### CLI Commands Failing
- Verify `n8nac` works in terminal: `n8nac list`
- Check `N8NAC_WORKSPACE` is set correctly
- Ensure workspace directory exists and has .n8nac.yml config

### Native Commands Falling Back to CLI
- Run `/n8n-config` to see current settings
- Verify `N8N_API_KEY` is set in environment
- Test API directly: `curl -H "X-N8N-API-KEY: $N8N_API_KEY" $N8N_URL/healthz`
- Check n8n instance is reachable from Zed's environment

### Completions Empty
- Run `n8nac list --remote` in terminal to verify data
- Run `n8nac list --local` to verify local files
- Restart Zed after changing environment variables

### Build Issues
- Run `cargo check` to validate syntax
- Run `cargo build --target wasm32-wasip1 --release` for WASM build
- Install wasm32-wasip1 target: `rustup target add wasm32-wasip1`

## Testing Native Service Independently

To verify the native HTTP client works without Zed:

```bash
# Set environment
export N8N_URL=http://localhost:5678
export N8N_API_KEY=your-key

# Test health check
curl -H "X-N8N-API-KEY: $N8N_API_KEY" $N8N_URL/healthz
# Expected: {"status":"ok"}

# Test workflow list
curl -H "X-N8N-API-KEY: $N8N_API_KEY" $N8N_URL/api/v1/workflows
# Expected: {"data": [workflow objects...]}
```

If these work, the native commands should work in Zed with the same env vars.

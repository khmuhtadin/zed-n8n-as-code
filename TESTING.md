# Zed Testing Guide

## Prerequisites
- Zed installed
- `n8nac` installed and working in your shell
- Access to the initialized workspace used by `n8nac`

Recommended environment:

```bash
export N8NAC_WORKSPACE=/home/motului/.openclaw/n8nac
```

Optional sanity checks before opening Zed:

```bash
n8nac list
n8nac list --remote
n8nac list --local
```

## Install as a dev extension
1. Open Zed
2. Run: `zed: install dev extension`
3. Select the `zed-n8n-as-code` repository folder
4. Restart Zed if the slash commands do not appear immediately

## Slash command tests

### 1. List
```text
/n8n-list
```
Expected:
- Command runs successfully
- Workflow list appears in output

### 2. Verify a remote workflow
```text
/n8n-verify <workflow-id>
```
Expected:
- Verification output from `n8nac verify`

### 3. Pull a remote workflow
```text
/n8n-pull <workflow-id>
```
Expected:
- Workflow file appears in local n8nac workspace
- Filename can later appear in completion for push/validate

### 4. Validate a local workflow file
```text
/n8n-validate <filename.workflow.ts>
```
Expected:
- Validation result is displayed

### 5. Push a local workflow file
```text
/n8n-push <filename.workflow.ts>
```
Expected:
- Workflow uploads successfully to n8n

## Daily-use recommendation
Use this order when working safely:
1. `/n8n-list`
2. `/n8n-pull <id>`
3. edit locally
4. `/n8n-validate <filename.workflow.ts>`
5. `/n8n-push <filename.workflow.ts>`
6. `/n8n-verify <id>`

## Troubleshooting
- Open Zed logs with `zed: open log`
- Launch Zed from terminal with `zed --foreground`
- Confirm Zed inherits the same shell PATH as the terminal where `n8nac` works
- If completions are empty, ensure `n8nac list --remote` and/or `n8nac list --local` return data in a normal shell first

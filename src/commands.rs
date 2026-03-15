/// Slash command handlers - both CLI-backed and native implementations
use zed_extension_api::{self as zed, process::Command as ProcessCommand};

use crate::service::{N8nConfig, N8nService};
use crate::settings::ExtensionSettings;

/// Run a CLI command through n8nac subprocess
pub fn run_cli_command(
    settings: &ExtensionSettings,
    subcommand: &str,
    args: &[String],
) -> Result<(Option<i32>, String, String), String> {
    let mut command = ProcessCommand::new(&settings.cli_bin);
    command = command
        .arg(subcommand)
        .args(args.iter().cloned())
        .envs(settings.workspace_env());

    let output = command.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok((output.status, stdout, stderr))
}

/// Render command output in consistent format
pub fn render_cli_output(
    subcommand: &str,
    args: &[String],
    status: Option<i32>,
    stdout: &str,
    stderr: &str,
) -> zed::SlashCommandOutput {
    let display_args = if args.is_empty() {
        String::new()
    } else {
        format!(" {}", args.join(" "))
    };

    let mut text = String::new();
    text.push_str(&format!("$ n8nac {}{}\n", subcommand, display_args));
    text.push_str(&format!(
        "Status: {}\n\n",
        match status {
            Some(0) => "OK".to_string(),
            Some(code) => format!("FAILED (exit {})", code),
            None => "FAILED (no exit code)".to_string(),
        }
    ));

    if !stdout.trim().is_empty() {
        text.push_str("stdout:\n");
        text.push_str(stdout.trim_end());
        text.push_str("\n\n");
    }

    if !stderr.trim().is_empty() {
        text.push_str("stderr:\n");
        text.push_str(stderr.trim_end());
        text.push_str("\n\n");
    }

    if stdout.trim().is_empty() && stderr.trim().is_empty() {
        text.push_str("Command completed with no output.\n");
    }

    let mut sections = Vec::new();
    let full_len = text.len();
    sections.push(zed::SlashCommandOutputSection {
        range: (0..full_len).into(),
        label: format!("n8nac {}", subcommand),
    });

    zed::SlashCommandOutput { text, sections }
}

/// Execute list command via CLI
pub fn cmd_list_cli(settings: &ExtensionSettings) -> Result<zed::SlashCommandOutput, String> {
    let (status, stdout, stderr) = run_cli_command(settings, "list", &[])?;
    Ok(render_cli_output("list", &[], status, &stdout, &stderr))
}

/// Execute pull command via CLI
pub fn cmd_pull_cli(
    settings: &ExtensionSettings,
    workflow_id: &str,
) -> Result<zed::SlashCommandOutput, String> {
    let args = vec![workflow_id.to_string()];
    let (status, stdout, stderr) = run_cli_command(settings, "pull", &args)?;
    Ok(render_cli_output("pull", &args, status, &stdout, &stderr))
}

/// Execute push command via CLI
pub fn cmd_push_cli(
    settings: &ExtensionSettings,
    filename: &str,
) -> Result<zed::SlashCommandOutput, String> {
    let args = vec![filename.to_string()];
    let (status, stdout, stderr) = run_cli_command(settings, "push", &args)?;
    Ok(render_cli_output("push", &args, status, &stdout, &stderr))
}

/// Execute verify command via CLI
pub fn cmd_verify_cli(
    settings: &ExtensionSettings,
    workflow_id: &str,
) -> Result<zed::SlashCommandOutput, String> {
    let args = vec![workflow_id.to_string()];
    let (status, stdout, stderr) = run_cli_command(settings, "verify", &args)?;
    Ok(render_cli_output("verify", &args, status, &stdout, &stderr))
}

/// Execute validate command via CLI
pub fn cmd_validate_cli(
    settings: &ExtensionSettings,
    filename: &str,
) -> Result<zed::SlashCommandOutput, String> {
    let args = vec![filename.to_string()];
    let (status, stdout, stderr) = run_cli_command(settings, "validate", &args)?;
    Ok(render_cli_output("validate", &args, status, &stdout, &stderr))
}

/// NEW: Native browse command - rich workflow browser
pub fn cmd_browse_native(settings: &ExtensionSettings) -> Result<zed::SlashCommandOutput, String> {
    let config = N8nConfig {
        base_url: settings.n8n_url.clone(),
        api_key: settings.n8n_api_key.clone(),
    };

    let service = N8nService::new(config);
    let workflows = service.list_workflows()?;

    let mut text = String::new();
    text.push_str("# n8n Workflow Browser (Native)\n\n");

    if workflows.is_empty() {
        text.push_str("No workflows found.\n");
    } else {
        text.push_str(&format!("Found {} workflow(s):\n\n", workflows.len()));

        for wf in workflows.iter() {
            text.push_str(&format!(
                "## {} {}\n",
                wf.id,
                if wf.active { "🟢" } else { "⚫" }
            ));
            text.push_str(&format!("**Name:** {}\n", wf.name));
            text.push_str(&format!("**ID:** {}\n", wf.id));
            text.push_str(&format!("**Active:** {}\n", wf.active));
            if !wf.tags.is_empty() {
                let tag_names: Vec<&str> = wf.tags.iter().map(|t| t.name.as_str()).collect();
                text.push_str(&format!("**Tags:** {}\n", tag_names.join(", ")));
            }
            if !wf.updated_at.is_empty() {
                text.push_str(&format!("**Updated:** {}\n", wf.updated_at));
            }
            text.push_str("\n");
        }
    }

    let mut sections = vec![];
    let header_end = text.find('\n').unwrap_or(text.len());
    sections.push(zed::SlashCommandOutputSection {
        range: (0..header_end).into(),
        label: "Workflow Browser".to_string(),
    });

    Ok(zed::SlashCommandOutput { text, sections })
}

/// NEW: Native status command - show local/remote diff status
pub fn cmd_status_native(
    settings: &ExtensionSettings,
) -> Result<zed::SlashCommandOutput, String> {
    let config = N8nConfig {
        base_url: settings.n8n_url.clone(),
        api_key: settings.n8n_api_key.clone(),
    };

    let service = N8nService::new(config);

    // Get remote workflows
    let remote_workflows = service.list_workflows()?;

    // Get local workflows via CLI
    let (_, local_stdout, _) = run_cli_command(settings, "list", &["--local".to_string()])?;
    let local_files = parse_local_workflows(&local_stdout);

    let mut text = String::new();
    text.push_str("# n8n Workflow Status (Native)\n\n");

    text.push_str(&format!(
        "**Remote workflows:** {}\n",
        remote_workflows.len()
    ));
    text.push_str(&format!("**Local files:** {}\n\n", local_files.len()));

    // Simple comparison
    text.push_str("## Remote Only (not pulled yet)\n");
    let mut remote_only = Vec::new();
    for wf in remote_workflows.iter() {
        if !local_files.iter().any(|l| l.contains(&wf.id)) {
            remote_only.push(wf);
        }
    }
    if remote_only.is_empty() {
        text.push_str("(none)\n\n");
    } else {
        for wf in remote_only {
            text.push_str(&format!("- {} - {}\n", wf.id, wf.name));
        }
        text.push_str("\n");
    }

    text.push_str("## Local Files\n");
    if local_files.is_empty() {
        text.push_str("(none)\n\n");
    } else {
        for file in local_files {
            text.push_str(&format!("- {}\n", file));
        }
        text.push_str("\n");
    }

    let mut sections = vec![];
    sections.push(zed::SlashCommandOutputSection {
        range: (0..text.len()).into(),
        label: "Workflow Status".to_string(),
    });

    Ok(zed::SlashCommandOutput { text, sections })
}

/// NEW: Config command - show current settings
pub fn cmd_config(settings: &ExtensionSettings) -> Result<zed::SlashCommandOutput, String> {
    let mut text = String::new();
    text.push_str("# n8n-as-code Extension Configuration\n\n");
    text.push_str(&settings.summary());
    text.push_str("\n\n");
    text.push_str("## Environment Variables\n");
    text.push_str("Configure these in your shell or Zed settings:\n\n");
    text.push_str("- `N8NAC_BIN`: Path to n8nac CLI (default: n8nac)\n");
    text.push_str("- `N8N_URL` or `N8N_HOST`: n8n instance URL (default: http://localhost:5678)\n");
    text.push_str("- `N8N_API_KEY`: n8n API key for native service\n");
    text.push_str("- `N8NAC_WORKSPACE`: Override workspace directory\n");
    text.push_str("- `N8NAC_NATIVE`: Enable/disable native service (default: true)\n");

    let sections = vec![zed::SlashCommandOutputSection {
        range: (0..text.len()).into(),
        label: "Configuration".to_string(),
    }];

    Ok(zed::SlashCommandOutput { text, sections })
}

/// Parse local workflow files from CLI list output
fn parse_local_workflows(stdout: &str) -> Vec<String> {
    let mut files = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('│') {
            continue;
        }

        let columns: Vec<String> = trimmed
            .trim_matches('│')
            .split('│')
            .map(|part| part.trim().to_string())
            .collect();

        if columns.len() >= 4 {
            let local_path = &columns[3];
            if !local_path.is_empty()
                && local_path != "Local Path"
                && local_path.ends_with(".workflow.ts")
            {
                files.push(local_path.clone());
            }
        }
    }
    files
}

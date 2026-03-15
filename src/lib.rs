mod commands;
mod completions;
mod service;
mod settings;

use zed_extension_api::{self as zed};

use commands::{
    cmd_browse_native, cmd_config, cmd_list_cli, cmd_pull_cli, cmd_push_cli, cmd_status_native,
    cmd_validate_cli, cmd_verify_cli,
};
use completions::{filename_completions, workflow_id_completions};
use service::{N8nConfig, N8nService};
use settings::ExtensionSettings;

struct N8nAsCodeExtension {
    settings: ExtensionSettings,
}

impl N8nAsCodeExtension {
    /// Check if native service should be used
    /// Note: We do a lightweight check without storing state since Extension trait uses &self
    fn should_use_native(&self, command_name: &str) -> bool {
        // Only use native for browse and status commands
        // Keep existing commands CLI-backed for stability
        match command_name {
            "n8n-browse" | "n8n-status" => self.settings.should_use_native(),
            _ => false,
        }
    }
}

impl zed::Extension for N8nAsCodeExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            settings: ExtensionSettings::from_env(),
        }
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        match command.name.as_str() {
            // Existing CLI-backed commands
            "n8n-list" => cmd_list_cli(&self.settings),
            "n8n-pull" => {
                let workflow_id = args
                    .first()
                    .ok_or_else(|| "workflow id is required".to_string())?;
                cmd_pull_cli(&self.settings, workflow_id)
            }
            "n8n-push" => {
                let filename = args
                    .first()
                    .ok_or_else(|| "workflow filename is required".to_string())?;
                cmd_push_cli(&self.settings, filename)
            }
            "n8n-verify" => {
                let workflow_id = args
                    .first()
                    .ok_or_else(|| "workflow id is required".to_string())?;
                cmd_verify_cli(&self.settings, workflow_id)
            }
            "n8n-validate" => {
                let filename = args
                    .first()
                    .ok_or_else(|| "local workflow filename is required".to_string())?;
                cmd_validate_cli(&self.settings, filename)
            }

            // NEW: Native commands
            "n8n-browse" => {
                if self.should_use_native("n8n-browse") {
                    cmd_browse_native(&self.settings)
                } else {
                    // Fallback to CLI list
                    cmd_list_cli(&self.settings)
                }
            }
            "n8n-status" => {
                if self.should_use_native("n8n-status") {
                    cmd_status_native(&self.settings)
                } else {
                    // Fallback to basic CLI list
                    cmd_list_cli(&self.settings)
                }
            }
            "n8n-config" => cmd_config(&self.settings),

            other => Err(format!("unknown slash command: {other}")),
        }
    }

    fn complete_slash_command_argument(
        &self,
        command: zed::SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<zed::SlashCommandArgumentCompletion>, String> {
        match command.name.as_str() {
            "n8n-list" | "n8n-browse" | "n8n-status" | "n8n-config" => Ok(vec![]),
            "n8n-pull" | "n8n-verify" => {
                let completions = workflow_id_completions(&self.settings);
                if completions.is_empty() {
                    Ok(vec![zed::SlashCommandArgumentCompletion {
                        label: "workflow-id".to_string(),
                        new_text: "<workflow-id>".to_string(),
                        run_command: false,
                    }])
                } else {
                    Ok(completions)
                }
            }
            "n8n-push" | "n8n-validate" => {
                let completions = filename_completions(&self.settings);
                if completions.is_empty() {
                    Ok(vec![zed::SlashCommandArgumentCompletion {
                        label: "filename.workflow.ts".to_string(),
                        new_text: "example.workflow.ts".to_string(),
                        run_command: false,
                    }])
                } else {
                    Ok(completions)
                }
            }
            other => Err(format!("unknown slash command: {other}")),
        }
    }

    fn context_server_command(
        &mut self,
        context_server_id: &zed::ContextServerId,
        _project: &zed::Project,
    ) -> zed::Result<zed::Command> {
        if context_server_id.as_ref() != "n8nac" {
            return Err(format!(
                "unknown context server: {}",
                context_server_id.as_ref()
            ));
        }

        Ok(zed::Command {
            command: self.settings.cli_bin.clone(),
            args: vec!["skills".to_string(), "mcp".to_string()],
            env: self.settings.workspace_env(),
        })
    }
}

zed::register_extension!(N8nAsCodeExtension);

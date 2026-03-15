use std::env;
use zed_extension_api as zed;

struct N8nAsCodeExtension;

impl N8nAsCodeExtension {
    fn cli_bin(&self) -> String {
        env::var("N8NAC_BIN").unwrap_or_else(|_| "n8nac".to_string())
    }

    fn workspace_env(&self) -> Vec<(String, String)> {
        let mut envs = Vec::new();
        if let Ok(workspace) = env::var("N8NAC_WORKSPACE") {
            envs.push(("N8NAC_WORKSPACE".to_string(), workspace));
        }
        envs
    }

    fn run_cli_command(
        &self,
        subcommand: &str,
        args: Vec<String>,
    ) -> Result<zed::SlashCommandOutput, String> {
        let display_args = if args.is_empty() {
            String::new()
        } else {
            format!(" {}", args.join(" "))
        };

        let mut command = zed::process::Command::new(self.cli_bin());
        command = command.arg(subcommand).args(args.clone()).envs(self.workspace_env());

        let output = command.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let mut text = format!("$ n8nac {}{}\n\n", subcommand, display_args);
        if !stdout.trim().is_empty() {
            text.push_str(&stdout);
            if !stdout.ends_with('\n') {
                text.push('\n');
            }
        }
        if !stderr.trim().is_empty() {
            if !text.ends_with("\n\n") {
                text.push('\n');
            }
            text.push_str("stderr:\n");
            text.push_str(&stderr);
            if !stderr.ends_with('\n') {
                text.push('\n');
            }
        }
        if stdout.trim().is_empty() && stderr.trim().is_empty() {
            text.push_str("Command completed with no output.\n");
        }

        Ok(zed::SlashCommandOutput {
            sections: vec![zed::SlashCommandOutputSection {
                range: (0..text.len()).into(),
                label: format!("n8nac {}", subcommand),
            }],
            text,
        })
    }
}

impl zed::Extension for N8nAsCodeExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        match command.name.as_str() {
            "n8n-list" => self.run_cli_command("list", vec![]),
            "n8n-pull" => {
                let workflow_id = args
                    .first()
                    .ok_or_else(|| "workflow id is required".to_string())?;
                self.run_cli_command("pull", vec![workflow_id.clone()])
            }
            "n8n-push" => {
                let filename = args
                    .first()
                    .ok_or_else(|| "workflow filename is required".to_string())?;
                self.run_cli_command("push", vec![filename.clone()])
            }
            "n8n-verify" => {
                let workflow_id = args
                    .first()
                    .ok_or_else(|| "workflow id is required".to_string())?;
                self.run_cli_command("verify", vec![workflow_id.clone()])
            }
            "n8n-validate" => {
                let filename = args
                    .first()
                    .ok_or_else(|| "local workflow filename is required".to_string())?;
                self.run_cli_command("validate", vec![filename.clone()])
            }
            other => Err(format!("unknown slash command: {other}")),
        }
    }

    fn complete_slash_command_argument(
        &self,
        command: zed::SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<zed::SlashCommandArgumentCompletion>, String> {
        match command.name.as_str() {
            "n8n-list" => Ok(vec![]),
            "n8n-pull" | "n8n-verify" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "workflow-id".to_string(),
                new_text: "<workflow-id>".to_string(),
                run_command: false,
            }]),
            "n8n-push" | "n8n-validate" => Ok(vec![zed::SlashCommandArgumentCompletion {
                label: "filename.workflow.ts".to_string(),
                new_text: "example.workflow.ts".to_string(),
                run_command: false,
            }]),
            other => Err(format!("unknown slash command: {other}")),
        }
    }

    fn context_server_command(
        &mut self,
        context_server_id: &zed::ContextServerId,
        _project: &zed::Project,
    ) -> Result<zed::Command> {
        if context_server_id.as_ref() != "n8nac" {
            return Err(format!(
                "unknown context server: {}",
                context_server_id.as_ref()
            ));
        }

        Ok(zed::Command {
            command: self.cli_bin(),
            args: vec!["skills".to_string(), "mcp".to_string()],
            env: self.workspace_env(),
        })
    }
}

zed::register_extension!(N8nAsCodeExtension);

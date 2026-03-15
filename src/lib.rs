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

    fn run_cli_command(&self, subcommand: &str, args: Vec<String>) -> Result<zed::SlashCommandOutput, String> {
        let header = format!("$ {} {} {}\n\n", self.cli_bin(), subcommand, args.join(" "));
        let body = format!(
            "This Zed extension is configured to run the n8nac CLI backend.\n\nSubcommand: {}\nArgs: {}\n\nInstall and expose `n8nac` in your shell PATH for live execution from Zed.\nFor now this scaffold returns an execution plan that mirrors the native command invocation path.",
            subcommand,
            if args.is_empty() { "-".to_string() } else { args.join(" ") }
        );
        let text = format!("{}{}", header, body);
        Ok(zed::SlashCommandOutput {
            text: text.clone(),
            sections: vec![zed::SlashCommandOutputSection {
                range: (0..text.len()).into(),
                label: format!("n8nac {}", subcommand),
            }],
        })
    }
}

impl zed::Extension for N8nAsCodeExtension {
    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        match command.name.as_str() {
            "n8n-list" => self.run_cli_command("list", vec![]),
            "n8n-pull" => {
                let workflow_id = args.first().ok_or_else(|| "workflow id is required".to_string())?;
                self.run_cli_command("pull", vec![workflow_id.clone()])
            }
            "n8n-push" => {
                let filename = args.first().ok_or_else(|| "workflow filename is required".to_string())?;
                self.run_cli_command("push", vec![filename.clone()])
            }
            "n8n-verify" => {
                let workflow_id = args.first().ok_or_else(|| "workflow id is required".to_string())?;
                self.run_cli_command("verify", vec![workflow_id.clone()])
            }
            "n8n-validate" => {
                let filename = args.first().ok_or_else(|| "local workflow filename is required".to_string())?;
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
    ) -> Result<zed::Command, String> {
        if context_server_id.as_ref() != "n8nac" {
            return Err(format!("unknown context server: {}", context_server_id.as_ref()));
        }

        Ok(zed::Command {
            command: self.cli_bin(),
            args: vec!["skills".to_string(), "mcp".to_string()],
            env: self.workspace_env(),
        })
    }
}

zed::register_extension!(N8nAsCodeExtension);

use std::env;
use zed_extension_api::{self as zed, process::Command as ProcessCommand};

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

    fn run_process(
        &self,
        subcommand: &str,
        args: &[String],
    ) -> Result<(Option<i32>, String, String), String> {
        let mut command = ProcessCommand::new(self.cli_bin());
        command = command
            .arg(subcommand)
            .args(args.iter().cloned())
            .envs(self.workspace_env());

        let output = command.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Ok((output.status, stdout, stderr))
    }

    fn extract_workflow_candidates(
        &self,
        list_args: &[String],
        mode: CandidateMode,
    ) -> Vec<String> {
        let Ok((_status, stdout, _stderr)) = self.run_process("list", list_args) else {
            return Vec::new();
        };

        let mut values = Vec::new();
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

            if columns.len() < 4 {
                continue;
            }

            let id = columns[1].clone();
            let local_path = columns[3].clone();
            match mode {
                CandidateMode::WorkflowId => {
                    if !id.is_empty() && id != "ID" {
                        values.push(id);
                    }
                }
                CandidateMode::Filename => {
                    if !local_path.is_empty()
                        && local_path != "Local Path"
                        && local_path.ends_with(".workflow.ts")
                    {
                        values.push(local_path);
                    }
                }
            }
        }

        values.sort();
        values.dedup();
        values
    }

    fn workflow_id_completions(&self) -> Vec<zed::SlashCommandArgumentCompletion> {
        self.extract_workflow_candidates(&["--remote".to_string()], CandidateMode::WorkflowId)
            .into_iter()
            .take(50)
            .map(|id| zed::SlashCommandArgumentCompletion {
                label: id.clone(),
                new_text: id,
                run_command: false,
            })
            .collect()
    }

    fn filename_completions(&self) -> Vec<zed::SlashCommandArgumentCompletion> {
        self.extract_workflow_candidates(&["--local".to_string()], CandidateMode::Filename)
            .into_iter()
            .take(50)
            .map(|filename| zed::SlashCommandArgumentCompletion {
                label: filename.clone(),
                new_text: filename,
                run_command: false,
            })
            .collect()
    }

    fn render_output(
        &self,
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

    fn run_cli_command(
        &self,
        subcommand: &str,
        args: Vec<String>,
    ) -> Result<zed::SlashCommandOutput, String> {
        let (status, stdout, stderr) = self.run_process(subcommand, &args)?;
        Ok(self.render_output(subcommand, &args, status, &stdout, &stderr))
    }
}

#[derive(Clone, Copy)]
enum CandidateMode {
    WorkflowId,
    Filename,
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
            "n8n-pull" | "n8n-verify" => {
                let completions = self.workflow_id_completions();
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
                let completions = self.filename_completions();
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
            command: self.cli_bin(),
            args: vec!["skills".to_string(), "mcp".to_string()],
            env: self.workspace_env(),
        })
    }
}

zed::register_extension!(N8nAsCodeExtension);

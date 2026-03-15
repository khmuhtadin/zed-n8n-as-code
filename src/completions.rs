/// Argument completion logic for slash commands
use zed_extension_api::{self as zed};

use crate::commands::run_cli_command;
use crate::settings::ExtensionSettings;

#[derive(Clone, Copy)]
pub enum CandidateMode {
    WorkflowId,
    Filename,
}

/// Extract workflow candidates from CLI list output
pub fn extract_workflow_candidates(
    settings: &ExtensionSettings,
    list_args: &[String],
    mode: CandidateMode,
) -> Vec<String> {
    let Ok((_, stdout, _)) = run_cli_command(settings, "list", list_args) else {
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

/// Get workflow ID completions
pub fn workflow_id_completions(
    settings: &ExtensionSettings,
) -> Vec<zed::SlashCommandArgumentCompletion> {
    extract_workflow_candidates(settings, &["--remote".to_string()], CandidateMode::WorkflowId)
        .into_iter()
        .take(50)
        .map(|id| zed::SlashCommandArgumentCompletion {
            label: id.clone(),
            new_text: id,
            run_command: false,
        })
        .collect()
}

/// Get filename completions
pub fn filename_completions(
    settings: &ExtensionSettings,
) -> Vec<zed::SlashCommandArgumentCompletion> {
    extract_workflow_candidates(settings, &["--local".to_string()], CandidateMode::Filename)
        .into_iter()
        .take(50)
        .map(|filename| zed::SlashCommandArgumentCompletion {
            label: filename.clone(),
            new_text: filename,
            run_command: false,
        })
        .collect()
}

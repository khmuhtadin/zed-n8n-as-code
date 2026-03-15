/// Extension settings management
///
/// Note: Zed extension API does not currently expose a way for extensions to define
/// their own custom settings in settings.json. Instead, we rely on environment variables
/// which users can set in their shell or in Zed's terminal configuration.
///
/// Future work: When Zed adds extension settings API, migrate to proper settings schema.

use std::env;

/// Extension settings loaded from environment
#[derive(Debug, Clone)]
pub struct ExtensionSettings {
    /// Path to n8nac CLI binary
    pub cli_bin: String,
    /// n8n instance base URL
    pub n8n_url: String,
    /// n8n API key for authentication
    pub n8n_api_key: Option<String>,
    /// Workspace directory override
    pub workspace: Option<String>,
    /// Enable native service (vs CLI-only mode)
    pub enable_native_service: bool,
}

impl ExtensionSettings {
    /// Load settings from environment variables
    pub fn from_env() -> Self {
        Self {
            cli_bin: env::var("N8NAC_BIN").unwrap_or_else(|_| "n8nac".to_string()),
            n8n_url: env::var("N8N_URL")
                .or_else(|_| env::var("N8N_HOST"))
                .unwrap_or_else(|_| "http://localhost:5678".to_string()),
            n8n_api_key: env::var("N8N_API_KEY").ok(),
            workspace: env::var("N8NAC_WORKSPACE").ok(),
            enable_native_service: env::var("N8NAC_NATIVE")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(true), // Default to enabled
        }
    }

    /// Get workspace as env var tuple for subprocess
    pub fn workspace_env(&self) -> Vec<(String, String)> {
        if let Some(ref ws) = self.workspace {
            vec![("N8NAC_WORKSPACE".to_string(), ws.clone())]
        } else {
            vec![]
        }
    }

    /// Check if native service should be used for a given operation
    pub fn should_use_native(&self) -> bool {
        self.enable_native_service && self.n8n_api_key.is_some()
    }

    /// Display current settings for debugging
    pub fn summary(&self) -> String {
        format!(
            "n8n-as-code Extension Settings:\n\
             - CLI Binary: {}\n\
             - n8n URL: {}\n\
             - API Key: {}\n\
             - Workspace: {}\n\
             - Native Service: {}",
            self.cli_bin,
            self.n8n_url,
            if self.n8n_api_key.is_some() {
                "configured"
            } else {
                "not set"
            },
            self.workspace.as_deref().unwrap_or("default"),
            if self.enable_native_service {
                "enabled"
            } else {
                "disabled"
            }
        )
    }
}

impl Default for ExtensionSettings {
    fn default() -> Self {
        Self::from_env()
    }
}

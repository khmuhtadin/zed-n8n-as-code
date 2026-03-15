/// Native n8n API service layer using Zed's http_client
use serde::{Deserialize, Serialize};
use zed_extension_api::http_client;

/// Configuration for the n8n API client
#[derive(Debug, Clone)]
pub struct N8nConfig {
    pub base_url: String,
    pub api_key: Option<String>,
}

impl N8nConfig {
    /// Create from environment variables or defaults
    pub fn from_env() -> Self {
        let base_url = std::env::var("N8N_URL")
            .or_else(|_| std::env::var("N8N_HOST"))
            .unwrap_or_else(|_| "http://localhost:5678".to_string());
        let api_key = std::env::var("N8N_API_KEY").ok();

        Self { base_url, api_key }
    }

    /// Validate that the base URL looks reasonable
    pub fn is_valid(&self) -> bool {
        self.base_url.starts_with("http://") || self.base_url.starts_with("https://")
    }
}

/// Simplified workflow metadata from n8n API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub tags: Vec<WorkflowTag>,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTag {
    pub id: String,
    pub name: String,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
}

/// Native n8n API client
pub struct N8nService {
    config: N8nConfig,
}

impl N8nService {
    pub fn new(config: N8nConfig) -> Self {
        Self { config }
    }

    /// Build HTTP request with auth headers
    fn build_request(&self, path: &str, method: http_client::HttpMethod) -> Result<http_client::HttpRequest, String> {
        let url = format!("{}{}", self.config.base_url, path);
        let mut builder = http_client::HttpRequestBuilder::new()
            .method(method)
            .url(url);

        if let Some(ref api_key) = self.config.api_key {
            builder = builder.header("X-N8N-API-KEY", api_key);
        }

        builder.build()
    }

    /// Check if n8n instance is reachable and healthy
    pub fn health_check(&self) -> Result<HealthStatus, String> {
        let request = self.build_request("/healthz", http_client::HttpMethod::Get)?;

        let response = http_client::fetch(&request)
            .map_err(|e| format!("Health check failed: {}", e))?;

        // Note: Zed's HttpResponse doesn't include status code
        // fetch() returns Err for non-success status codes
        // So if we get here, the request succeeded

        // n8n healthz returns: {"status":"ok"}
        let body_str = String::from_utf8_lossy(&response.body);
        serde_json::from_str(&body_str)
            .map_err(|e| format!("Failed to parse health response: {}", e))
    }

    /// List all workflows (lightweight metadata only)
    pub fn list_workflows(&self) -> Result<Vec<WorkflowMetadata>, String> {
        let request = self.build_request("/api/v1/workflows", http_client::HttpMethod::Get)?;

        let response = http_client::fetch(&request)
            .map_err(|e| format!("Failed to fetch workflows: {}", e))?;

        // n8n returns: {"data": [workflows...]}
        #[derive(Deserialize)]
        struct ListResponse {
            data: Vec<WorkflowMetadata>,
        }

        let body_str = String::from_utf8_lossy(&response.body);
        let parsed: ListResponse = serde_json::from_str(&body_str)
            .map_err(|e| format!("Failed to parse workflows: {}", e))?;

        Ok(parsed.data)
    }

    /// Get single workflow metadata
    pub fn get_workflow(&self, id: &str) -> Result<WorkflowMetadata, String> {
        let path = format!("/api/v1/workflows/{}", id);
        let request = self.build_request(&path, http_client::HttpMethod::Get)?;

        let response = http_client::fetch(&request)
            .map_err(|e| format!("Failed to fetch workflow: {}", e))?;

        let body_str = String::from_utf8_lossy(&response.body);
        serde_json::from_str(&body_str)
            .map_err(|e| format!("Failed to parse workflow: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let valid = N8nConfig {
            base_url: "http://localhost:5678".to_string(),
            api_key: None,
        };
        assert!(valid.is_valid());

        let invalid = N8nConfig {
            base_url: "localhost:5678".to_string(),
            api_key: None,
        };
        assert!(!invalid.is_valid());
    }
}

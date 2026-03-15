//! Claude settings.json generator for hook configuration.

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Claude Code settings.json structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<HooksConfig>,
}

/// Hooks configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HooksConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_tool_use: Option<Vec<HookMatcher>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_tool_use: Option<Vec<HookMatcher>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_start: Option<Vec<HookMatcher>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_end: Option<Vec<HookMatcher>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<HookMatcher>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_stop: Option<Vec<HookMatcher>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_request: Option<Vec<HookMatcher>>,
}

/// Hook matcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMatcher {
    pub matcher: String,
    pub hooks: Vec<Hook>,
}

/// Individual hook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    #[serde(rename = "type")]
    pub hook_type: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
}

/// Generate Claude settings with hooks that POST events to the axel server.
///
/// **WARNING: Shared File Problem**
///
/// This writes to `.claude/settings.json` which is **workspace-level** and shared by
/// ALL Claude instances in the workspace. Each call overwrites the previous hooks config,
/// meaning all Claude instances will send their hook events to the LAST terminal's server.
///
/// This is the root cause of the "events going to wrong terminal" bug. The OTEL-based
/// session→pane mapping in `routes.rs` corrects for this at the server level.
///
/// A proper fix would involve per-session hook configuration, which Claude Code
/// doesn't currently support.
pub fn generate_hooks_settings(port: u16, pane_id: &str) -> ClaudeSettings {
    let endpoint = format!("http://localhost:{}/events/{}", port, pane_id);

    // Create a curl command that reads from stdin and POSTs to the endpoint
    let curl_command = format!(
        "curl -s -X POST -H 'Content-Type: application/json' -d @- {}",
        endpoint
    );

    let create_hook = |_event_type: &str| -> Vec<HookMatcher> {
        vec![HookMatcher {
            matcher: "*".to_string(),
            hooks: vec![Hook {
                hook_type: "command".to_string(),
                command: curl_command.clone(),
                timeout: Some(5),
            }],
        }]
    };

    ClaudeSettings {
        hooks: Some(HooksConfig {
            pre_tool_use: Some(create_hook("PreToolUse")),
            post_tool_use: Some(create_hook("PostToolUse")),
            session_start: Some(create_hook("SessionStart")),
            session_end: Some(create_hook("SessionEnd")),
            stop: Some(create_hook("Stop")),
            subagent_stop: Some(create_hook("SubagentStop")),
            permission_request: Some(create_hook("PermissionRequest")),
        }),
    }
}

/// Get the OTEL exporter metrics endpoint URL with pane_id
/// Returns the full URL for OTEL_EXPORTER_OTLP_METRICS_ENDPOINT
pub fn otel_metrics_endpoint(port: u16, pane_id: &str) -> String {
    format!("http://localhost:{}/v1/metrics/{}", port, pane_id)
}

/// Get the OTEL exporter traces endpoint URL with pane_id
/// Returns the full URL for OTEL_EXPORTER_OTLP_TRACES_ENDPOINT
pub fn otel_traces_endpoint(port: u16, pane_id: &str) -> String {
    format!("http://localhost:{}/v1/traces/{}", port, pane_id)
}

/// Get the OTEL exporter logs endpoint URL with pane_id
/// Returns the full URL for OTEL_EXPORTER_OTLP_LOGS_ENDPOINT
pub fn otel_logs_endpoint(port: u16, pane_id: &str) -> String {
    format!("http://localhost:{}/v1/logs/{}", port, pane_id)
}

/// Write the Claude settings to a file
pub fn write_settings(settings: &ClaudeSettings, path: &Path) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Check if there's an existing settings file
    let final_settings = if path.exists() {
        // Read existing settings
        let content = std::fs::read_to_string(path)?;
        let mut existing: serde_json::Value = serde_json::from_str(&content)?;

        // Merge hooks into existing settings
        if let Some(hooks) = &settings.hooks {
            existing["hooks"] = serde_json::to_value(hooks)?;
        }

        existing
    } else {
        serde_json::to_value(settings)?
    };

    // Write the settings
    let json = serde_json::to_string_pretty(&final_settings)?;
    std::fs::write(path, json)?;

    Ok(())
}

/// Get the path to the Claude settings file in a workspace
pub fn settings_path(workspace_dir: &Path) -> std::path::PathBuf {
    workspace_dir.join(".claude").join("settings.json")
}

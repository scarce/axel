//! Codex skill driver.
//!
//! Codex expects skills in `.codex/skills/<skill-name>/SKILL.md` format.
//! This driver creates skill directories with symlinked SKILL.md files,
//! making them discoverable via the `/skills` slash command.
//!
//! Codex also uses AGENTS.md for project context (similar to Claude's CLAUDE.md).
//!
//! ## OpenTelemetry Support
//!
//! Codex supports OTEL telemetry export via `-c`/`--config` CLI flags.
//! Unlike Claude which uses environment variables, Codex requires config
//! file settings or CLI overrides for OTEL export.
//! See: https://developers.openai.com/codex/config-advanced/

use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{SkillDriver, claude::install_index_symlink};
use crate::{
    config::WorkspaceConfig,
    hooks::{otel_logs_endpoint, otel_metrics_endpoint, otel_traces_endpoint},
};

/// Codex skill driver
pub struct CodexDriver;

impl SkillDriver for CodexDriver {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn skills_dir(&self, workspace_dir: &Path) -> PathBuf {
        workspace_dir.join(".codex").join("skills")
    }

    fn skill_patterns(&self) -> &'static [&'static str] {
        &["AGENTS.md", ".codex/skills/*/SKILL.md"]
    }

    fn install_skills(&self, workspace_dir: &Path, skill_paths: &[PathBuf]) -> Result<usize> {
        if skill_paths.is_empty() {
            return Ok(0);
        }

        let skills_dir = self.skills_dir(workspace_dir);
        std::fs::create_dir_all(&skills_dir)?;

        let mut count = 0;
        for source_path in skill_paths {
            let name = derive_skill_name(source_path);

            // Codex expects: .codex/skills/<skill-name>/SKILL.md
            let skill_dir = skills_dir.join(&name);
            let link_path = skill_dir.join("SKILL.md");

            // Remove existing skill directory if present
            if skill_dir.exists() {
                std::fs::remove_dir_all(&skill_dir).ok();
            }

            // Create skill directory
            std::fs::create_dir_all(&skill_dir)?;

            // Canonicalize the source path to get a clean absolute path
            let canonical_source = source_path
                .canonicalize()
                .unwrap_or_else(|_| source_path.clone());

            // Create symlink to SKILL.md
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&canonical_source, &link_path)?;
                count += 1;
            }
        }

        Ok(count)
    }

    fn cleanup(&self, workspace_dir: &Path) -> bool {
        let mut cleaned = false;

        // Remove skill directories from .codex/skills/
        let skills_dir = self.skills_dir(workspace_dir);
        if skills_dir.exists()
            && let Ok(entries) = std::fs::read_dir(&skills_dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Check if this directory contains a SKILL.md symlink
                    let skill_md = path.join("SKILL.md");
                    if skill_md
                        .symlink_metadata()
                        .map(|m| m.file_type().is_symlink())
                        .unwrap_or(false)
                        && std::fs::remove_dir_all(&path).is_ok()
                    {
                        cleaned = true;
                    }
                }
            }
        }

        // Remove AGENTS.md symlink
        let agents_md = workspace_dir.join("AGENTS.md");
        if agents_md
            .symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
            && std::fs::remove_file(&agents_md).is_ok()
        {
            cleaned = true;
        }

        cleaned
    }

    fn supports_otel(&self) -> bool {
        true
    }

    fn otel_cli_args(&self, port: u16, pane_id: &str) -> Vec<String> {
        // Codex uses -c/--config flags for configuration overrides.
        // Unlike Claude which uses env vars, Codex requires config file or CLI flags.
        // See: https://developers.openai.com/codex/config-advanced/
        //
        // The values need to be shell-quoted because they contain special characters.
        let logs_endpoint = otel_logs_endpoint(port, pane_id);
        let metrics_endpoint = otel_metrics_endpoint(port, pane_id);
        let traces_endpoint = otel_traces_endpoint(port, pane_id);

        vec![
            // Enable analytics (required for metrics export)
            "-c".to_string(),
            "'analytics_enabled=true'".to_string(),
            // Enable bell notifications for approvals (allows tmux to detect them)
            "-c".to_string(),
            "'tui_notifications=\"always\"'".to_string(),
            "-c".to_string(),
            "'tui_notification_method=\"bel\"'".to_string(),
            // Disable paste burst detection so tmux send-keys works correctly
            // (otherwise Enter is treated as newline when sent shortly after text)
            "-c".to_string(),
            "'disable_paste_burst=true'".to_string(),
            // Configure log exporter (OTLP HTTP with JSON protocol)
            "-c".to_string(),
            format!(
                r#"'otel.exporter={{otlp-http={{endpoint="{}",protocol="json"}}}}'"#,
                logs_endpoint
            ),
            // Configure trace exporter
            "-c".to_string(),
            format!(
                r#"'otel.trace_exporter={{otlp-http={{endpoint="{}",protocol="json"}}}}'"#,
                traces_endpoint
            ),
            // Configure metrics exporter (override default Statsig)
            "-c".to_string(),
            format!(
                r#"'otel.metrics_exporter={{otlp-http={{endpoint="{}",protocol="json"}}}}'"#,
                metrics_endpoint
            ),
        ]
    }

    fn tmux_bell_hook_command(&self, port: u16, pane_id: &str) -> Option<String> {
        // Generate the command that tmux should run when a bell is detected.
        // This captures the pane content, checks for approval patterns, and sends to axel server.
        // The payload matches the HookEvent structure so it works with /events/{pane_id}.
        Some(format!(
            r#"run-shell 'content=$(tmux capture-pane -t {pane} -p -S -30 2>/dev/null); \
if echo "$content" | grep -q "Yes, proceed"; then \
  cmd=$(echo "$content" | grep -B10 "Yes, proceed" | grep -E "^[[:space:]]*[a-z]" | tail -1 | sed "s/^[[:space:]]*//"); \
  curl -s -X POST "http://localhost:{port}/events/{pane}" \
    -H "Content-Type: application/json" \
    -d "{{\\"hook_event_name\\":\\"PermissionRequest\\",\\"tool_name\\":\\"Bash\\",\\"session_id\\":\\"{pane}\\",\\"tool_input\\":{{\\"command\\":\\"$cmd\\"}}}}" >/dev/null 2>&1; \
elif echo "$content" | grep -q "Codex wants to edit"; then \
  file=$(echo "$content" | grep "Codex wants to edit" | sed "s/.*edit //" | tr -d "\\n"); \
  curl -s -X POST "http://localhost:{port}/events/{pane}" \
    -H "Content-Type: application/json" \
    -d "{{\\"hook_event_name\\":\\"PermissionRequest\\",\\"tool_name\\":\\"Edit\\",\\"session_id\\":\\"{pane}\\",\\"tool_input\\":{{\\"file_path\\":\\"$file\\"}}}}" >/dev/null 2>&1; \
fi'"#,
            pane = pane_id,
            port = port
        ))
    }

    fn index_filename(&self) -> Option<&'static str> {
        Some("AGENTS.md")
    }

    fn install_index(&self, config: &WorkspaceConfig, workspace_dir: &Path) -> Result<bool> {
        install_index_symlink(config, workspace_dir, "AGENTS.md")
    }
}

/// Derive skill name from file path.
///
/// Handles two naming conventions:
/// - `<name>/SKILL.md` -> uses the directory name
/// - `<name>.md` -> uses the file stem
fn derive_skill_name(path: &Path) -> String {
    if path.file_name().map(|n| n == "SKILL.md").unwrap_or(false) {
        path.parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "skill".to_string())
    } else {
        path.file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "skill".to_string())
    }
}

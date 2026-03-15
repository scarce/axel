//! Claude Code skill driver
//!
//! Installs skills to `.claude/skills/` directory as symlinks. Each skill
//! becomes available as `/skill-name` in Claude Code. Creates CLAUDE.md
//! symlink pointing to AXEL.md for project context.

use std::path::{Path, PathBuf};

use anyhow::Result;

use super::SkillDriver;
use crate::{
    config::WorkspaceConfig,
    hooks::{otel_metrics_endpoint, otel_traces_endpoint},
};

/// Helper to create index file symlink (e.g., CLAUDE.md, AGENTS.md) pointing to AXEL.md
pub(super) fn install_index_symlink(
    config: &WorkspaceConfig,
    workspace_dir: &Path,
    filename: &str,
) -> Result<bool> {
    let Some(manifest_path) = &config.manifest_path else {
        return Ok(false);
    };

    if !manifest_path.exists() {
        return Ok(false);
    }

    let link_path = workspace_dir.join(filename);

    // Remove existing symlink/file if present
    if link_path.exists() || link_path.is_symlink() {
        std::fs::remove_file(&link_path).ok();
    }

    // Canonicalize the source path
    let canonical_source = manifest_path
        .canonicalize()
        .unwrap_or_else(|_| manifest_path.clone());

    // Create symlink
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&canonical_source, &link_path)?;
        Ok(true)
    }

    #[cfg(not(unix))]
    Ok(false)
}

/// Claude Code skill driver
pub struct ClaudeDriver;

impl SkillDriver for ClaudeDriver {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn skills_dir(&self, workspace_dir: &Path) -> PathBuf {
        workspace_dir.join(".claude").join("skills")
    }

    fn skill_patterns(&self) -> &'static [&'static str] {
        &["CLAUDE.md", ".claude/skills/*/SKILL.md"]
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

            // Claude Code expects: .claude/skills/<skill-name>/SKILL.md
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

        // Remove skill directories from .claude/skills/
        // Each skill is a directory containing SKILL.md symlink
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

        // Remove CLAUDE.md symlink
        let claude_md = workspace_dir.join("CLAUDE.md");
        if claude_md
            .symlink_metadata()
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
            && std::fs::remove_file(&claude_md).is_ok()
        {
            cleaned = true;
        }

        cleaned
    }

    fn supports_otel(&self) -> bool {
        true
    }

    fn otel_env_vars(&self, port: u16, pane_id: &str) -> Vec<(String, String)> {
        vec![
            // Required: Enable telemetry
            ("CLAUDE_CODE_ENABLE_TELEMETRY".to_string(), "1".to_string()),
            // Required: Enable OTLP exporter for metrics and traces
            ("OTEL_METRICS_EXPORTER".to_string(), "otlp".to_string()),
            ("OTEL_TRACES_EXPORTER".to_string(), "otlp".to_string()),
            // Use HTTP JSON protocol (our server accepts this)
            (
                "OTEL_EXPORTER_OTLP_PROTOCOL".to_string(),
                "http/json".to_string(),
            ),
            // Set specific endpoints (not base OTEL_EXPORTER_OTLP_ENDPOINT which appends paths)
            (
                "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT".to_string(),
                otel_metrics_endpoint(port, pane_id),
            ),
            (
                "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT".to_string(),
                otel_traces_endpoint(port, pane_id),
            ),
            // Fast export intervals (2 seconds for near-realtime gauge updates)
            // OTEL_METRIC_EXPORT_INTERVAL - periodic metric reader export interval (ms)
            (
                "OTEL_METRIC_EXPORT_INTERVAL".to_string(),
                "2000".to_string(),
            ),
            // OTEL_BSP_SCHEDULE_DELAY - batch span processor schedule delay (ms)
            ("OTEL_BSP_SCHEDULE_DELAY".to_string(), "2000".to_string()),
        ]
    }

    fn index_filename(&self) -> Option<&'static str> {
        Some("CLAUDE.md")
    }

    fn install_index(&self, config: &WorkspaceConfig, workspace_dir: &Path) -> Result<bool> {
        install_index_symlink(config, workspace_dir, "CLAUDE.md")
    }
}

/// Derive skill name from file path
///
/// - For SKILL.md files, use parent directory name
/// - For other .md files, use filename without extension
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

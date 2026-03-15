//! Skill driver implementations
//!
//! This module provides the `SkillDriver` trait and implementations for various
//! AI coding tools (Claude Code, Codex, OpenCode, Antigravity). Drivers handle
//! installing skill files in tool-specific formats.

mod antigravity;
mod claude;
mod codex;
mod opencode;

use std::path::{Path, PathBuf};

pub use antigravity::AntigravityDriver;
use anyhow::Result;
pub use claude::ClaudeDriver;
pub use codex::CodexDriver;
pub use opencode::OpenCodeDriver;

use crate::config::WorkspaceConfig;

/// Trait for skill installation drivers
///
/// Each driver knows how to install skills for a specific tool (Claude Code, Codex, etc.)
pub trait SkillDriver {
    /// Driver name for display/config
    fn name(&self) -> &'static str;

    /// Directory where skills should be installed (relative to workspace)
    fn skills_dir(&self, workspace_dir: &Path) -> PathBuf;

    /// File patterns used by this driver for skill discovery
    ///
    /// Returns patterns like "CLAUDE.md", ".claude/skills/*.md", etc.
    fn skill_patterns(&self) -> &'static [&'static str];

    /// Install skills to the target directory
    ///
    /// Returns the number of skills installed.
    fn install_skills(&self, workspace_dir: &Path, skill_paths: &[PathBuf]) -> Result<usize>;

    /// Clean up installed skills from the workspace
    ///
    /// Returns true if any cleanup was performed.
    fn cleanup(&self, workspace_dir: &Path) -> bool;

    /// Get environment variables for OpenTelemetry configuration.
    ///
    /// Returns a list of (key, value) pairs to set when launching the shell.
    /// Default implementation returns empty vec (no OTEL support).
    fn otel_env_vars(&self, _port: u16, _pane_id: &str) -> Vec<(String, String)> {
        Vec::new()
    }

    /// Get CLI arguments for OpenTelemetry configuration.
    ///
    /// Returns CLI arguments to append to the command (e.g., `-c key=value` for Codex).
    /// Default implementation returns empty vec (use env vars instead).
    fn otel_cli_args(&self, _port: u16, _pane_id: &str) -> Vec<String> {
        Vec::new()
    }

    /// Whether this driver supports OpenTelemetry telemetry export.
    fn supports_otel(&self) -> bool {
        false
    }

    /// Generate a tmux hook command for bell-based approval detection.
    ///
    /// Some tools (like Codex) send terminal bells when they need approval.
    /// This method returns a tmux command that should be set as an alert-bell hook
    /// to capture pane content and send approval requests to the axel server.
    ///
    /// Returns None if the driver doesn't support bell-based approval detection.
    fn tmux_bell_hook_command(&self, _port: u16, _pane_id: &str) -> Option<String> {
        None
    }

    /// Install index file (e.g., CLAUDE.md, AGENTS.md) as symlink to AXEL.md.
    ///
    /// Each tool expects project context in a specific file:
    /// - Claude Code: CLAUDE.md
    /// - Codex: AGENTS.md
    /// - OpenCode: OPENCODE.md
    ///
    /// Returns true if the symlink was created.
    fn install_index(&self, _config: &WorkspaceConfig, _workspace_dir: &Path) -> Result<bool> {
        Ok(false)
    }

    /// The name of the index file this driver expects (e.g., "CLAUDE.md", "AGENTS.md").
    fn index_filename(&self) -> Option<&'static str> {
        None
    }
}

/// Get a driver by name
pub fn get_driver(name: &str) -> Option<Box<dyn SkillDriver>> {
    match name {
        "claude" => Some(Box::new(ClaudeDriver)),
        "codex" => Some(Box::new(CodexDriver)),
        "opencode" => Some(Box::new(OpenCodeDriver)),
        "antigravity" => Some(Box::new(AntigravityDriver)),
        _ => None,
    }
}

/// Get all available drivers
pub fn all_drivers() -> Vec<Box<dyn SkillDriver>> {
    vec![
        Box::new(ClaudeDriver),
        Box::new(CodexDriver),
        Box::new(OpenCodeDriver),
        Box::new(AntigravityDriver),
    ]
}

/// Get all skill file patterns from all drivers
pub fn all_skill_patterns() -> Vec<&'static str> {
    let mut patterns = Vec::new();
    // Also include generic skill patterns
    patterns.extend_from_slice(&["skills/*.md", "skills/*/SKILL.md"]);
    for driver in all_drivers() {
        patterns.extend_from_slice(driver.skill_patterns());
    }
    patterns
}

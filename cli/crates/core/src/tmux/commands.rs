//! Low-level tmux command wrappers
//!
//! This module provides builder-pattern wrappers for common tmux commands.

use std::process::{Command, Output};

use anyhow::{Context, Result};
use serde::Serialize;

/// Execute a tmux command and return the output
fn tmux(args: &[&str]) -> Result<Output> {
    Command::new("tmux")
        .args(args)
        .output()
        .context("Failed to execute tmux command")
}

/// Execute a tmux command and check if it succeeded (suppressing stderr)
fn tmux_status(args: &[&str]) -> Result<bool> {
    Ok(Command::new("tmux")
        .args(args)
        .stderr(std::process::Stdio::null())
        .status()?
        .success())
}

/// Execute a tmux command, returning an error if it fails
fn tmux_run(args: &[&str]) -> Result<()> {
    let status = Command::new("tmux").args(args).status()?;
    if !status.success() {
        anyhow::bail!("tmux command failed: {:?}", args);
    }
    Ok(())
}

// =============================================================================
// Session Commands
// =============================================================================

/// Check if we're currently inside a tmux session
pub fn in_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

/// Get the current tmux session name (if inside tmux)
pub fn current_session() -> Option<String> {
    if !in_tmux() {
        return None;
    }
    let output = tmux(&["display-message", "-p", "#S"]).ok()?;
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}

/// Check if a tmux session exists
pub fn has_session(name: &str) -> bool {
    tmux_status(&["has-session", "-t", name]).unwrap_or(false)
}

/// Information about a tmux session
#[derive(Debug, Clone, Serialize)]
pub struct SessionInfo {
    /// Session name
    pub name: String,
    /// Number of windows
    pub windows: u32,
    /// Number of panes (across all windows)
    pub panes: u32,
    /// Creation time (Unix timestamp)
    pub created: u64,
    /// Whether clients are attached
    pub attached: bool,
    /// Working directory (from axel environment)
    pub working_dir: Option<String>,
    /// Server port (from AXEL_PORT environment)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// Axel pane ID (from AXEL_PANE_ID environment)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axel_pane_id: Option<String>,
}

/// Get the total number of panes in a session
fn count_session_panes(session: &str) -> u32 {
    // list-panes -s lists all panes across all windows in a session
    tmux(&["list-panes", "-s", "-t", session])
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count() as u32)
        .unwrap_or(0)
}

/// List all tmux sessions (optionally filtered to axel sessions only)
pub fn list_sessions(axel_only: bool) -> Result<Vec<SessionInfo>> {
    let output = tmux(&[
        "list-sessions",
        "-F",
        "#{session_name}\t#{session_windows}\t#{session_created}\t#{session_attached}",
    ])?;

    if !output.status.success() {
        // No sessions exist
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut sessions = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 4 {
            let name = parts[0].to_string();

            // Check if this is an axel session by looking for AXEL_MANIFEST env var
            let manifest = get_environment(&name, "AXEL_MANIFEST");

            if axel_only && manifest.is_none() {
                continue;
            }

            // Extract working directory from manifest path
            let working_dir = manifest.as_ref().and_then(|m| {
                std::path::Path::new(m)
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
            });

            // Read port and pane_id from session environment
            let port = get_environment(&name, "AXEL_PORT").and_then(|p| p.parse::<u16>().ok());
            let axel_pane_id = get_environment(&name, "AXEL_PANE_ID");

            let panes = count_session_panes(&name);

            sessions.push(SessionInfo {
                name,
                windows: parts[1].parse().unwrap_or(0),
                panes,
                created: parts[2].parse().unwrap_or(0),
                attached: parts[3] == "1",
                working_dir,
                port,
                axel_pane_id,
            });
        }
    }

    Ok(sessions)
}

/// Kill a tmux session
pub fn kill_session(name: &str) -> Result<()> {
    tmux_run(&["kill-session", "-t", name])
}

/// Set an environment variable on a tmux session
pub fn set_environment(session: &str, key: &str, value: &str) -> Result<()> {
    tmux_run(&["set-environment", "-t", session, key, value])
}

/// Get an environment variable from a tmux session
pub fn get_environment(session: &str, key: &str) -> Option<String> {
    let output = tmux(&["show-environment", "-t", session, key]).ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Output format is "KEY=value" or "-KEY" (if unset)
    stdout
        .trim()
        .strip_prefix(&format!("{}=", key))
        .map(|v| v.to_string())
}

/// Attach to a tmux session
pub fn attach_session(name: &str) -> Result<()> {
    Command::new("tmux")
        .args(["attach-session", "-t", name])
        .status()?;
    Ok(())
}

/// Detach all clients from a tmux session
pub fn detach_session(name: &str) -> Result<()> {
    // Detach all clients from the session (silently ignore if no clients attached)
    Command::new("tmux")
        .args(["detach-client", "-s", name])
        .stderr(std::process::Stdio::null())
        .status()
        .ok();
    Ok(())
}

/// Builder for creating new tmux sessions
#[derive(Default)]
pub struct NewSession<'a> {
    name: Option<&'a str>,
    detached: bool,
    start_dir: Option<&'a str>,
    window_name: Option<&'a str>,
    shell_command: Option<&'a str>,
}

#[allow(dead_code)]
impl<'a> NewSession<'a> {
    /// Create a new session builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the session name
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Start the session detached
    pub fn detached(mut self) -> Self {
        self.detached = true;
        self
    }

    /// Set the starting directory
    pub fn start_directory(mut self, dir: &'a str) -> Self {
        self.start_dir = Some(dir);
        self
    }

    /// Set the initial window name
    pub fn window_name(mut self, name: &'a str) -> Self {
        self.window_name = Some(name);
        self
    }

    /// Set the shell command to run in the session
    pub fn shell_command(mut self, cmd: &'a str) -> Self {
        self.shell_command = Some(cmd);
        self
    }

    /// Execute the new-session command
    pub fn run(self) -> Result<()> {
        let mut args = vec!["new-session"];

        if self.detached {
            args.push("-d");
        }

        if let Some(name) = self.name {
            args.push("-s");
            args.push(name);
        }

        if let Some(dir) = self.start_dir {
            args.push("-c");
            args.push(dir);
        }

        if let Some(name) = self.window_name {
            args.push("-n");
            args.push(name);
        }

        // Shell command must come last
        if let Some(cmd) = self.shell_command {
            args.push(cmd);
        }

        tmux_run(&args)
    }
}

// =============================================================================
// Window Commands
// =============================================================================

/// Rename a tmux window
pub fn rename_window(target: &str, new_name: &str) -> Result<()> {
    tmux_run(&["rename-window", "-t", target, new_name])
}

// =============================================================================
// Pane Commands
// =============================================================================

/// Split direction
pub enum SplitDirection {
    /// Horizontal split
    Horizontal,
    /// Vertical split
    Vertical,
}

/// Builder for splitting windows
#[derive(Default)]
pub struct SplitWindow<'a> {
    target: Option<&'a str>,
    direction: Option<SplitDirection>,
    percentage: Option<u32>,
    start_dir: Option<&'a str>,
    shell_command: Option<&'a str>,
}

impl<'a> SplitWindow<'a> {
    /// Create a new split window builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the target pane
    pub fn target(mut self, target: &'a str) -> Self {
        self.target = Some(target);
        self
    }

    /// Split horizontally
    pub fn horizontal(mut self) -> Self {
        self.direction = Some(SplitDirection::Horizontal);
        self
    }

    /// Split vertically
    pub fn vertical(mut self) -> Self {
        self.direction = Some(SplitDirection::Vertical);
        self
    }

    /// Set the split percentage
    pub fn percentage(mut self, pct: u32) -> Self {
        self.percentage = Some(pct);
        self
    }

    /// Set the starting directory
    pub fn start_directory(mut self, dir: &'a str) -> Self {
        self.start_dir = Some(dir);
        self
    }

    /// Set the command to run in the new pane
    pub fn command(mut self, cmd: &'a str) -> Self {
        self.shell_command = Some(cmd);
        self
    }

    /// Run the split-window command and return the new pane ID
    pub fn run(self) -> Result<String> {
        let mut args = vec!["split-window".to_string()];

        if let Some(target) = self.target {
            args.push("-t".to_string());
            args.push(target.to_string());
        }

        match self.direction {
            Some(SplitDirection::Horizontal) => args.push("-h".to_string()),
            Some(SplitDirection::Vertical) => args.push("-v".to_string()),
            None => {}
        }

        if let Some(pct) = self.percentage {
            args.push("-p".to_string());
            args.push(pct.to_string());
        }

        if let Some(dir) = self.start_dir {
            args.push("-c".to_string());
            args.push(dir.to_string());
        }

        // Add -P -F to get the new pane ID
        args.push("-P".to_string());
        args.push("-F".to_string());
        args.push("#{pane_id}".to_string());

        if let Some(cmd) = self.shell_command {
            args.push(cmd.to_string());
        }

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let output = tmux(&args_ref)?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

/// Builder for selecting and configuring panes
#[derive(Default)]
pub struct SelectPane<'a> {
    target: Option<&'a str>,
    title: Option<&'a str>,
    style: Option<String>,
}

impl<'a> SelectPane<'a> {
    /// Create a new select pane builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the target pane
    pub fn target(mut self, target: &'a str) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the pane title
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Set the pane background color
    pub fn background(mut self, color: &str) -> Self {
        self.style = Some(format!("bg={}", color));
        self
    }

    /// Execute the select-pane command
    pub fn run(self) -> Result<()> {
        // Apply style if set
        if let Some(style) = &self.style {
            let mut args = vec!["select-pane"];
            if let Some(target) = self.target {
                args.push("-t");
                args.push(target);
            }
            args.push("-P");
            args.push(style);
            tmux_run(&args)?;
        }

        // Apply title if set
        if let Some(title) = self.title {
            let mut args = vec!["select-pane"];
            if let Some(target) = self.target {
                args.push("-t");
                args.push(target);
            }
            args.push("-T");
            args.push(title);
            tmux_run(&args)?;
        }

        // Just select if no style or title
        if self.style.is_none() && self.title.is_none() {
            let mut args = vec!["select-pane"];
            if let Some(target) = self.target {
                args.push("-t");
                args.push(target);
            }
            tmux_run(&args)?;
        }

        Ok(())
    }
}

/// Get the pane ID for a target
pub fn get_pane_id(target: &str) -> Result<String> {
    let output = tmux(&["display-message", "-t", target, "-p", "#{pane_id}"])?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Send keys to a pane
pub fn send_keys(target: &str, keys: &str) -> Result<()> {
    tmux_run(&["send-keys", "-t", target, keys, "Enter"])
}

/// Bind a key in a specific key table
pub fn bind_key(table: &str, key: &str, command: &[&str]) -> Result<()> {
    let mut args = vec!["bind-key", "-T", table, key];
    args.extend(command);
    tmux_run(&args)
}

// =============================================================================
// Option Commands
// =============================================================================

/// Builder for setting tmux options
#[derive(Default)]
pub struct SetOption<'a> {
    target: Option<&'a str>,
    global: bool,
    server: bool,
    window: bool,
    option: Option<&'a str>,
    value: Option<&'a str>,
}

#[allow(dead_code)]
impl<'a> SetOption<'a> {
    /// Create a new set option builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the target session/window
    pub fn target(mut self, target: &'a str) -> Self {
        self.target = Some(target);
        self
    }

    /// Set as a global option
    pub fn global(mut self) -> Self {
        self.global = true;
        self
    }

    /// Set as a server option
    pub fn server(mut self) -> Self {
        self.server = true;
        self
    }

    /// Set as a window option
    pub fn window(mut self) -> Self {
        self.window = true;
        self
    }

    /// Set the option name
    pub fn option(mut self, opt: &'a str) -> Self {
        self.option = Some(opt);
        self
    }

    /// Set the option value
    pub fn value(mut self, val: &'a str) -> Self {
        self.value = Some(val);
        self
    }

    /// Execute the set-option command
    pub fn run(self) -> Result<()> {
        let cmd = if self.window {
            "set-window-option"
        } else {
            "set-option"
        };

        let mut args = vec![cmd];

        if self.global {
            args.push("-g");
        }

        if self.server {
            args.push("-s");
        }

        if let Some(target) = self.target {
            args.push("-t");
            args.push(target);
        }

        if let Some(opt) = self.option {
            args.push(opt);
        }

        if let Some(val) = self.value {
            args.push(val);
        }

        tmux_run(&args)
    }
}

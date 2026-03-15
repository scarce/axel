//! Tmux workspace session management.
//!
//! This module provides high-level workspace creation using tmux sessions.
//! It handles the complex layout algorithm for arranging panes in a grid,
//! installing skills for each AI tool, and configuring tmux with axel styling.
//!
//! # Layout Algorithm
//!
//! Panes are organized in a column-major grid:
//! 1. Panes are sorted by (col, row)
//! 2. Columns are created via horizontal splits from left to right
//! 3. Rows within each column are created via vertical splits
//! 4. Width/height percentages are applied during splits
//!
//! # Session Features
//!
//! - Mouse support with clipboard integration
//! - Pane border titles showing shell names
//! - Color-coded panes based on shell configuration
//! - Automatic skill installation per driver type
//! - Manifest path stored in session environment for cleanup

use std::{collections::HashMap, io::Write};

use anyhow::Result;
use colored::Colorize;

use super::commands::{
    NewSession, SelectPane, SetOption, SplitWindow, bind_key, get_pane_id, rename_window,
    send_keys, set_environment,
};
use crate::{
    claude::ClaudeCommand,
    config::{
        AiPaneConfig, PaneConfig, ResolvedPane, WorkspaceConfig, WorkspaceIndex, expand_path,
        to_fg_rgb, to_tmux_color,
    },
    drivers,
    hooks::{otel_logs_endpoint, otel_metrics_endpoint, otel_traces_endpoint},
};

/// OTEL configuration for pane commands (used by macOS app integration)
#[derive(Clone)]
pub struct OtelConfig {
    pub port: u16,
    pub pane_id: String,
}

// =============================================================================
// Tmux option keys
// =============================================================================

const OPT_MOUSE: &str = "mouse";
const OPT_SET_CLIPBOARD: &str = "set-clipboard";
const OPT_ALLOW_PASSTHROUGH: &str = "allow-passthrough";
const OPT_EXTENDED_KEYS: &str = "extended-keys";
const OPT_PANE_BORDER_STATUS: &str = "pane-border-status";
const OPT_PANE_BORDER_FORMAT: &str = "pane-border-format";
const OPT_PANE_ACTIVE_BORDER_STYLE: &str = "pane-active-border-style";
const OPT_STATUS: &str = "status";
const OPT_ALLOW_RENAME: &str = "allow-rename";

// =============================================================================
// Tmux option values
// =============================================================================

const VAL_ON: &str = "on";
const VAL_OFF: &str = "off";
const VAL_TOP: &str = "top";

// =============================================================================
// Tmux key bindings
// =============================================================================

const KEY_TABLE_COPY_MODE: &str = "copy-mode";
const KEY_TABLE_ROOT: &str = "root";
const KEY_MOUSE_DRAG_END: &str = "MouseDragEnd1Pane";
const KEY_WHEEL_UP: &str = "WheelUpPane";
const KEY_WHEEL_DOWN: &str = "WheelDownPane";

// =============================================================================
// Axel-specific constants
// =============================================================================

/// Axel accent color (purple)
const AXEL_COLOR: &str = "#A78BFA";
/// Pane border format template
const PANE_BORDER_FORMAT: &str = "#[align=centre] #{pane_title} ";

/// Environment variable name for storing manifest path in tmux session
pub const AXEL_MANIFEST_ENV: &str = "AXEL_MANIFEST";

/// Environment variable name for storing the server port in tmux session
pub const AXEL_PORT_ENV: &str = "AXEL_PORT";

/// Environment variable name for storing the pane ID in tmux session
pub const AXEL_PANE_ID_ENV: &str = "AXEL_PANE_ID";

/// Environment variable name for storing the parent worktree path (when --isolate is used)
pub const AXEL_ISOLATE_PARENT_ENV: &str = "AXEL_ISOLATE_PARENT";

/// Build the command string for an AI pane (Claude or OpenCode).
///
/// Both Claude Code and OpenCode use similar CLI interfaces, so this function
/// handles both by parameterizing the command name. The command is built using
/// `ClaudeCommand` builder which handles argument escaping and formatting.
///
/// Note: The `_index` parameter is unused because index content is handled via
/// CLAUDE.md symlink for Claude (installed by the driver).
fn build_ai_command(
    command_name: &str,
    config: &AiPaneConfig,
    _index: Option<&WorkspaceIndex>,
) -> String {
    let mut cmd = ClaudeCommand::new();

    if let Some(model) = &config.model {
        cmd = cmd.model(model);
    }
    if !config.allowed_tools.is_empty() {
        cmd = cmd.allowed_tools(config.allowed_tools.clone());
    }
    if !config.disallowed_tools.is_empty() {
        cmd = cmd.disallowed_tools(config.disallowed_tools.clone());
    }
    // Only use explicit prompt - index is handled via CLAUDE.md symlink for Claude
    if let Some(prompt) = &config.prompt {
        cmd = cmd.prompt(prompt);
    }
    for arg in &config.args {
        cmd = cmd.extra_arg(arg);
    }

    let built = cmd.build();
    // Replace "claude" with actual command if different
    if command_name != "claude" {
        built.replacen("claude", command_name, 1)
    } else {
        built
    }
}

/// Build the command string for Antigravity CLI.
///
/// Antigravity is Google's AI coding assistant. It automatically discovers
/// project rules from `.antigravity/rules.md` (where axel installs skills).
///
/// The CLI interface supports:
/// - `-m` for model selection
/// - Initial prompt as a positional argument
fn build_antigravity_command(config: &AiPaneConfig, index: Option<&WorkspaceIndex>) -> String {
    let mut parts = vec!["antigravity".to_string()];

    if let Some(model) = &config.model {
        parts.push("-m".to_string());
        parts.push(model.clone());
    }

    for arg in &config.args {
        parts.push(arg.clone());
    }

    // Use single quotes for shell safety
    if let Some(prompt) = &config.prompt {
        let escaped = prompt.replace('\'', "'\\''");
        parts.push(format!("'{}'", escaped));
    } else if let Some(idx) = index {
        let escaped = idx.to_initial_prompt().replace('\'', "'\\''");
        parts.push(format!("'{}'", escaped));
    }

    parts.join(" ")
}

/// Build the command string for Codex CLI.
///
/// Codex has a different CLI interface than Claude/OpenCode. Key differences:
/// - Uses `-c` for config options instead of dedicated flags
/// - Agents are discovered via `project_doc_fallback_filenames` config
/// - Initial prompt is passed as a positional argument
///
/// The command includes `-c 'project_doc_fallback_filenames=[".codex/AGENTS.md"]'`
/// to ensure Codex discovers the merged skills file created by the driver.
fn build_codex_command(
    config: &AiPaneConfig,
    _workspace_dir: Option<&std::path::Path>,
    index: Option<&WorkspaceIndex>,
    otel_config: Option<&OtelConfig>,
) -> String {
    let mut parts = vec!["codex".to_string()];

    // Add .codex/AGENTS.md to fallback filenames so Codex discovers it
    parts.push("-c".to_string());
    parts.push("'project_doc_fallback_filenames=[\".codex/AGENTS.md\"]'".to_string());

    // Add OTEL configuration if provided (macOS app integration)
    if let Some(otel) = otel_config {
        let logs_endpoint = otel_logs_endpoint(otel.port, &otel.pane_id);
        let traces_endpoint = otel_traces_endpoint(otel.port, &otel.pane_id);
        let metrics_endpoint = otel_metrics_endpoint(otel.port, &otel.pane_id);

        // Enable analytics (required for metrics export)
        parts.push("-c".to_string());
        parts.push("'analytics_enabled=true'".to_string());
        // Enable bell notifications for approvals (allows tmux to detect them)
        parts.push("-c".to_string());
        parts.push("'tui_notifications=\"always\"'".to_string());
        parts.push("-c".to_string());
        parts.push("'tui_notification_method=\"bel\"'".to_string());
        // Disable paste burst detection so tmux send-keys works correctly
        parts.push("-c".to_string());
        parts.push("'disable_paste_burst=true'".to_string());
        // Configure OTEL exporters
        parts.push("-c".to_string());
        parts.push(format!(
            r#"'otel.exporter={{otlp-http={{endpoint="{}",protocol="json"}}}}'"#,
            logs_endpoint
        ));
        parts.push("-c".to_string());
        parts.push(format!(
            r#"'otel.trace_exporter={{otlp-http={{endpoint="{}",protocol="json"}}}}'"#,
            traces_endpoint
        ));
        parts.push("-c".to_string());
        parts.push(format!(
            r#"'otel.metrics_exporter={{otlp-http={{endpoint="{}",protocol="json"}}}}'"#,
            metrics_endpoint
        ));
    }

    if let Some(model) = &config.model {
        parts.push("-m".to_string());
        parts.push(model.clone());
    }

    for arg in &config.args {
        parts.push(arg.clone());
    }

    // Use single quotes for shell safety
    if let Some(prompt) = &config.prompt {
        let escaped = prompt.replace('\'', "'\\''");
        parts.push(format!("'{}'", escaped));
    } else if let Some(idx) = index {
        let escaped = idx.to_initial_prompt().replace('\'', "'\\''");
        parts.push(format!("'{}'", escaped));
    }

    parts.join(" ")
}

/// Build the command to run for a pane
pub fn build_pane_command(
    pane: &ResolvedPane,
    workspace_dir: Option<&std::path::Path>,
    index: Option<&WorkspaceIndex>,
    otel_config: Option<&OtelConfig>,
) -> Option<String> {
    match &pane.config {
        PaneConfig::Claude(config) => Some(build_ai_command("claude", config, index)),
        PaneConfig::Codex(config) => Some(build_codex_command(
            config,
            workspace_dir,
            index,
            otel_config,
        )),
        PaneConfig::Opencode(config) => Some(build_ai_command("opencode", config, index)),
        PaneConfig::Antigravity(config) => Some(build_antigravity_command(config, index)),
        PaneConfig::Custom(config) => config.command.clone(),
    }
}

/// Create a tmux workspace from a configuration.
///
/// This is the main entry point for workspace creation. It:
///
/// 1. **Resolves panes** from the profile configuration
/// 2. **Installs skills** for each AI driver (Claude, Codex, OpenCode)
/// 3. **Creates the tmux session** with the first pane
/// 4. **Configures session options** (mouse, clipboard, styling)
/// 5. **Builds the grid layout** via horizontal/vertical splits
/// 6. **Sends commands** to each pane to launch the shells
///
/// The layout algorithm groups panes by column, creates columns via horizontal
/// splits, then creates rows within each column via vertical splits. Width/height
/// percentages are applied during the split operations.
///
/// The optional `otel_config` parameter enables OTEL telemetry for non-Claude
/// AI panes (Codex, OpenCode) when launched from the macOS app.
pub fn create_workspace(
    session_name: &str,
    config: &WorkspaceConfig,
    profile: Option<&str>,
    otel_config: Option<OtelConfig>,
) -> Result<()> {
    let mut panes = config.resolve_panes(profile);
    let workspace_dir = config.workspace_dir();
    let index = config.load_index();

    if panes.is_empty() {
        anyhow::bail!("No panes defined");
    }

    // Collect skill names per driver type from AI panes
    let mut claude_skills: Vec<String> = Vec::new();
    let mut codex_skills: Vec<String> = Vec::new();
    let mut opencode_skills: Vec<String> = Vec::new();
    let mut antigravity_skills: Vec<String> = Vec::new();

    for pane in &panes {
        match &pane.config {
            PaneConfig::Claude(c) => claude_skills.extend(c.skills.iter().cloned()),
            PaneConfig::Codex(c) => codex_skills.extend(c.skills.iter().cloned()),
            PaneConfig::Opencode(c) => opencode_skills.extend(c.skills.iter().cloned()),
            PaneConfig::Antigravity(c) => antigravity_skills.extend(c.skills.iter().cloned()),
            PaneConfig::Custom(_) => {}
        }
    }
    claude_skills.dedup();
    codex_skills.dedup();
    opencode_skills.dedup();
    antigravity_skills.dedup();

    // Install skills for each driver that has panes
    if let Some(ref workspace_dir) = workspace_dir {
        for (driver_name, skill_names) in [
            ("claude", &claude_skills),
            ("codex", &codex_skills),
            ("opencode", &opencode_skills),
            ("antigravity", &antigravity_skills),
        ] {
            if skill_names.is_empty() {
                continue;
            }
            let Some(driver) = drivers::get_driver(driver_name) else {
                continue;
            };
            let skill_paths = config.resolve_skills(skill_names);

            if let Some(count) = driver
                .install_skills(workspace_dir, &skill_paths)
                .ok()
                .filter(|&c| c > 0)
            {
                let skills_word = if count == 1 { "skill" } else { "skills" };
                eprintln!(
                    "{} {} {} {} for {}",
                    "✔".green(),
                    "Installed".dimmed(),
                    count,
                    skills_word,
                    driver.name()
                );
            }
        }

        // Install index files (CLAUDE.md, AGENTS.md, etc.) for each driver type with panes
        let driver_names: Vec<&str> = panes
            .iter()
            .filter_map(|p| match &p.config {
                PaneConfig::Claude(_) => Some("claude"),
                PaneConfig::Codex(_) => Some("codex"),
                PaneConfig::Opencode(_) => Some("opencode"),
                PaneConfig::Antigravity(_) => Some("antigravity"),
                PaneConfig::Custom(_) => None,
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for driver_name in driver_names {
            if let Some(driver) = drivers::get_driver(driver_name)
                && let Some(filename) = driver.index_filename()
                && driver.install_index(config, workspace_dir).unwrap_or(false)
            {
                eprintln!(
                    "{} {} {} symlink",
                    "✔".green(),
                    "Created".dimmed(),
                    filename
                );
            }
        }
    }

    // Sort panes by col, then row
    panes.sort_by(|a, b| a.col.cmp(&b.col).then(a.row.cmp(&b.row)));

    // Group panes by column
    let mut columns: HashMap<u32, Vec<&ResolvedPane>> = HashMap::new();
    let mut col_widths: HashMap<u32, u32> = HashMap::new();
    let mut max_col = 0;

    for pane in &panes {
        columns.entry(pane.col).or_default().push(pane);
        if let Some(width) = pane.width {
            col_widths.insert(pane.col, width);
        }
        if pane.col > max_col {
            max_col = pane.col;
        }
    }

    // Create session with first pane
    let first_pane = &panes[0];
    let first_path = first_pane
        .path()
        .map(expand_path)
        .unwrap_or_else(|| ".".to_string());

    NewSession::new()
        .name(session_name)
        .detached()
        .start_directory(&first_path)
        .run()?;

    // Store manifest path in session environment for cleanup on kill
    if let Some(manifest_path) = &config.manifest_path
        && let Some(path_str) = manifest_path.to_str()
    {
        set_environment(session_name, AXEL_MANIFEST_ENV, path_str).ok();
    }

    // Store OTEL config (port and pane_id) in session environment for recovery
    if let Some(ref otel) = otel_config {
        set_environment(session_name, AXEL_PORT_ENV, &otel.port.to_string()).ok();
        set_environment(session_name, AXEL_PANE_ID_ENV, &otel.pane_id).ok();
    }

    // Configure session options
    SetOption::new()
        .server()
        .option(OPT_MOUSE)
        .value(VAL_ON)
        .run()?;

    SetOption::new()
        .global()
        .option(OPT_MOUSE)
        .value(VAL_ON)
        .run()?;

    SetOption::new()
        .target(session_name)
        .option(OPT_MOUSE)
        .value(VAL_ON)
        .run()?;

    SetOption::new()
        .target(session_name)
        .option(OPT_SET_CLIPBOARD)
        .value(VAL_ON)
        .run()?;

    SetOption::new()
        .global()
        .option(OPT_ALLOW_PASSTHROUGH)
        .value(VAL_ON)
        .run()
        .ok();

    SetOption::new()
        .target(session_name)
        .option(OPT_EXTENDED_KEYS)
        .value(VAL_ON)
        .run()
        .ok();

    SetOption::new()
        .target(session_name)
        .option(OPT_PANE_BORDER_STATUS)
        .value(VAL_TOP)
        .run()?;

    SetOption::new()
        .target(session_name)
        .option(OPT_PANE_BORDER_FORMAT)
        .value(PANE_BORDER_FORMAT)
        .run()?;

    SetOption::new()
        .target(session_name)
        .option(OPT_PANE_ACTIVE_BORDER_STYLE)
        .value(&format!("fg={}", AXEL_COLOR))
        .run()?;

    configure_status_bar(session_name)?;

    SetOption::new()
        .window()
        .target(session_name)
        .option(OPT_ALLOW_RENAME)
        .value(VAL_OFF)
        .run()?;

    // Fix mouse behavior after copy
    bind_key(
        KEY_TABLE_COPY_MODE,
        KEY_MOUSE_DRAG_END,
        &["send-keys", "-X", "copy-pipe-and-cancel"],
    )?;

    // Slow down mouse wheel scroll in copy-mode
    bind_key(
        KEY_TABLE_COPY_MODE,
        KEY_WHEEL_UP,
        &["send-keys", "-X", "scroll-up"],
    )
    .ok();
    bind_key(
        KEY_TABLE_COPY_MODE,
        KEY_WHEEL_DOWN,
        &["send-keys", "-X", "scroll-down"],
    )
    .ok();

    // Enable mouse wheel scrolling in root mode
    // - If in alternate screen (vim, less, etc.), send mouse events to the app
    // - Otherwise, enter copy-mode and scroll the scrollback buffer
    bind_key(
        KEY_TABLE_ROOT,
        KEY_WHEEL_UP,
        &[
            "if-shell",
            "-F",
            "#{alternate_on}",
            "send-keys -M",
            "copy-mode -e; send-keys -M",
        ],
    )
    .ok();
    bind_key(
        KEY_TABLE_ROOT,
        KEY_WHEEL_DOWN,
        &[
            "if-shell",
            "-F",
            "#{alternate_on}",
            "send-keys -M",
            "copy-mode -e; send-keys -M",
        ],
    )
    .ok();

    rename_window(session_name, &config.workspace)?;

    // Track pane IDs per column and collect all panes for later configuration
    let mut col_first_ids: HashMap<u32, String> = HashMap::new();
    let mut col_last_ids: HashMap<u32, String> = HashMap::new();
    let mut all_panes: Vec<(String, ResolvedPane)> = Vec::new();

    // Get first pane ID and send command if needed
    let first_pane_target = format!("{}:0.0", session_name);
    let first_id = get_pane_id(&first_pane_target)?;

    if let Some(cmd) = build_pane_command(
        first_pane,
        workspace_dir.as_deref(),
        index.as_ref(),
        otel_config.as_ref(),
    ) {
        std::thread::sleep(std::time::Duration::from_millis(200));
        send_keys(&first_id, &cmd)?;
    }
    col_first_ids.insert(0, first_id.clone());
    col_last_ids.insert(0, first_id.clone());
    all_panes.push((first_id, first_pane.clone()));

    let mut pane_counter = 1;

    // Create columns (horizontal splits)
    for col in 1..=max_col {
        let Some(col_panes) = columns.get(&col) else {
            continue;
        };
        let first_col_pane = col_panes[0];

        let path = first_col_pane
            .path()
            .map(expand_path)
            .unwrap_or_else(|| ".".to_string());

        let wrapper = create_wrapper_script(pane_counter, first_col_pane)?;

        let prev_col = col - 1;
        let target_id = col_first_ids.get(&prev_col).unwrap();

        let mut split = SplitWindow::new()
            .target(target_id)
            .horizontal()
            .start_directory(&path)
            .command(&wrapper);

        if let Some(width) = col_widths.get(&col) {
            split = split.percentage(*width);
        }

        let new_id = split.run()?;
        all_panes.push((new_id.clone(), first_col_pane.clone()));

        if let Some(cmd) = build_pane_command(
            first_col_pane,
            workspace_dir.as_deref(),
            index.as_ref(),
            otel_config.as_ref(),
        ) {
            std::thread::sleep(std::time::Duration::from_millis(200));
            send_keys(&new_id, &cmd)?;
        }

        col_first_ids.insert(col, new_id.clone());
        col_last_ids.insert(col, new_id);
        pane_counter += 1;
    }

    // Create rows within each column (vertical splits)
    for col in 0..=max_col {
        let Some(col_panes) = columns.get(&col) else {
            continue;
        };

        let num_rows = col_panes.len();

        for (row_idx, &pane) in col_panes.iter().enumerate().skip(1) {
            let path = pane
                .path()
                .map(expand_path)
                .unwrap_or_else(|| ".".to_string());

            let wrapper = create_wrapper_script(pane_counter, pane)?;

            let target_id = col_last_ids.get(&col).unwrap();

            let height_pct = pane.height.unwrap_or_else(|| {
                let remaining = num_rows - row_idx;
                (remaining as u32 * 100) / (remaining as u32 + 1)
            });

            let new_id = SplitWindow::new()
                .target(target_id)
                .vertical()
                .percentage(height_pct)
                .start_directory(&path)
                .command(&wrapper)
                .run()?;

            all_panes.push((new_id.clone(), pane.clone()));

            if let Some(cmd) = build_pane_command(
                pane,
                workspace_dir.as_deref(),
                index.as_ref(),
                otel_config.as_ref(),
            ) {
                std::thread::sleep(std::time::Duration::from_millis(200));
                send_keys(&new_id, &cmd)?;
            }

            col_last_ids.insert(col, new_id);
            pane_counter += 1;
        }
    }

    // Wait for all shells to initialize, then configure panes
    std::thread::sleep(std::time::Duration::from_millis(500));
    for (pane_id, pane) in &all_panes {
        configure_pane(pane_id, pane)?;
    }

    // Select first pane
    SelectPane::new()
        .target(&format!("{}:0.0", session_name))
        .run()?;

    Ok(())
}

/// Disable the tmux status bar for a session.
pub fn configure_status_bar(session_name: &str) -> Result<()> {
    SetOption::new()
        .target(session_name)
        .option(OPT_STATUS)
        .value(VAL_OFF)
        .run()?;

    Ok(())
}

/// Configure a pane's title and background color.
///
/// Called after all panes are created to set visual properties. The title
/// appears in the pane border, and the background color is set if configured.
fn configure_pane(target: &str, pane: &ResolvedPane) -> Result<()> {
    let mut select = SelectPane::new().target(target).title(&pane.name);

    if let Some(color) = pane.color() {
        let tmux_color = to_tmux_color(color);
        if tmux_color != "default" {
            select = select.background(tmux_color);
        }
    }

    select.run()
}

/// Create a temporary bash wrapper script for a pane.
///
/// The wrapper script:
/// 1. Clears the terminal
/// 2. Displays pane notes (if configured) or a simple title
/// 3. Removes itself from disk (self-cleaning)
/// 4. Execs into fish shell with greeting and title disabled
///
/// This approach allows displaying startup information before the shell
/// takes over, while keeping the pane in a clean state.
fn create_wrapper_script(id: usize, pane: &ResolvedPane) -> Result<String> {
    let wrapper_path = format!("/tmp/axel_ws_{}", id);
    let mut file = std::fs::File::create(&wrapper_path)?;

    writeln!(file, "#!/bin/bash")?;
    writeln!(file, "clear")?;

    let fg_rgb = pane.color().map(to_fg_rgb).unwrap_or("255;255;255");

    if !pane.notes().is_empty() {
        writeln!(file, "COLS=$(tput cols)")?;
        writeln!(file, "printf '\\e[38;2;{}m'", fg_rgb)?;

        let first_note = pane.notes().first().map(|s| s.trim()).unwrap_or("");
        let first_note_len = first_note.chars().count();
        writeln!(
            file,
            "printf ' notes | {}%*s\\n' \"$((COLS - {} - 10))\" ''",
            first_note.replace('\'', "'\\''"),
            first_note_len
        )?;

        for note in pane.notes().iter().skip(1) {
            let note = note.trim();
            let note_len = note.chars().count();
            writeln!(
                file,
                "printf '       | {}%*s\\n' \"$((COLS - {} - 10))\" ''",
                note.replace('\'', "'\\''"),
                note_len
            )?;
        }

        writeln!(file, "printf '\\e[0m'")?;
    } else {
        writeln!(
            file,
            "printf '%b\\n' $'\\e'\"[38;2;{}m- {} -\"$'\\e'\"[0m\"",
            fg_rgb, pane.name
        )?;
    }

    writeln!(file, "rm '{}'", wrapper_path)?;
    writeln!(file, "if command -v fish >/dev/null 2>&1; then")?;
    writeln!(
        file,
        "  exec fish -C 'set fish_greeting; function fish_title; end'"
    )?;
    writeln!(file, "else")?;
    writeln!(file, "  exec \"$SHELL\"")?;
    writeln!(file, "fi")?;

    drop(file);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrapper_path, std::fs::Permissions::from_mode(0o755))?;
    }

    Ok(wrapper_path)
}

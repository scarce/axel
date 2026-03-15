//! Session management commands for axel.
//!
//! This module handles tmux session lifecycle:
//! - Listing running sessions
//! - Launching workspaces (shell, tmux, tmux_cc modes)
//! - Killing sessions with cleanup

use std::path::{Path, PathBuf};

use anyhow::Result;
use axel_core::{
    GridType, PaneConfig,
    claude::ClaudeCommand,
    config::{expand_path, load_config},
    drivers, generate_hooks_settings, git, settings_path,
    tmux::{
        AXEL_ISOLATE_PARENT_ENV, AXEL_MANIFEST_ENV, AXEL_PANE_ID_ENV, AXEL_PORT_ENV, NewSession,
        OtelConfig, SetOption, attach_session, configure_status_bar,
        create_workspace as tmux_create_workspace, detach_session, get_environment, has_session,
        kill_session, list_sessions, set_environment,
    },
    write_settings,
};
use colored::Colorize;

use crate::{
    commands::skill::{cleanup_skills, format_cleaned_drivers},
    display_path,
};

// =============================================================================
// Session Listing
// =============================================================================

/// List running tmux sessions.
///
/// If `axel_only` is true, only shows sessions created by axel
/// (identified by the AXEL_MANIFEST environment variable).
/// If `json_output` is true, outputs JSON format for programmatic access.
pub fn do_list_sessions(axel_only: bool, json_output: bool) -> Result<()> {
    let sessions = list_sessions(axel_only)?;

    if json_output {
        let json = serde_json::to_string_pretty(&sessions)?;
        println!("{}", json);
        return Ok(());
    }

    if sessions.is_empty() {
        if axel_only {
            println!("{}", "No axel sessions running".dimmed());
        } else {
            println!("{}", "No tmux sessions running".dimmed());
        }
        return Ok(());
    }

    use comfy_table::{Table, presets::NOTHING};

    let mut table = Table::new();
    table.load_preset(NOTHING);

    for session in &sessions {
        let attached = if session.attached {
            "(attached)".green().to_string()
        } else {
            String::new()
        };

        let location = session
            .working_dir
            .as_ref()
            .map(|d| display_path(Path::new(d)))
            .unwrap_or_else(|| "-".to_string());

        let panes_label = if session.panes == 1 { "pane" } else { "panes" };
        table.add_row(vec![
            session.name.blue().to_string(),
            location.dimmed().to_string(),
            format!("{} {}", session.panes, panes_label)
                .dimmed()
                .to_string(),
            attached,
        ]);
    }

    println!("{table}");

    Ok(())
}

// =============================================================================
// Session Killing
// =============================================================================

/// Kill all running axel sessions.
pub fn do_kill_all_sessions(
    _workspaces_dir: &Path,
    keep_skills: bool,
    skip_confirm: bool,
) -> Result<()> {
    let sessions = list_sessions(true)?; // true = axel_only

    if sessions.is_empty() {
        println!("{}", "No axel sessions running".dimmed());
        return Ok(());
    }

    println!(
        "{} {} axel session(s) running:",
        "Found".dimmed(),
        sessions.len()
    );
    for session in &sessions {
        let attached = if session.attached { " (attached)" } else { "" };
        println!(
            "  {} {}{}",
            "-".dimmed(),
            session.name.blue(),
            attached.dimmed()
        );
    }
    println!();

    if !skip_confirm {
        use dialoguer::{Confirm, theme::ColorfulTheme};
        let theme = ColorfulTheme::default();
        let confirmed = Confirm::with_theme(&theme)
            .with_prompt(format!("Kill all {} session(s)?", sessions.len()))
            .default(false)
            .interact()?;

        if !confirmed {
            println!("{}", "Cancelled".dimmed());
            return Ok(());
        }
    }

    let mut killed = 0;
    for session in &sessions {
        // Detach clients first to avoid issues
        detach_session(&session.name)?;

        // Clean up skills if not keeping them
        if !keep_skills && let Some(ref working_dir) = session.working_dir {
            let dir = PathBuf::from(working_dir);
            cleanup_skills(&dir);
        }

        // Kill the session
        if kill_session(&session.name).is_ok() {
            killed += 1;
            println!("{} {} {}", "✔".green(), "Killed".dimmed(), session.name);
        } else {
            eprintln!("{} Failed to kill {}", "✘".red(), session.name);
        }
    }

    println!();
    println!("{} {} session(s)", "Killed".green(), killed);

    Ok(())
}

/// Kill a workspace session with optional cleanup.
pub fn do_kill_workspace(
    workspaces_dir: &Path,
    name: &str,
    keep_skills: bool,
    prune_worktree: bool,
    worktree_branch: Option<&str>,
    skip_confirm: bool,
) -> Result<()> {
    let resolved_name = if has_session(name) {
        name.to_string()
    } else {
        let sessions = list_sessions(false).unwrap_or_default();
        let matched = sessions
            .iter()
            .find(|session| session.axel_pane_id.as_deref() == Some(name))
            .map(|session| session.name.clone());

        if let Some(found) = matched {
            found
        } else {
            eprintln!("{} Session '{}' not found", "✘".red(), name);
            eprintln!();
            let _ = do_list_sessions(false, false);
            return Ok(());
        }
    };

    if !skip_confirm {
        use dialoguer::{Confirm, theme::ColorfulTheme};
        let theme = ColorfulTheme::default();
        let confirmed = Confirm::with_theme(&theme)
            .with_prompt(format!("Kill session '{}'?", resolved_name))
            .default(true)
            .interact()?;

        if !confirmed {
            println!("{}", "Cancelled".dimmed());
            return Ok(());
        }
    }

    // Skip skill cleanup for worktree sessions - the worktree directory
    // may be pruned anyway, and we don't want to accidentally clean the main repo
    let cleaned = if !keep_skills && worktree_branch.is_none() {
        let session_manifest =
            get_environment(&resolved_name, AXEL_MANIFEST_ENV).map(PathBuf::from);
        let config_path = workspaces_dir.join(&resolved_name).join("AXEL.md");
        let local_config = std::env::current_dir().ok().map(|d| d.join("AXEL.md"));

        let cfg = session_manifest
            .and_then(|p| load_config(&p).ok())
            .or_else(|| load_config(&config_path).ok())
            .or_else(|| local_config.and_then(|p| load_config(&p).ok()));

        cfg.and_then(|c| c.workspace_dir())
            .map(|dir| cleanup_skills(&dir))
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    detach_session(&resolved_name)?;
    kill_session(&resolved_name)?;

    println!(
        "{} {} {}",
        "✔".green(),
        "Killed workspace".dimmed(),
        resolved_name
    );

    if !cleaned.is_empty() {
        println!(
            "{} {} {} skills",
            "✔".green(),
            "Cleaned".dimmed(),
            format_cleaned_drivers(&cleaned)
        );
    }

    // Handle worktree pruning if requested
    if prune_worktree {
        if let Some(branch) = worktree_branch {
            let cwd = std::env::current_dir()?;
            if git::is_git_repo(&cwd) {
                match git::remove_worktree(&cwd, branch, true) {
                    Ok(true) => {
                        println!(
                            "{} {} {}",
                            "✔".green(),
                            "Removed worktree for".dimmed(),
                            branch.blue()
                        );
                    }
                    Ok(false) => {
                        eprintln!("{} No worktree found for branch '{}'", "⚠".yellow(), branch);
                    }
                    Err(e) => {
                        eprintln!("{} Failed to remove worktree: {}", "✘".red(), e);
                    }
                }
            }
        } else {
            eprintln!(
                "{} --prune requires -w/--worktree to specify which branch",
                "⚠".yellow()
            );
        }
    }

    Ok(())
}

// =============================================================================
// Session Launching
// =============================================================================

/// Launch a specific grid layout by name.
///
/// This allows launching a non-default grid from `axel session new --grid <name>`.
/// When `pane_id` and `port` are provided (macOS app mode), the embedded server is started
/// and Claude hooks are configured for the first AI pane in the grid.
pub fn launch_grid_by_name(
    config_path: &Path,
    grid_name: &str,
    session_name: Option<&str>,
    pane_id: Option<&str>,
    server_port: Option<u16>,
    verbose: bool,
) -> Result<()> {
    if !config_path.exists() {
        eprintln!(
            "{}",
            format!("Manifest not found: {}", config_path.display()).red()
        );
        std::process::exit(1);
    }

    // Use provided port or default to 4318
    let port = server_port.unwrap_or(4318);

    // If port is provided (macOS app mode), start embedded server in background thread
    if server_port.is_some() {
        start_embedded_server(port, pane_id, false, None, verbose)?;
    }

    let config = load_config(config_path)?;

    // Validate grid exists
    if !config.layouts.grids.contains_key(grid_name) {
        let available: Vec<&str> = config.layouts.grids.keys().map(|s| s.as_str()).collect();
        eprintln!(
            "{} Grid '{}' not found. Available grids: {}",
            "✘".red(),
            grid_name,
            available.join(", ")
        );
        std::process::exit(1);
    }

    // Configure hooks/OTEL for AI panes if pane_id is provided (macOS app mode)
    if let Some(pane_id) = pane_id {
        let current_dir = std::env::current_dir().ok();
        if let Some(ref install_dir) = current_dir {
            let panes = config.resolve_panes(Some(grid_name));

            // Configure Claude hooks (uses settings file)
            let has_claude = panes
                .iter()
                .any(|p| matches!(p.config, PaneConfig::Claude(_)));
            if has_claude {
                let hooks_settings = generate_hooks_settings(port, pane_id);
                let hooks_path = settings_path(install_dir);
                if write_settings(&hooks_settings, &hooks_path).is_ok() {
                    eprintln!(
                        "{} {} Claude hooks for pane {} (port {})",
                        "✔".green(),
                        "Configured".dimmed(),
                        &pane_id[..8.min(pane_id.len())],
                        port
                    );
                }
            }

            // Note: Codex/OpenCode OTEL is configured via CLI args at tmux pane creation time.
            // The OtelConfig is passed to tmux_create_workspace() which adds the -c flags to Codex commands.
        }
    }

    let grid_type = config.grid_type(Some(grid_name));

    // Use provided session name or derive from workspace
    let session = session_name.map(|s| s.to_string()).unwrap_or_else(|| {
        config_path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| config.workspace.clone())
    });

    if has_session(&session) {
        println!(
            "{}",
            format!("Attaching to existing session: {}", session).blue()
        );
        return match grid_type {
            GridType::TmuxCC => {
                std::process::Command::new("tmux")
                    .args(["-CC", "attach-session", "-t", &session])
                    .status()?;
                Ok(())
            }
            _ => attach_session(&session),
        };
    }

    // Create OTEL config if pane_id is provided (macOS app mode)
    let otel_config = pane_id.map(|id| OtelConfig {
        port,
        pane_id: id.to_string(),
    });

    match grid_type {
        GridType::Shell => launch_shell_mode(&config, Some(grid_name)),
        GridType::TmuxCC => {
            launch_tmux_cc_mode_with_grid(config_path, &config, grid_name, &session, otel_config)
        }
        GridType::Tmux => launch_tmux_mode_with_grid(&config, grid_name, &session, otel_config),
    }
}

/// Launch in tmux control mode (-CC) for iTerm2 integration with a specific grid.
fn launch_tmux_cc_mode_with_grid(
    config_path: &Path,
    config: &axel_core::WorkspaceConfig,
    grid_name: &str,
    session_name: &str,
    otel_config: Option<OtelConfig>,
) -> Result<()> {
    if has_session(session_name) {
        println!(
            "{}",
            format!("Attaching to existing session (CC mode): {}", session_name).blue()
        );
        std::process::Command::new("tmux")
            .args(["-CC", "attach-session", "-t", session_name])
            .status()?;
        return Ok(());
    }

    tmux_create_workspace(session_name, config, Some(grid_name), otel_config)?;

    // Tag session with manifest path
    let manifest_str = config_path.to_string_lossy();
    set_environment(session_name, AXEL_MANIFEST_ENV, &manifest_str).ok();

    println!(
        "{} {} {} (grid: {})",
        "✔".green(),
        "Created tmux session (CC mode)".dimmed(),
        session_name,
        grid_name
    );

    std::process::Command::new("tmux")
        .args(["-CC", "attach-session", "-t", session_name])
        .status()?;

    Ok(())
}

/// Launch in standard tmux mode with a specific grid.
fn launch_tmux_mode_with_grid(
    config: &axel_core::WorkspaceConfig,
    grid_name: &str,
    session_name: &str,
    otel_config: Option<OtelConfig>,
) -> Result<()> {
    if has_session(session_name) {
        println!(
            "{}",
            format!("Attaching to existing session: {}", session_name).blue()
        );
        attach_session(session_name)?;
        return Ok(());
    }

    tmux_create_workspace(session_name, config, Some(grid_name), otel_config)?;

    // Tag session with manifest path
    if let Some(ref manifest_path) = config.manifest_path {
        let manifest_str = manifest_path.to_string_lossy();
        set_environment(session_name, AXEL_MANIFEST_ENV, &manifest_str).ok();
    }

    println!(
        "{} {} {} (grid: {})",
        "✔".green(),
        "Created tmux session".dimmed(),
        session_name,
        grid_name
    );
    attach_session(session_name)?;

    Ok(())
}

/// Launch a workspace from a manifest file.
///
/// This is the main launch path when running `axel` with an `AXEL.md` present.
pub fn launch_from_manifest(config_path: &Path, profile: Option<&str>) -> Result<()> {
    if !config_path.exists() {
        eprintln!(
            "{}",
            format!("Manifest not found: {}", config_path.display()).red()
        );
        std::process::exit(1);
    }

    let session_name = config_path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let config = load_config(config_path)?;
    let grid_type = config.grid_type(profile);

    if !session_name.is_empty() && has_session(&session_name) {
        // Check if this session belongs to a different workspace
        let current_manifest = config_path.to_path_buf();

        if let Some(existing_manifest) = get_environment(&session_name, AXEL_MANIFEST_ENV) {
            let existing_path = PathBuf::from(&existing_manifest);
            if existing_path != current_manifest {
                eprintln!(
                    "{} A session named '{}' already exists for a different workspace:",
                    "✘".red(),
                    session_name
                );
                eprintln!(
                    "  {} {}",
                    "existing:".dimmed(),
                    display_path(&existing_path)
                );
                eprintln!(
                    "  {} {}",
                    "current: ".dimmed(),
                    display_path(&current_manifest)
                );
                eprintln!();
                eprintln!(
                    "{}",
                    "To fix this, update the 'workspace' field in your AXEL.md to use a unique name.".yellow()
                );
                std::process::exit(1);
            }
        }

        println!(
            "{}",
            format!("Attaching to existing session: {}", session_name).blue()
        );
        return match grid_type {
            GridType::TmuxCC => {
                std::process::Command::new("tmux")
                    .args(["-CC", "attach-session", "-t", &session_name])
                    .status()?;
                Ok(())
            }
            _ => attach_session(&session_name),
        };
    }

    match grid_type {
        GridType::Shell => launch_shell_mode(&config, profile),
        GridType::TmuxCC => launch_tmux_cc_mode(config_path, &config, profile),
        GridType::Tmux => launch_tmux_mode(&config, profile),
    }
}

/// Launch in shell mode (no tmux, just run the first shell).
fn launch_shell_mode(config: &axel_core::WorkspaceConfig, profile: Option<&str>) -> Result<()> {
    use std::os::unix::process::CommandExt;

    let panes = config.resolve_panes(profile);
    let index = config.load_index();

    if panes.is_empty() {
        anyhow::bail!("No shells defined in profile");
    }

    let first_pane = &panes[0];

    let work_dir = first_pane
        .path()
        .map(|p| PathBuf::from(expand_path(p)))
        .or_else(|| config.workspace_dir());

    if let Some(ref workspace_dir) = work_dir {
        let (driver_name, skill_names) = match &first_pane.config {
            PaneConfig::Claude(c) => ("claude", &c.skills),
            PaneConfig::Codex(c) => ("codex", &c.skills),
            PaneConfig::Opencode(c) => ("opencode", &c.skills),
            PaneConfig::Antigravity(c) => ("antigravity", &c.skills),
            PaneConfig::Custom(_) => ("", &Vec::new()),
        };

        if !skill_names.is_empty()
            && let Some(driver) = drivers::get_driver(driver_name)
        {
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

        // Install index file (CLAUDE.md, AGENTS.md, etc.) for the driver
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

    let command = build_pane_command(&first_pane.config, index.as_ref(), None);

    if let Some(ref dir) = work_dir {
        std::env::set_current_dir(dir)?;
    }

    if let Some(cmd) = command {
        let err = std::process::Command::new("sh").arg("-c").arg(&cmd).exec();
        Err(err.into())
    } else {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
        let err = std::process::Command::new(&shell).exec();
        Err(err.into())
    }
}

/// Launch a specific pane by name from the manifest.
pub fn launch_pane_by_name(
    manifest_path: &Path,
    pane_name: &str,
    prompt_override: Option<&str>,
    pane_id: Option<&str>,
    server_port: Option<u16>,
    use_tmux: bool,
    session_name: Option<&str>,
    review_post_completion: bool,
    parent_worktree_path: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    // Use provided port or default to 4318
    let port = server_port.unwrap_or(4318);

    // If port is provided (macOS app mode), start embedded server in background thread
    // The server will automatically terminate when this process exits
    if server_port.is_some() {
        start_embedded_server(
            port,
            pane_id,
            review_post_completion,
            parent_worktree_path.clone(),
            verbose,
        )?;
    }

    let config = load_config(manifest_path)?;
    let index = config.load_index();

    let pane_config = config
        .layouts
        .panes
        .iter()
        .find(|s| s.pane_type() == pane_name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Pane '{}' not found in manifest. Available panes: {}",
                pane_name,
                config
                    .layouts
                    .panes
                    .iter()
                    .map(|s| s.pane_type())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;

    let current_dir = std::env::current_dir().ok();

    if let Some(ref install_dir) = current_dir {
        let (driver_name, skill_names) = match pane_config {
            PaneConfig::Claude(c) => ("claude", &c.skills),
            PaneConfig::Codex(c) => ("codex", &c.skills),
            PaneConfig::Opencode(c) => ("opencode", &c.skills),
            PaneConfig::Antigravity(c) => ("antigravity", &c.skills),
            PaneConfig::Custom(_) => ("", &Vec::new()),
        };

        if !skill_names.is_empty()
            && let Some(driver) = drivers::get_driver(driver_name)
        {
            let skill_paths = config.resolve_skills(skill_names);
            if let Some(count) = driver
                .install_skills(install_dir, &skill_paths)
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

        // Install index file (CLAUDE.md, AGENTS.md, etc.) for the driver
        if let Some(driver) = drivers::get_driver(driver_name)
            && let Some(filename) = driver.index_filename()
            && driver.install_index(&config, install_dir).unwrap_or(false)
        {
            eprintln!(
                "{} {} {} symlink",
                "✔".green(),
                "Created".dimmed(),
                filename
            );
        }

        // Configure Claude hooks if pane_id is provided (for macOS app integration)
        if matches!(pane_config, PaneConfig::Claude(_))
            && let Some(pane_id) = pane_id
        {
            let hooks_settings = generate_hooks_settings(port, pane_id);
            let hooks_path = settings_path(install_dir);
            if write_settings(&hooks_settings, &hooks_path).is_ok() {
                eprintln!(
                    "{} {} Claude hooks for pane {} (port {})",
                    "✔".green(),
                    "Configured".dimmed(),
                    &pane_id[..8.min(pane_id.len())],
                    port
                );
            }
        }
    }

    let command = build_pane_command(pane_config, index.as_ref(), prompt_override);

    // Get the driver for this pane type to check OTEL support
    let driver_name = match pane_config {
        PaneConfig::Claude(_) => "claude",
        PaneConfig::Codex(_) => "codex",
        PaneConfig::Opencode(_) => "opencode",
        PaneConfig::Antigravity(_) => "antigravity",
        PaneConfig::Custom(_) => "",
    };

    // If --tmux is specified, create a tmux session instead of running directly
    if use_tmux {
        let base_cmd =
            command.unwrap_or_else(|| std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string()));

        // Generate session name if not provided
        let session = if let Some(name) = session_name {
            name.to_string()
        } else {
            generate_session_name(&config.workspace, pane_name)
        };

        // Build command with OTEL support if driver supports it and server is running
        let cmd = if server_port.is_some() {
            if let Some(driver) = drivers::get_driver(driver_name) {
                if driver.supports_otel() {
                    // Use session name as pane_id for OTEL
                    let otel_vars = driver.otel_env_vars(port, &session);
                    let otel_args = driver.otel_cli_args(port, &session);

                    if !otel_vars.is_empty() {
                        // Use environment variables (Claude, OpenCode)
                        let env_prefix: String = otel_vars
                            .iter()
                            .map(|(k, v)| format!("{}={}", k, v))
                            .collect::<Vec<_>>()
                            .join(" ");
                        eprintln!(
                            "{} {} OTEL telemetry for {}",
                            "✔".green(),
                            "Enabled".dimmed(),
                            driver.name()
                        );
                        format!("{} {}", env_prefix, base_cmd)
                    } else if !otel_args.is_empty() {
                        // Use CLI arguments (Codex)
                        // Insert OTEL args after the command name but before the prompt
                        let args_str = otel_args.join(" ");
                        eprintln!(
                            "{} {} OTEL telemetry for {}",
                            "✔".green(),
                            "Enabled".dimmed(),
                            driver.name()
                        );
                        // Find where to insert args (after "codex" but before prompt)
                        if let Some(space_idx) = base_cmd.find(' ') {
                            let (cmd_name, rest) = base_cmd.split_at(space_idx);
                            format!("{} {}{}", cmd_name, args_str, rest)
                        } else {
                            format!("{} {}", base_cmd, args_str)
                        }
                    } else {
                        base_cmd
                    }
                } else {
                    base_cmd
                }
            } else {
                base_cmd
            }
        } else {
            base_cmd
        };

        let cwd = current_dir
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        // Create tmux session with the command
        NewSession::new()
            .name(&session)
            .detached()
            .start_directory(&cwd)
            .window_name(pane_name)
            .shell_command(&cmd)
            .run()?;

        // Ensure mouse support is enabled for scrollback in this session.
        SetOption::new()
            .target(&session)
            .option("mouse")
            .value("on")
            .run()?;

        // Apply axel status bar styling
        configure_status_bar(&session)?;

        // Tag session with manifest path so it shows up in `axel session ls`
        let manifest_str = manifest_path.to_string_lossy();
        set_environment(&session, AXEL_MANIFEST_ENV, &manifest_str).ok();

        // Store port and pane_id in session environment for recovery
        if server_port.is_some() {
            set_environment(&session, AXEL_PORT_ENV, &port.to_string()).ok();
            if let Some(id) = pane_id {
                set_environment(&session, AXEL_PANE_ID_ENV, id).ok();
            }
        }

        // Store parent worktree path for isolation recovery
        if let Some(ref parent_path) = parent_worktree_path {
            set_environment(
                &session,
                AXEL_ISOLATE_PARENT_ENV,
                &parent_path.to_string_lossy(),
            )
            .ok();
        }

        // Set up bell monitoring for Codex approval detection
        if let Some(driver) = drivers::get_driver(driver_name)
            && let Some(hook_cmd) = driver.tmux_bell_hook_command(port, &session)
        {
            // Enable bell monitoring on the window
            let _ = std::process::Command::new("tmux")
                .args(["set-option", "-t", &session, "monitor-bell", "on"])
                .status();

            // Set up the alert-bell hook
            let _ = std::process::Command::new("tmux")
                .args(["set-hook", "-t", &session, "alert-bell", &hook_cmd])
                .status();

            eprintln!(
                "{} {} bell monitoring for {} approvals",
                "✔".green(),
                "Enabled".dimmed(),
                driver.name()
            );
        }


        // Auto-approve workspace trust for isolated worktrees.
        //
        // Claude Code shows a "Is this a project you trust?" prompt when it starts
        // in a directory it hasn't seen before. Isolated worktrees are always new
        // temp directories, so this prompt always appears. There's no CLI flag to
        // skip it (--trust-workspace doesn't exist), so we spawn a background thread
        // that sends "1" (select "Yes") + Enter via tmux after Claude has started.
        //
        // The 3-second delay accounts for Claude's startup time before the trust
        // prompt renders. The 200ms gap between "1" and Enter ensures the selection
        // registers before confirmation.
        if parent_worktree_path.is_some() {
            let session_for_trust = session.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(3));
                let _ = std::process::Command::new("tmux")
                    .args(["send-keys", "-t", &session_for_trust, "-l", "1"])
                    .status();
                std::thread::sleep(std::time::Duration::from_millis(200));
                let _ = std::process::Command::new("tmux")
                    .args(["send-keys", "-t", &session_for_trust, "Enter"])
                    .status();
            });
            eprintln!(
                "{} {} workspace trust for isolated worktree",
                "✔".green(),
                "Auto-approving".dimmed(),
            );
        }

        eprintln!(
            "{} {} tmux session '{}'",
            "✔".green(),
            "Created".dimmed(),
            session
        );

        // Attach to the session
        attach_session(&session)?;

        // Cleanup after session ends (user detached or shell exited)
        if let Some(ref install_dir) = current_dir {
            let cleaned = cleanup_skills(install_dir);
            if !cleaned.is_empty() {
                eprintln!(
                    "{} {} {} artifacts",
                    "✔".green(),
                    "Cleaned".dimmed(),
                    format_cleaned_drivers(&cleaned)
                );
            }
        }

        return Ok(());
    }

    let status = if let Some(mut cmd) = command {
        let mut process = std::process::Command::new("sh");

        // Enable OTEL telemetry if driver supports it and we have a pane_id
        if let (Some(pane_id), Some(driver)) = (pane_id, drivers::get_driver(driver_name))
            && driver.supports_otel()
        {
            let otel_vars = driver.otel_env_vars(port, pane_id);
            let otel_args = driver.otel_cli_args(port, pane_id);

            if !otel_args.is_empty() {
                // Append CLI args to the command (Codex)
                let args_str = otel_args.join(" ");
                // Insert OTEL args after the command name but before the prompt
                if let Some(space_idx) = cmd.find(' ') {
                    let (cmd_name, rest) = cmd.split_at(space_idx);
                    cmd = format!("{} {}{}", cmd_name, args_str, rest);
                } else {
                    cmd = format!("{} {}", cmd, args_str);
                }
                eprintln!(
                    "{} {} OTEL telemetry for {}",
                    "✔".green(),
                    "Enabled".dimmed(),
                    driver.name()
                );
            }

            // Set environment variables (Claude, OpenCode)
            for (key, value) in &otel_vars {
                process.env(key, value);
            }
            if !otel_vars.is_empty() && otel_args.is_empty() {
                eprintln!(
                    "{} {} OTEL telemetry for {}",
                    "✔".green(),
                    "Enabled".dimmed(),
                    driver.name()
                );
            }
        }

        process.arg("-c").arg(&cmd);
        process.status()
    } else {
        eprintln!("{}", "No command built, falling back to shell".red());
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
        std::process::Command::new(&shell).status()
    };

    if let Some(ref install_dir) = current_dir {
        let cleaned = cleanup_skills(install_dir);
        if !cleaned.is_empty() {
            eprintln!(
                "{} {} {} artifacts",
                "✔".green(),
                "Cleaned".dimmed(),
                format_cleaned_drivers(&cleaned)
            );
        }
    }

    status?;
    Ok(())
}

/// Generate a unique session name for a shell.
///
/// Format: `{workspace}-{shell}-{index}` where index increments to avoid collisions.
fn generate_session_name(workspace: &str, shell_name: &str) -> String {
    let base = format!("{}-{}", workspace, shell_name);

    // Check if base name is available
    if !has_session(&base) {
        return base;
    }

    // Find next available index
    for i in 1..100 {
        let name = format!("{}-{}", base, i);
        if !has_session(&name) {
            return name;
        }
    }

    // Fallback with timestamp if too many sessions
    format!(
        "{}-{}",
        base,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    )
}

/// Launch in tmux control mode (-CC) for iTerm2 integration.
fn launch_tmux_cc_mode(
    config_path: &Path,
    config: &axel_core::WorkspaceConfig,
    profile: Option<&str>,
) -> Result<()> {
    let session_name = config_path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| config.workspace.clone());

    if has_session(&session_name) {
        println!(
            "{}",
            format!("Attaching to existing session (CC mode): {}", session_name).blue()
        );
        std::process::Command::new("tmux")
            .args(["-CC", "attach-session", "-t", &session_name])
            .status()?;
        return Ok(());
    }

    tmux_create_workspace(&session_name, config, profile, None)?;
    println!(
        "{} {} {}",
        "✔".green(),
        "Created tmux session (CC mode)".dimmed(),
        config.workspace
    );

    std::process::Command::new("tmux")
        .args(["-CC", "attach-session", "-t", &session_name])
        .status()?;

    Ok(())
}

/// Launch in standard tmux mode.
fn launch_tmux_mode(config: &axel_core::WorkspaceConfig, profile: Option<&str>) -> Result<()> {
    let session_name = config
        .manifest_path
        .as_ref()
        .and_then(|p| p.parent())
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| config.workspace.clone());

    if has_session(&session_name) {
        println!(
            "{}",
            format!("Attaching to existing session: {}", session_name).blue()
        );
        attach_session(&session_name)?;
        return Ok(());
    }

    tmux_create_workspace(&session_name, config, profile, None)?;
    println!(
        "{} {} {}",
        "✔".green(),
        "Created tmux session".dimmed(),
        config.workspace
    );
    attach_session(&session_name)?;

    Ok(())
}

// =============================================================================
// Helpers
// =============================================================================

/// Build the command string for a given pane config.
///
/// If `prompt_override` is provided, it takes precedence over the prompt
/// defined in the pane config or the workspace index.
fn build_pane_command(
    pane_config: &PaneConfig,
    index: Option<&axel_core::WorkspaceIndex>,
    prompt_override: Option<&str>,
) -> Option<String> {
    match pane_config {
        PaneConfig::Claude(c) => {
            let mut cmd = ClaudeCommand::new();
            if let Some(model) = &c.model {
                cmd = cmd.model(model);
            }
            if !c.allowed_tools.is_empty() {
                cmd = cmd.allowed_tools(c.allowed_tools.clone());
            }
            if !c.disallowed_tools.is_empty() {
                cmd = cmd.disallowed_tools(c.disallowed_tools.clone());
            }
            if let Some(prompt) = prompt_override.or(c.prompt.as_deref()) {
                cmd = cmd.prompt(prompt);
            }
            for arg in &c.args {
                cmd = cmd.extra_arg(arg);
            }
            Some(cmd.build())
        }
        PaneConfig::Codex(c) => {
            let mut parts = vec!["codex".to_string()];
            if let Some(model) = &c.model {
                parts.push("-m".to_string());
                parts.push(model.clone());
            }
            for arg in &c.args {
                parts.push(arg.clone());
            }
            if let Some(prompt) = prompt_override.or(c.prompt.as_deref()) {
                let escaped = prompt.replace('\'', "'\\''");
                parts.push(format!("'{}'", escaped));
            } else if let Some(idx) = index {
                let escaped = idx.to_initial_prompt().replace('\'', "'\\''");
                parts.push(format!("'{}'", escaped));
            }
            Some(parts.join(" "))
        }
        PaneConfig::Opencode(c) => {
            let mut parts = vec!["opencode".to_string()];
            if let Some(model) = &c.model {
                parts.push("-m".to_string());
                parts.push(model.clone());
            }
            for arg in &c.args {
                parts.push(arg.clone());
            }
            if let Some(prompt) = prompt_override.or(c.prompt.as_deref()) {
                let escaped = prompt.replace('\'', "'\\''");
                parts.push(format!("'{}'", escaped));
            } else if let Some(idx) = index {
                let escaped = idx.to_initial_prompt().replace('\'', "'\\''");
                parts.push(format!("'{}'", escaped));
            }
            Some(parts.join(" "))
        }
        PaneConfig::Antigravity(c) => {
            let mut parts = vec!["antigravity".to_string()];
            if let Some(model) = &c.model {
                parts.push("-m".to_string());
                parts.push(model.clone());
            }
            for arg in &c.args {
                parts.push(arg.clone());
            }
            if let Some(prompt) = prompt_override.or(c.prompt.as_deref()) {
                let escaped = prompt.replace('\'', "'\\''");
                parts.push(format!("'{}'", escaped));
            } else if let Some(idx) = index {
                let escaped = idx.to_initial_prompt().replace('\'', "'\\''");
                parts.push(format!("'{}'", escaped));
            }
            Some(parts.join(" "))
        }
        PaneConfig::Custom(c) => c.command.clone(),
    }
}

/// Start the event server in a background thread.
/// The server will automatically terminate when this process exits.
fn start_embedded_server(
    port: u16,
    pane_id: Option<&str>,
    review_post_completion: bool,
    parent_worktree_path: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    use axel_core::server::{ServerConfig, run_server};

    // Create log path in current directory
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let log_path = cwd.join(".axel").join("events.jsonl");

    // Ensure log directory exists
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let config = ServerConfig {
        port,
        // Use pane_id as the session name - this enables tmux send-keys for outbox responses
        session: pane_id.map(|s| s.to_string()).unwrap_or_default(),
        log_path,
        pane_id: pane_id.map(|s| s.to_string()),
        review_post_completion,
        working_dir: if review_post_completion {
            Some(cwd)
        } else {
            None
        },
        parent_worktree_path,
        verbose,
    };

    let pane_display = pane_id
        .map(|id| format!(" for pane {}", &id[..8.min(id.len())]))
        .unwrap_or_default();

    eprintln!(
        "{} {} event server on port {}{}",
        "✔".green(),
        "Starting".dimmed(),
        port,
        pane_display
    );

    // Spawn the server in a background thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            if let Err(e) = run_server(config).await {
                eprintln!("Server error: {}", e);
            }
        });
    });

    // Give the server a moment to start
    std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(())
}

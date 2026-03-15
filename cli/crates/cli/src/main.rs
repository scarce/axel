//! Axel CLI - AI-assisted development workspace manager.
//!
//! Axel provides portable skills across LLMs (Claude Code, Codex, OpenCode) and
//! reproducible terminal workspaces via tmux. Write skills once and use them with
//! any supported AI assistant.
//!
//! # Architecture
//!
//! The CLI is organized into modules:
//! - **cli**: Command-line argument definitions (clap)
//! - **skill**: Skill management commands (list, new, import, fork, link, rm)
//! - **session**: Session management commands (list, new, join, kill, launch)
//!
//! # Workflow
//!
//! 1. User runs `axel` in a project directory
//! 2. CLI finds `AXEL.md` by walking up the directory tree
//! 3. Profile type determines launch mode (tmux, iTerm2, or single shell)
//! 4. Skills are installed via drivers (symlinks for Claude/OpenCode, merged for Codex)
//! 5. Tmux session is created with configured panes, or shell is exec'd directly
//!
//! Core functionality (config parsing, drivers, tmux commands) is in `axel-core`.

mod cli;
mod commands;

use std::path::{Path, PathBuf};

use anyhow::Result;
use axel_core::{
    config::{generate_config, workspaces_dir},
    git,
    tmux::{attach_session, current_session, has_session},
};
use clap::{CommandFactory, Parser};
use cli::{Cli, Commands, LayoutCommands, SessionCommands, SkillCommands};
use colored::Colorize;
use commands::{
    session::{
        do_kill_all_sessions, do_kill_workspace, do_list_sessions, launch_from_manifest,
        launch_grid_by_name, launch_pane_by_name,
    },
    skill::{fork_skill, import_skill, link_skill, list_skills, new_skill, rm_skill},
};

// =============================================================================
// Main Entry Point
// =============================================================================

/// Entry point for the axel CLI.
///
/// Parses command-line arguments and dispatches to the appropriate handler:
///
/// - **Subcommands** (`init`, `bootstrap`, `skill`): Handled first
/// - **Flags** (`-k`): Kill workspace
/// - **Shell name**: Launch specific shell from manifest (e.g., `axel claude`)
/// - **No args**: Launch full workspace from `AXEL.md` or show help
///
/// The manifest path is resolved by walking up the directory tree from the
/// current directory until `AXEL.md` is found, or uses the path specified
/// with `-m/--manifest-path`.
fn main() -> Result<()> {
    let cli = Cli::parse();
    let workspaces_dir = workspaces_dir();

    // Handle git worktree if specified
    let _worktree_info = if let Some(ref branch) = cli.worktree {
        let cwd = std::env::current_dir()?;
        if !git::is_git_repo(&cwd) {
            eprintln!("{} Not a git repository", "✘".red());
            std::process::exit(1);
        }

        match git::ensure_worktree(&cwd, branch) {
            Ok(info) => {
                if info.created {
                    if info.branch_created {
                        eprintln!(
                            "{} {} {} (from {})",
                            "✔".green(),
                            "Created branch".dimmed(),
                            info.branch.blue(),
                            git::default_branch(&cwd).unwrap_or_else(|_| "HEAD".to_string())
                        );
                    }
                    eprintln!(
                        "{} {} {}",
                        "✔".green(),
                        "Created worktree at".dimmed(),
                        display_path(&info.path)
                    );
                } else {
                    eprintln!(
                        "{} {} {}",
                        "✔".green(),
                        "Using existing worktree at".dimmed(),
                        display_path(&info.path)
                    );
                }
                // Change to worktree directory
                std::env::set_current_dir(&info.path)?;
                Some(info)
            }
            Err(e) => {
                eprintln!("{} Failed to create worktree: {}", "✘".red(), e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    let manifest_path = resolve_manifest_path(cli.manifest_path.as_deref());
    let base_dir = manifest_base_dir(&manifest_path);

    // Handle subcommands first
    if let Some(command) = cli.command {
        return match command {
            Commands::Init { workspace } => init_workspace(workspace),
            Commands::Bootstrap => bootstrap_skills(),
            Commands::Skill { action } => match action {
                SkillCommands::List => list_skills(&manifest_path, &base_dir),
                SkillCommands::New { name } => new_skill(name.as_deref(), &base_dir),
                SkillCommands::Import { path } => import_skill(&path),
                SkillCommands::Fork { name } => fork_skill(&name, &manifest_path, &base_dir),
                SkillCommands::Link { name } => link_skill(&name, &manifest_path, &base_dir),
                SkillCommands::Rm { name } => rm_skill(&name, &manifest_path, &base_dir),
            },
            Commands::Session { action } => match action {
                SessionCommands::List { all, json } => do_list_sessions(!all, json),
                SessionCommands::New {
                    pane,
                    grid,
                    session_name,
                    pane_id,
                    port,
                    prompt,
                    worktree,
                    auto_worktree,
                    allow_dirty,
                    review_post_completion,
                    tmux,
                    verbose,
                } => {
                    // Track parent worktree path for isolation
                    let mut parent_worktree_path: Option<PathBuf> = None;

                    // Handle git worktree if specified at subcommand level
                    if let Some(ref branch) = worktree {
                        let cwd = std::env::current_dir()?;
                        if !git::is_git_repo(&cwd) {
                            eprintln!("{} Not a git repository", "✘".red());
                            std::process::exit(1);
                        }

                        match git::ensure_worktree(&cwd, branch) {
                            Ok(info) => {
                                if info.created {
                                    if info.branch_created {
                                        eprintln!(
                                            "{} {} {} (from {})",
                                            "✔".green(),
                                            "Created branch".dimmed(),
                                            info.branch.blue(),
                                            git::default_branch(&cwd)
                                                .unwrap_or_else(|_| "HEAD".to_string())
                                        );
                                    }
                                    eprintln!(
                                        "{} {} {}",
                                        "✔".green(),
                                        "Created worktree at".dimmed(),
                                        display_path(&info.path)
                                    );
                                } else {
                                    eprintln!(
                                        "{} {} {}",
                                        "✔".green(),
                                        "Using existing worktree at".dimmed(),
                                        display_path(&info.path)
                                    );
                                }
                                // Change to worktree directory
                                std::env::set_current_dir(&info.path)?;
                            }
                            Err(e) => {
                                eprintln!("{} Failed to create worktree: {}", "✘".red(), e);
                                std::process::exit(1);
                            }
                        }
                    }

                    // Handle --auto-worktree: create a temporary worktree
                    if auto_worktree {
                        let cwd = std::env::current_dir()?;
                        if !git::is_git_repo(&cwd) {
                            eprintln!(
                                "{} Not a git repository (--auto-worktree requires git)",
                                "✘".red()
                            );
                            std::process::exit(1);
                        }

                        let source_branch = git::current_branch(&cwd)?;
                        let isolate_pane_id = pane_id.as_deref().unwrap_or("default");

                        match git::create_isolated_worktree(
                            &cwd,
                            &source_branch,
                            isolate_pane_id,
                            allow_dirty,
                        ) {
                            Ok(info) => {
                                eprintln!(
                                    "{} {} {} (branch: {})",
                                    "✔".green(),
                                    "Created isolated worktree at".dimmed(),
                                    display_path(&info.path),
                                    info.branch.blue()
                                );
                                parent_worktree_path = Some(cwd.clone());
                                std::env::set_current_dir(&info.path)?;
                            }
                            Err(e) => {
                                eprintln!(
                                    "{} Failed to create isolated worktree: {}",
                                    "✘".red(),
                                    e
                                );
                                std::process::exit(1);
                            }
                        }
                    }

                    // Re-resolve manifest path after potential worktree change
                    let manifest_path = resolve_manifest_path(cli.manifest_path.as_deref());

                    if let Some(name) = pane {
                        // Launch a specific pane
                        launch_pane_by_name(
                            &manifest_path,
                            &name,
                            prompt.as_deref(),
                            pane_id.as_deref(),
                            port,
                            tmux,
                            session_name.as_deref(),
                            review_post_completion,
                            parent_worktree_path,
                            verbose,
                        )
                    } else if let Some(grid_name) = grid {
                        // Launch a specific grid layout
                        launch_grid_by_name(
                            &manifest_path,
                            &grid_name,
                            session_name.as_deref(),
                            pane_id.as_deref(),
                            port,
                            verbose,
                        )
                    } else {
                        // Launch the default grid (full workspace)
                        launch_from_manifest(&manifest_path, cli.profile.as_deref())
                    }
                }
                SessionCommands::Join { name } => {
                    if !has_session(&name) {
                        eprintln!("{} Session '{}' not found", "✘".red(), name);
                        eprintln!();
                        do_list_sessions(false, false)?;
                        std::process::exit(1);
                    }
                    attach_session(&name)
                }
                SessionCommands::Kill {
                    name,
                    all,
                    keep_skills,
                    confirm,
                } => {
                    if all {
                        do_kill_all_sessions(&workspaces_dir, keep_skills, confirm)
                    } else {
                        let session_name = match name {
                            Some(n) => n,
                            None => current_session().ok_or_else(|| {
                                anyhow::anyhow!(
                                    "Not inside a tmux session. Specify a session name or use --all: axel session kill <name>"
                                )
                            })?,
                        };
                        do_kill_workspace(
                            &workspaces_dir,
                            &session_name,
                            keep_skills,
                            false,
                            None,
                            confirm,
                        )
                    }
                }
            },
            Commands::Server { port, session, log } => {
                // Run the server in async context
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(async {
                    commands::server::run(commands::server::ServerArgs {
                        port,
                        session,
                        log,
                        review_post_completion: false,
                        verbose: false,
                    })
                    .await
                })
            }
            Commands::Layout { action } => match action {
                LayoutCommands::List { json } => {
                    commands::layout::list_panes(cli.manifest_path.as_deref(), json)
                }
            },
        };
    }

    // Handle --pane-id without a shell name: send prompt to an existing tmux pane
    if cli.name.is_none()
        && let Some(ref pane_id) = cli.pane_id
    {
        let prompt = cli.prompt.as_deref().unwrap_or("");
        if prompt.is_empty() {
            eprintln!("{} --pane-id requires --prompt", "✘".red());
            std::process::exit(1);
        }
        axel_core::tmux::send_keys(pane_id, prompt)?;
        return Ok(());
    }

    if let Some(name) = cli.kill {
        let session_name = if name.is_empty() {
            // No workspace specified, try to detect current tmux session
            current_session().ok_or_else(|| {
                anyhow::anyhow!(
                    "Not inside a tmux session. Specify a workspace name: axel -k <workspace>"
                )
            })?
        } else {
            name
        };
        do_kill_workspace(
            &workspaces_dir,
            &session_name,
            cli.keep_skills,
            cli.prune_worktree,
            cli.worktree.as_deref(),
            cli.confirm,
        )?;
    } else if let Some(ref name) = cli.name {
        if name == "setup" {
            setup_axel()?;
        } else if manifest_path.exists() {
            // Handle --auto-worktree at the top level
            let mut parent_wt: Option<PathBuf> = None;
            if cli.auto_worktree {
                let cwd = std::env::current_dir()?;
                if git::is_git_repo(&cwd) {
                    let source_branch = git::current_branch(&cwd)?;
                    let isolate_pane_id = cli.pane_id.as_deref().unwrap_or("default");
                    match git::create_isolated_worktree(
                        &cwd,
                        &source_branch,
                        isolate_pane_id,
                        cli.allow_dirty,
                    ) {
                        Ok(info) => {
                            eprintln!(
                                "{} {} {} (branch: {})",
                                "✔".green(),
                                "Created isolated worktree at".dimmed(),
                                display_path(&info.path),
                                info.branch.blue()
                            );
                            parent_wt = Some(cwd);
                            std::env::set_current_dir(&info.path)?;
                        }
                        Err(e) => {
                            eprintln!("{} Failed to create isolated worktree: {}", "✘".red(), e);
                            std::process::exit(1);
                        }
                    }
                }
            }

            launch_pane_by_name(
                &manifest_path,
                name,
                cli.prompt.as_deref(),
                cli.pane_id.as_deref(),
                cli.server_port,
                cli.tmux,
                cli.session_name.as_deref(),
                cli.review_post_completion,
                parent_wt,
                cli.verbose,
            )?;
        } else {
            eprintln!(
                "{} No AXEL.md found. Run '{}' to create one.",
                "✘".red(),
                "axel init".blue()
            );
            std::process::exit(1);
        }
    } else if cli.manifest_path.is_some() || manifest_path.exists() {
        launch_from_manifest(&manifest_path, cli.profile.as_deref())?;
    } else {
        Cli::command().print_help()?;
    }

    Ok(())
}

// =============================================================================
// Path Resolution
// =============================================================================

/// Make a path absolute without resolving symlinks
fn make_absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

/// Resolve manifest path from CLI option or default to ./AXEL.md
fn resolve_manifest_path(cli_path: Option<&str>) -> PathBuf {
    if let Some(p) = cli_path {
        let path = PathBuf::from(p);
        return make_absolute(&path);
    }

    // Walk up directory tree looking for AXEL.md
    let mut current = std::env::current_dir().unwrap_or_default();
    loop {
        let md_candidate = current.join("AXEL.md");
        if md_candidate.exists() {
            return md_candidate;
        }

        match current.parent() {
            Some(parent) if parent != current => {
                current = parent.to_path_buf();
            }
            _ => break,
        }
    }

    std::env::current_dir().unwrap_or_default().join("AXEL.md")
}

/// Get the base directory (parent of manifest) for resolving relative paths
fn manifest_base_dir(manifest_path: &Path) -> PathBuf {
    manifest_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))
}

/// Convert absolute path to display path (replace home with ~)
pub fn display_path(path: &Path) -> String {
    dirs::home_dir()
        .and_then(|home| {
            path.strip_prefix(&home)
                .ok()
                .map(|rel| Path::new("~").join(rel).display().to_string())
        })
        .unwrap_or_else(|| path.display().to_string())
}

// =============================================================================
// Workspace Commands
// =============================================================================

/// Initialize an axel workspace in the current directory
fn init_workspace(workspace_name: Option<String>) -> Result<()> {
    use dialoguer::{Input, theme::ColorfulTheme};

    let current_dir = std::env::current_dir()?;
    let config_path = current_dir.join("AXEL.md");

    if config_path.exists() {
        eprintln!("{}", "AXEL.md already exists in this directory".red());
        std::process::exit(1);
    }

    // Default name from directory
    let default_name = current_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "workspace".to_string());

    // Use provided name or prompt interactively
    let name = if let Some(name) = workspace_name {
        name
    } else {
        let theme = ColorfulTheme::default();
        Input::with_theme(&theme)
            .with_prompt("Workspace name")
            .default(default_name)
            .interact_text()?
    };

    // Create AXEL.md (includes project context after frontmatter)
    let config_content = generate_config(&name, &current_dir.to_string_lossy());
    std::fs::write(&config_path, config_content)?;
    println!("{} {} AXEL.md", "✔".green(), "Created".dimmed());

    println!();
    println!("Launch with: {}", "axel".blue());

    Ok(())
}

/// Scan for existing skills and consolidate them using AI.
///
/// This experimental command discovers skill files across the filesystem by:
/// 1. Prompting for a directory to scan
/// 2. Walking the directory tree (ignoring .gitignore) looking for known skill patterns
/// 3. Copying found files to a staging directory (`~/.config/axel/skills/.bootstrap-staging/`)
/// 4. Launching an AI assistant to consolidate and organize the skills
///
/// The AI is given instructions to merge duplicates, create proper directory structures
/// (`<name>/SKILL.md`), and clean up the staging directory when done.
///
/// For more controlled imports, prefer `axel skill import`.
fn bootstrap_skills() -> Result<()> {
    use std::os::unix::process::CommandExt;

    use axel_core::all_skill_patterns;
    use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
    use ignore::WalkBuilder;

    let theme = ColorfulTheme::default();
    let current_dir = std::env::current_dir()?;

    // Prompt for directory to scan
    let scan_dir: String = Input::with_theme(&theme)
        .with_prompt("Directory to scan for skills")
        .default(current_dir.to_string_lossy().to_string())
        .interact_text()?;

    // Expand ~ to home directory
    let expanded_dir = if let Some(rest) = scan_dir.strip_prefix("~/") {
        home_dir()?.join(rest).to_string_lossy().to_string()
    } else {
        scan_dir.clone()
    };

    let scan_path = PathBuf::from(&expanded_dir);
    if !scan_path.exists() {
        eprintln!("{}", format!("Directory not found: {}", expanded_dir).red());
        std::process::exit(1);
    }

    println!();
    println!(
        "{} Scanning {} for skill files...",
        "...".dimmed(),
        scan_dir
    );
    println!();

    // Get skill file patterns from all drivers
    let skill_patterns = all_skill_patterns();

    // Use ignore crate (ripgrep's directory walker) for fast traversal
    // Don't respect .gitignore since skill files are often gitignored
    let mut found_skills: Vec<PathBuf> = Vec::new();

    let walker = WalkBuilder::new(&scan_path)
        .hidden(false) // Include hidden directories like .claude
        .git_ignore(false) // Don't respect .gitignore - skill files are often ignored
        .git_global(false)
        .git_exclude(false)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();

        // Skip if not a file or if it's a symlink
        let metadata = match path.symlink_metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            continue;
        }

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let path_str = path.to_string_lossy();

        // Check if this matches any of our patterns
        let is_skill = skill_patterns.iter().any(|pattern| {
            if pattern.contains('*') {
                // Simple glob matching
                let parts: Vec<&str> = pattern.split('*').collect();
                if parts.len() == 2 {
                    let prefix = parts[0];
                    let suffix = parts[1];
                    path_str.contains(prefix.trim_start_matches('.')) && path_str.ends_with(suffix)
                } else {
                    false
                }
            } else {
                file_name == *pattern || path_str.ends_with(pattern)
            }
        });

        if is_skill {
            found_skills.push(path.to_path_buf());
        }
    }

    if found_skills.is_empty() {
        println!("{}", "No skill files found.".yellow());
        return Ok(());
    }

    // Display found skills
    println!("{} {} skill files:", "✔".green(), found_skills.len());
    println!();
    for skill in &found_skills {
        let rel_path = skill
            .strip_prefix(&scan_path)
            .unwrap_or(skill)
            .display()
            .to_string();
        println!("  {} {}", "-".dimmed(), rel_path);
    }
    println!();

    // Confirm consolidation
    let proceed = Confirm::with_theme(&theme)
        .with_prompt("Consolidate these skills to ~/.config/axel/skills?")
        .default(true)
        .interact()?;

    if !proceed {
        println!("{}", "Cancelled".dimmed());
        return Ok(());
    }

    // Copy skills to global directory
    let global_skills_dir = home_dir()?.join(".config/axel/skills");
    std::fs::create_dir_all(&global_skills_dir)?;

    let staging_dir = global_skills_dir.join(".bootstrap-staging");
    std::fs::create_dir_all(&staging_dir)?;

    for (i, skill_path) in found_skills.iter().enumerate() {
        let dest_name = format!(
            "{:03}-{}.md",
            i,
            skill_path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("skill")
        );
        let dest_path = staging_dir.join(&dest_name);
        std::fs::copy(skill_path, &dest_path)?;
    }

    println!();
    println!(
        "{} {} files to ~/.config/axel/skills/.bootstrap-staging/",
        "✔".green(),
        found_skills.len()
    );

    // Select AI for consolidation
    let ai_options = ["Claude Code", "Codex", "OpenCode"];
    let ai_selection = Select::with_theme(&theme)
        .with_prompt("Which AI should consolidate these skills?")
        .items(&ai_options)
        .default(0)
        .interact()?;

    let ai_command = match ai_selection {
        0 => "claude",
        1 => "codex",
        2 => "opencode",
        _ => unreachable!(),
    };

    println!();
    println!(
        "{} Starting {} to consolidate skills...",
        "✔".green(),
        ai_command
    );
    println!();

    // Build the consolidation prompt
    let prompt = format!(
        r#"I have {} skill files in .bootstrap-staging/ that were discovered from various projects.

Please consolidate and organize them into clean skills:

## Instructions

1. Read all files in .bootstrap-staging/
2. Identify unique, valuable skills (merge duplicates, remove redundant ones)
3. For each unique skill, create a directory structure:

   <skill-name>/SKILL.md

   Example: code-reviewer/SKILL.md, rust-developer/SKILL.md

4. Each SKILL.md must have this format:
   ```markdown
   ---
   name: <skill-name>
   description: <one-line description of what this skill does>
   ---

   # <Skill Name>

   <skill instructions here>
   ```

5. After creating all skills, delete the .bootstrap-staging/ directory

## Guidelines

- Use kebab-case for directory names (e.g., code-reviewer, not CodeReviewer)
- Merge similar skills into one comprehensive skill
- Focus on quality over quantity
- Keep the best instructions from duplicates"#,
        found_skills.len()
    );

    // Change to the global skills directory and launch AI
    std::env::set_current_dir(&global_skills_dir)?;

    let err = std::process::Command::new(ai_command).arg(&prompt).exec();

    Err(err.into())
}

// =============================================================================
// Setup Command
// =============================================================================

fn setup_axel() -> Result<()> {
    use dialoguer::{Input, theme::ColorfulTheme};

    let theme = ColorfulTheme::default();
    let home = home_dir()?;

    println!("{}", "Axel Setup".blue().bold());
    println!();

    let global_dir = home.join(".axel");
    let global_skills = global_dir.join("skills");

    if !global_dir.exists() {
        std::fs::create_dir_all(&global_dir)?;
        println!("{} {} ~/.axel/", "✔".green(), "Created".dimmed());
    }

    if !global_skills.exists() {
        std::fs::create_dir_all(&global_skills)?;
        println!("{} {} ~/.axel/skills/", "✔".green(), "Created".dimmed());
    }

    let global_config = global_dir.join("AXEL.md");
    if !global_config.exists() {
        std::fs::write(&global_config, "---\n# Global axel configuration\n---\n")?;
        println!("{} {} ~/.axel/AXEL.md", "✔".green(), "Created".dimmed());
    }

    println!();

    let setup_org: String = Input::with_theme(&theme)
        .with_prompt("Organization name (leave empty to skip)")
        .allow_empty(true)
        .interact_text()?;

    if !setup_org.is_empty() {
        let org_base: String = Input::with_theme(&theme)
            .with_prompt("Organization base path")
            .default(format!("{}/Coding/{}", home.display(), setup_org))
            .interact_text()?;

        let org_path = PathBuf::from(&org_base);
        let org_skills = org_path.join("skills");
        let org_workspaces = org_path.join("workspaces");

        if !org_path.exists() {
            std::fs::create_dir_all(&org_path)?;
            println!("{} {} {}/", "✔".green(), "Created".dimmed(), org_base);
        }

        if !org_skills.exists() {
            std::fs::create_dir_all(&org_skills)?;
            println!(
                "{} {} {}/skills/",
                "✔".green(),
                "Created".dimmed(),
                org_base
            );
        }

        if !org_workspaces.exists() {
            std::fs::create_dir_all(&org_workspaces)?;
            println!(
                "{} {} {}/workspaces/",
                "✔".green(),
                "Created".dimmed(),
                org_base
            );
        }
    }

    println!();
    println!("{}", "Setup complete!".green().bold());

    Ok(())
}

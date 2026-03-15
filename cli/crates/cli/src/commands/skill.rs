//! Skill management commands for axel.
//!
//! This module handles all skill-related operations:
//! - Listing skills (local and global)
//! - Creating new skills
//! - Importing skills from files/directories
//! - Forking global skills to local
//! - Linking global skills to local
//! - Removing skills

use std::path::{Path, PathBuf};

use anyhow::Result;
use axel_core::{config::load_config, drivers};
use colored::Colorize;

use crate::{display_path, home_dir};

// =============================================================================
// Constants
// =============================================================================

const SKILL_FILE: &str = "SKILL.md";
const SKILLS_DIR: &str = "skills";
const AXEL_DIR: &str = "axel";
const CONFIG_DIR: &str = ".config";

// =============================================================================
// Skill Path Helpers
// =============================================================================

fn global_skills_dir() -> Result<PathBuf> {
    Ok(home_dir()?.join(CONFIG_DIR).join(AXEL_DIR).join(SKILLS_DIR))
}

/// Represents a skill's location in the filesystem.
///
/// Skills follow the convention `<base>/<name>/SKILL.md` where:
/// - Local skills: `./skills/<name>/SKILL.md`
/// - Global skills: `~/.config/axel/skills/<name>/SKILL.md`
struct SkillPath {
    /// Directory containing the SKILL.md file
    dir: PathBuf,
    /// Whether this is a global skill (affects display formatting)
    is_global: bool,
}

impl SkillPath {
    fn local(name: &str, base_dir: &Path) -> Self {
        Self {
            dir: base_dir.join(SKILLS_DIR).join(name),
            is_global: false,
        }
    }

    fn global(name: &str) -> Result<Self> {
        Ok(Self {
            dir: global_skills_dir()?.join(name),
            is_global: true,
        })
    }

    fn exists(&self) -> bool {
        self.dir.exists()
    }

    fn skill_file(&self) -> PathBuf {
        self.dir.join(SKILL_FILE)
    }

    fn display(&self) -> String {
        if self.is_global {
            display_path(&self.dir)
        } else {
            Path::new(SKILLS_DIR)
                .join(self.dir.file_name().unwrap_or_default())
                .display()
                .to_string()
        }
    }

    fn display_with_file(&self) -> String {
        if self.is_global {
            display_path(&self.skill_file())
        } else {
            Path::new(SKILLS_DIR)
                .join(self.dir.file_name().unwrap_or_default())
                .join(SKILL_FILE)
                .display()
                .to_string()
        }
    }
}

/// Get all global skill directories to search
fn global_skill_dirs() -> Vec<PathBuf> {
    global_skills_dir()
        .ok()
        .filter(|p| p.exists())
        .into_iter()
        .collect()
}

/// Metadata for a discovered skill, used for listing.
struct SkillInfo {
    /// Skill name (directory name or file stem)
    name: String,
    /// First non-empty, non-heading line from the skill file (truncated to 60 chars)
    description: String,
    /// Full path to the skill file
    #[allow(dead_code)]
    path: PathBuf,
    /// Location label for display (workspace name or "global")
    location: String,
}

/// Find all skills in a directory.
///
/// Discovers skills in two formats:
/// - Directory format: `<name>/SKILL.md`
/// - File format: `<name>.md` (excluding `index.md`)
fn find_skills_in_dir(dir: &Path, location: &str) -> Vec<SkillInfo> {
    let mut skills = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return skills,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let (skill_name, skill_path) = if path.is_dir() {
            let skill_file = path.join("SKILL.md");
            if skill_file.exists() {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                (name, skill_file)
            } else {
                continue;
            }
        } else if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            if path.file_name().is_some_and(|n| n == "index.md") {
                continue;
            }
            let name = path
                .file_stem()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            (name, path)
        } else {
            continue;
        };

        if skill_name.is_empty() {
            continue;
        }

        let description = std::fs::read_to_string(&skill_path)
            .ok()
            .and_then(|content| {
                let content = if content.starts_with("---") {
                    content
                        .find("\n---")
                        .map(|i| &content[i + 4..])
                        .unwrap_or(&content)
                } else {
                    &content
                };

                content
                    .lines()
                    .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
                    .or_else(|| {
                        content
                            .lines()
                            .find(|l| l.starts_with('#'))
                            .map(|l| l.trim_start_matches('#').trim())
                    })
                    .map(|s| {
                        let s = s.trim();
                        if s.len() > 60 {
                            format!("{}...", &s[..57])
                        } else {
                            s.to_string()
                        }
                    })
            })
            .unwrap_or_else(|| "No description".to_string());

        skills.push(SkillInfo {
            name: skill_name,
            description,
            path: skill_path,
            location: location.to_string(),
        });
    }

    skills
}

// =============================================================================
// Public Commands
// =============================================================================

/// Clean up installed skill symlinks for all drivers
pub fn cleanup_skills(workspace_dir: &Path) -> Vec<&'static str> {
    let mut cleaned = Vec::new();

    for driver in drivers::all_drivers() {
        if driver.cleanup(workspace_dir) {
            cleaned.push(driver.name());
        }
    }

    cleaned
}

/// Format cleaned drivers list for display
pub fn format_cleaned_drivers(cleaned: &[&str]) -> String {
    if cleaned.len() == 1 {
        cleaned[0].to_string()
    } else {
        let last = cleaned.last().unwrap();
        let rest = &cleaned[..cleaned.len() - 1];
        format!("{} and {}", rest.join(", "), last)
    }
}

/// List all available skills (local and global)
pub fn list_skills(manifest_path: &Path, base_dir: &Path) -> Result<()> {
    let mut all_skills: Vec<SkillInfo> = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    let global_dir = global_skills_dir().ok();

    let skill_sources: Vec<(PathBuf, String)> = if manifest_path.exists() {
        let cfg = load_config(manifest_path)?;
        cfg.skills_dirs()
            .into_iter()
            .map(|dir| {
                let name = if dir.starts_with(base_dir) {
                    base_dir
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "local".to_string())
                } else if global_dir.as_ref().is_some_and(|g| &dir == g) {
                    "global".to_string()
                } else {
                    display_path(&dir)
                };
                (dir, name)
            })
            .collect()
    } else {
        let mut sources = Vec::new();
        let local_dir = base_dir.join(SKILLS_DIR);
        if local_dir.exists() {
            let name = base_dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "local".to_string());
            sources.push((local_dir, name));
        }
        for dir in global_skill_dirs() {
            sources.push((dir, "global".to_string()));
        }
        sources
    };

    for (dir, location) in &skill_sources {
        for skill in find_skills_in_dir(dir, location) {
            if !seen_names.contains(&skill.name) {
                seen_names.insert(skill.name.clone());
                all_skills.push(skill);
            }
        }
    }

    if all_skills.is_empty() {
        println!("{}", "No skills found".dimmed());
        return Ok(());
    }

    use comfy_table::{Table, presets::NOTHING};

    let workspace_name = base_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut table = Table::new();
    table.load_preset(NOTHING);

    for skill in &all_skills {
        let location = if skill.location == workspace_name {
            skill.location.yellow().to_string()
        } else {
            skill.location.purple().to_string()
        };

        table.add_row(vec![
            skill.name.green().to_string(),
            location,
            skill.description.dimmed().to_string(),
        ]);
    }

    println!("{table}");

    Ok(())
}

/// Create a new skill interactively
pub fn new_skill(name: Option<&str>, base_dir: &Path) -> Result<()> {
    use dialoguer::{Input, Select, theme::ColorfulTheme};

    let theme = ColorfulTheme::default();

    let skill_name: String = match name {
        Some(n) => n.to_string(),
        None => Input::with_theme(&theme)
            .with_prompt("Skill name")
            .interact_text()?,
    };

    let local = SkillPath::local(&skill_name, base_dir);
    let global = SkillPath::global(&skill_name)?;

    let options = [
        format!("Local ({})", local.display()),
        format!("Global ({})", global.display()),
    ];
    let selection = Select::with_theme(&theme)
        .with_prompt("Where should this skill be created?")
        .items(&options)
        .default(0)
        .interact()?;

    let skill = match selection {
        0 => local,
        1 => global,
        _ => unreachable!(),
    };

    if skill.exists() {
        let collision_options = ["Replace", "Cancel"];
        let collision_selection = Select::with_theme(&theme)
            .with_prompt(format!("Skill '{}' already exists", skill_name))
            .items(&collision_options)
            .default(1)
            .interact()?;

        match collision_selection {
            0 => {
                std::fs::remove_dir_all(&skill.dir)?;
            }
            1 => {
                println!("{}", "Cancelled".dimmed());
                return Ok(());
            }
            _ => unreachable!(),
        }
    }

    std::fs::create_dir_all(&skill.dir)?;

    let content = format!(
        r#"---
name: {name}
description: Describe what this skill does
---

# {name}

You are a {name} skill.

## Guidelines

- Add your guidelines here
"#,
        name = skill_name
    );
    let skill_file = skill.skill_file();

    std::fs::write(&skill_file, content)?;

    println!(
        "{} {} {}",
        "✔".green(),
        "Created".dimmed(),
        skill.display_with_file()
    );

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "code".to_string());
    std::process::Command::new(editor)
        .arg(&skill_file)
        .status()?;

    Ok(())
}

/// Import skill file(s) to the global skills directory
pub fn import_skill(path: &str) -> Result<()> {
    // Expand ~ to home directory
    let expanded_path = if let Some(rest) = path.strip_prefix("~/") {
        home_dir()?.join(rest)
    } else {
        PathBuf::from(path)
    };

    if !expanded_path.exists() {
        eprintln!("{} Path not found: {}", "✘".red(), path);
        std::process::exit(1);
    }

    // Skip symlinks
    let metadata = expanded_path.symlink_metadata()?;
    if metadata.file_type().is_symlink() {
        eprintln!("{} Cannot import symlinks", "✘".red());
        std::process::exit(1);
    }

    // If it's a directory, import all .md files in it
    if expanded_path.is_dir() {
        let mut count = 0;
        for entry in std::fs::read_dir(&expanded_path)?.flatten() {
            let entry_path = entry.path();

            // Skip symlinks
            if entry_path
                .symlink_metadata()
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(true)
            {
                continue;
            }

            // Import .md files
            if entry_path.is_file() && entry_path.extension().map(|e| e == "md").unwrap_or(false) {
                import_single_skill(&entry_path)?;
                count += 1;
            }
        }

        if count == 0 {
            eprintln!("{} No .md files found in directory", "✘".red());
            std::process::exit(1);
        }

        return Ok(());
    }

    // Single file import
    import_single_skill(&expanded_path)
}

fn import_single_skill(source_path: &Path) -> Result<()> {
    // Derive skill name from path
    let skill_name = if source_path
        .file_name()
        .map(|n| n == "SKILL.md")
        .unwrap_or(false)
    {
        // Use parent directory name for SKILL.md files
        source_path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "skill".to_string())
    } else {
        // Use filename without extension
        source_path
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "skill".to_string())
    };

    // Skip index.md
    if skill_name == "index" {
        return Ok(());
    }

    // Create target directory in global skills
    let global_skills_dir = home_dir()?.join(".config/axel/skills");
    let target_dir = global_skills_dir.join(&skill_name);
    let target_file = target_dir.join("SKILL.md");

    if target_dir.exists() {
        // Silently skip existing skills when importing from directory
        println!(
            "{} {} {}/SKILL.md (already exists)",
            "-".dimmed(),
            "Skipped".dimmed(),
            skill_name
        );
        return Ok(());
    }

    std::fs::create_dir_all(&target_dir)?;
    std::fs::copy(source_path, &target_file)?;

    println!(
        "{} {} {}/SKILL.md",
        "✔".green(),
        "Imported".dimmed(),
        skill_name
    );

    Ok(())
}

/// Fork (copy) a global skill to the current workspace
pub fn fork_skill(name: &str, manifest_path: &Path, base_dir: &Path) -> Result<()> {
    let global = SkillPath::global(name)?;
    let local = SkillPath::local(name, base_dir);

    if !global.exists() {
        eprintln!("{}", format!("Global skill '{}' not found", name).red());
        eprintln!();
        let _ = list_skills(manifest_path, base_dir);
        std::process::exit(1);
    }

    if local.exists() {
        eprintln!(
            "{}",
            format!("Skill '{}' already exists in workspace", name).red()
        );
        std::process::exit(1);
    }

    std::fs::create_dir_all(&local.dir)?;
    std::fs::copy(global.skill_file(), local.skill_file())?;

    println!(
        "{} {} {}",
        "✔".green(),
        "Forked".dimmed(),
        local.display_with_file()
    );

    Ok(())
}

/// Link (symlink) a global skill to the current workspace
pub fn link_skill(name: &str, manifest_path: &Path, base_dir: &Path) -> Result<()> {
    let global = SkillPath::global(name)?;
    let local = SkillPath::local(name, base_dir);

    if !global.exists() {
        eprintln!("{}", format!("Global skill '{}' not found", name).red());
        eprintln!();
        let _ = list_skills(manifest_path, base_dir);
        std::process::exit(1);
    }

    if local.exists() {
        eprintln!(
            "{}",
            format!("Skill '{}' already exists in workspace", name).red()
        );
        std::process::exit(1);
    }

    std::fs::create_dir_all(base_dir.join(SKILLS_DIR))?;

    #[cfg(unix)]
    std::os::unix::fs::symlink(&global.dir, &local.dir)?;

    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&global.dir, &local.dir)?;

    println!(
        "{} {} {} -> {}",
        "✔".green(),
        "Linked".dimmed(),
        local.display(),
        global.display()
    );

    Ok(())
}

/// Remove a skill
pub fn rm_skill(name: &str, manifest_path: &Path, base_dir: &Path) -> Result<()> {
    use dialoguer::{Confirm, Select, theme::ColorfulTheme};

    let theme = ColorfulTheme::default();

    let local = SkillPath::local(name, base_dir);
    let global = SkillPath::global(name)?;

    let skill_to_remove = if local.exists() && global.exists() {
        let options = [
            format!("Local ({})", local.display()),
            format!("Global ({})", global.display()),
        ];
        let selection = Select::with_theme(&theme)
            .with_prompt(format!(
                "Skill '{}' exists in both locations. Which one to remove?",
                name
            ))
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => local,
            1 => global,
            _ => unreachable!(),
        }
    } else if local.exists() {
        local
    } else if global.exists() {
        global
    } else {
        eprintln!("{}", format!("Skill '{}' not found", name).red());
        eprintln!();
        let _ = list_skills(manifest_path, base_dir);
        std::process::exit(1);
    };

    let confirmed = Confirm::with_theme(&theme)
        .with_prompt(format!("Remove {}?", skill_to_remove.display()))
        .default(false)
        .interact()?;

    if !confirmed {
        println!("{}", "Cancelled".dimmed());
        return Ok(());
    }

    std::fs::remove_dir_all(&skill_to_remove.dir)?;
    println!(
        "{} {} {}",
        "✔".green(),
        "Removed".dimmed(),
        skill_to_remove.display()
    );

    Ok(())
}

//! OpenCode skill driver.
//!
//! OpenCode uses a similar symlink strategy to Claude Code. Skills are installed
//! as symlinks in `.opencode/skill/` directory within the workspace.
//!
//! This driver:
//! 1. Creates `.opencode/skill/` if it doesn't exist
//! 2. Symlinks each skill file as `<name>.md`
//! 3. On cleanup, removes only symlinks (preserving any manually created files)

use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{SkillDriver, claude::install_index_symlink};
use crate::config::WorkspaceConfig;

/// OpenCode skill driver
pub struct OpenCodeDriver;

impl SkillDriver for OpenCodeDriver {
    fn name(&self) -> &'static str {
        "opencode"
    }

    fn skills_dir(&self, workspace_dir: &Path) -> PathBuf {
        workspace_dir.join(".opencode").join("skill")
    }

    fn skill_patterns(&self) -> &'static [&'static str] {
        &[".opencode/skill/*.md", ".opencode/SKILL.md"]
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
            let link_path = skills_dir.join(format!("{}.md", name));

            // Remove existing symlink/file if present
            if link_path.exists() || link_path.is_symlink() {
                std::fs::remove_file(&link_path).ok();
            }

            // Canonicalize the source path to get a clean absolute path
            let canonical_source = source_path
                .canonicalize()
                .unwrap_or_else(|_| source_path.clone());

            // Create symlink
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

        // Remove skill symlinks from .opencode/skill/
        let skills_dir = self.skills_dir(workspace_dir);
        if skills_dir.exists()
            && let Ok(entries) = std::fs::read_dir(&skills_dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .symlink_metadata()
                    .map(|m| m.file_type().is_symlink())
                    .unwrap_or(false)
                    && std::fs::remove_file(&path).is_ok()
                {
                    cleaned = true;
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

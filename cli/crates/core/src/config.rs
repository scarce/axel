//! Configuration types and parsing for axel workspaces
//!
//! This module provides the core configuration types for axel workspaces,
//! including workspace configuration, shell definitions, terminal profiles,
//! and skill management.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use colored::Colorize;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

// =============================================================================
// Workspace Configuration
// =============================================================================

/// Main workspace configuration loaded from AXEL.md (YAML frontmatter)
#[derive(Debug, Deserialize)]
pub struct WorkspaceConfig {
    /// Workspace name (used as tmux session name)
    #[serde(alias = "name")]
    pub workspace: String,
    /// Layout configurations (panes + grids)
    pub layouts: LayoutsConfig,
    /// Agent directories configuration
    #[serde(default)]
    pub skills: Vec<SkillPathConfig>,
    /// Path to the manifest file (set during loading, not from YAML)
    #[serde(skip)]
    pub manifest_path: Option<PathBuf>,
}

/// Layout configuration containing pane definitions and grid layouts
#[derive(Debug, Deserialize, Default)]
pub struct LayoutsConfig {
    /// Pane definitions (AI shells, regular shells, custom commands)
    #[serde(default)]
    pub panes: Vec<PaneConfig>,
    /// Grid layouts (named configurations of pane arrangements)
    #[serde(default)]
    pub grids: HashMap<String, Grid>,
}

/// Configuration for an skill search path
#[derive(Debug, Deserialize, Clone)]
pub struct SkillPathConfig {
    /// Path to skills directory (relative to manifest or absolute)
    pub path: String,
}

impl WorkspaceConfig {
    /// Get all resolved skill directories that exist
    pub fn skills_dirs(&self) -> Vec<PathBuf> {
        let manifest_dir = self
            .manifest_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf());

        self.skills
            .iter()
            .filter_map(|skill_config| {
                let path = &skill_config.path;
                let resolved = if path.starts_with('/') || path.starts_with('~') {
                    PathBuf::from(expand_path(path))
                } else if let Some(ref base) = manifest_dir {
                    base.join(path)
                } else {
                    PathBuf::from(path)
                };

                if resolved.exists() {
                    Some(resolved)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find an skill file by name across all skill directories
    ///
    /// Supports both flat files (name.md) and directory structure (name/SKILL.md).
    /// Returns the first match (priority order defined by skills config).
    /// Warns if skill is found in multiple directories.
    pub fn find_skill(&self, name: &str) -> Option<PathBuf> {
        let dirs = self.skills_dirs();
        let mut first_match: Option<PathBuf> = None;
        let mut _first_dir: Option<PathBuf> = None;

        for dir in &dirs {
            // Check for directory structure: skills/<name>/SKILL.md
            let dir_path = dir.join(name).join("SKILL.md");
            if dir_path.exists() {
                if first_match.is_some() {
                    eprintln!(
                        "{} Duplicate skill '{}', ignoring {}",
                        "!".yellow(),
                        name,
                        dir.display()
                    );
                } else {
                    first_match = Some(dir_path);
                    _first_dir = Some(dir.clone());
                }
                continue;
            }

            // Check for flat file: skills/<name>.md (but not index.md)
            if name == "index" {
                continue;
            }
            let flat_path = dir.join(format!("{}.md", name));
            if flat_path.exists() {
                if first_match.is_some() {
                    eprintln!(
                        "{} Duplicate skill '{}', ignoring {}",
                        "!".yellow(),
                        name,
                        dir.display()
                    );
                } else {
                    first_match = Some(flat_path);
                    _first_dir = Some(dir.clone());
                }
            }
        }

        first_match
    }

    /// Find all skill files across all skill directories
    ///
    /// Uses priority order from config - first directory wins for conflicting names.
    /// Returns skills in priority order (preserves insertion order via IndexMap internally).
    pub fn find_all_skills(&self) -> Vec<PathBuf> {
        let mut skills_by_name: IndexMap<String, (PathBuf, PathBuf)> = IndexMap::new();

        for dir in self.skills_dirs() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
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
                        // Skip index.md - it's used as workspace context, not an skill
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

                    if let Some((existing_path, existing_dir)) = skills_by_name.get(&skill_name) {
                        eprintln!(
                            "{} Duplicate skill '{}', ignoring {}",
                            "!".yellow(),
                            skill_name,
                            dir.display()
                        );
                        let _ = (existing_path, existing_dir);
                    } else {
                        skills_by_name.insert(skill_name, (skill_path, dir.clone()));
                    }
                }
            }
        }

        skills_by_name.into_values().map(|(path, _)| path).collect()
    }

    /// Resolve skill paths based on config (supports "*" for all)
    pub fn resolve_skills(&self, skill_names: &[String]) -> Vec<PathBuf> {
        if skill_names.iter().any(|n| n == "*") {
            self.find_all_skills()
        } else {
            skill_names
                .iter()
                .filter_map(|name| self.find_skill(name))
                .collect()
        }
    }

    /// Load and parse skills from paths
    ///
    /// Returns skills in priority order (IndexMap preserves insertion order).
    #[allow(dead_code)]
    pub fn load_skills(&self, skill_names: &[String]) -> IndexMap<String, Skill> {
        let paths = self.resolve_skills(skill_names);
        let mut skills = IndexMap::new();

        for path in paths {
            if let Ok(skill) = Skill::from_file(&path) {
                skills.entry(skill.name.clone()).or_insert(skill);
            }
        }

        skills
    }

    /// Get workspace directory (parent of manifest)
    pub fn workspace_dir(&self) -> Option<PathBuf> {
        self.manifest_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
    }

    /// Load the workspace context from AXEL.md
    ///
    /// Reads the content after the YAML frontmatter from the manifest file.
    /// This content is used as initial context for AI assistants.
    pub fn load_index(&self) -> Option<WorkspaceIndex> {
        self.manifest_path
            .as_ref()
            .and_then(|path| WorkspaceIndex::from_manifest(path, &self.workspace).ok())
    }

    /// Get the grid type for a given grid name (defaults to "default")
    pub fn grid_type(&self, grid_name: Option<&str>) -> GridType {
        let grid_name = grid_name.unwrap_or("default");
        self.layouts
            .grids
            .get(grid_name)
            .map(|g| g.grid_type)
            .unwrap_or_default()
    }

    /// Resolve panes using the specified grid (defaults to "default")
    pub fn resolve_panes(&self, grid_name: Option<&str>) -> Vec<ResolvedPane> {
        let grid_name = grid_name.unwrap_or("default");
        let Some(grid) = self.layouts.grids.get(grid_name) else {
            return vec![];
        };

        // Build lookup map of pane templates by type
        let templates: HashMap<&str, &PaneConfig> = self
            .layouts
            .panes
            .iter()
            .map(|p| (p.pane_type(), p))
            .collect();

        // Default path from manifest directory
        let default_path = self
            .workspace_dir()
            .map(|p| p.to_string_lossy().to_string());

        grid.cells
            .iter()
            .filter_map(|(cell_name, grid_cell)| {
                let pane_type = grid_cell.pane_type.as_deref().unwrap_or(cell_name.as_str());

                let template = templates.get(pane_type)?;

                let mut config = (*template).clone();

                if config.path().is_none()
                    && let Some(ref default) = default_path
                {
                    config.set_path(default.clone());
                }

                if let Some(ref color) = grid_cell.color {
                    config.set_color(color.clone());
                }

                Some(ResolvedPane {
                    name: cell_name.clone(),
                    col: grid_cell.col,
                    row: grid_cell.row,
                    width: grid_cell.width,
                    height: grid_cell.height,
                    config,
                })
            })
            .collect()
    }

    /// Get the profile type for a given profile name (legacy alias for grid_type)
    #[deprecated(note = "Use grid_type instead")]
    pub fn profile_type(&self, profile_name: Option<&str>) -> GridType {
        self.grid_type(profile_name)
    }
}

// =============================================================================
// Skill Types
// =============================================================================

/// Parsed skill ready for AI tool configuration
#[derive(Debug, Clone, Serialize)]
pub struct Skill {
    /// Agent name (derived from filename or frontmatter)
    #[serde(skip)]
    pub name: String,
    /// Description of when to use this skill
    pub description: String,
    /// The system prompt content
    pub prompt: String,
    /// Optional list of allowed tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    /// Optional model to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// YAML frontmatter for skill files
#[derive(Debug, Deserialize, Default)]
struct SkillFrontmatter {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    tools: Option<String>,
    #[serde(default)]
    model: Option<String>,
}

impl Skill {
    /// Parse an skill from a markdown file with optional YAML frontmatter
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;

        // Derive name from path
        let name = if path.file_name().map(|n| n == "SKILL.md").unwrap_or(false) {
            path.parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "skill".to_string())
        } else {
            path.file_stem()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "skill".to_string())
        };

        // Parse YAML frontmatter
        let (frontmatter, prompt) = if let Some(after_start) = content.strip_prefix("---") {
            if let Some(end_idx) = after_start.find("\n---") {
                let fm_content = &after_start[..end_idx];
                let rest = &after_start[end_idx + 4..];
                let fm: SkillFrontmatter = serde_yaml::from_str(fm_content).unwrap_or_default();
                (fm, rest.trim().to_string())
            } else {
                (SkillFrontmatter::default(), content)
            }
        } else {
            (SkillFrontmatter::default(), content)
        };

        let name = frontmatter.name.unwrap_or(name);

        let description = frontmatter.description.unwrap_or_else(|| {
            prompt
                .lines()
                .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .or_else(|| prompt.lines().next())
                .map(|l| l.trim_start_matches('#').trim().to_string())
                .unwrap_or_else(|| format!("{} skill", name))
        });

        let tools = frontmatter.tools.map(|t| {
            t.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        });

        Ok(Skill {
            name,
            description,
            prompt,
            tools,
            model: frontmatter.model,
        })
    }
}

// =============================================================================
// Workspace Index
// =============================================================================

/// Workspace index - project context from AXEL.md
///
/// This is NOT a skill - it's a project description used as initial context.
/// The content is extracted from AXEL.md after the YAML frontmatter.
#[derive(Debug, Clone)]
pub struct WorkspaceIndex {
    /// Project name (from workspace config)
    pub name: String,
    /// Project description from frontmatter
    pub description: Option<String>,
    /// Full markdown content (after frontmatter)
    pub content: String,
}

impl WorkspaceIndex {
    /// Parse a workspace index from the AXEL.md manifest file
    ///
    /// Extracts the content after the YAML frontmatter, which contains
    /// project documentation used as context for AI assistants.
    pub fn from_manifest(path: &Path, workspace_name: &str) -> Result<Self> {
        let raw_content = std::fs::read_to_string(path)?;

        // Extract content after YAML frontmatter
        let content = if let Some(after_start) = raw_content.strip_prefix("---") {
            if let Some(end_idx) = after_start.find("\n---") {
                after_start[end_idx + 4..].trim().to_string()
            } else {
                String::new()
            }
        } else {
            raw_content.trim().to_string()
        };

        // Return None-equivalent if no content after frontmatter
        if content.is_empty() {
            anyhow::bail!("No content after frontmatter in AXEL.md");
        }

        Ok(WorkspaceIndex {
            name: workspace_name.to_string(),
            description: None,
            content,
        })
    }

    /// Build the initial prompt to send to Claude/Codex
    pub fn to_initial_prompt(&self) -> String {
        format!(
            "Context: You're working on a project called {}. Here's the project documentation:\n\n{}\n\n---\nAwaiting your instructions.",
            self.name, self.content
        )
    }
}

// =============================================================================
// Grid Configuration
// =============================================================================

/// Grid type (how the layout is rendered)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GridType {
    /// Standard tmux session with panes
    #[default]
    Tmux,
    /// iTerm2 tmux control mode (-CC)
    TmuxCC,
    /// Direct shell execution (no tmux, first pane only)
    Shell,
}

impl<'de> serde::Deserialize<'de> for GridType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "tmux" => Ok(GridType::Tmux),
            "tmux_cc" => Ok(GridType::TmuxCC),
            "shell" => Ok(GridType::Shell),
            _ => Err(serde::de::Error::custom(format!(
                "unknown grid type: {} (expected tmux, tmux_cc, or shell)",
                s
            ))),
        }
    }
}

/// A grid layout with type and cell definitions
#[derive(Debug, Clone)]
pub struct Grid {
    /// Grid type (tmux, tmux_cc, shell)
    pub grid_type: GridType,
    /// Cell definitions (pane placements)
    pub cells: IndexMap<String, GridCell>,
}

impl<'de> serde::Deserialize<'de> for Grid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map: IndexMap<String, serde_yaml::Value> = IndexMap::deserialize(deserializer)?;

        let grid_type = if let Some(type_value) = map.shift_remove("type") {
            serde_yaml::from_value(type_value).map_err(serde::de::Error::custom)?
        } else {
            GridType::default()
        };

        let cells: IndexMap<String, GridCell> = map
            .into_iter()
            .filter_map(|(k, v)| serde_yaml::from_value(v).ok().map(|cell| (k, cell)))
            .collect();

        Ok(Grid { grid_type, cells })
    }
}

/// Cell entry in a grid (references a pane definition)
#[derive(Debug, Deserialize, Default, Clone)]
pub struct GridCell {
    /// Reference to a pane type defined in layouts.panes
    pub pane_type: Option<String>,
    /// Column position
    #[serde(default)]
    pub col: u32,
    /// Row position
    #[serde(default)]
    pub row: u32,
    /// Width percentage
    #[serde(default)]
    pub width: Option<u32>,
    /// Height percentage
    #[serde(default)]
    pub height: Option<u32>,
    /// Override color from pane definition
    #[serde(default)]
    pub color: Option<String>,
}

// =============================================================================
// Pane Configuration
// =============================================================================

/// Raw pane config for deserialization
#[derive(Debug, Deserialize)]
struct PaneConfigRaw {
    #[serde(rename = "type")]
    pane_type: String,
    /// Unique name for the pane (used to reference in grids)
    /// For AI types (claude, codex, etc.), defaults to the type name
    /// For custom types, this must be provided to create unique identifiers
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    notes: Vec<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    skills: Vec<String>,
    #[serde(default)]
    allowed_tools: Vec<String>,
    #[serde(default)]
    disallowed_tools: Vec<String>,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    command: Option<String>,
}

/// Pane configuration - known AI types or custom shell types
#[derive(Debug, Clone)]
pub enum PaneConfig {
    /// Claude Code shell
    Claude(AiPaneConfig),
    /// Codex shell
    Codex(AiPaneConfig),
    /// OpenCode shell
    Opencode(AiPaneConfig),
    /// Google Antigravity shell
    Antigravity(AiPaneConfig),
    /// Custom shell with arbitrary command
    Custom(CustomPaneConfig),
}

impl<'de> serde::Deserialize<'de> for PaneConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = PaneConfigRaw::deserialize(deserializer)?;

        match raw.pane_type.as_str() {
            "claude" => Ok(PaneConfig::Claude(AiPaneConfig {
                pane_type: raw.pane_type.clone(),
                name: raw.name.or(Some(raw.pane_type)),
                path: raw.path,
                color: raw.color,
                notes: raw.notes,
                model: raw.model,
                skills: raw.skills,
                allowed_tools: raw.allowed_tools,
                disallowed_tools: raw.disallowed_tools,
                prompt: raw.prompt,
                args: raw.args,
            })),
            "codex" => Ok(PaneConfig::Codex(AiPaneConfig {
                pane_type: raw.pane_type.clone(),
                name: raw.name.or(Some(raw.pane_type)),
                path: raw.path,
                color: raw.color,
                notes: raw.notes,
                model: raw.model,
                skills: raw.skills,
                allowed_tools: raw.allowed_tools,
                disallowed_tools: raw.disallowed_tools,
                prompt: raw.prompt,
                args: raw.args,
            })),
            "opencode" => Ok(PaneConfig::Opencode(AiPaneConfig {
                pane_type: raw.pane_type.clone(),
                name: raw.name.or(Some(raw.pane_type)),
                path: raw.path,
                color: raw.color,
                notes: raw.notes,
                model: raw.model,
                skills: raw.skills,
                allowed_tools: raw.allowed_tools,
                disallowed_tools: raw.disallowed_tools,
                prompt: raw.prompt,
                args: raw.args,
            })),
            "antigravity" => Ok(PaneConfig::Antigravity(AiPaneConfig {
                pane_type: raw.pane_type.clone(),
                name: raw.name.or(Some(raw.pane_type)),
                path: raw.path,
                color: raw.color,
                notes: raw.notes,
                model: raw.model,
                skills: raw.skills,
                allowed_tools: raw.allowed_tools,
                disallowed_tools: raw.disallowed_tools,
                prompt: raw.prompt,
                args: raw.args,
            })),
            // "custom" type requires a name field
            "custom" => {
                let name = raw.name.ok_or_else(|| {
                    serde::de::Error::custom("custom pane type requires a 'name' field")
                })?;
                Ok(PaneConfig::Custom(CustomPaneConfig {
                    pane_type: raw.pane_type,
                    name,
                    path: raw.path,
                    color: raw.color,
                    command: raw.command,
                    notes: raw.notes,
                }))
            }
            // Legacy: "shell" and other unknown types become custom panes
            // The type becomes the name for backwards compatibility
            _ => Ok(PaneConfig::Custom(CustomPaneConfig {
                pane_type: "custom".to_string(),
                name: raw.name.unwrap_or(raw.pane_type),
                path: raw.path,
                color: raw.color,
                command: raw.command,
                notes: raw.notes,
            })),
        }
    }
}

impl PaneConfig {
    /// Get the unique pane identifier (name) for referencing in grids
    /// For AI panes, this defaults to the type (claude, codex, etc.) unless overridden
    /// For custom panes, this is the required name field
    pub fn pane_type(&self) -> &str {
        match self {
            PaneConfig::Claude(c)
            | PaneConfig::Codex(c)
            | PaneConfig::Opencode(c)
            | PaneConfig::Antigravity(c) => c.name.as_deref().unwrap_or(&c.pane_type),
            PaneConfig::Custom(c) => &c.name,
        }
    }

    /// Get the actual type (claude, codex, custom, etc.)
    pub fn actual_type(&self) -> &str {
        match self {
            PaneConfig::Claude(c)
            | PaneConfig::Codex(c)
            | PaneConfig::Opencode(c)
            | PaneConfig::Antigravity(c) => &c.pane_type,
            PaneConfig::Custom(c) => &c.pane_type,
        }
    }

    /// Get the color if set
    pub fn color(&self) -> Option<&str> {
        match self {
            PaneConfig::Claude(c)
            | PaneConfig::Codex(c)
            | PaneConfig::Opencode(c)
            | PaneConfig::Antigravity(c) => c.color.as_deref(),
            PaneConfig::Custom(c) => c.color.as_deref(),
        }
    }

    /// Set the color
    pub fn set_color(&mut self, color: String) {
        match self {
            PaneConfig::Claude(c)
            | PaneConfig::Codex(c)
            | PaneConfig::Opencode(c)
            | PaneConfig::Antigravity(c) => {
                c.color = Some(color);
            }
            PaneConfig::Custom(c) => c.color = Some(color),
        }
    }

    /// Get the path if set
    pub fn path(&self) -> Option<&str> {
        match self {
            PaneConfig::Claude(c)
            | PaneConfig::Codex(c)
            | PaneConfig::Opencode(c)
            | PaneConfig::Antigravity(c) => c.path.as_deref(),
            PaneConfig::Custom(c) => c.path.as_deref(),
        }
    }

    /// Set the path
    pub fn set_path(&mut self, path: String) {
        match self {
            PaneConfig::Claude(c)
            | PaneConfig::Codex(c)
            | PaneConfig::Opencode(c)
            | PaneConfig::Antigravity(c) => {
                c.path = Some(path);
            }
            PaneConfig::Custom(c) => c.path = Some(path),
        }
    }

    /// Get notes
    pub fn notes(&self) -> &[String] {
        match self {
            PaneConfig::Claude(c)
            | PaneConfig::Codex(c)
            | PaneConfig::Opencode(c)
            | PaneConfig::Antigravity(c) => &c.notes,
            PaneConfig::Custom(c) => &c.notes,
        }
    }
}

/// Configuration for AI panes (claude, codex, opencode, antigravity)
#[derive(Debug, Deserialize, Clone, Default)]
pub struct AiPaneConfig {
    /// The pane type identifier (claude, codex, etc.)
    #[serde(default, rename = "type")]
    pub pane_type: String,
    /// Unique name for referencing in grids (defaults to pane_type)
    #[serde(default)]
    pub name: Option<String>,
    /// Working directory path
    #[serde(default)]
    pub path: Option<String>,
    /// Pane background color
    #[serde(default)]
    pub color: Option<String>,
    /// Notes to display in pane header
    #[serde(default)]
    pub notes: Vec<String>,
    /// Model to use (e.g., "sonnet", "opus")
    #[serde(default)]
    pub model: Option<String>,
    /// Agents to load - use "*" for all, or list specific names
    #[serde(default)]
    pub skills: Vec<String>,
    /// Allowed tools
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// Disallowed tools
    #[serde(default)]
    pub disallowed_tools: Vec<String>,
    /// Initial prompt to send
    #[serde(default)]
    pub prompt: Option<String>,
    /// Additional CLI arguments
    #[serde(default)]
    pub args: Vec<String>,
}

/// Configuration for custom pane types
#[derive(Debug, Clone)]
pub struct CustomPaneConfig {
    /// The type (e.g., "custom", "shell", or a custom type name)
    pub pane_type: String,
    /// Unique name for referencing in grids (required for custom panes)
    pub name: String,
    /// Working directory path
    pub path: Option<String>,
    /// Pane background color
    pub color: Option<String>,
    /// Command to execute
    pub command: Option<String>,
    /// Notes to display in pane header
    pub notes: Vec<String>,
}

impl Default for CustomPaneConfig {
    fn default() -> Self {
        Self {
            pane_type: "custom".to_string(),
            name: "shell".to_string(),
            path: None,
            color: None,
            command: None,
            notes: Vec::new(),
        }
    }
}

/// Resolved pane with config and layout merged
#[derive(Debug, Clone)]
pub struct ResolvedPane {
    /// Pane name
    pub name: String,
    /// Column position
    pub col: u32,
    /// Row position
    pub row: u32,
    /// Width percentage
    pub width: Option<u32>,
    /// Height percentage
    pub height: Option<u32>,
    /// Pane configuration
    pub config: PaneConfig,
}

impl ResolvedPane {
    /// Get the path if set
    pub fn path(&self) -> Option<&str> {
        self.config.path()
    }

    /// Get the color if set
    pub fn color(&self) -> Option<&str> {
        self.config.color()
    }

    /// Get notes
    pub fn notes(&self) -> &[String] {
        self.config.notes()
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Get the workspaces directory
pub fn workspaces_dir() -> PathBuf {
    PathBuf::from("/Users/ludovic/Coding/barrel/workspaces")
}

/// Extract YAML frontmatter from a markdown file.
/// Frontmatter is delimited by `---` at the start of the file.
fn extract_frontmatter(content: &str) -> Result<&str> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        anyhow::bail!("No frontmatter found: file must start with ---");
    }
    let after_opening = &trimmed[3..];
    let after_opening = after_opening.strip_prefix('\n').unwrap_or(after_opening);
    match after_opening.find("\n---") {
        Some(end) => Ok(&after_opening[..end]),
        None => anyhow::bail!("No closing --- found for frontmatter"),
    }
}

/// Load workspace configuration from a file.
/// Parses YAML from markdown frontmatter.
pub fn load_config(path: &Path) -> Result<WorkspaceConfig> {
    let content = std::fs::read_to_string(path)?;
    let yaml = extract_frontmatter(&content)?;
    let mut config: WorkspaceConfig = serde_yaml::from_str(yaml)?;
    config.manifest_path = Some(path.to_path_buf());
    Ok(config)
}

/// Generate a new workspace configuration as a markdown file with YAML frontmatter
pub fn generate_config(workspace: &str, _workspace_path: &str) -> String {
    format!(
        r#"---
workspace: {workspace}

# =============================================================================
# Skill directories
# =============================================================================
# Search paths for skill files (first match wins for duplicate names)
# Supports: ./relative, ~/home, /absolute paths

skills:
  - path: ./skills
  - path: ~/.config/axel/skills

# =============================================================================
# Layouts
# =============================================================================

layouts:
  # ---------------------------------------------------------------------------
  # Pane definitions
  # ---------------------------------------------------------------------------
  # Define panes that can be used in grid layouts
  #
  # Built-in types: claude, codex, opencode, antigravity, shell
  # Custom types use the 'command' field

  panes:
    # Claude Code - AI coding assistant
    - type: claude
      color: gray
      skills:
        - "*"                    # Load all skills, or list specific: ["skill1", "skill2"]
      # model: sonnet            # Model: sonnet, opus, haiku
      # prompt: "Your task..."   # Initial prompt
      # allowed_tools: []        # Restrict to specific tools
      # disallowed_tools: []     # Block specific tools
      # args: []                 # Additional CLI arguments

    # Codex - OpenAI coding assistant
    - type: codex
      color: green
      skills:
        - "*"
      # model: o3-mini           # Model to use
      # prompt: "Your task..."   # Initial prompt
      # args: []                 # Additional CLI arguments

    # OpenCode - Open-source coding assistant
    # - type: opencode
    #   color: blue
    #   skills: ["*"]

    # Antigravity - Google coding assistant
    # - type: antigravity
    #   color: orange
    #   skills: ["*"]
    #   # model: gemini-3-pro    # Model to use

    # Regular shell with notes displayed on startup
    - type: shell
      notes:
        - "$ axel -k {workspace}"

    # Custom command example
    # - type: logs
    #   command: "tail -f /var/log/app.log"
    #   color: red

  # ---------------------------------------------------------------------------
  # Grid layouts
  # ---------------------------------------------------------------------------
  # Layout configurations for tmux sessions
  #
  # Grid types:
  #   tmux    - Standard tmux session (default)
  #   tmux_cc - iTerm2 tmux integration mode
  #   shell   - No tmux, run first pane directly
  #
  # Cell positioning:
  #   col: 0, 1, 2...  - Column position (left to right)
  #   row: 0, 1, 2...  - Row position within column (top to bottom)
  #   width: 50        - Column width percentage
  #   height: 30       - Row height percentage
  #
  # Colors: purple, yellow, red, green, blue, gray, orange

  grids:
    # Default grid - two columns
    default:
      type: tmux
      claude:
        col: 0
        row: 0
      shell:
        col: 1
        row: 0
        color: yellow

    # Solo mode - single AI pane
    # solo:
    #   type: shell
    #   claude:
    #     col: 0
    #     row: 0

    # Three column layout
    # wide:
    #   type: tmux
    #   claude:
    #     col: 0
    #     row: 0
    #     width: 40
    #   codex:
    #     col: 1
    #     row: 0
    #     width: 40
    #   shell:
    #     col: 2
    #     row: 0
    #     width: 20
---

# {workspace}

<!-- Project context for AI assistants. This content is used as initial context when launching panes. -->

## Overview

<!-- Brief description of what this project does -->

## Getting Started

<!-- How to set up and run the project -->

## Architecture

<!-- High-level architecture overview -->

## Key Files

<!-- Important files and directories -->
"#,
        workspace = workspace,
    )
}

/// Convert color name to tmux color code
pub fn to_tmux_color(color: &str) -> &'static str {
    match color {
        "purple" => "#251F2B",
        "yellow" => "#2B2011",
        "red" => "#231517",
        "green" => "#122322",
        "blue" => "#1E202E",
        "gray" | "grey" => "#1a1a1a",
        "orange" => "#2B2011",
        _ => "default",
    }
}

/// Convert color name to RGB for terminal escape sequences
pub fn to_fg_rgb(color: &str) -> &'static str {
    match color {
        "purple" => "198;147;241",
        "yellow" => "255;182;21",
        "red" => "251;109;136",
        "green" => "0;217;146",
        "blue" => "133;162;255",
        "gray" | "grey" => "150;150;150",
        "orange" => "255;182;21",
        _ => "255;255;255",
    }
}

/// Expand ~ to home directory in paths
pub fn expand_path(path: &str) -> String {
    path.strip_prefix("~/")
        .and_then(|stripped| dirs::home_dir().map(|home| home.join(stripped)))
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_parsing_without_frontmatter() {
        let content = "# Test Agent\n\nYou are a helpful skill.";
        let temp_dir = std::env::temp_dir();
        let skill_path = temp_dir.join("test-skill.md");
        std::fs::write(&skill_path, content).unwrap();

        let skill = Skill::from_file(&skill_path).unwrap();
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.prompt, content);
        assert!(skill.description.contains("Test Agent") || skill.description.contains("helpful"));

        std::fs::remove_file(&skill_path).ok();
    }

    #[test]
    fn test_skill_parsing_with_frontmatter() {
        let content = r#"---
name: custom-name
description: A custom description
tools: Read, Write, Bash
model: opus
---

# My Agent

You are a specialized skill."#;

        let temp_dir = std::env::temp_dir();
        let skill_path = temp_dir.join("frontmatter-skill.md");
        std::fs::write(&skill_path, content).unwrap();

        let skill = Skill::from_file(&skill_path).unwrap();
        assert_eq!(skill.name, "custom-name");
        assert_eq!(skill.description, "A custom description");
        assert_eq!(
            skill.tools,
            Some(vec![
                "Read".to_string(),
                "Write".to_string(),
                "Bash".to_string()
            ])
        );
        assert_eq!(skill.model, Some("opus".to_string()));
        assert!(skill.prompt.contains("My Agent"));
        assert!(skill.prompt.contains("specialized skill"));

        std::fs::remove_file(&skill_path).ok();
    }

    #[test]
    fn test_skill_dir_structure() {
        let temp_dir = std::env::temp_dir().join("axel-test-skills");
        let skill_dir = temp_dir.join("my-skill");
        std::fs::create_dir_all(&skill_dir).ok();

        let skill_file = skill_dir.join("SKILL.md");
        std::fs::write(&skill_file, "# My Agent\n\nHello").unwrap();

        let skill = Skill::from_file(&skill_file).unwrap();
        assert_eq!(skill.name, "my-skill");

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}

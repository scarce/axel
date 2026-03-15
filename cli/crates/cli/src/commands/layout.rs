//! Layout listing commands for axel.
//!
//! Provides commands to query layout configurations from AXEL.md manifests.

use std::path::Path;

use anyhow::Result;
use axel_core::config::{Grid, GridType, PaneConfig, load_config};
use serde::Serialize;

/// JSON output format for a pane configuration
#[derive(Serialize)]
pub struct PaneInfo {
    /// Pane type identifier (e.g., "claude", "codex", "shell")
    #[serde(rename = "type")]
    pub pane_type: String,
    /// Display name for the pane
    pub name: String,
    /// Color associated with this pane
    pub color: Option<String>,
    /// Whether this is an AI pane (vs custom command)
    pub is_ai: bool,
}

impl From<&PaneConfig> for PaneInfo {
    fn from(config: &PaneConfig) -> Self {
        // Get the unique identifier (name) and actual type
        let pane_id = config.pane_type().to_string();
        let actual_type = config.actual_type();

        let (color, is_ai) = match config {
            PaneConfig::Claude(c) => (c.color.clone(), true),
            PaneConfig::Codex(c) => (c.color.clone(), true),
            PaneConfig::Opencode(c) => (c.color.clone(), true),
            PaneConfig::Antigravity(c) => (c.color.clone(), true),
            PaneConfig::Custom(c) => (c.color.clone(), false),
        };

        // Generate display name from actual type or pane_id
        let name = match actual_type {
            "claude" => "Claude".to_string(),
            "codex" => "Codex".to_string(),
            "opencode" => "OpenCode".to_string(),
            "antigravity" => "Antigravity".to_string(),
            "custom" => {
                // For custom panes, use the pane_id (name) as display name
                // Capitalize first letter
                let mut chars = pane_id.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                    None => pane_id.clone(),
                }
            }
            other => {
                // Legacy: capitalize first letter for unknown types
                let mut chars = other.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                    None => other.to_string(),
                }
            }
        };

        PaneInfo {
            pane_type: actual_type.to_string(),
            name,
            color,
            is_ai,
        }
    }
}

/// JSON output format for a grid configuration
#[derive(Serialize)]
pub struct GridInfo {
    /// Grid name (e.g., "default", "wide")
    pub name: String,
    /// Grid type: tmux, tmux_cc, or shell
    #[serde(rename = "type")]
    pub grid_type: String,
    /// Number of pane cells in this grid
    pub pane_count: usize,
    /// Cell configurations
    pub cells: Vec<GridCellInfo>,
}

/// JSON output format for a grid cell
#[derive(Serialize)]
pub struct GridCellInfo {
    /// Pane type this cell references
    pub pane_type: String,
    /// Column position
    pub col: u32,
    /// Row position
    pub row: u32,
    /// Width percentage (if specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Height percentage (if specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Color override (if specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl GridInfo {
    fn from_grid(name: &str, grid: &Grid) -> Self {
        let grid_type = match grid.grid_type {
            GridType::Tmux => "tmux",
            GridType::TmuxCC => "tmux_cc",
            GridType::Shell => "shell",
        };

        let cells: Vec<GridCellInfo> = grid
            .cells
            .iter()
            .map(|(pane_type, cell)| GridCellInfo {
                pane_type: pane_type.clone(),
                col: cell.col,
                row: cell.row,
                width: cell.width,
                height: cell.height,
                color: cell.color.clone(),
            })
            .collect();

        GridInfo {
            name: name.to_string(),
            grid_type: grid_type.to_string(),
            pane_count: cells.len(),
            cells,
        }
    }
}

/// Combined layout output including both panes and grids
#[derive(Serialize)]
pub struct LayoutInfo {
    pub panes: Vec<PaneInfo>,
    pub grids: Vec<GridInfo>,
}

/// List all panes defined in the workspace AXEL.md
pub fn list_panes(manifest_path: Option<&str>, _json: bool) -> Result<()> {
    let path = manifest_path.unwrap_or("./AXEL.md");
    let config = load_config(Path::new(path))?;

    let panes: Vec<PaneInfo> = config.layouts.panes.iter().map(PaneInfo::from).collect();
    let grids: Vec<GridInfo> = config
        .layouts
        .grids
        .iter()
        .map(|(name, grid)| GridInfo::from_grid(name, grid))
        .collect();

    let layout = LayoutInfo { panes, grids };

    // Always output JSON for now (the flag is for future plain text support)
    let json = serde_json::to_string_pretty(&layout)?;
    println!("{}", json);

    Ok(())
}

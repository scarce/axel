//! Axel Core - Core library for the axel workspace launcher
//!
//! This crate provides the core functionality for axel including:
//! - Configuration parsing and types
//! - Tmux session management
//! - Skill driver implementations
//! - Claude command building
//! - Git worktree management
//! - Claude hooks configuration

pub mod claude;
pub mod config;
pub mod drivers;
pub mod git;
pub mod hooks;
pub mod server;
pub mod tmux;

// Re-export commonly used types at crate root
pub use config::{
    AiPaneConfig, CustomPaneConfig, Grid, GridCell, GridType, LayoutsConfig, PaneConfig,
    ResolvedPane, Skill, SkillPathConfig, WorkspaceConfig, WorkspaceIndex,
};
pub use drivers::{ClaudeDriver, CodexDriver, OpenCodeDriver, SkillDriver, all_skill_patterns};
pub use hooks::{
    generate_hooks_settings, otel_logs_endpoint, otel_metrics_endpoint, otel_traces_endpoint,
    settings_path, write_settings,
};

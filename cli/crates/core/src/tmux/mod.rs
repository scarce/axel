//! Tmux session management for axel workspaces.
//!
//! This module provides utilities for creating and managing tmux sessions,
//! including session creation, pane layout, and workspace configuration.
//!
//! # Submodules
//!
//! - [`commands`]: Low-level tmux command builders (NewSession, SplitWindow, etc.)
//! - [`session`]: High-level workspace creation from axel configuration
//!
//! # Usage
//!
//! The primary entry point is [`create_workspace`], which takes a session name
//! and [`WorkspaceConfig`](crate::WorkspaceConfig) to create a fully configured
//! tmux session with the specified pane layout and AI shells.
//!
//! ```ignore
//! use axel_core::tmux::create_workspace;
//!
//! create_workspace("my-project", &config, Some("default"))?;
//! attach_session("my-project")?;
//! ```
//!
//! For session management, use [`has_session`], [`attach_session`], [`kill_session`],
//! and [`current_session`] to query and control tmux sessions.

mod commands;
mod session;

pub use commands::*;
pub use session::*;

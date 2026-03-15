//! Claude Code hooks configuration.
//!
//! Provides utilities for generating Claude settings.json with hooks
//! that send events to the axel event server.

mod settings;

pub use settings::{
    ClaudeSettings, Hook, HookMatcher, HooksConfig, generate_hooks_settings, otel_logs_endpoint,
    otel_metrics_endpoint, otel_traces_endpoint, settings_path, write_settings,
};

//! Axum route handlers for the event server.

use std::{
    collections::HashMap,
    convert::Infallible,
    io::{BufRead, Write},
    path::PathBuf,
    process::Command,
    sync::Arc,
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post},
};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use super::{
    events::{
        FILE_COMMITTED_EVENT, HookEvent, METRICS_UPDATE_EVENT, OtelEventType, OutboxResponse,
        TimestampedEvent, WORKTREE_COMPLETE_EVENT,
    },
    otel,
};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub event_tx: mpsc::Sender<TimestampedEvent>,
    pub inbox_tx: broadcast::Sender<TimestampedEvent>,
    /// Tmux session name for sending responses back to Claude
    pub tmux_session: Option<String>,
    /// Mapping from Claude session_id to pane_id (for correlating OTEL metrics)
    pub session_to_pane: Arc<RwLock<HashMap<String, String>>>,
    /// Default pane_id for this server (one server per terminal)
    pub default_pane_id: Option<String>,
    /// Whether to auto-approve file edits and commit them
    pub review_post_completion: bool,
    /// Working directory for git operations (isolated worktree path)
    pub working_dir: Option<PathBuf>,
    /// Parent worktree path (if --isolate was used)
    pub parent_worktree_path: Option<PathBuf>,
    /// Tracks pending tool uses (tool_use_id -> file_path) for PostToolUse matching
    pub pending_edits: Arc<RwLock<HashMap<String, String>>>,
    /// Path for debug log file (.axel/server.log)
    pub debug_log_path: Option<PathBuf>,
}

impl AppState {
    fn debug_log(&self, msg: &str) {
        if let Some(ref path) = self.debug_log_path {
            eprintln!("{}", msg);
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(f, "[{}] {}", chrono::Utc::now().format("%H:%M:%S%.3f"), msg);
            }
        }
    }
}

/// Build the router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/inbox", get(handle_inbox_sse))
        .route("/outbox", post(handle_outbox))
        .route("/events/{pane_id}", post(handle_hook_event))
        .route("/worktree/complete", post(handle_worktree_complete))
        // OTEL routes — CLI parses raw OTEL and emits metrics_update SSE events
        .route("/v1/metrics/{pane_id}", post(handle_otel_metrics_with_pane))
        .route("/v1/traces/{pane_id}", post(handle_otel_traces_with_pane))
        .route("/v1/logs/{pane_id}", post(handle_otel_logs_with_pane))
        // OTEL routes without pane_id (Claude Code may not use per-signal endpoints)
        .route("/v1/metrics", post(handle_otel_metrics))
        .route("/v1/traces", post(handle_otel_traces))
        .route("/v1/logs", post(handle_otel_logs))
        .with_state(Arc::new(state))
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// SSE endpoint for inbox events
async fn handle_inbox_sse(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.inbox_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| {
        match result {
            Ok(event) => {
                // Serialize the event to JSON
                match serde_json::to_string(&event) {
                    Ok(json) => Some(Ok(Event::default().data(json))),
                    Err(_) => None,
                }
            }
            Err(_) => None, // Skip lagged messages
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Handle Claude Code hook events (PermissionRequest, Stop, PreToolUse, etc.).
///
/// Claude Code's hooks system sends event data via stdin to a curl command that
/// POSTs to this endpoint. The URL path includes the `pane_id`.
///
/// ## The Shared Settings Problem
///
/// Hooks are configured in `.claude/settings.json` at the workspace level.
/// This file is **shared by ALL Claude instances** in the workspace. When axel creates
/// a new terminal, it overwrites settings.json with hooks pointing to the new terminal's
/// server (port) and pane_id. This means:
///
/// - **All hook events** from every Claude instance arrive at the **last terminal's** server
/// - **All events carry the last terminal's pane_id**, not their own
///
/// ## OTEL-Based Remapping
///
/// OTEL telemetry uses per-process env vars (`OTEL_EXPORTER_OTLP_METRICS_ENDPOINT`)
/// set at process start time. These contain the correct pane_id and can't be overwritten.
/// We build a `session_id → pane_id` mapping from OTEL data and use it to remap
/// hook events to the correct pane_id before broadcasting.
async fn handle_hook_event(
    State(state): State<Arc<AppState>>,
    Path(pane_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Parse event type from raw JSON.
    // NOTE: Claude Code sends "hook_event_name" but HookEvent expects "type",
    // so HookEvent parsing always fails. Read directly from the raw payload.
    let hook_event_name = payload
        .get("hook_event_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let event_type = match serde_json::from_value::<HookEvent>(payload.clone()) {
        Ok(hook_event) => hook_event.event_type.to_string(),
        Err(_) => "unknown_hook".to_string(),
    };

    let tool_name = payload
        .get("tool_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    state.debug_log(&format!(
        "[hooks] event={} tool={} review={}",
        hook_event_name, tool_name, state.review_post_completion
    ));

    // Resolve the correct pane_id using OTEL session mapping
    let resolved_pane_id =
        if let Some(session_id) = payload.get("session_id").and_then(|v| v.as_str()) {
            let mut mapping = state.session_to_pane.write().await;

            if !mapping.contains_key(session_id) {
                mapping.insert(session_id.to_string(), pane_id.clone());
                pane_id.clone()
            } else {
                let correct = mapping.get(session_id).cloned().unwrap_or(pane_id.clone());
                if correct != pane_id {
                    eprintln!(
                        "[hooks] Remapping session {} from pane {} to {} (OTEL-derived)",
                        &session_id[..8.min(session_id.len())],
                        &pane_id[..8.min(pane_id.len())],
                        &correct[..8.min(correct.len())]
                    );
                }
                correct
            }
        } else {
            pane_id.clone()
        };

    // Auto-approve file edits when review_post_completion is enabled.
    // Uses Claude Code's PreToolUse hook response mechanism: returning {"decision": "approve"}
    // in the hook's stdout causes Claude Code to auto-approve without showing a permission prompt.
    let mut approve_response = false;
    if state.review_post_completion {
        if hook_event_name == "PreToolUse" && (tool_name == "Edit" || tool_name == "Write") {
            approve_response = true;

            // Store file_path → tool_use_id for PostToolUse commit matching
            let file_path = payload
                .get("tool_input")
                .and_then(|v| v.get("file_path"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let tool_use_id = payload
                .get("tool_use_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if !tool_use_id.is_empty() && !file_path.is_empty() {
                let mut pending = state.pending_edits.write().await;
                pending.insert(tool_use_id, file_path);
            }
        }

        if hook_event_name == "PostToolUse" && (tool_name == "Edit" || tool_name == "Write") {
            let tool_use_id = payload
                .get("tool_use_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let file_path = {
                let mut pending = state.pending_edits.write().await;
                pending.remove(&tool_use_id)
            };

            if let Some(ref path) = file_path {
                if let Some(ref working_dir) = state.working_dir {
                    let basename = std::path::Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file");
                    let commit_msg = format!("{}({}): edit", tool_name.to_lowercase(), basename);

                    let git_result = Command::new("git")
                        .args(["add", path])
                        .current_dir(working_dir)
                        .output();

                    if git_result.is_ok() {
                        let commit_result = Command::new("git")
                            .args(["commit", "-m", &commit_msg])
                            .current_dir(working_dir)
                            .output();

                        if let Ok(output) = commit_result {
                            if output.status.success() {
                                // Get the commit hash
                                let hash = Command::new("git")
                                    .args(["rev-parse", "--short", "HEAD"])
                                    .current_dir(working_dir)
                                    .output()
                                    .ok()
                                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                                    .unwrap_or_default();

                                // Get the diff for this commit
                                let diff = Command::new("git")
                                    .args(["diff", "HEAD~1..HEAD"])
                                    .current_dir(working_dir)
                                    .output()
                                    .ok()
                                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                                    .unwrap_or_default();

                                state.debug_log(&format!(
                                    "[review] Committed {} ({}) +{}b diff",
                                    basename,
                                    hash,
                                    diff.len()
                                ));

                                // Broadcast FileCommitted event
                                let committed_payload = serde_json::json!({
                                    "commit_hash": hash,
                                    "file_path": path,
                                    "message": commit_msg,
                                    "tool_name": tool_name,
                                    "diff": diff,
                                });

                                let committed_event = TimestampedEvent::new(
                                    FILE_COMMITTED_EVENT,
                                    &resolved_pane_id,
                                    committed_payload,
                                );
                                let _ = state.event_tx.send(committed_event.clone()).await;
                                let _ = state.inbox_tx.send(committed_event);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut event = TimestampedEvent::new(event_type, resolved_pane_id, payload);

    // Attach worktree commits and versioning info to Stop events when review_post_completion is enabled.
    // This lets the macOS app show the commit review UI without needing paneId matching.
    if state.review_post_completion && hook_event_name == "Stop" {
        if let (Some(working_dir), Some(parent_dir)) =
            (&state.working_dir, &state.parent_worktree_path)
        {
            event.worktree_commits = gather_worktree_commits(working_dir, parent_dir);
            state.debug_log(&format!(
                "[review] Attached {} commits to Stop event",
                event.worktree_commits.as_ref().map_or(0, |c| c.len())
            ));
        }

        // Extract VERSIONING commit message from the agent's transcript
        if let Some(transcript_path) = event.event.get("transcript_path").and_then(|v| v.as_str()) {
            if let Some((msg, desc)) = extract_versioning_from_transcript(transcript_path) {
                state.debug_log(&format!("[review] Extracted versioning: {}", msg));
                event.versioning_message = Some(msg);
                event.versioning_description = Some(desc);
            }
        }
    }

    // Send to file logger
    if state.event_tx.send(event.clone()).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to log event");
    }

    // Broadcast to SSE subscribers (ignore errors if no subscribers)
    let _ = state.inbox_tx.send(event);

    // Return approve decision for PreToolUse hooks when auto-approving.
    // Claude Code reads the hook's stdout and acts on hookSpecificOutput.permissionDecision.
    if approve_response {
        (
            StatusCode::OK,
            "{\"hookSpecificOutput\":{\"hookEventName\":\"PreToolUse\",\"permissionDecision\":\"allow\",\"permissionDecisionReason\":\"Auto-approved by review-post-completion\"}}",
        )
    } else {
        (StatusCode::OK, "OK")
    }
}

/// Handle outbox responses from the macOS app.
///
/// The app sends permission responses (allow/deny) and text responses (for questions)
/// back to this endpoint. The server then injects the response into the Claude process
/// via `tmux send-keys`.
///
/// ## Pane ID Correction
///
/// The `pane_id` in the payload may be **wrong** due to the shared `.claude/settings.json`
/// hooks problem (see `handle_hook_event` for details). To fix this, we look up the
/// correct tmux target using the `session_to_pane` mapping (built from OTEL data, which
/// uses per-process env vars and is therefore reliable).
///
/// Flow: app sends `(session_id, pane_id)` → we check `session_to_pane[session_id]`
/// → if it differs from `pane_id`, use the OTEL-derived one → `tmux send-keys -t <correct>`
async fn handle_outbox(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OutboxResponse>,
) -> impl IntoResponse {
    let event_type = payload.response_type.to_string();
    let session_id = payload.session_id.clone();
    let response_text = payload.response_text.clone();

    // Convert to JSON value for storage
    let json_payload = match serde_json::to_value(&payload) {
        Ok(v) => v,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid payload"),
    };

    // Use session_id as the pane_id for outbox responses
    let event = TimestampedEvent::new(event_type, session_id.clone(), json_payload);

    // Send to file logger
    if state.event_tx.send(event.clone()).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to log event");
    }

    // Broadcast to SSE subscribers (so other clients can see the response)
    let _ = state.inbox_tx.send(event);

    // Inject the response into the Claude process
    if let Some(ref tmux_session) = state.tmux_session {
        // Tmux mode: send keys to the appropriate pane.
        // Use session→pane mapping from OTEL to correct for shared .claude/settings.json
        // hooks that may cause events to arrive with the wrong pane_id.
        let target = if let Some(ref pane_id) = payload.pane_id {
            // Check if OTEL has a more reliable mapping for this session_id
            let mapping = state.session_to_pane.read().await;
            let resolved = mapping.get(&payload.session_id).cloned();
            drop(mapping);
            if let Some(ref correct) = resolved {
                if correct != pane_id {
                    eprintln!(
                        "[outbox] Correcting target from {} to {} (OTEL-derived)",
                        &pane_id[..8.min(pane_id.len())],
                        &correct[..8.min(correct.len())]
                    );
                }
                correct.clone()
            } else {
                pane_id.clone()
            }
        } else {
            // Default to first pane in the session (pane 0.0)
            // Skip pane 0 which is the server pane, target pane 1
            format!("{}:0.1", tmux_session)
        };

        // Send the response text literally (handles special chars, spaces, newlines)
        let text_result = Command::new("tmux")
            .args(["send-keys", "-t", &target, "-l", &response_text])
            .output();

        if let Err(e) = text_result {
            eprintln!("[outbox] Failed to send text to tmux: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to send response to tmux",
            );
        }

        // Send Enter key to submit the prompt
        // Use C-m (Ctrl+M / carriage return) which works better with TUI apps like Codex
        let enter_result = Command::new("tmux")
            .args(["send-keys", "-t", &target, "C-m"])
            .output();

        if let Err(e) = enter_result {
            eprintln!("[outbox] Failed to send Enter to tmux: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to send response to tmux",
            );
        }
    } else {
        // Non-tmux mode: write response to a file
        let response_dir = std::path::PathBuf::from(".axel");
        let response_file = response_dir.join(format!("response_{}.txt", session_id));

        // Ensure directory exists
        if let Err(e) = std::fs::create_dir_all(&response_dir) {
            eprintln!("[outbox] Failed to create response directory: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to write response file",
            );
        }

        // Write the response
        if let Err(e) = std::fs::write(&response_file, &response_text) {
            eprintln!("[outbox] Failed to write response file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to write response file",
            );
        }
    }

    (StatusCode::OK, "OK")
}

/// Handle OTEL metrics with pane_id in URL — parses and emits metrics_update events
async fn handle_otel_metrics_with_pane(
    State(state): State<Arc<AppState>>,
    Path(pane_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Log raw OTEL payload for debugging
    let raw_event = TimestampedEvent::new(
        OtelEventType::Metrics.to_string(),
        &pane_id,
        payload.clone(),
    );
    let _ = state.event_tx.send(raw_event).await;

    // Parse OTEL metrics into flat MetricsUpdate structs
    let updates = otel::parse_claude_otel_metrics(&payload);

    // Register session→pane mapping and broadcast each MetricsUpdate
    for update in &updates {
        let mut mapping = state.session_to_pane.write().await;
        mapping.insert(update.session_id.clone(), pane_id.clone());
    }

    broadcast_metrics_updates(&state, &pane_id, &updates).await;

    (StatusCode::OK, "OK")
}

/// Handle OTEL traces with pane_id in URL — just log, no metrics extraction
async fn handle_otel_traces_with_pane(
    State(state): State<Arc<AppState>>,
    Path(pane_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let event = TimestampedEvent::new(OtelEventType::Traces.to_string(), pane_id, payload);
    let _ = state.event_tx.send(event).await;
    (StatusCode::OK, "OK")
}

/// Handle OTEL logs with pane_id in URL — parses Codex logs and emits metrics_update events
async fn handle_otel_logs_with_pane(
    State(state): State<Arc<AppState>>,
    Path(pane_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Log raw OTEL payload for debugging
    let raw_event =
        TimestampedEvent::new(OtelEventType::Logs.to_string(), &pane_id, payload.clone());
    let _ = state.event_tx.send(raw_event).await;

    // Parse Codex OTEL logs into MetricsUpdate structs
    let updates = otel::parse_codex_otel_logs(&payload);

    // Register session→pane mapping and broadcast
    for update in &updates {
        if !update.session_id.is_empty() {
            let mut mapping = state.session_to_pane.write().await;
            mapping.insert(update.session_id.clone(), pane_id.clone());
        }
    }

    broadcast_metrics_updates(&state, &pane_id, &updates).await;

    (StatusCode::OK, "OK")
}

/// Handle OTEL metrics without pane_id — uses server's default pane_id
async fn handle_otel_metrics(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pane_id = state
        .default_pane_id
        .clone()
        .unwrap_or_else(|| "otel".into());

    let raw_event = TimestampedEvent::new(
        OtelEventType::Metrics.to_string(),
        &pane_id,
        payload.clone(),
    );
    let _ = state.event_tx.send(raw_event).await;

    let updates = otel::parse_claude_otel_metrics(&payload);

    for update in &updates {
        let mut mapping = state.session_to_pane.write().await;
        mapping.insert(update.session_id.clone(), pane_id.clone());
    }

    broadcast_metrics_updates(&state, &pane_id, &updates).await;

    (StatusCode::OK, "OK")
}

/// Handle OTEL traces without pane_id — just log
async fn handle_otel_traces(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pane_id = state
        .default_pane_id
        .clone()
        .unwrap_or_else(|| "otel".into());
    let event = TimestampedEvent::new(OtelEventType::Traces.to_string(), pane_id, payload);
    let _ = state.event_tx.send(event).await;
    (StatusCode::OK, "OK")
}

/// Handle OTEL logs without pane_id — uses server's default pane_id
async fn handle_otel_logs(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let pane_id = state
        .default_pane_id
        .clone()
        .unwrap_or_else(|| "otel".into());

    let raw_event =
        TimestampedEvent::new(OtelEventType::Logs.to_string(), &pane_id, payload.clone());
    let _ = state.event_tx.send(raw_event).await;

    let updates = otel::parse_codex_otel_logs(&payload);

    for update in &updates {
        if !update.session_id.is_empty() {
            let mut mapping = state.session_to_pane.write().await;
            mapping.insert(update.session_id.clone(), pane_id.clone());
        }
    }

    broadcast_metrics_updates(&state, &pane_id, &updates).await;

    (StatusCode::OK, "OK")
}

/// Request body for /worktree/complete
#[derive(Debug, Deserialize)]
struct WorktreeCompleteRequest {
    commit_message: String,
    #[serde(default)]
    commit_description: String,
    /// If true, keep the worktree alive by resetting it to the parent's HEAD
    /// instead of removing it. This allows the same Claude session to continue
    /// working in the same directory for chained tasks.
    #[serde(default)]
    keep_worktree: bool,
}

/// Response from /worktree/complete
#[derive(Debug, Serialize)]
struct WorktreeCompleteResponse {
    commit_hash: String,
    files_changed: Vec<String>,
    insertions: u32,
    deletions: u32,
}

/// Handle worktree completion: squash all commits in the isolated worktree
/// and cherry-pick the result into the parent worktree.
async fn handle_worktree_complete(
    State(state): State<Arc<AppState>>,
    Json(request): Json<WorktreeCompleteRequest>,
) -> impl IntoResponse {
    let working_dir = match &state.working_dir {
        Some(dir) => dir.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "No working directory configured"})),
            );
        }
    };

    let parent_dir = match &state.parent_worktree_path {
        Some(dir) => dir.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    serde_json::json!({"error": "No parent worktree configured (not running with --isolate)"}),
                ),
            );
        }
    };

    match crate::git::complete_isolated_worktree(
        &working_dir,
        &parent_dir,
        &request.commit_message,
        &request.commit_description,
        request.keep_worktree,
    ) {
        Ok(result) => {
            let pane_id = state
                .default_pane_id
                .clone()
                .unwrap_or_else(|| "unknown".to_string());

            // Broadcast worktree_complete event
            let complete_payload = serde_json::json!({
                "commit_hash": result.commit_hash,
                "files_changed": result.files_changed,
                "insertions": result.insertions,
                "deletions": result.deletions,
                "commit_message": request.commit_message,
            });

            let complete_event =
                TimestampedEvent::new(WORKTREE_COMPLETE_EVENT, &pane_id, complete_payload);
            let _ = state.event_tx.send(complete_event.clone()).await;
            let _ = state.inbox_tx.send(complete_event);

            (
                StatusCode::OK,
                Json(serde_json::json!(WorktreeCompleteResponse {
                    commit_hash: result.commit_hash,
                    files_changed: result.files_changed,
                    insertions: result.insertions,
                    deletions: result.deletions,
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        ),
    }
}

/// Gather all commits from the isolated worktree since it diverged from the parent.
/// Returns a list of commit objects with hash, message, file_path, tool_name, and diff.
fn gather_worktree_commits(
    working_dir: &std::path::Path,
    parent_dir: &std::path::Path,
) -> Option<Vec<serde_json::Value>> {
    // Get the parent worktree's HEAD (the point from which the temp worktree diverged)
    let parent_head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(parent_dir)
        .output()
        .ok()?;
    if !parent_head.status.success() {
        return None;
    }
    let base = String::from_utf8_lossy(&parent_head.stdout)
        .trim()
        .to_string();

    // Get all commits since the base, oldest first
    let log_output = Command::new("git")
        .args([
            "log",
            "--reverse",
            "--format=%h|%s",
            &format!("{}..HEAD", base),
        ])
        .current_dir(working_dir)
        .output()
        .ok()?;
    if !log_output.status.success() {
        return None;
    }

    let log_str = String::from_utf8_lossy(&log_output.stdout);
    let mut commits = Vec::new();

    for line in log_str.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, '|').collect();
        if parts.len() != 2 {
            continue;
        }

        let hash = parts[0];
        let message = parts[1];

        // Get changed files for this commit
        let file_path = Command::new("git")
            .args(["diff-tree", "--no-commit-id", "--name-only", "-r", hash])
            .current_dir(working_dir)
            .output()
            .ok()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string()
            })
            .unwrap_or_default();

        // Get diff for this commit
        let diff = Command::new("git")
            .args(["show", "--format=", hash])
            .current_dir(working_dir)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        // Extract tool name from commit message pattern "edit(file): edit" or "write(file): edit"
        let tool_name = if message.starts_with("edit(") {
            "Edit"
        } else if message.starts_with("write(") {
            "Write"
        } else {
            "Unknown"
        };

        commits.push(serde_json::json!({
            "commit_hash": hash,
            "file_path": file_path,
            "message": message,
            "tool_name": tool_name,
            "diff": diff,
        }));
    }

    if commits.is_empty() {
        None
    } else {
        Some(commits)
    }
}

/// Extract VERSIONING commit message/description from a Claude Code transcript file.
///
/// The transcript is a JSONL file where each line is a JSON object with a `message` field.
/// Agent responses have `message.role == "assistant"` and content containing:
/// `VERSIONING: {"versioningMessage": "...", "versioningDescription": "..."}`
///
/// We read the file backwards to find the LAST assistant message containing VERSIONING,
/// since the agent may produce multiple responses (e.g., after retries).
fn extract_versioning_from_transcript(transcript_path: &str) -> Option<(String, String)> {
    let file = std::fs::File::open(transcript_path).ok()?;
    let reader = std::io::BufReader::new(file);

    let mut last_versioning: Option<(String, String)> = None;

    for line in reader.lines() {
        let line = line.ok()?;
        if line.is_empty() {
            continue;
        }

        // Quick string check before parsing JSON (performance optimization)
        if !line.contains("VERSIONING") {
            continue;
        }

        let entry: serde_json::Value = serde_json::from_str(&line).ok()?;
        let message = entry.get("message")?;

        // Only look at assistant messages
        if message.get("role")?.as_str()? != "assistant" {
            continue;
        }

        // Content can be a string or an array of blocks
        let content = message.get("content")?;
        let text = if let Some(s) = content.as_str() {
            s.to_string()
        } else if let Some(arr) = content.as_array() {
            arr.iter()
                .filter_map(|block| block.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            continue;
        };

        // Find the VERSIONING line
        for text_line in text.lines() {
            if let Some(json_str) = text_line.strip_prefix("VERSIONING: ") {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) {
                    let msg = v
                        .get("versioningMessage")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let desc = v
                        .get("versioningDescription")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if !msg.is_empty() {
                        last_versioning = Some((msg, desc));
                    }
                }
            }
        }
    }

    last_versioning
}

/// Broadcast parsed MetricsUpdate structs as `metrics_update` SSE events.
async fn broadcast_metrics_updates(
    state: &AppState,
    pane_id: &str,
    updates: &[otel::MetricsUpdate],
) {
    for update in updates {
        let json_payload = match serde_json::to_value(update) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let event = TimestampedEvent::new(METRICS_UPDATE_EVENT, pane_id, json_payload);

        // Broadcast to SSE subscribers (don't log parsed events — raw is already logged)
        let _ = state.inbox_tx.send(event);
    }
}

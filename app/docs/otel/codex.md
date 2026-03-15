# Codex OTEL Architecture

How token usage and event data flows from Codex through Axel CLI to the macOS app.

## Pipeline Overview

```
Codex (Rust, codex-rs)
    │  OTEL Logs via tracing -> OpenTelemetryTracingBridge
    │  OTEL Metrics via MetricsClient (separate)
    │
    ▼  POST /v1/logs/{pane_id}   (per-response token counts)
    ▼  POST /v1/metrics/{pane_id} (analytics metrics)
Axel CLI Server (Rust, axum)
    │  Wraps as TimestampedEvent { event_type: "otel_logs", pane_id }
    │
    ▼  SSE broadcast on /inbox
Axel macOS App (Swift)
    │  InboxService.parseOTELLogs()
    │  parseOTELLogRecord() — extracts token data from attributes
    │  handleCodexOTELEvent() — creates synthetic Stop events
    │
    ▼  CostTracker.recordMetrics(forSession:...)
```

## Key Difference from Claude

| | Claude Code | Codex |
|--|------------|-------|
| **Transport** | OTEL Metrics (`resourceMetrics`) | OTEL Logs (`resourceLogs`) |
| **Token values** | Cumulative per session | Per-response (each API call) |
| **Session ID** | `session.id` attribute | `conversation.id` attribute |
| **Config method** | Environment variables | `-c` CLI args (TOML config) |
| **Metric prefix** | `claude_code.*` | N/A (uses log event attributes) |
| **Task completion** | Hook event (`Stop`) | Synthetic from `response.completed` log |

## 1. Codex (Source)

### OTEL Implementation

**Files**: `codex-rs/otel/src/`

Codex uses Rust's `tracing` crate with an `OpenTelemetryTracingBridge` to export structured events as OTEL logs. This is fundamentally different from Claude's OTEL metrics approach.

```rust
// codex-rs/otel/src/otel_provider.rs
// Events with target "codex_otel" are filtered and exported as OTEL logs
OpenTelemetryTracingBridge::new(&logger_provider)
```

### Configuration (set by `axel-cli/crates/core/src/drivers/codex.rs`)

Codex uses `-c` CLI flags instead of environment variables:

| Config | Value | Purpose |
|--------|-------|---------|
| `analytics_enabled` | `true` | Enable metrics export |
| `otel.exporter` | `{otlp-http={endpoint="...",protocol="json"}}` | Log exporter (OTLP HTTP) |
| `otel.trace_exporter` | `{otlp-http={endpoint="...",protocol="json"}}` | Trace exporter |
| `otel.metrics_exporter` | `{otlp-http={endpoint="...",protocol="json"}}` | Metrics exporter |

### Log Events

**File**: `codex-rs/otel/src/traces/otel_manager.rs`

All events are emitted via `tracing::event!()` with `target: "codex_otel"`. Each event has an `event.name` attribute.

#### `codex.sse_event` — API Response Events

Emitted for each Server-Sent Event from the OpenAI API.

**Attributes**:
- `event.name`: `"codex.sse_event"`
- `event.kind`: The SSE event type (e.g., `"response.completed"`, `"response.output_item.done"`)
- `conversation.id`: Session UUID
- `model`: Model name
- `app.version`: Codex version

When `event.kind == "response.completed"`, additional token attributes are included:
- `input_token_count`: Input tokens for this response
- `output_token_count`: Output tokens for this response
- `cached_token_count`: Cached input tokens
- `reasoning_token_count`: Reasoning tokens (o-series models)
- `tool_token_count`: Tokens used by tool calls

**Critical**: These are **per-response** counts (not cumulative). Each API call reports its own usage independently. The macOS app must **accumulate** these into cumulative values before passing to CostTracker (which expects cumulative).

#### `codex.tool_decision` — Tool Approval/Rejection

**Attributes**:
- `event.name`: `"codex.tool_decision"`
- `tool_name`: Name of the tool
- `decision`: `"approved"` or `"rejected"`
- `source`: Who made the decision (e.g., `"user"`, `"auto"`)
- `conversation.id`: Session UUID

#### `codex.tool_result` — Tool Execution Result

**Attributes**:
- `event.name`: `"codex.tool_result"`
- `tool_name`: Name of the tool
- `call_id`: Tool call identifier
- `success`: `"true"` or `"false"`
- `output`: Tool output (may be truncated)
- `conversation.id`: Session UUID

#### Other Events

- `codex.conversation_starts` — New conversation started
- `codex.api_request` — API request made
- `codex.user_prompt` — User submitted a prompt

## 2. Axel CLI Server (Relay)

Same server as Claude, different routes:

| Route | Event Type | Purpose |
|-------|-----------|---------|
| `POST /v1/logs/{pane_id}` | `otel_logs` | Primary: Codex log events with token data |
| `POST /v1/metrics/{pane_id}` | `otel_metrics` | Codex analytics metrics |
| `POST /v1/traces/{pane_id}` | `otel_traces` | Codex traces |

The pane_id is embedded in the URL (configured via `-c` CLI args).

### No Hook Events

Unlike Claude, Codex does not have a hook system. All lifecycle events come through OTEL logs. The Axel CLI uses tmux bell hooks for permission request detection (see `tmux_bell_hook_command()` in `codex.rs`).

## 3. Axel macOS App (Consumer)

**File**: `Axel/Services/InboxService.swift`

### Log Parsing Flow

1. **Route**: `event_type == "otel_logs"` -> `parseOTELLogs()`
2. **Iterate**: `resourceLogs[].scopeLogs[].logRecords[]`
3. **For each record**, call `parseOTELLogRecord()`

### `parseOTELLogRecord()`

Builds an attribute map from the record's `attributes[]` array, then:

1. **Check `event.name`** -> route to `handleCodexOTELEvent()` for event-specific handling
2. **Try parsing body as JSON** — checks for `usage.prompt_tokens` or `tokens.input` objects
3. **Extract tokens** using multiple naming conventions (prioritized):
   - `input_token_count` / `output_token_count` (Codex native)
   - `input_tokens` / `output_tokens` (Claude style)
   - `prompt_tokens` / `completion_tokens` (OpenAI API style)
   - `gen_ai.usage.*` (OTEL semantic conventions)
   - `llm.usage.*` (alternative conventions)
4. **Extract session ID** from: `session.id`, `session_id`, `conversation.id`, `gen_ai.session.id`
5. **Record** to CostTracker via `recordMetrics(forSession:...)`

### `handleCodexOTELEvent()`

Handles Codex-specific event types:

- **`codex.sse_event` with `event.kind == "response.completed"`**: Creates a synthetic `Stop` inbox event (since Codex has no hook-based Stop event). Includes 5-second debounce to avoid duplicates.
- **`codex.tool_decision`**: Logs tool approval/rejection (informational).
- **`codex.tool_result`**: Logs tool execution results (informational).

### Token Accumulation Model — BUG (fixed)

Codex sends **per-response** token counts, but `CostTracker.recordMetrics()` expects **cumulative** values.

**The bug**: `parseOTELLogRecord()` called `metrics.updateFromOTEL()` which **assigns** values. Then it passed `metrics.inputTokens` to CostTracker. Since each response overwrites the previous count, smaller responses would be lost:

```
Response 1: input_token_count=1500 → metrics.inputTokens=1500 → CostTracker sees 1500, delta=1500 ✓
Response 2: input_token_count=800  → metrics.inputTokens=800  → CostTracker sees 800 < lastSession(1500), delta=0 ✗
Response 3: input_token_count=2000 → metrics.inputTokens=2000 → CostTracker sees 2000, delta=500 (should be 2000) ✗
```

**The fix**: `parseOTELLogRecord()` now **adds** per-response tokens to a running total before passing to CostTracker, making the values cumulative:

```
Response 1: += 1500 → cumulative=1500 → CostTracker delta=1500 ✓
Response 2: += 800  → cumulative=2300 → CostTracker delta=800 ✓
Response 3: += 2000 → cumulative=4300 → CostTracker delta=2000 ✓
```

## Debugging

Look for `[OTEL Log]` prefixes in Xcode console:

```
[OTEL Log] pane=abc-123 body=nil attrs=[conversation.id, event.kind, event.name, input_token_count, ...]
[OTEL Log] Found token data: session=abc-123 in=1500 out=200 cost=0.0
[InboxService] Created Codex completion event for session a1b2c3d4...
```

### Common Issues

1. **No token data appearing**: Check if `analytics_enabled=true` is in Codex config. Without it, no OTEL data is exported.
2. **Missing completion events**: `response.completed` must appear in `event.kind`. If the event name format changes, the synthetic Stop won't fire.
3. **Duplicate completions**: The 5-second debounce in `handleCodexOTELEvent()` prevents duplicate Stop events. If completions are spaced > 5s apart, each gets its own event.
4. **Permission detection via bell**: Codex doesn't use hooks for permissions. Instead, tmux bell hooks capture pane content and grep for approval patterns (`"Yes, proceed"`, `"Codex wants to edit"`). This is fragile and depends on Codex's TUI output format.

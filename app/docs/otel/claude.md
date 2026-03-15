# Claude Code OTEL Architecture

How token usage, cost, and activity data flows from Claude Code through Axel CLI to the macOS app.

## Pipeline Overview

```
Claude Code (Node.js)
    │  OTEL_METRICS_EXPORTER=otlp
    │  OTEL_EXPORTER_OTLP_PROTOCOL=http/json
    │  OTEL_METRIC_EXPORT_INTERVAL=2000ms
    │
    ▼  POST /v1/metrics/{pane_id}  (cumulative counters)
Axel CLI Server (Rust, axum)
    │  Wraps payload as TimestampedEvent { event_type: "otel_metrics", pane_id }
    │
    ▼  SSE broadcast on /inbox
Axel macOS App (Swift)
    │  InboxService.parseOTELMetrics()
    │  InboxService.parseMetric() — aggregates across models per session
    │
    ▼  CostTracker.recordMetrics(forSession:...)
    │  TerminalCostTracker — computes deltas from cumulative values
    │
    ▼  SpeedometerGauge — reads recentThroughput(forProvider:)
```

## 1. Claude Code (Source)

Claude Code exports OTEL **metrics** (not logs or traces for token data). It uses standard OTEL SDK environment variables.

### Environment Variables (set by `axel-cli/crates/core/src/drivers/claude.rs`)

| Variable | Value | Purpose |
|----------|-------|---------|
| `CLAUDE_CODE_ENABLE_TELEMETRY` | `1` | Master switch for telemetry |
| `OTEL_METRICS_EXPORTER` | `otlp` | Enable OTLP metrics export |
| `OTEL_TRACES_EXPORTER` | `otlp` | Enable OTLP traces export |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | `http/json` | Use HTTP+JSON (not gRPC) |
| `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` | `http://localhost:{port}/v1/metrics/{pane_id}` | Metrics endpoint with pane routing |
| `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` | `http://localhost:{port}/v1/traces/{pane_id}` | Traces endpoint with pane routing |
| `OTEL_METRIC_EXPORT_INTERVAL` | `2000` | Export every 2 seconds |
| `OTEL_BSP_SCHEDULE_DELAY` | `2000` | Batch span processor delay |

Note: No `OTEL_LOGS_EXPORTER` is set — Claude uses metrics, not logs.

### Metric Names

Claude Code exports metrics with the prefix `claude_code.`:

| Metric | Type Attribute | Description |
|--------|---------------|-------------|
| `claude_code.token.usage` | `input` | Cumulative input tokens (per model) |
| `claude_code.token.usage` | `output` | Cumulative output tokens (per model) |
| `claude_code.token.usage` | `cacheRead` | Cumulative cache read tokens |
| `claude_code.token.usage` | `cacheCreation` | Cumulative cache creation tokens |
| `claude_code.cost.usage` | — | Cumulative cost in USD |
| `claude_code.lines_of_code.count` | `added` | Lines added |
| `claude_code.lines_of_code.count` | `removed` | Lines removed |
| `claude_code.active_time.total` | — | Active time in seconds |

### Data Point Structure

Each metric contains `sum.dataPoints[]`. Each data point has:
- `asDouble`: The cumulative value
- `attributes[]`: Array of `{key, value: {stringValue}}` pairs including:
  - `session.id`: Claude session UUID
  - `type`: Metric subtype (e.g., `input`, `output`, `cacheRead`)

**Critical**: Values are **cumulative per model** since session start. A single session using multiple models (e.g., Haiku + Sonnet) will have separate data points for each model. The macOS app must sum across all data points for the same session.

## 2. Axel CLI Server (Relay)

**File**: `axel-cli/crates/core/src/server/routes.rs`

The server accepts OTEL data and relays it as SSE events.

### Routes

| Route | Handler | Purpose |
|-------|---------|---------|
| `POST /v1/metrics/{pane_id}` | `handle_otel_metrics_with_pane` | Primary: pane_id in URL |
| `POST /v1/traces/{pane_id}` | `handle_otel_traces_with_pane` | Traces with pane routing |
| `POST /v1/logs/{pane_id}` | `handle_otel_logs_with_pane` | Logs with pane routing |
| `POST /v1/metrics` | `handle_otel_metrics` | Legacy: extracts session.id, looks up pane mapping |
| `GET /inbox` | SSE stream | macOS app connects here |
| `POST /events/{pane_id}` | Hook events | Claude hook payloads (SessionStart, Stop, etc.) |

### Event Wrapping

OTEL payloads are wrapped as `TimestampedEvent`:
```json
{
  "timestamp": "2026-02-09T...",
  "event_type": "otel_metrics",
  "pane_id": "abc-123",
  "event": { /* raw OTEL JSON payload */ }
}
```

### Session-to-Pane Mapping

Hook events (`/events/{pane_id}`) include `session_id` in the payload. The server stores `session_id -> pane_id` mappings in `AppState.session_to_pane` for legacy OTEL routes that don't include pane_id in the URL.

### Hook Events (separate from OTEL)

Claude Code also sends hook events via curl for lifecycle tracking:

| Event | Purpose |
|-------|---------|
| `SessionStart` | New Claude session began |
| `Stop` | Task completed |
| `SubagentStop` | Subagent task completed |
| `PreToolUse` / `PostToolUse` | Tool execution lifecycle |
| `PermissionRequest` | User approval needed |

These are configured in `.claude/settings.json` by `axel-cli/crates/core/src/hooks/settings.rs`.

## 3. Axel macOS App (Consumer)

**File**: `Axel/Services/InboxService.swift`

### SSE Connection

The app connects to `http://localhost:{port}/inbox` and receives all events. Routing:
- `event_type == "otel_metrics"` -> `parseOTELMetrics()`
- `event_type == "otel_logs"` -> `parseOTELLogs()` (used by Codex)
- Other types -> hook event processing

### Parsing Flow (`parseOTELMetrics` -> `parseMetric`)

1. **Iterate** `resourceMetrics[].scopeMetrics[].metrics[]`
2. **For each metric**, extract `name` and `sum.dataPoints[]`
3. **For each data point**, extract:
   - `asDouble` (the cumulative value)
   - `session.id` and `type` from attributes
4. **Aggregate** data points by session ID (sums across models)
5. **Match metric suffix** (strips provider prefix): `token.usage`, `cost.usage`, etc.
6. **Register** pane/session mapping via `CostTracker.shared.registerSession()`
7. **Update** `TaskMetrics` with cumulative values (assigns, not adds)
8. **Record** to `CostTracker.shared.recordMetrics(forSession:...)`

### CostTracker (`Axel/Services/CostTracker.swift`)

**File**: `Axel/Services/CostTracker.swift`

The CostTracker is a singleton that manages per-terminal token tracking.

#### Key Components

- **`CostTracker`** (singleton): Maps `paneId -> TerminalCostTracker`, `sessionId -> paneId`
- **`TerminalCostTracker`**: Per-terminal state with cumulative totals and time series
- **`registerSession(paneId:sessionId:)`**: Maps session to pane; detects session resets

#### Delta Calculation

OTEL sends cumulative values, but the gauge needs deltas:

```
incoming cumulative = 5000 tokens
lastSessionCumulative = 4500 tokens
delta = max(0, 5000 - 4500) = 500 tokens
```

The delta is:
1. Added to `allTimeTotalTokens` (lifetime counter)
2. Appended to `timeSeries[]` (for activity gauges)
3. Stored as `lastSession*` for next delta calculation

#### Session Reset Detection

When `registerSession()` detects a new `sessionId` for a pane, it calls `resetSessionTracking()` which zeroes `lastSession*` values. This prevents the first OTEL batch of a new session from being treated as a massive delta.

Fallback: if `incomingTotal < lastSessionTotal / 3`, assume a session reset.

#### Gauge: `recentThroughput(forProvider:maxAge:)`

Returns the sum of the latest delta per terminal for a provider, if the data point is younger than `maxAge` (default 3s). This keeps the gauge steady between 2s OTEL batches and decays to 0 when work stops.

## Timing & Tuning

| Parameter | Value | Location |
|-----------|-------|----------|
| OTEL export interval | 2s | `claude.rs` (`OTEL_METRIC_EXPORT_INTERVAL`) |
| Gauge maxAge | 3s | `CostTracker.recentThroughput()` |
| Gauge tremble timer | 0.15s | `SpeedometerGauge` in `MacHelpers.swift` |
| Boot animation | 3.5s | `WorkspaceContentView.swift` |

The maxAge (3s) must exceed the OTEL export interval (2s) to prevent flickering. The 1s margin allows for network jitter.

## Debugging

Enable debug logging by looking for `[OTEL]` and `[CostTracker]` prefixes in Xcode console:

```
[OTEL] parseMetric: name=claude_code.token.usage dataPoints.count=2 paneId=abc-123
[OTEL]   dataPoint: session=a1b2c3d4 type=input value=1234.0
[OTEL]   dataPoint: session=a1b2c3d4 type=output value=567.0
[OTEL] -> CostTracker.recordMetrics(session=a1b2c3d4, in:1234 out:567 ...)
```

### Common Issues

1. **Gauge stuck at 0**: Check `recentThroughput` maxAge vs OTEL interval. If maxAge < interval, data expires before refresh.
2. **Tokens double-counting**: Each OTEL batch has cumulative values. If delta calculation is wrong (e.g., treating as additive), tokens will grow too fast.
3. **Session reset causes spike**: First batch of new session has cumulative=0, but if `lastSession*` isn't reset, delta = 0. If `lastSession*` IS reset but previous session's cumulative was high, that's fine — delta starts from 0.
4. **No pane mapping**: If hook events don't arrive before OTEL metrics, the legacy route can't resolve session->pane. The pane_id-in-URL approach avoids this.

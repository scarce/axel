# OTEL Telemetry Architecture

How Axel captures and displays real-time token usage from AI coding assistants.

## Documents

- [Claude Code](./claude.md) — OTEL Metrics pipeline (cumulative counters, 2s interval)
- [Codex](./codex.md) — OTEL Logs pipeline (per-response tokens, tracing bridge)

## Architecture Summary

```
AI Assistant ──► Axel CLI Server ──► SSE /inbox ──► macOS App
  (OTEL export)    (Rust, axum)       (broadcast)    (Swift)
```

Each provider has a different OTEL strategy:

| Provider | OTEL Signal | Token Model | Config |
|----------|------------|-------------|--------|
| Claude Code | Metrics | Cumulative per session | Env vars |
| Codex | Logs | Per-response | CLI `-c` args |

The macOS app normalizes both into a common `CostTracker` that tracks per-terminal deltas for gauge display.

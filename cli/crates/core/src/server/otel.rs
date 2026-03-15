//! OTEL parsing module.
//!
//! Parses raw OTEL metric/log payloads and produces flat `MetricsUpdate` structs
//! that the server broadcasts as `metrics_update` SSE events. This moves the
//! heavy OTEL-walking logic out of the Swift app and into the CLI.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Flat metrics update emitted as an SSE event.
/// Values are **cumulative** (delta computation stays in Swift's CostTracker).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsUpdate {
    pub session_id: String,
    pub provider: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost_usd: f64,
    pub lines_added: i64,
    pub lines_removed: i64,
    pub active_time_seconds: f64,
}

/// Intermediate per-session accumulator used during OTEL metric parsing.
#[derive(Default)]
struct SessionAccumulator {
    input_tokens: i64,
    output_tokens: i64,
    cache_read_tokens: i64,
    cache_creation_tokens: i64,
    cost_usd: f64,
    lines_added: i64,
    lines_removed: i64,
    active_time_seconds: f64,
}

/// Parse a Claude OTEL metrics payload into one `MetricsUpdate` per session.
///
/// Walks `resourceMetrics[].scopeMetrics[].metrics[]` and aggregates data points
/// by session_id (summing across models).
pub fn parse_claude_otel_metrics(payload: &serde_json::Value) -> Vec<MetricsUpdate> {
    let Some(resource_metrics) = payload.get("resourceMetrics").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    let mut sessions: HashMap<String, SessionAccumulator> = HashMap::new();

    for rm in resource_metrics {
        let Some(scope_metrics) = rm.get("scopeMetrics").and_then(|v| v.as_array()) else {
            continue;
        };

        for sm in scope_metrics {
            let Some(metrics) = sm.get("metrics").and_then(|v| v.as_array()) else {
                continue;
            };

            for metric in metrics {
                let Some(name) = metric.get("name").and_then(|v| v.as_str()) else {
                    continue;
                };

                let Some(sum) = metric.get("sum") else {
                    continue;
                };
                let Some(data_points) = sum.get("dataPoints").and_then(|v| v.as_array()) else {
                    continue;
                };

                // Extract suffix after first dot: "claude_code.token.usage" → "token.usage"
                let metric_suffix = name.find('.').map(|i| &name[i + 1..]).unwrap_or(name);

                for dp in data_points {
                    let Some(value) = dp.get("asDouble").and_then(|v| v.as_f64()) else {
                        continue;
                    };

                    let Some(attributes) = dp.get("attributes").and_then(|v| v.as_array()) else {
                        continue;
                    };

                    let mut session_id: Option<String> = None;
                    let mut attr_type: Option<String> = None;

                    for attr in attributes {
                        let Some(key) = attr.get("key").and_then(|k| k.as_str()) else {
                            continue;
                        };
                        let string_val = attr
                            .get("value")
                            .and_then(|v| v.get("stringValue"))
                            .and_then(|v| v.as_str());

                        match key {
                            "session.id" => session_id = string_val.map(|s| s.to_string()),
                            "type" => attr_type = string_val.map(|s| s.to_string()),
                            _ => {}
                        }
                    }

                    let Some(ref sid) = session_id else {
                        continue;
                    };

                    let acc = sessions.entry(sid.clone()).or_default();
                    let int_val = value as i64;

                    match metric_suffix {
                        "token.usage" => match attr_type.as_deref() {
                            Some("input") => acc.input_tokens += int_val,
                            Some("output") => acc.output_tokens += int_val,
                            Some("cacheRead") => acc.cache_read_tokens += int_val,
                            Some("cacheCreation") => acc.cache_creation_tokens += int_val,
                            _ => {}
                        },
                        "cost.usage" => acc.cost_usd += value,
                        "lines_of_code.count" => match attr_type.as_deref() {
                            Some("added") => acc.lines_added += int_val,
                            Some("removed") => acc.lines_removed += int_val,
                            _ => {}
                        },
                        "active_time.total" => acc.active_time_seconds += value,
                        _ => {}
                    }
                }
            }
        }
    }

    sessions
        .into_iter()
        .map(|(session_id, acc)| MetricsUpdate {
            session_id,
            provider: "claude".to_string(),
            input_tokens: acc.input_tokens,
            output_tokens: acc.output_tokens,
            cache_read_tokens: acc.cache_read_tokens,
            cache_creation_tokens: acc.cache_creation_tokens,
            cost_usd: acc.cost_usd,
            lines_added: acc.lines_added,
            lines_removed: acc.lines_removed,
            active_time_seconds: acc.active_time_seconds,
        })
        .collect()
}

/// Parse a Codex OTEL logs payload into `MetricsUpdate` structs.
///
/// Walks `resourceLogs[].scopeLogs[].logRecords[]` and extracts token counts
/// from body/attributes.
pub fn parse_codex_otel_logs(payload: &serde_json::Value) -> Vec<MetricsUpdate> {
    let Some(resource_logs) = payload.get("resourceLogs").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    let mut updates = Vec::new();

    for rl in resource_logs {
        let Some(scope_logs) = rl.get("scopeLogs").and_then(|v| v.as_array()) else {
            continue;
        };

        for sl in scope_logs {
            let Some(log_records) = sl.get("logRecords").and_then(|v| v.as_array()) else {
                continue;
            };

            for record in log_records {
                if let Some(update) = parse_codex_log_record(record) {
                    updates.push(update);
                }
            }
        }
    }

    updates
}

/// Parse a single Codex OTEL log record.
fn parse_codex_log_record(record: &serde_json::Value) -> Option<MetricsUpdate> {
    let attributes = record
        .get("attributes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Build attribute map (skip malformed attributes)
    let mut attr_map: HashMap<String, serde_json::Value> = HashMap::new();
    for attr in &attributes {
        let Some(key) = attr.get("key").and_then(|k| k.as_str()) else {
            continue;
        };
        let Some(value_dict) = attr.get("value") else {
            continue;
        };
        if let Some(sv) = value_dict.get("stringValue") {
            attr_map.insert(key.to_string(), sv.clone());
        } else if let Some(iv) = value_dict.get("intValue") {
            attr_map.insert(key.to_string(), iv.clone());
        } else if let Some(dv) = value_dict.get("doubleValue") {
            attr_map.insert(key.to_string(), dv.clone());
        }
    }

    // Also try to parse body as JSON (Codex embeds usage data there)
    if let Some(body_str) = record
        .get("body")
        .and_then(|b| b.get("stringValue"))
        .and_then(|v| v.as_str())
    {
        if body_str.starts_with('{') {
            if let Ok(body_json) = serde_json::from_str::<serde_json::Value>(body_str) {
                // OpenAI response format: { "usage": { "prompt_tokens": N, ... } }
                if let Some(usage) = body_json.get("usage") {
                    if let Some(pt) = usage.get("prompt_tokens") {
                        attr_map
                            .entry("prompt_tokens".to_string())
                            .or_insert(pt.clone());
                    }
                    if let Some(ct) = usage.get("completion_tokens") {
                        attr_map
                            .entry("completion_tokens".to_string())
                            .or_insert(ct.clone());
                    }
                }
                // Also check { "tokens": { "input": N, "output": N } }
                if let Some(tokens) = body_json.get("tokens") {
                    if let Some(inp) = tokens.get("input") {
                        attr_map
                            .entry("input_tokens".to_string())
                            .or_insert(inp.clone());
                    }
                    if let Some(outp) = tokens.get("output") {
                        attr_map
                            .entry("output_tokens".to_string())
                            .or_insert(outp.clone());
                    }
                }
            }
        }
    }

    // Extract session ID (multiple naming conventions)
    let session_id = extract_string(&attr_map, "session.id")
        .or_else(|| extract_string(&attr_map, "session_id"))
        .or_else(|| extract_string(&attr_map, "conversation.id"))
        .or_else(|| extract_string(&attr_map, "gen_ai.session.id"));

    // Extract token counts (multiple naming conventions)
    let input_tokens = extract_int(&attr_map, "input_token_count")
        .or_else(|| extract_int(&attr_map, "input_tokens"))
        .or_else(|| extract_int(&attr_map, "prompt_tokens"))
        .or_else(|| extract_int(&attr_map, "gen_ai.usage.prompt_tokens"))
        .or_else(|| extract_int(&attr_map, "gen_ai.usage.input_tokens"))
        .or_else(|| extract_int(&attr_map, "llm.usage.prompt_tokens"))
        .unwrap_or(0);

    let output_tokens = extract_int(&attr_map, "output_token_count")
        .or_else(|| extract_int(&attr_map, "output_tokens"))
        .or_else(|| extract_int(&attr_map, "completion_tokens"))
        .or_else(|| extract_int(&attr_map, "gen_ai.usage.completion_tokens"))
        .or_else(|| extract_int(&attr_map, "gen_ai.usage.output_tokens"))
        .or_else(|| extract_int(&attr_map, "llm.usage.completion_tokens"))
        .unwrap_or(0);

    let cost_usd = extract_f64(&attr_map, "cost")
        .or_else(|| extract_f64(&attr_map, "cost_usd"))
        .or_else(|| extract_f64(&attr_map, "gen_ai.usage.cost"))
        .unwrap_or(0.0);

    // Only return if we found token data
    if input_tokens == 0 && output_tokens == 0 {
        return None;
    }

    Some(MetricsUpdate {
        session_id: session_id.unwrap_or_default(),
        provider: "codex".to_string(),
        input_tokens,
        output_tokens,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_usd,
        lines_added: 0,
        lines_removed: 0,
        active_time_seconds: 0.0,
    })
}

/// Extract a string from the attribute map.
fn extract_string(map: &HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
    map.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Extract an integer from the attribute map (handles both number and string values).
fn extract_int(map: &HashMap<String, serde_json::Value>, key: &str) -> Option<i64> {
    map.get(key).and_then(|v| {
        v.as_i64()
            .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
    })
}

/// Extract a float from the attribute map.
fn extract_f64(map: &HashMap<String, serde_json::Value>, key: &str) -> Option<f64> {
    map.get(key).and_then(|v| {
        v.as_f64()
            .or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
    })
}

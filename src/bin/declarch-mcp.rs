use serde::Deserialize;
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines().map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<RpcRequest>(&line) {
            Ok(req) => handle_request(req),
            Err(e) => error_response(None, &format!("Invalid JSON request: {}", e)),
        };

        let _ = writeln!(stdout, "{}", response);
        let _ = stdout.flush();
    }
}

fn handle_request(req: RpcRequest) -> Value {
    match req.method.as_str() {
        "tools/list" => tools_list_response(req.id),
        "tools/call" => tools_call_response(req.id, req.params),
        other => error_response(req.id, &format!("Unknown method: {}", other)),
    }
}

fn tools_list_response(id: Option<Value>) -> Value {
    let tools = vec![
        json!({
            "name": "declarch_info",
            "description": "Run `declarch info` in machine-output mode (v1).",
            "inputSchema": {"type":"object","properties":{"query":{"type":"string"}}}
        }),
        json!({
            "name": "declarch_list",
            "description": "Run `declarch info --list` in machine-output mode (v1).",
            "inputSchema": {"type":"object","properties":{"orphans":{"type":"boolean"},"synced":{"type":"boolean"},"backend":{"type":"string"}}}
        }),
        json!({
            "name": "declarch_lint",
            "description": "Run `declarch lint` in machine-output mode (v1).",
            "inputSchema": {"type":"object","properties":{"mode":{"type":"string"},"strict":{"type":"boolean"}}}
        }),
        json!({
            "name": "declarch_search",
            "description": "Run `declarch search` in machine-output mode (v1).",
            "inputSchema": {"type":"object","required":["query"],"properties":{"query":{"type":"string"},"limit":{"type":"integer"},"local":{"type":"boolean"},"installed_only":{"type":"boolean"},"available_only":{"type":"boolean"},"backends":{"oneOf":[{"type":"string"},{"type":"array","items":{"type":"string"}}]}}}
        }),
        json!({
            "name": "declarch_sync_preview",
            "description": "Run `declarch sync preview` in machine-output mode (v1).",
            "inputSchema": {"type":"object","properties":{"target":{"type":"string"},"profile":{"type":"string"},"host":{"type":"string"},"modules":{"type":"array","items":{"type":"string"}}}}
        }),
    ];

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "tools": tools }
    })
}

fn tools_call_response(id: Option<Value>, params: Value) -> Value {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let arguments = params
        .get("arguments")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let args = match build_declarch_args(&name, &arguments) {
        Ok(args) => args,
        Err(msg) => return error_response(id, &msg),
    };

    match run_declarch_json(&args) {
        Ok(payload) => json!({
            "jsonrpc":"2.0",
            "id": id,
            "result": {
                "content": [
                    {
                        "type":"text",
                        "text": serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
                    }
                ],
                "isError": false
            }
        }),
        Err(msg) => json!({
            "jsonrpc":"2.0",
            "id": id,
            "result": {
                "content": [{"type":"text","text": msg}],
                "isError": true
            }
        }),
    }
}

fn build_declarch_args(
    name: &str,
    arguments: &serde_json::Map<String, Value>,
) -> Result<Vec<String>, String> {
    let mut args: Vec<String> = Vec::new();

    match name {
        "declarch_info" => {
            args.push("info".into());
            if let Some(q) = arguments.get("query").and_then(Value::as_str) {
                args.push(q.to_string());
            }
        }
        "declarch_list" => {
            args.push("info".into());
            args.push("--list".into());
            if arguments
                .get("orphans")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                args.push("--orphans".into());
            }
            if arguments
                .get("synced")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                args.push("--synced".into());
            }
            if let Some(backend) = arguments.get("backend").and_then(Value::as_str) {
                args.push("--backend".into());
                args.push(backend.to_string());
            }
        }
        "declarch_lint" => {
            args.push("lint".into());
            if let Some(mode) = arguments.get("mode").and_then(Value::as_str) {
                args.push("--mode".into());
                args.push(mode.to_string());
            }
            if arguments
                .get("strict")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                args.push("--strict".into());
            }
        }
        "declarch_search" => {
            args.push("search".into());
            let Some(query) = arguments.get("query").and_then(Value::as_str) else {
                return Err("declarch_search requires arguments.query".to_string());
            };
            args.push(query.to_string());

            if let Some(limit) = arguments.get("limit").and_then(Value::as_i64) {
                args.push("--limit".into());
                args.push(limit.to_string());
            }
            if arguments
                .get("local")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                args.push("--local".into());
            }
            if arguments
                .get("installed_only")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                args.push("--installed-only".into());
            }
            if arguments
                .get("available_only")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                args.push("--available-only".into());
            }
            match arguments.get("backends") {
                Some(Value::String(b)) => {
                    args.push("--backends".into());
                    args.push(b.clone());
                }
                Some(Value::Array(arr)) => {
                    let list: Vec<String> = arr
                        .iter()
                        .filter_map(Value::as_str)
                        .map(ToString::to_string)
                        .collect();
                    if !list.is_empty() {
                        args.push("--backends".into());
                        args.push(list.join(","));
                    }
                }
                _ => {}
            }
        }
        "declarch_sync_preview" => {
            args.push("sync".into());
            args.push("preview".into());
            if let Some(target) = arguments.get("target").and_then(Value::as_str) {
                args.push("--target".into());
                args.push(target.to_string());
            }
            if let Some(profile) = arguments.get("profile").and_then(Value::as_str) {
                args.push("--profile".into());
                args.push(profile.to_string());
            }
            if let Some(host) = arguments.get("host").and_then(Value::as_str) {
                args.push("--host".into());
                args.push(host.to_string());
            }
            if let Some(modules) = arguments.get("modules").and_then(Value::as_array) {
                let list: Vec<String> = modules
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect();
                if !list.is_empty() {
                    args.push("--modules".into());
                    args.extend(list);
                }
            }
        }
        _ => return Err(format!("Unknown tool name: {}", name)),
    }

    args.push("--format".into());
    args.push("json".into());
    args.push("--output-version".into());
    args.push("v1".into());
    Ok(args)
}

fn run_declarch_json(args: &[String]) -> Result<Value, String> {
    let bin = std::env::var("DECLARCH_BIN").unwrap_or_else(|_| "declarch".to_string());

    let output = Command::new(&bin)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute '{}': {}", bin, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "declarch command failed (status: {:?})\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            stdout,
            stderr
        ));
    }

    let stdout = String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8: {}", e))?;
    serde_json::from_str::<Value>(&stdout)
        .map_err(|e| format!("Failed to parse declarch JSON output: {}\n{}", e, stdout))
}

fn error_response(id: Option<Value>, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": -32000,
            "message": message
        }
    })
}

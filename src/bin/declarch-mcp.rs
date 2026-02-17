use declarch::config::loader;
use declarch::project_identity;
use declarch::utils::paths;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;
use std::io::{self, BufRead, Write};
use std::process::Command;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum McpMode {
    ReadOnly,
    WriteEnabled,
}

#[derive(Debug, Clone)]
struct McpRuntimePolicy {
    mode: McpMode,
    allow_tools: HashSet<String>,
}

impl Default for McpRuntimePolicy {
    fn default() -> Self {
        Self {
            mode: McpMode::ReadOnly,
            allow_tools: HashSet::new(),
        }
    }
}

impl McpRuntimePolicy {
    fn load() -> Self {
        let Ok(config_path) = paths::config_file() else {
            return Self::default();
        };
        if !config_path.exists() {
            return Self::default();
        }

        let Ok(cfg) = loader::load_root_config(&config_path) else {
            return Self::default();
        };

        let mode = match cfg.mcp_mode().to_lowercase().as_str() {
            "write-enabled" => McpMode::WriteEnabled,
            _ => McpMode::ReadOnly,
        };

        let allow_tools = cfg
            .mcp
            .as_ref()
            .map(|m| m.allow_tools.iter().cloned().collect())
            .unwrap_or_default();

        Self { mode, allow_tools }
    }

    fn allows_write_tool(&self, tool_name: &str) -> Result<(), String> {
        if self.mode != McpMode::WriteEnabled {
            return Err(
                "MCP write actions are disabled by config (mcp.mode is read-only).".to_string(),
            );
        }
        if !self.allow_tools.contains(tool_name) {
            return Err(format!(
                "MCP write action '{}' is not allowed. Add it to mcp.allow_tools.",
                tool_name
            ));
        }
        Ok(())
    }
}

fn policy() -> &'static McpRuntimePolicy {
    static POLICY: OnceLock<McpRuntimePolicy> = OnceLock::new();
    POLICY.get_or_init(McpRuntimePolicy::load)
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

fn stable_tool_name(suffix: &str) -> String {
    format!("{}_{}", project_identity::STABLE_PROJECT_ID, suffix)
}

fn normalize_tool_name(name: &str) -> String {
    let stable = project_identity::STABLE_PROJECT_ID;
    let binary = project_identity::BINARY_NAME;
    if stable != binary {
        let binary_prefix = format!("{}_", binary);
        if let Some(suffix) = name.strip_prefix(&binary_prefix) {
            return format!("{}_{}", stable, suffix);
        }
    }
    name.to_string()
}

fn tools_list_response(id: Option<Value>) -> Value {
    let info_tool = stable_tool_name("info");
    let list_tool = stable_tool_name("list");
    let lint_tool = stable_tool_name("lint");
    let search_tool = stable_tool_name("search");
    let sync_dry_tool = stable_tool_name("sync_dry_run");
    let sync_apply_tool = stable_tool_name("sync_apply");

    let mut tools = vec![
        json!({
            "name": info_tool,
            "description": format!("Run `{}` in machine-output mode (v1).", project_identity::cli_with("info")),
            "inputSchema": {"type":"object","properties":{"query":{"type":"string"}}}
        }),
        json!({
            "name": list_tool,
            "description": format!("Run `{}` in machine-output mode (v1).", project_identity::cli_with("info --list")),
            "inputSchema": {"type":"object","properties":{"scope":{"type":"string","enum":["all","orphans","synced","unmanaged"]},"backend":{"type":"string"}}}
        }),
        json!({
            "name": lint_tool,
            "description": format!("Run `{}` in machine-output mode (v1).", project_identity::cli_with("lint")),
            "inputSchema": {"type":"object","properties":{"mode":{"type":"string"},"strict":{"type":"boolean"}}}
        }),
        json!({
            "name": search_tool,
            "description": format!("Run `{}` in machine-output mode (v1).", project_identity::cli_with("search")),
            "inputSchema": {"type":"object","required":["query"],"properties":{"query":{"type":"string"},"limit":{"type":"integer"},"local":{"type":"boolean"},"installed_only":{"type":"boolean"},"available_only":{"type":"boolean"},"backends":{"oneOf":[{"type":"string"},{"type":"array","items":{"type":"string"}}]}}}
        }),
        json!({
            "name": sync_dry_tool,
            "description": format!("Run `{}` in machine-output mode (v1).", project_identity::cli_with("--dry-run sync")),
            "inputSchema": {"type":"object","properties":{"target":{"type":"string"},"profile":{"type":"string"},"host":{"type":"string"},"modules":{"type":"array","items":{"type":"string"}}}}
        }),
    ];

    if policy().allows_write_tool(&sync_apply_tool).is_ok() {
        tools.push(json!({
            "name": sync_apply_tool,
            "description": format!("Run `{}` (apply). Requires mcp config allow + {}=1 + confirm=\"APPLY_SYNC\".", project_identity::cli_with("sync"), project_identity::env_key("MCP_ALLOW_APPLY")),
            "inputSchema": {"type":"object","required":["confirm"],"properties":{"confirm":{"type":"string"},"target":{"type":"string"},"profile":{"type":"string"},"host":{"type":"string"},"modules":{"type":"array","items":{"type":"string"}}}}
        }));
    }

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "tools": tools }
    })
}

fn tools_call_response(id: Option<Value>, params: Value) -> Value {
    let raw_name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let name = normalize_tool_name(&raw_name);
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
    let info_tool = stable_tool_name("info");
    let list_tool = stable_tool_name("list");
    let lint_tool = stable_tool_name("lint");
    let search_tool = stable_tool_name("search");
    let sync_dry_tool = stable_tool_name("sync_dry_run");
    let sync_apply_tool = stable_tool_name("sync_apply");
    let mut args: Vec<String> = Vec::new();

    match name {
        n if n == info_tool => {
            args.push("info".into());
            if let Some(q) = arguments.get("query").and_then(Value::as_str) {
                args.push(q.to_string());
            }
        }
        n if n == list_tool => {
            args.push("info".into());
            args.push("--list".into());
            if let Some(scope) = arguments.get("scope").and_then(Value::as_str) {
                args.push("--scope".into());
                args.push(scope.to_string());
            }
            if let Some(backend) = arguments.get("backend").and_then(Value::as_str) {
                args.push("--backend".into());
                args.push(backend.to_string());
            }
        }
        n if n == lint_tool => {
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
        n if n == search_tool => {
            args.push("search".into());
            let Some(query) = arguments.get("query").and_then(Value::as_str) else {
                return Err(format!("{} requires arguments.query", search_tool));
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
        n if n == sync_dry_tool => {
            args.push("--dry-run".into());
            args.push("sync".into());
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
        n if n == sync_apply_tool => {
            policy().allows_write_tool(&sync_apply_tool)?;
            enforce_apply_safety(arguments)?;
            args.push("sync".into());
            args.push("--yes".into());

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
            return Ok(args);
        }
        _ => return Err(format!("Unknown tool name: {}", name)),
    }

    args.push("--format".into());
    args.push("json".into());
    args.push("--output-version".into());
    args.push("v1".into());
    Ok(args)
}

fn enforce_apply_safety(arguments: &serde_json::Map<String, Value>) -> Result<(), String> {
    let sync_apply_tool = stable_tool_name("sync_apply");
    let allow_apply_key = project_identity::env_key("MCP_ALLOW_APPLY");
    let allow_apply = project_identity::env_get("MCP_ALLOW_APPLY").unwrap_or_default();
    if allow_apply != "1" {
        return Err(format!(
            "Apply is disabled. Set {}=1 to allow {}.",
            allow_apply_key, sync_apply_tool
        ));
    }
    let confirm = arguments
        .get("confirm")
        .and_then(Value::as_str)
        .unwrap_or("");
    if confirm != "APPLY_SYNC" {
        return Err(format!(
            "Invalid confirm token. Pass confirm=\"APPLY_SYNC\" to run {}.",
            sync_apply_tool
        ));
    }
    Ok(())
}

fn run_declarch_json(args: &[String]) -> Result<Value, String> {
    let bin = project_identity::env_get("BIN")
        .unwrap_or_else(|| project_identity::BINARY_NAME.to_string());

    let output = Command::new(&bin)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute '{}': {}", bin, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "{} command failed (status: {:?})\nstdout:\n{}\nstderr:\n{}",
            project_identity::BINARY_NAME,
            output.status.code(),
            stdout,
            stderr
        ));
    }

    let stdout = String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8: {}", e))?;
    serde_json::from_str::<Value>(&stdout).map_err(|e| {
        format!(
            "Failed to parse {} JSON output: {}\n{}",
            project_identity::BINARY_NAME,
            e,
            stdout
        )
    })
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

//! An ADK agent's `tools`: built-in tools, Python functions and MCP toolsets. `AgentTool`s are
//! drawn as connections instead (see `flow`).
use crate::convert::{Builder, Fidelity};
use crate::model::{Agent, EnvValue, McpServer, Restrictions, SecretRef, Settings, Tool, ToolRule};
use serde_json::{Map, Value};

/// `McpToolset` arguments the document says in its own terms.
const MCP_KEYS: &[&str] = &[
    "stdio_server_params",
    "stdio_connection_params",
    "sse_connection_params",
    "streamable_http_connection_params",
    "tool_filter",
    "tool_name_prefix",
];

/// Adds the agent's tools, and returns the `args` of the tools that are not MCP toolsets, by tool
/// id, for the extension.
pub(super) fn add(b: &mut Builder, node: &super::load::Node, agent: &mut Agent) -> Map<String, Value> {
    let mut kept = Map::new();
    let Some(list) = node.config.get("tools").and_then(Value::as_array) else { return kept };
    let (mut allow, mut filtered) = (Vec::new(), false);
    for (i, t) in list.iter().enumerate() {
        let at = format!("{}/tools/{i}", node.at);
        let name = t.get("name").and_then(Value::as_str).unwrap_or_default();
        let args = t.get("args").and_then(Value::as_object);
        let (mut tool, only) = match name {
            "AgentTool" => continue,
            "McpToolset" => mcp(b, &at, args.cloned().unwrap_or_default()),
            _ => (builtin(b, &at, name, args.is_some()), None),
        };
        tool.id = free_tool_id(agent, &tool.id);
        if let (Some(args), None) = (args, &tool.mcp) {
            kept.insert(tool.id.clone(), Value::Object(args.clone()));
        }
        filtered |= only.is_some();
        match only {
            Some(names) => allow.extend(names.iter().map(|n| format!("{}/{n}", tool.id))),
            None => allow.push(tool.id.clone()),
        }
        agent.tools.push(tool);
    }
    if filtered {
        let settings = agent.settings.get_or_insert_with(Settings::default);
        let restrictions = settings.restrictions.get_or_insert_with(Restrictions::default);
        restrictions.tools = Some(ToolRule { allow: Some(allow), deny: Vec::new() });
    }
    kept
}

/// A tool ADK finds by name in `google.adk.tools`, or a Python function by its import path.
fn builtin(b: &mut Builder, at: &str, name: &str, has_args: bool) -> Tool {
    if name.contains('.') {
        let why = format!("a Python function: only an ADK runner that can import `{name}` can call it");
        b.note(Fidelity::Approximated, at, why);
    }
    if has_args {
        let why = format!(
            "the tool's `args` are kept under extensions `{}` (`tool_args`), which other tools ignore",
            super::NS
        );
        b.note(Fidelity::Unsupported, format!("{at}/args"), why);
    }
    let id = name.rsplit('.').next().unwrap_or(name);
    Tool { id: id.to_string(), builtin: Some(format!("google-adk:{name}")), ..Default::default() }
}

/// An MCP toolset, and the tools `tool_filter` limits it to.
fn mcp(b: &mut Builder, at: &str, args: Map<String, Value>) -> (Tool, Option<Vec<String>>) {
    for key in args.keys().filter(|k| !MCP_KEYS.contains(&k.as_str())) {
        b.note(Fidelity::Unsupported, format!("{at}/args/{key}"), format!("`{key}` was not imported"));
    }
    let prefix = args.get("tool_name_prefix").and_then(Value::as_str).map(|p| p.trim_end_matches('_'));
    let id = prefix.filter(|p| !p.is_empty()).unwrap_or("mcp").to_string();
    let only = args
        .get("tool_filter")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).map(str::to_string).collect());
    (Tool { id, mcp: Some(server(b, at, &args)), ..Default::default() }, only)
}

/// How to reach the server: a command (stdio) or a URL (SSE, streamable HTTP).
fn server(b: &mut Builder, at: &str, args: &Map<String, Value>) -> McpServer {
    let stdio = args.get("stdio_server_params").or_else(|| args.get("stdio_connection_params")?.get("server_params"));
    if let Some(p) = stdio {
        return stdio_server(b, at, p);
    }
    let remote = ["sse_connection_params", "streamable_http_connection_params"].iter().find_map(|k| args.get(*k));
    let url = remote.and_then(|r| r.get("url")).and_then(Value::as_str).map(str::to_string);
    if url.is_none() {
        let why = "no connection the importer understands; the server is not reachable from the document";
        b.note(Fidelity::Unsupported, format!("{at}/args"), why);
    }
    McpServer { url, ..Default::default() }
}

/// A server started by a command. Environment values become secret references by name.
fn stdio_server(b: &mut Builder, at: &str, p: &Value) -> McpServer {
    let env: Vec<String> =
        p.get("env").and_then(Value::as_object).into_iter().flat_map(|e| e.keys().cloned()).collect();
    if !env.is_empty() {
        let names = env.join(", ");
        let why = format!("environment values became secret references ({names}): documents never hold secret values");
        b.note(Fidelity::Approximated, format!("{at}/args"), why);
    }
    let args = p.get("args").and_then(Value::as_array).into_iter().flatten();
    McpServer {
        command: p.get("command").and_then(Value::as_str).map(str::to_string),
        args: args.filter_map(Value::as_str).map(str::to_string).collect(),
        env: env.into_iter().map(|k| (k.clone(), EnvValue::Secret(SecretRef { secret: k }))).collect(),
        url: None,
    }
}

fn free_tool_id(agent: &Agent, wanted: &str) -> String {
    let taken = |id: &str| agent.tools.iter().any(|t| t.id == id);
    let mut id = wanted.to_string();
    let mut n = 1;
    while taken(&id) {
        n += 1;
        id = format!("{wanted}-{n}");
    }
    id
}

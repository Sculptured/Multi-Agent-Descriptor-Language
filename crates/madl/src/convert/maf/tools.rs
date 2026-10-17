//! A Prompt agent's `tools`: MCP servers, functions the host registers, and the providers' hosted
//! tools (`web_search`, `file_search`, `code_interpreter`).
use super::load::File;
use crate::convert::{Builder, Fidelity, sanitize};
use crate::model::{Agent, Approvals, McpServer, Restrictions, Settings, Tool, ToolRule};
use serde_json::{Map, Value};

/// Hosted tools the provider runs.
const HOSTED: &[&str] = &["web_search", "file_search", "code_interpreter"];

/// What a tool's entry says beyond its kind, name and description.
const SAID: &[&str] = &["kind", "name", "description"];

/// Adds the agent's tools, and returns what the document can't say about them, by tool id.
pub(super) fn add(b: &mut Builder, file: &File, agent: &mut Agent) -> Map<String, Value> {
    let mut kept = Map::new();
    let Some(list) = file.value.get("tools").and_then(Value::as_array) else { return kept };
    let mut rules = Rules::default();
    for (i, t) in list.iter().enumerate() {
        let at = format!("{}#/tools/{i}", file.rel);
        let kind = t.get("kind").and_then(Value::as_str).unwrap_or_default().to_ascii_lowercase();
        let name = t.get("name").and_then(Value::as_str).filter(|n| !n.is_empty()).unwrap_or(&kind);
        let id = free_tool_id(agent, &sanitize(name));
        let description = t.get("description").and_then(Value::as_str).map(str::to_string);
        let mut tool = Tool { id: id.clone(), description, ..Default::default() };
        let (rest, only) = if kind == "mcp" {
            let (server, only) = mcp(b, t, &at, &id, &mut rules);
            tool.mcp = Some(server);
            (rest_of(t, &["url", "allowedTools", "approvalMode"]), only)
        } else {
            tool.builtin = Some(builtin(b, &kind, name, &at));
            (rest_of(t, &[]), None)
        };
        if !rest.is_empty() {
            let why = format!("the tool's other settings are kept under extensions `{}` (`tool_args`)", super::NS);
            b.note(Fidelity::Unsupported, &at, why);
            kept.insert(id.clone(), super::agent::scrub(b, Value::Object(rest), &at));
        }
        rules.filtered |= only.is_some();
        rules.allow.extend(only.unwrap_or_else(|| vec![id]));
        agent.tools.push(tool);
    }
    rules.apply(agent);
    kept
}

/// What `allowedTools` and `approvalMode` add up to across the agent's tools.
#[derive(Default)]
struct Rules {
    allow: Vec<String>,
    filtered: bool,
    approve: Vec<String>,
}

impl Rules {
    fn apply(self, agent: &mut Agent) {
        if !self.filtered && self.approve.is_empty() {
            return;
        }
        let settings = agent.settings.get_or_insert_with(Settings::default);
        let restrictions = settings.restrictions.get_or_insert_with(Restrictions::default);
        if self.filtered {
            restrictions.tools = Some(ToolRule { allow: Some(self.allow), deny: Vec::new() });
        }
        if !self.approve.is_empty() {
            restrictions.approvals = Some(Approvals { required_for: self.approve });
        }
    }
}

/// A function the host registers, or a tool the provider hosts, by name.
fn builtin(b: &mut Builder, kind: &str, name: &str, at: &str) -> String {
    if HOSTED.contains(&kind) {
        return format!("microsoft-agent-framework:{kind}");
    }
    let why = match kind {
        "function" => format!("a function: only a runner where the host registers `{name}` can call it"),
        "openapi" => "an OpenAPI tool: its specification is kept under extensions".to_string(),
        _ => format!("`{kind}` is a tool kind the importer does not know; it is named, not described"),
    };
    let fidelity = if kind == "function" || kind == "openapi" { Fidelity::Approximated } else { Fidelity::Unsupported };
    b.note(fidelity, at, why);
    format!("microsoft-agent-framework:{name}")
}

/// An MCP server by URL, and the tools `allowedTools` limits the agent to (`<id>/<tool>`).
/// `approvalMode` says which calls need a person's approval.
fn mcp(b: &mut Builder, t: &Value, at: &str, id: &str, rules: &mut Rules) -> (McpServer, Option<Vec<String>>) {
    let url = t.get("url").and_then(Value::as_str).map(str::to_string);
    if url.as_deref().is_none_or(|u| u.starts_with('=')) {
        let why = "the server's URL is not written in the file; the server is not reachable from the document";
        b.note(Fidelity::Unsupported, format!("{at}/url"), why);
    }
    let names = |v: Option<&Value>| -> Vec<String> {
        v.and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(|n| format!("{id}/{n}"))
            .collect()
    };
    let only = Some(names(t.get("allowedTools"))).filter(|n| !n.is_empty());
    let mode = t.get("approvalMode");
    let kind = mode.and_then(|m| m.as_str().or_else(|| m.get("kind")?.as_str())).map(str::to_ascii_lowercase);
    match kind.as_deref() {
        Some("always") => rules.approve.push(id.to_string()),
        Some("specify") => rules.approve.extend(names(mode.and_then(|m| m.get("alwaysRequireApprovalTools")))),
        _ => {}
    }
    (McpServer { url: url.filter(|u| !u.starts_with('=')), ..Default::default() }, only)
}

/// The entry's keys beyond those the document says, and beyond `also`.
fn rest_of(t: &Value, also: &[&str]) -> Map<String, Value> {
    let Some(m) = t.as_object() else { return Map::new() };
    m.iter()
        .filter(|(k, _)| !SAID.contains(&k.as_str()) && !also.contains(&k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
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

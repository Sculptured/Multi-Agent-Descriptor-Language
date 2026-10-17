//! ADK agents as MADL agents, and ADK workflow agents (sequential, parallel, loop) as groups.
use super::NS;
use super::load::{Class, Node};
use crate::convert::{Builder, Fidelity};
use crate::model::{Agent, Group, ModelRef, Output, Preferences, Settings};
use serde_json::{Map, Value};

/// Keys of an `LlmAgent` config that the document says in its own terms.
const LLM_KEYS: &[&str] = &[
    "agent_class",
    "name",
    "description",
    "instruction",
    "model",
    "sub_agents",
    "tools",
    "output_key",
    "generate_content_config",
    "include_contents",
    "disallow_transfer_to_parent",
    "disallow_transfer_to_peers",
];

/// Keys of a workflow agent config that the document says in its own terms.
const GROUP_KEYS: &[&str] = &["agent_class", "name", "description", "sub_agents", "max_iterations"];

/// Adds the agent for `node`, in `group`, and returns its id. `inherited` is the model of the
/// nearest enclosing `LlmAgent`, which ADK uses when an agent names none.
pub(super) fn agent(b: &mut Builder, node: &Node, group: Option<&str>, inherited: Option<&str>) -> String {
    let id = b.id_for(node.name());
    let mut agent = Agent {
        id: id.clone(),
        name: (id != node.name()).then(|| node.name().to_string()),
        description: node.text("description").map(str::to_string),
        instructions: node.text("instruction").map(str::to_string),
        groups: group.map(str::to_string).into_iter().collect(),
        ..Default::default()
    };
    if let Class::Code(path) = &node.class {
        defined_in_python(b, node, path, agent);
        return id;
    }
    agent.model = model(b, node, inherited).map(|m| ModelRef::Id(qualified(&m)));
    agent.outputs = node.text("output_key").map(output).into_iter().collect();
    agent.settings = options(node);
    let tool_args = super::tools::add(b, node, &mut agent);
    let ext = extension(b, node, tool_args);
    if !ext.is_empty() {
        agent.extensions.insert(NS.into(), Value::Object(ext));
    }
    b.doc.agents.push(agent);
    id
}

/// What only ADK uses: the config's other keys, tool arguments, and a custom class.
fn extension(b: &mut Builder, node: &Node, tool_args: Map<String, Value>) -> Map<String, Value> {
    let mut ext = leftovers(b, node, LLM_KEYS);
    if !tool_args.is_empty() {
        ext.insert("tool_args".into(), Value::Object(tool_args));
    }
    if let Class::Custom(class) = &node.class {
        let why = format!("`{class}` is a custom agent class: how it runs is defined in Python");
        b.note(Fidelity::Unsupported, format!("{}/agent_class", node.at), why);
        ext.insert("agent_class".into(), class.clone().into());
    }
    ext
}

/// An agent ADK builds from Python: its place in the team, and where to find it.
fn defined_in_python(b: &mut Builder, node: &Node, path: &str, mut agent: Agent) {
    let why = format!("`{path}` is an agent defined in Python; only its place in the team was imported");
    b.note(Fidelity::Unsupported, &node.at, why);
    agent.extensions.insert(NS.into(), serde_json::json!({ "code": path }));
    b.doc.agents.push(agent);
}

/// The model the agent runs: its own, else the inherited one.
pub(super) fn model(b: &mut Builder, node: &Node, inherited: Option<&str>) -> Option<String> {
    let own = node.text("model");
    if own.is_none() && inherited.is_none() && node.class == Class::Llm {
        b.note(
            Fidelity::Approximated,
            &node.at,
            "names no model, and no enclosing agent does: ADK uses its default model",
        );
    }
    own.or(inherited).map(str::to_string)
}

/// `gemini-2.5-flash` → `google/gemini-2.5-flash`; a name with a provider keeps it.
fn qualified(model: &str) -> String {
    if model.contains('/') { model.to_string() } else { format!("google/{model}") }
}

fn output(key: &str) -> Output {
    let description = format!("The agent's final answer, kept in session state as `{key}`");
    Output { id: key.to_string(), description: Some(description), ..Default::default() }
}

/// `generate_content_config` and `include_contents` are ADK's own options.
fn options(node: &Node) -> Option<Settings> {
    let options: std::collections::BTreeMap<String, Value> = ["generate_content_config", "include_contents"]
        .iter()
        .filter_map(|k| Some((k.to_string(), node.config.get(*k)?.clone())))
        .collect();
    let preferences = Preferences { options, ..Default::default() };
    (!preferences.options.is_empty()).then(|| Settings { preferences: Some(preferences), ..Default::default() })
}

/// Adds the group for a sequential, parallel or loop agent, inside `parent`, and returns its id.
pub(super) fn group(b: &mut Builder, node: &Node, parent: Option<&str>) -> String {
    let id = b.id_for(node.name());
    let mut ext = leftovers(b, node, GROUP_KEYS);
    let class = node.config.get("agent_class").cloned().unwrap_or_else(|| "LlmAgent".into());
    ext.insert("agent_class".into(), class);
    if let Some(n) = node.config.get("max_iterations") {
        ext.insert("max_iterations".into(), n.clone());
    }
    let mut extensions = crate::model::Extensions::new();
    extensions.insert(NS.into(), Value::Object(ext));
    b.doc.groups.push(Group {
        id: id.clone(),
        name: (id != node.name()).then(|| node.name().to_string()),
        purpose: node.text("description").map(str::to_string),
        parent: parent.map(str::to_string),
        settings: None,
        extensions,
    });
    id
}

/// The config's keys that have no MADL counterpart: kept for ADK under `extensions`, with a note.
fn leftovers(b: &mut Builder, node: &Node, known: &[&str]) -> Map<String, Value> {
    let mut out = Map::new();
    for (key, value) in node.config.iter().filter(|(k, _)| !known.contains(&k.as_str())) {
        let why =
            format!("`{key}` has no MADL counterpart; it is kept under extensions `{NS}`, which other tools ignore");
        b.note(Fidelity::Unsupported, format!("{}/{key}", node.at), why);
        out.insert(key.clone(), value.clone());
    }
    out
}

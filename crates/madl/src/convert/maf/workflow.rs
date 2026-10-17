//! A declarative workflow's agents: one MADL agent for each agent the workflow invokes by name, and
//! one for each invocation whose agent is chosen at run time. How work passes between them is in
//! `links`; what the agents share is in `channels`.
use super::NS;
use super::flow::{Graph, Node, expression};
use super::load::File;
use crate::convert::{Builder, Fidelity, sanitize};
use crate::model::{Agent, Output};
use serde_json::Value;
use std::collections::HashMap;

/// Keys of a workflow file that the document says in its own terms.
const WORKFLOW_KEYS: &[&str] = &["kind", "name", "description", "trigger", "actions"];

/// Keys of an `InvokeAzureAgent` action that the document says in its own terms.
const INVOKE_KEYS: &[&str] = &["kind", "id", "displayName", "conversationId", "agent", "agentName", "input", "output"];

pub(super) fn is_agent(n: &Node) -> bool {
    n.kind == "invokeazureagent"
}

/// Adds the workflow's agents, the handoffs between them and what they share, and returns the
/// agents the workflow starts with.
pub(super) fn import(b: &mut Builder, w: &File, defs: &[File]) -> Result<Vec<String>, String> {
    let (actions, at) = actions(b, w);
    let g = Graph::new(&actions, &at);
    for (n, target) in &g.lost {
        let why = format!("goes to `{target}`, which no action has; drawn as the end of the workflow");
        b.note(Fidelity::Unsupported, &g.nodes[*n].at, why);
    }
    let order = reachable(&g);
    if order.is_empty() {
        return Err("invokes no agent, so there is nothing to describe".into());
    }
    let mut names = HashMap::new();
    let ids: Vec<String> = order.iter().map(|&n| agent_for(b, &g.nodes[n], defs, &mut names)).collect();
    super::links::handoffs(b, &g, &order, &ids);
    super::channels::add(b, &g, &order, &ids);
    super::passed::note(b, &g, &at);
    unused(b, defs, &names);
    b.doc.extensions.insert(NS.into(), serde_json::json!({ "workflow": w.value }));
    Ok(super::links::entries(b, &g, &order, &ids))
}

/// The actions, under `trigger` or at the top as the Python loader also accepts, and where they are.
fn actions(b: &mut Builder, w: &File) -> (Vec<Value>, String) {
    for (key, _) in w.value.iter().filter(|(k, _)| !WORKFLOW_KEYS.contains(&k.as_str())) {
        let why = match key.as_str() {
            "inputs" => "the workflow's named inputs were drawn as one input given at start".to_string(),
            _ => format!("`{key}` has no MADL counterpart; it is kept with the workflow under extensions `{NS}`"),
        };
        let fidelity = if key == "inputs" { Fidelity::Approximated } else { Fidelity::Unsupported };
        b.note(fidelity, format!("{}#/{key}", w.rel), why);
    }
    let list = |v: Option<&Value>| v.and_then(Value::as_array).cloned().unwrap_or_default();
    let Some(trigger) = w.value.get("trigger") else {
        return (list(w.value.get("actions")), format!("{}#/actions", w.rel));
    };
    let kind = trigger.get("kind").and_then(Value::as_str).unwrap_or_default();
    if !kind.eq_ignore_ascii_case("OnConversationStart") {
        let why = format!("`{kind}` starts the workflow; drawn as an input given at start");
        b.note(Fidelity::Approximated, format!("{}#/trigger/kind", w.rel), why);
    }
    (list(trigger.get("actions")), format!("{}#/trigger/actions", w.rel))
}

/// The agent invocations work can reach from the start, in the order work reaches them.
fn reachable(g: &Graph) -> Vec<usize> {
    let mut order: Vec<usize> =
        g.following(std::slice::from_ref(&g.start), is_agent).into_iter().map(|f| f.0).collect();
    let mut i = 0;
    while i < order.len() {
        for (n, _) in g.following(&g.nodes[order[i]].next, is_agent) {
            if !order.contains(&n) {
                order.push(n);
            }
        }
        i += 1;
    }
    order
}

/// The agent an invocation runs: added the first time its name comes up, and kept in `names`.
/// Returns its id.
fn agent_for(b: &mut Builder, n: &Node, defs: &[File], names: &mut HashMap<String, String>) -> String {
    let name = n.text(&["agent", "name"]).or(n.text(&["agentName"]));
    let id = match name.filter(|n| !n.starts_with('=')) {
        Some(name) => match names.get(name) {
            Some(id) => id.clone(),
            None => {
                let id = named(b, n, name, defs);
                names.insert(name.to_string(), id.clone());
                id
            }
        },
        None => chosen_at_run_time(b, n, name),
    };
    invocation(b, n, &id);
    id
}

/// An agent the workflow names: from the Prompt agent of that name, else only its name.
fn named(b: &mut Builder, n: &Node, name: &str, defs: &[File]) -> String {
    let id = b.id_for(name);
    let agent = match defs.iter().find(|d| d.text("name") == Some(name)) {
        Some(def) => super::agent::agent(b, &id, def),
        None => {
            let why = format!("`{name}` is defined outside these files; only its place in the workflow was imported");
            b.note(Fidelity::Approximated, format!("{}/agent", n.at), why);
            Agent { id: id.clone(), name: (id != name).then(|| name.to_string()), ..Default::default() }
        }
    };
    b.doc.agents.push(agent);
    id
}

/// An invocation whose agent an expression picks while the workflow runs.
fn chosen_at_run_time(b: &mut Builder, n: &Node, expr: Option<&str>) -> String {
    let expr = expression(expr.map(|e| Value::String(e.into())).as_ref());
    let id = b.id_for(n.text(&["id"]).unwrap_or("chosen-agent"));
    let why = format!("the agent is chosen while the workflow runs (`{expr}`); drawn as one agent standing for it");
    b.note(Fidelity::Approximated, format!("{}/agent", n.at), why);
    b.doc.agents.push(Agent {
        id: id.clone(),
        name: Some(n.text(&["displayName"]).unwrap_or("Agent chosen at run time").to_string()),
        description: Some(format!("Whichever agent `{expr}` names when the workflow reaches it")),
        ..Default::default()
    });
    id
}

/// What one invocation adds to its agent: the variables it keeps the answer in, as outputs, and
/// the action's id under `extensions`.
fn invocation(b: &mut Builder, n: &Node, id: &str) {
    for key in n.def.keys().filter(|k| !INVOKE_KEYS.contains(&k.as_str())) {
        let why = format!("`{key}` has no MADL counterpart; it is kept with the workflow under extensions `{NS}`");
        b.note(Fidelity::Unsupported, format!("{}/{key}", n.at), why);
    }
    if let Some(until) = n.get(&["input", "externalLoop", "when"]) {
        let why = format!("the agent talks with the user while `{}` holds; drawn as one turn", expression(Some(until)));
        b.note(Fidelity::Approximated, format!("{}/input/externalLoop", n.at), why);
    }
    let Some(agent) = b.doc.agents.iter_mut().find(|a| a.id == id) else { return };
    for var in ["messages", "responseObject"].iter().filter_map(|k| n.text(&["output", k])) {
        let out = sanitize(var.rsplit('.').next().unwrap_or(var));
        if !agent.outputs.iter().any(|o| o.id == out) {
            let description = Some(format!("The agent's answer, kept in the workflow variable `{var}`"));
            agent.outputs.push(Output { id: out, description, ..Default::default() });
        }
    }
    let ext = agent.extensions.entry(NS.to_string()).or_insert_with(|| serde_json::json!({}));
    if let (Some(map), Some(action)) = (ext.as_object_mut(), n.text(&["id"])) {
        let actions = map.entry("actions").or_insert_with(|| Value::Array(Vec::new()));
        actions.as_array_mut().into_iter().for_each(|a| a.push(action.into()));
    }
}

/// Prompt agents the workflow never invokes, and second definitions of a name.
fn unused(b: &mut Builder, defs: &[File], names: &HashMap<String, String>) {
    for (i, def) in defs.iter().enumerate() {
        let name = def.text("name").unwrap_or_default();
        let why = if let Some(first) = defs[..i].iter().find(|d| d.text("name") == Some(name)) {
            format!("defines `{name}` again; the definition in {} was used", first.rel)
        } else if !names.contains_key(name) {
            format!("`{name}` is not invoked by the workflow; it was not imported")
        } else {
            continue;
        };
        b.note(Fidelity::Unsupported, format!("{}#", def.rel), why);
    }
}

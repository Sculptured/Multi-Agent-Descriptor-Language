//! How work moves through an ADK tree, as connections.
//!
//! - A sequential agent hands work from each sub-agent to the next.
//! - A loop agent does the same, then hands it back to the first, at most `max_iterations` times.
//! - A parallel agent gives the work to every sub-agent.
//! - An `LlmAgent` with sub-agents may transfer the conversation to each (`delegate`), and each
//!   `LlmAgent` sub-agent may transfer it back (`report`), as in MADL's supervisor pattern, and on
//!   to its peers (`handoff`), unless `disallow_transfer_to_parent` or `disallow_transfer_to_peers`
//!   says otherwise.
//! - An `AgentTool` is a call that waits for the answer (`delegate`, `mode: sync`).
use super::agent;
use super::load::{Class, Node};
use crate::convert::{Builder, Fidelity};
use crate::model::{ConnectionKind, SyncMode};

/// The agents where work enters a part of the tree, and the agents it leaves from.
#[derive(Clone, Debug, Default)]
pub(super) struct Ends {
    pub entries: Vec<String>,
    pub exits: Vec<String>,
}

/// Adds `node` and everything under it; `model` is the nearest enclosing `LlmAgent`'s model.
pub(super) fn place(b: &mut Builder, node: &Node, group: Option<&str>, model: Option<&str>) -> Ends {
    match node.class {
        Class::Sequential | Class::Parallel | Class::Loop => workflow(b, node, group, model),
        _ => single(b, node, group, model),
    }
}

fn single(b: &mut Builder, node: &Node, group: Option<&str>, inherited: Option<&str>) -> Ends {
    let id = agent::agent(b, node, group, inherited);
    let model = node.text("model").or(inherited).map(str::to_string);
    for called in &node.agent_tools {
        let why = "an `AgentTool`: the caller waits for the answer, drawn as `delegate` with `mode: sync`; the called agent never takes over the conversation";
        b.note(Fidelity::Approximated, called.at.trim_end_matches("/args/agent"), why);
        let ends = place(b, called, None, model.as_deref());
        for to in &ends.entries {
            if let Some(c) = b.link(&id, to, ConnectionKind::Delegate) {
                c.mode = Some(SyncMode::Sync);
            }
        }
    }
    transfers(b, node, &id, group, model.as_deref());
    Ends { entries: vec![id.clone()], exits: vec![id] }
}

/// An agent's sub-agents, and the transfers between it and them.
fn transfers(b: &mut Builder, node: &Node, id: &str, group: Option<&str>, model: Option<&str>) {
    let parts: Vec<(&Node, Ends)> = node.children.iter().map(|c| (c, place(b, c, group, model))).collect();
    if node.class != Class::Llm && !parts.is_empty() {
        b.note(
            Fidelity::Approximated,
            &node.at,
            "how this agent runs its sub-agents is code; drawn as delegating to each",
        );
    }
    for (_, ends) in &parts {
        for to in &ends.entries {
            b.link(id, to, ConnectionKind::Delegate);
        }
    }
    if node.class == Class::Llm {
        returns(b, node, id, &parts);
    }
}

/// Transfers from an `LlmAgent`'s sub-agents: back to it, and on to their peers.
fn returns(b: &mut Builder, node: &Node, id: &str, parts: &[(&Node, Ends)]) {
    let back = parts.iter().any(|(c, _)| c.class == Class::Llm && !c.flag("disallow_transfer_to_parent"));
    if back {
        let why = "ADK hands the whole conversation back when a sub-agent transfers to its parent; drawn as `report`";
        b.note(Fidelity::Approximated, format!("{}/sub_agents", node.at), why);
    }
    for (child, ends) in parts.iter().filter(|(c, _)| c.class == Class::Llm) {
        let from = &ends.entries[0];
        if !child.flag("disallow_transfer_to_parent") {
            b.link(from, id, ConnectionKind::Report);
        }
        if child.flag("disallow_transfer_to_peers") {
            continue;
        }
        for (_, peer) in parts.iter().filter(|(p, _)| !std::ptr::eq(*p, *child)) {
            for to in &peer.entries {
                b.link(from, to, ConnectionKind::Handoff);
            }
        }
    }
}

/// A sequential, parallel or loop agent: a group, with its sub-agents inside.
fn workflow(b: &mut Builder, node: &Node, group: Option<&str>, model: Option<&str>) -> Ends {
    let gid = agent::group(b, node, group);
    let parts: Vec<(&Node, Ends)> = node.children.iter().map(|c| (c, place(b, c, Some(&gid), model))).collect();
    if parts.is_empty() {
        b.note(Fidelity::Unsupported, &node.at, "has no sub-agents, so no work passes through it");
        return Ends::default();
    }
    if node.class == Class::Parallel {
        let all = |f: fn(&Ends) -> &Vec<String>| parts.iter().flat_map(|(_, e)| f(e).clone()).collect();
        return Ends { entries: all(|e| &e.entries), exits: all(|e| &e.exits) };
    }
    for pair in parts.windows(2) {
        hand_on(b, &pair[0], &pair[1].1);
    }
    if node.class == Class::Loop {
        back(b, node, &parts);
    }
    Ends { entries: parts[0].1.entries.clone(), exits: parts[parts.len() - 1].1.exits.clone() }
}

/// Work passes from one part of a sequence to the next.
fn hand_on(b: &mut Builder, (node, from): &(&Node, Ends), to: &Ends) {
    if node.class == Class::Parallel && from.exits.len() > 1 {
        let why = "ADK waits for every branch before going on; in MADL the next agent receives each branch's handoff as it comes";
        b.note(Fidelity::Approximated, &node.at, why);
    }
    for f in &from.exits {
        for t in &to.entries {
            b.link(f, t, ConnectionKind::Handoff);
        }
    }
}

/// The handoff from a loop's last part back to its first, with `max_iterations` as its round limit.
fn back(b: &mut Builder, node: &Node, parts: &[(&Node, Ends)]) {
    let rounds = node.config.get("max_iterations").and_then(serde_json::Value::as_u64);
    let rounds = rounds.map(|n| u32::try_from(n).unwrap_or(u32::MAX));
    let (first, last) = (&parts[0].1, &parts[parts.len() - 1].1);
    let mut linked = false;
    for f in &last.exits {
        for t in &first.entries {
            if let Some(c) = b.link(f, t, ConnectionKind::Handoff) {
                c.max_rounds = rounds;
                linked = true;
            }
        }
    }
    let why = match (linked, rounds) {
        (false, _) => {
            "a loop around one agent: MADL has no connection from an agent to itself, so the repeats are not drawn"
        }
        (true, None) => {
            "ADK repeats until an agent escalates (for example with `exit_loop`); with no `max_iterations`, the runner's own round limit applies"
        }
        (true, Some(_)) => {
            "ADK also leaves the loop early when an agent escalates (for example with `exit_loop`); MADL has only the round limit"
        }
    };
    b.note(if linked { Fidelity::Approximated } else { Fidelity::Unsupported }, &node.at, why);
}

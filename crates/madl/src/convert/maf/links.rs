//! How work passes between a workflow's agents: a `handoff` from each agent to every agent the
//! workflow can run next, past the actions that are not agents. A branch's Power Fx condition is
//! not MADL's expression language, so it is kept under the connection's `extensions`.
use super::NS;
use super::cond;
use super::flow::Graph;
use super::workflow::is_agent;
use crate::convert::{Builder, Fidelity};
use crate::model::ConnectionKind;

/// Where work may go next, from agent to agent.
struct Next {
    from: String,
    to: String,
    /// The first invocation of `from` that leads to `to`.
    at: String,
    /// The ways there, from every invocation of `from`.
    ways: Vec<Vec<String>>,
}

/// Adds a handoff for every way from one invocation to the next. `ids[i]` is the agent of
/// invocation `order[i]`.
pub(super) fn handoffs(b: &mut Builder, g: &Graph, order: &[usize], ids: &[String]) {
    let mut next: Vec<Next> = Vec::new();
    let mut again = Vec::new();
    for (i, &n) in order.iter().enumerate() {
        for (m, ways) in g.following(&g.nodes[n].next, is_agent) {
            let to = &ids[order.iter().position(|&o| o == m).expect("every reachable invocation is in order")];
            if *to == ids[i] {
                again.push((g.nodes[n].at.clone(), to.clone()));
                continue;
            }
            match next.iter_mut().find(|p| p.from == ids[i] && p.to == *to) {
                Some(p) => p.ways.extend(ways),
                None => next.push(Next { from: ids[i].clone(), to: to.clone(), at: g.nodes[n].at.clone(), ways }),
            }
        }
    }
    for (at, id) in again {
        let why = format!("runs `{id}` again right after itself; MADL has no connection from an agent to itself");
        b.note(Fidelity::Unsupported, at, why);
    }
    for Next { from, to, at, ways } in next {
        let Some(c) = b.link(&from, &to, ConnectionKind::Handoff) else { continue };
        let Some(when) = cond::render(&cond::simplify(ways)) else { continue };
        c.extensions.insert(NS.into(), serde_json::json!({ "when": when }));
        let why = format!(
            "work goes from `{from}` to `{to}` only when {when} (Power Fx); the condition is kept under extensions `{NS}` and the handoff is drawn without it"
        );
        b.note(Fidelity::Approximated, at, why);
    }
}

/// The agents the workflow starts with. When a condition decides which, the note says so.
pub(super) fn entries(b: &mut Builder, g: &Graph, order: &[usize], ids: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (m, ways) in g.following(std::slice::from_ref(&g.start), is_agent) {
        let id = &ids[order.iter().position(|&o| o == m).expect("every reachable invocation is in order")];
        if !out.contains(id) {
            out.push(id.clone());
        }
        if let Some(when) = cond::render(&ways) {
            let why =
                format!("the workflow starts with `{id}` only when {when} (Power Fx); the input goes to it always");
            b.note(Fidelity::Approximated, &g.nodes[m].at, why);
        }
    }
    out
}

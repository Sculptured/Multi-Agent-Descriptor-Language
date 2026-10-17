//! A Mermaid flowchart of a document: groups as boxes, agents, steps and inputs as nodes, one
//! arrow style per connection kind.
use crate::expand::kind_name;
use crate::model::{ConnectionKind, Direction, Document, StepKind};
use std::collections::BTreeMap;
use std::fmt::Write;

/// The flowchart, for an expanded document (patterns already turned into connections).
pub fn flowchart(doc: &Document) -> String {
    let mut out = String::from("flowchart LR\n");
    for input in &doc.inputs {
        let given = if input.text.trim().is_empty() {
            format!("{} (given at start)", input.id)
        } else {
            first_line(&input.text)
        };
        let text = label(&format!("Input: {given}"));
        let _ = writeln!(out, "  {}([\"{text}\"])", node(&input.id));
    }
    let rt = runtimes(doc);
    for g in (0..doc.groups.len()).filter(|&i| doc.groups[i].parent.is_none()) {
        group(doc, &rt, g, 1, &mut out);
    }
    for a in doc.agents.iter().filter(|a| a.groups.is_empty()) {
        agent_line(doc, &rt, &a.id, 1, &mut out);
    }
    steps(doc, &mut out);
    arrows(doc, &mut out);
    bans(doc, &mut out);
    out
}

fn arrows(doc: &Document, out: &mut String) {
    for input in &doc.inputs {
        let targets: Vec<String> = match input.to.is_empty() {
            true => crate::graph::starting_agents(doc).into_iter().collect(),
            false => input.to.clone(),
        };
        for t in targets {
            let _ = writeln!(out, "  {} -.-> {}", node(&input.id), node(&t));
        }
    }
    for c in &doc.connections {
        let arrow = arrow(c.kind, c.direction == Some(Direction::Both));
        let (from, to) = (node(&c.from), node(&c.to));
        let _ = writeln!(out, "  {from} {arrow}|{}| {to}", kind_name(c.kind));
    }
}

fn steps(doc: &Document, out: &mut String) {
    for s in &doc.steps {
        let name = label(s.name.as_deref().unwrap_or(&s.id));
        let shape = match s.kind {
            StepKind::Check => format!("{{{{\"{name}\"}}}}"),
            StepKind::Human => format!("[/\"{name}\"/]"),
            StepKind::Merge | StepKind::FanIn => format!("[[\"{name}\"]]"),
            StepKind::FanOut => format!("[\\\"{name}\"\\]"),
        };
        let _ = writeln!(out, "  {}{shape}", node(&s.id));
    }
}

/// Each agent's effective runtime (its own, a group's, or the defaults').
fn runtimes(doc: &Document) -> BTreeMap<String, String> {
    let resolved = crate::resolve::resolve(doc);
    resolved.agents.into_iter().filter_map(|a| Some((a.id, a.settings.preferences.runtime?))).collect()
}

fn group(doc: &Document, rt: &BTreeMap<String, String>, g: usize, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    let gr = &doc.groups[g];
    let _ = writeln!(out, "{pad}subgraph {}[\"{}\"]", node(&gr.id), label(gr.name.as_deref().unwrap_or(&gr.id)));
    for (i, child) in doc.groups.iter().enumerate() {
        if child.parent.as_deref() == Some(&gr.id) && i != g {
            group(doc, rt, i, depth + 1, out);
        }
    }
    for a in doc.agents.iter().filter(|a| a.groups.first() == Some(&gr.id)) {
        agent_line(doc, rt, &a.id, depth + 1, out);
    }
    let _ = writeln!(out, "{pad}end");
}

fn agent_line(doc: &Document, rt: &BTreeMap<String, String>, id: &str, depth: usize, out: &mut String) {
    let Some(a) = doc.agents.iter().find(|a| a.id == id) else {
        return;
    };
    let name = label(a.name.as_deref().unwrap_or(&a.id));
    let runtime = rt.get(&a.id).map(|r| format!("<br/><small>{}</small>", label(r))).unwrap_or_default();
    let open = if rt.get(&a.id).map(String::as_str) == Some("human") { "((" } else { "[" };
    let close = if open == "((" { "))" } else { "]" };
    let _ = writeln!(out, "{}{}{open}\"{name}{runtime}\"{close}", "  ".repeat(depth), node(&a.id));
}

fn arrow(kind: ConnectionKind, both: bool) -> &'static str {
    match (kind, both) {
        (ConnectionKind::Watch, false) => "-.->",
        (ConnectionKind::Watch, true) => "<-.->",
        (ConnectionKind::Report | ConnectionKind::Ask, false) => "-->",
        (ConnectionKind::Report | ConnectionKind::Ask, true) => "<-->",
        (_, false) => "==>",
        (_, true) => "<==>",
    }
}

fn bans(doc: &Document, out: &mut String) {
    let rules = doc.communication.iter().flat_map(|c| c.deny.iter());
    for r in rules {
        let (a, b) = match (&r.from, &r.to, &r.between) {
            (Some(f), Some(t), _) => (f.clone(), t.clone()),
            (_, _, Some([x, y])) => (x.clone(), y.clone()),
            _ => continue,
        };
        if a != "*" && b != "*" {
            let _ = writeln!(out, "  %% banned: {a} → {b}");
        }
    }
}

/// Mermaid node ids: letters, digits and `_` only.
fn node(id: &str) -> String {
    let safe: String = id.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    format!("n_{safe}")
}

fn label(text: &str) -> String {
    text.replace('"', "#quot;")
}

fn first_line(text: &str) -> String {
    let line = text.lines().next().unwrap_or("").trim();
    if line.chars().count() > 40 {
        format!("{}…", line.chars().take(40).collect::<String>())
    } else {
        line.to_string()
    }
}

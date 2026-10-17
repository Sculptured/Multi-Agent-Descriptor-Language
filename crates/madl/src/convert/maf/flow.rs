//! A workflow's actions as a graph: where control can go after each action. Branches (`If`,
//! `ConditionGroup`) carry their Power Fx condition, `GotoAction` jumps, `Foreach` loops back, and
//! the `End…` and `Cancel…` actions stop.
use super::cond;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashMap};

/// How many different condition lists one search follows through the same action; past that, the
/// action is not followed again. Real workflows stay far below it.
const WAYS_PER_ACTION: usize = 32;

/// Actions after which nothing runs.
const ENDS: &[&str] =
    &["endworkflow", "endconversation", "enddialog", "cancelworkflow", "canceldialog", "cancelalldialogs"];

/// One action, and the edges control may take after it.
pub(super) struct Node {
    /// The file and the JSON Pointer of the action in it.
    pub at: String,
    /// `kind`, lowercased: kinds ignore case.
    pub kind: String,
    pub def: Map<String, Value>,
    pub next: Vec<Edge>,
}

impl Node {
    /// The value at `path` (`["agent", "name"]`).
    pub fn get(&self, path: &[&str]) -> Option<&Value> {
        let (first, rest) = path.split_first()?;
        rest.iter().try_fold(self.def.get(*first)?, |v, k| v.get(*k))
    }

    pub fn text(&self, path: &[&str]) -> Option<&str> {
        self.get(path).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty())
    }
}

/// Control passes to `to` (`None`: the workflow ends), when every condition in `when` holds.
#[derive(Clone, Debug)]
pub(super) struct Edge {
    pub to: Option<usize>,
    pub when: Vec<String>,
}

pub(super) struct Graph {
    pub nodes: Vec<Node>,
    /// The edge from the start of the workflow to its first action.
    pub start: Edge,
    /// `GotoAction`s whose target no action has, by node, with the target.
    pub lost: Vec<(usize, String)>,
}

/// While building: the loops we are in, innermost last, as (the `Foreach`, what follows it).
struct Build {
    nodes: Vec<Node>,
    gotos: Vec<(usize, String)>,
    loops: Vec<(usize, Option<usize>)>,
}

impl Graph {
    /// `actions` sits at `at` (`file#/trigger/actions`).
    pub fn new(actions: &[Value], at: &str) -> Graph {
        let mut b = Build { nodes: Vec::new(), gotos: Vec::new(), loops: Vec::new() };
        let first = b.block(actions, at, None);
        let ids: BTreeMap<String, usize> =
            b.nodes.iter().enumerate().filter_map(|(i, n)| Some((n.def.get("id")?.as_str()?.to_string(), i))).collect();
        let mut lost = Vec::new();
        for (n, target) in b.gotos {
            match ids.get(&target) {
                Some(&t) => b.nodes[n].next = vec![Edge { to: Some(t), when: Vec::new() }],
                None => lost.push((n, target)),
            }
        }
        Graph { nodes: b.nodes, start: Edge { to: first, when: Vec::new() }, lost }
    }

    /// The nodes `is_stop` picks that control reaches from `edges` without passing another such
    /// node, in the order found, each with the ways to it (see [`cond::simplify`]).
    pub fn following(&self, edges: &[Edge], is_stop: impl Fn(&Node) -> bool) -> Vec<(usize, Vec<Vec<String>>)> {
        let mut found: Vec<(usize, Vec<Vec<String>>)> = Vec::new();
        let mut seen: HashMap<usize, Vec<Vec<String>>> = HashMap::new();
        let mut queue: std::collections::VecDeque<(&Edge, Vec<String>)> =
            edges.iter().map(|e| (e, Vec::new())).collect();
        while let Some((edge, mut way)) = queue.pop_front() {
            for c in &edge.when {
                if !way.contains(c) {
                    way.push(c.clone());
                }
            }
            let Some(n) = edge.to else { continue };
            if is_stop(&self.nodes[n]) {
                match found.iter_mut().find(|(f, _)| *f == n) {
                    Some((_, ways)) => ways.push(way),
                    None => found.push((n, vec![way])),
                }
                continue;
            }
            let mut key = way.clone();
            key.sort();
            let ways = seen.entry(n).or_default();
            if ways.len() < WAYS_PER_ACTION && !ways.contains(&key) {
                ways.push(key);
                queue.extend(self.nodes[n].next.iter().map(|e| (e, way.clone())));
            }
        }
        found.into_iter().map(|(n, ways)| (n, cond::simplify(ways))).collect()
    }
}

impl Build {
    /// Adds a list of actions that continues with `then`; returns its first node.
    fn block(&mut self, list: &[Value], at: &str, then: Option<usize>) -> Option<usize> {
        let mut next = then;
        for (i, a) in list.iter().enumerate().rev() {
            next = Some(self.action(a, &format!("{at}/{i}"), next));
        }
        next
    }

    fn action(&mut self, a: &Value, at: &str, next: Option<usize>) -> usize {
        let def = a.as_object().cloned().unwrap_or_default();
        let kind = def.get("kind").and_then(Value::as_str).unwrap_or_default().to_ascii_lowercase();
        let n = self.nodes.len();
        self.nodes.push(Node { at: at.to_string(), kind: kind.clone(), def: def.clone(), next: Vec::new() });
        let plain = |to| vec![Edge { to, when: Vec::new() }];
        let edges = match kind.as_str() {
            "conditiongroup" => self.branches(&def, at, next),
            "if" => self.branches(&if_as_group(&def), at, next),
            "foreach" => self.foreach(n, &def, at, next),
            "gotoaction" => {
                let target = def.get("actionId").or_else(|| def.get("target")).and_then(Value::as_str);
                self.gotos.push((n, target.unwrap_or_default().to_string()));
                Vec::new()
            }
            "breakloop" => plain(self.loops.last().map_or(next, |l| l.1)),
            "continueloop" => plain(self.loops.last().map_or(next, |l| Some(l.0))),
            k if ENDS.contains(&k) => Vec::new(),
            _ => plain(next),
        };
        self.nodes[n].next = edges;
        n
    }

    /// Each condition in turn, the first that holds wins; else `elseActions`, else what follows.
    fn branches(&mut self, def: &Map<String, Value>, at: &str, next: Option<usize>) -> Vec<Edge> {
        let (mut edges, mut not) = (Vec::new(), Vec::new());
        for (i, c) in list(def, "conditions").iter().enumerate() {
            let held = expression(c.get("condition"));
            let actions = c.get("actions").and_then(Value::as_array).cloned().unwrap_or_default();
            let first = self.block(&actions, &format!("{at}/conditions/{i}/actions"), next);
            edges.push(Edge { to: first, when: not.iter().cloned().chain([held.clone()]).collect() });
            not.push(cond::not(&held));
        }
        let first = self.block(&list(def, "elseActions"), &format!("{at}/elseActions"), next);
        edges.push(Edge { to: first, when: not });
        edges
    }

    /// The body runs for each item and comes back; then what follows. The body runs at all only
    /// when there are items.
    fn foreach(&mut self, n: usize, def: &Map<String, Value>, at: &str, next: Option<usize>) -> Vec<Edge> {
        self.loops.push((n, next));
        let body = self.block(&list(def, "actions"), &format!("{at}/actions"), Some(n));
        self.loops.pop();
        let items = expression(def.get("items").or_else(|| def.get("source")));
        let some = cond::not(&format!("IsEmpty({items})"));
        vec![Edge { to: body, when: vec![some] }, Edge { to: next, when: Vec::new() }]
    }
}

/// `If` (`condition`, `then` or `actions`, `else` or `elseActions`) as a one-condition group.
fn if_as_group(def: &Map<String, Value>) -> Map<String, Value> {
    let then = def.get("then").or_else(|| def.get("actions")).cloned().unwrap_or(Value::Array(Vec::new()));
    let otherwise = def.get("else").or_else(|| def.get("elseActions")).cloned().unwrap_or(Value::Array(Vec::new()));
    let condition = def.get("condition").cloned().unwrap_or(Value::Null);
    let mut out = Map::new();
    out.insert("conditions".into(), serde_json::json!([{ "condition": condition, "actions": then }]));
    out.insert("elseActions".into(), otherwise);
    out
}

fn list(def: &Map<String, Value>, key: &str) -> Vec<Value> {
    def.get(key).and_then(Value::as_array).cloned().unwrap_or_default()
}

/// A Power Fx expression without its leading `=`, on one line.
pub(super) fn expression(v: Option<&Value>) -> String {
    let text = match v {
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => "true".into(),
    };
    let text = text.trim().trim_start_matches('=');
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

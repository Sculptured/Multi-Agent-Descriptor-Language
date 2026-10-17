//! Notes for the workflow's actions that are not drawn: work passes over them from one agent to
//! the next. The whole workflow is kept under `extensions`.
use super::NS;
use super::flow::Graph;
use crate::convert::{Builder, Fidelity};
use std::collections::BTreeMap;

/// Actions that keep variables, send messages or manage conversations.
const BOOKKEEPING: &[&str] = &[
    "setvariable",
    "setvalue",
    "settextvariable",
    "resetvariable",
    "clearallvariables",
    "parsevalue",
    "edittable",
    "edittablev2",
    "sendactivity",
    "sendmessage",
    "createconversation",
    "addconversationmessage",
    "copyconversationmessages",
    "retrieveconversationmessage",
    "retrieveconversationmessages",
];

/// Actions that wait for the user.
const ASKS: &[&str] = &["question", "requestexternalinput", "requesthumaninput", "waitforhumaninput"];

/// Actions that call a tool from the workflow itself.
const CALLS: &[&str] = &["invokemcptool", "invokefunctiontool", "httprequestaction"];

/// Actions the connections already say: agents, branches, jumps, loops and ends.
const DRAWN: &[&str] = &[
    "invokeazureagent",
    "conditiongroup",
    "if",
    "foreach",
    "gotoaction",
    "breakloop",
    "continueloop",
    "endworkflow",
    "endconversation",
    "enddialog",
    "cancelworkflow",
    "canceldialog",
    "cancelalldialogs",
];

/// One note for each action that is not drawn, except bookkeeping, which gets one note in all at
/// `at`, the workflow's actions.
pub(super) fn note(b: &mut Builder, g: &Graph, at: &str) {
    let mut kept: BTreeMap<&str, usize> = BTreeMap::new();
    for n in &g.nodes {
        let kind = n.def.get("kind").and_then(|k| k.as_str()).unwrap_or_default();
        let k = n.kind.as_str();
        if DRAWN.contains(&k) {
            continue;
        } else if BOOKKEEPING.contains(&k) {
            *kept.entry(kind).or_default() += 1;
        } else if ASKS.contains(&k) {
            let why = "the workflow waits for the user's answer here; not drawn, so the agents before and after it are linked directly";
            b.note(Fidelity::Approximated, &n.at, why);
        } else if CALLS.contains(&k) {
            let why = format!("`{kind}` calls a tool from the workflow itself, outside any agent; not drawn");
            b.note(Fidelity::Unsupported, &n.at, why);
        } else {
            let why = format!("`{kind}` is not an action the importer knows; work passes over it");
            b.note(Fidelity::Unsupported, &n.at, why);
        }
    }
    if !kept.is_empty() {
        let list: Vec<String> = kept.iter().map(|(k, n)| format!("{k} ×{n}")).collect();
        let why = format!(
            "actions that keep variables, send messages or manage conversations ({}) are not drawn; the workflow is kept under extensions `{NS}`",
            list.join(", ")
        );
        b.note(Fidelity::Unsupported, at, why);
    }
}

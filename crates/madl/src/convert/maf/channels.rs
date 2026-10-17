//! What a workflow's agents share, as `shared-memory` channels: each conversation more than one
//! agent takes part in (`conversationId`), and the workflow's variables, which agents write
//! through `output` and read through `input`.
use super::flow::{Graph, expression};
use crate::convert::Builder;
use crate::model::{Access, Channel};
use std::collections::BTreeMap;

/// The conversation the workflow was started from.
const USER_CONVERSATION: &str = "System.ConversationId";

/// Adds the channels. `ids[i]` is the agent of invocation `order[i]`.
pub(super) fn add(b: &mut Builder, g: &Graph, order: &[usize], ids: &[String]) {
    let mut conversations: Vec<(String, Vec<String>)> = Vec::new();
    let mut variables: BTreeMap<String, Access> = BTreeMap::new();
    for (&n, id) in order.iter().zip(ids) {
        let node = &g.nodes[n];
        if let Some(c) = node.get(&["conversationId"]).map(|c| expression(Some(c))) {
            match conversations.iter_mut().find(|(k, _)| *k == c) {
                Some((_, members)) if !members.contains(id) => members.push(id.clone()),
                Some(_) => {}
                None => conversations.push((c, vec![id.clone()])),
            }
        }
        let writes = node.get(&["output", "messages"]).or(node.get(&["output", "responseObject"])).is_some();
        let reads = node.get(&["input"]).is_some_and(|i| i.to_string().contains("Local."));
        for access in [writes.then_some(Access::Write), reads.then_some(Access::Read)].into_iter().flatten() {
            grant(&mut variables, id, access);
        }
    }
    for (c, members) in conversations.into_iter().filter(|(_, m)| m.len() > 1) {
        conversation(b, &c, members);
    }
    if !variables.is_empty() {
        let id = b.id_for("workflow-variables");
        push(b, id, "Workflow variables".into(), variables);
    }
}

/// Adds `access` to what `id` already has.
fn grant(access_by: &mut BTreeMap<String, Access>, id: &str, access: Access) {
    let joined = match access_by.get(id) {
        Some(had) if *had != access => Access::ReadWrite,
        _ => access,
    };
    access_by.insert(id.to_string(), joined);
}

/// A conversation the `members` take part in, each reading and writing it.
fn conversation(b: &mut Builder, c: &str, members: Vec<String>) {
    let access = members.into_iter().map(|m| (m, Access::ReadWrite)).collect();
    let (id, name) = match c {
        USER_CONVERSATION => (b.id_for("conversation"), "The user's conversation".to_string()),
        _ => (b.id_for(c.rsplit('.').next().unwrap_or(c)), format!("Conversation `{c}`")),
    };
    push(b, id, name, access);
}

fn push(b: &mut Builder, id: String, name: String, access: BTreeMap<String, Access>) {
    let medium = "shared-memory".into();
    b.doc.channels.push(Channel { id, name: Some(name), medium, access, ..Default::default() });
}

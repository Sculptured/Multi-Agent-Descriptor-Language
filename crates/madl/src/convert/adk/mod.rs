//! Google Agent Development Kit (ADK) agent configs: `root_agent.yaml` and the files it names.
//!
//! The importer reads the YAML and never runs Python. Agents defined in Python, callbacks and
//! schemas are named in the notes and kept under the `dev.adk` extension.
mod agent;
mod flow;
mod load;
mod state;
mod tools;

use super::{Builder, ImportError, Imported};
use crate::model::{
    Communication, CommunicationMode, ConnectionKind, Direction, Document, Input, Preferences, Settings, System,
};
use std::path::Path;

/// The extension namespace for what only ADK uses.
pub const NS: &str = "dev.adk";

/// Imports the agent tree rooted at `path`: a config file, or a folder holding `root_agent.yaml`.
pub fn import(path: &Path) -> Result<Imported, ImportError> {
    let file = if path.is_dir() { path.join("root_agent.yaml") } else { path.to_path_buf() };
    let root = load::Loader::new(&file)?.file(&file)?;
    let mut b = Builder::new(document(&root));
    let ends = flow::place(&mut b, &root, None, None);
    if ends.entries.is_empty() {
        return Err(ImportError(format!("{}: no agent to import", file.display())));
    }
    let input = b.id_for("user-message");
    b.doc.inputs.push(Input { id: input, text: String::new(), to: ends.entries });
    state::channel(&mut b);
    join_both_ways(&mut b.doc);
    Ok(Imported { document: b.doc, notes: b.notes })
}

/// The document around the agents: ADK's runtime by default, and closed communication, since ADK
/// transfers work only along the tree.
fn document(root: &load::Node) -> Document {
    let mut b = Builder::new(Document::default());
    let preferences = Preferences { runtime: Some("google-adk".into()), ..Default::default() };
    Document {
        madl: "draft".into(),
        system: System {
            id: b.id_for(root.name()),
            name: root.name().to_string(),
            description: root.text("description").map(str::to_string),
            budget: None,
        },
        defaults: Some(Settings { preferences: Some(preferences), ..Default::default() }),
        communication: Some(Communication { mode: Some(CommunicationMode::Closed), deny: Vec::new() }),
        ..Default::default()
    }
}

/// Two plain handoffs between the same agents, one each way, become one with `direction: both`.
fn join_both_ways(doc: &mut Document) {
    let plain = |c: &crate::model::Connection| {
        c.kind == ConnectionKind::Handoff && c.direction.is_none() && c.max_rounds.is_none() && c.mode.is_none()
    };
    let mut i = 0;
    while i < doc.connections.len() {
        let c = &doc.connections[i];
        let back = (i + 1..doc.connections.len()).find(|&j| {
            plain(c) && plain(&doc.connections[j]) && doc.connections[j].from == c.to && doc.connections[j].to == c.from
        });
        if let Some(j) = back {
            doc.connections.remove(j);
            doc.connections[i].direction = Some(Direction::Both);
        }
        i += 1;
    }
}

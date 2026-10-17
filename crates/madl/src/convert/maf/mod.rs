//! Microsoft Agent Framework declarative files: a workflow (`kind: Workflow`), the `kind: Prompt`
//! agents it invokes by name, or Prompt agents alone.
//!
//! The importer reads the YAML and never runs it. A workflow becomes the agents it invokes and
//! a `handoff` from each to every agent it can run next; its branch conditions, variables and the
//! actions between agents are named in the notes, and the workflow is kept under the
//! `com.microsoft.agent-framework` extension.
mod agent;
mod channels;
mod cond;
mod flow;
mod links;
mod load;
mod passed;
mod tools;
mod workflow;

use super::{Builder, Fidelity, ImportError, Imported};
use crate::model::{Communication, CommunicationMode, Document, Input, Preferences, Settings, System};
use std::path::Path;

/// The extension namespace for what only the Agent Framework uses.
pub const NS: &str = "com.microsoft.agent-framework";

/// Imports a workflow file, a Prompt agent file, or a folder holding one workflow and the Prompt
/// agents it invokes.
pub fn import(path: &Path) -> Result<Imported, ImportError> {
    let src = load::read(path)?;
    let mut b = Builder::new(document(&src));
    for (rel, why) in &src.skipped {
        b.note(Fidelity::Unsupported, rel.as_str(), why.as_str());
    }
    let entries = match &src.workflow {
        Some(w) => workflow::import(&mut b, w, &src.agents).map_err(|why| {
            let file = if path.is_dir() { path.join(&w.rel) } else { path.to_path_buf() };
            ImportError(format!("{}: {why}", file.display()))
        })?,
        None => alone(&mut b, &src.agents),
    };
    let input = b.id_for("user-message");
    b.doc.inputs.push(Input { id: input, text: String::new(), to: entries });
    Ok(Imported { document: b.doc, notes: b.notes })
}

/// The document around the agents: the Agent Framework's runtime by default, and closed
/// communication, since the workflow decides who runs next.
fn document(src: &load::Sources) -> Document {
    let mut b = Builder::new(Document::default());
    let only = (src.workflow.is_none() && src.agents.len() == 1).then(|| &src.agents[0]);
    let main = src.workflow.as_ref().or(only);
    let stem = main.and_then(|f| Path::new(&f.rel).file_stem()).map(|s| s.to_string_lossy().into_owned());
    let name = main.and_then(|f| f.text("name")).map(str::to_string).or(stem).unwrap_or_else(|| src.stem.clone());
    let preferences = Preferences { runtime: Some("microsoft-agent-framework".into()), ..Default::default() };
    let workflow = src.workflow.is_some();
    Document {
        madl: "draft".into(),
        system: System {
            id: b.id_for(&name),
            name,
            description: main.and_then(|f| f.text("description")).map(str::to_string),
            budget: None,
        },
        defaults: Some(Settings { preferences: Some(preferences), ..Default::default() }),
        communication: workflow.then_some(Communication { mode: Some(CommunicationMode::Closed), deny: Vec::new() }),
        ..Default::default()
    }
}

/// Prompt agents without a workflow: each one starts on its own.
fn alone(b: &mut Builder, defs: &[load::File]) -> Vec<String> {
    for def in defs {
        let id = b.id_for(def.text("name").unwrap_or("agent"));
        let agent = agent::agent(b, &id, def);
        b.doc.agents.push(agent);
    }
    if defs.len() > 1 {
        let why = "no workflow says who runs when; every agent starts on its own";
        b.note(Fidelity::Approximated, format!("{}#", defs[0].rel), why);
    }
    b.doc.agents.iter().map(|a| a.id.clone()).collect()
}

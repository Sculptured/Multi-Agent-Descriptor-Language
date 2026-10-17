//! Importers: MADL documents built from other frameworks' files, with a note for every part that
//! did not carry over exactly. What the notes do not mention was kept as it was.
pub mod adk;

use crate::model::{Connection, ConnectionKind, Document};
use serde::Serialize;
use std::collections::HashSet;
use std::fmt;

/// How well a part of the source carried over.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Fidelity {
    /// Kept, but MADL says something close to the source rather than the same.
    Approximated,
    /// Not kept, or kept only under `extensions` where other tools ignore it.
    Unsupported,
}

impl fmt::Display for Fidelity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Fidelity::Approximated => "approximated",
            Fidelity::Unsupported => "unsupported",
        })
    }
}

/// One part of the source that did not carry over exactly. `at` is the source file, relative to
/// the folder of the file the import started from, and a JSON Pointer into it: `loop.yaml#/tools/0`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Note {
    pub fidelity: Fidelity,
    pub at: String,
    pub message: String,
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: {}", self.fidelity, self.at, self.message)
    }
}

/// A document built from another framework's files, and what did not carry over exactly.
#[derive(Clone, Debug)]
pub struct Imported {
    pub document: Document,
    pub notes: Vec<Note>,
}

/// The source could not be read as the framework's files.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct ImportError(pub String);

/// A document being built: ids handed out so far, and the notes.
pub(crate) struct Builder {
    pub doc: Document,
    pub notes: Vec<Note>,
    taken: HashSet<String>,
}

impl Builder {
    pub fn new(doc: Document) -> Self {
        Builder { doc, notes: Vec::new(), taken: HashSet::new() }
    }

    pub fn note(&mut self, fidelity: Fidelity, at: impl Into<String>, message: impl Into<String>) {
        self.notes.push(Note { fidelity, at: at.into(), message: message.into() });
    }

    /// An unused MADL id for a name from the source: characters an id can't hold become `_`,
    /// and a name already used gets `-2`, `-3`…
    pub fn id_for(&mut self, name: &str) -> String {
        let mut base: String =
            name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect();
        if !base.starts_with(|c: char| c.is_ascii_alphanumeric()) {
            base.insert(0, 'a');
        }
        base.truncate(60);
        let mut id = base.clone();
        let mut n = 1;
        while self.taken.contains(&id) {
            n += 1;
            id = format!("{base}-{n}");
        }
        self.taken.insert(id.clone());
        id
    }

    /// Adds a connection, or returns the one already there with the same ends and kind. A
    /// connection from a node to itself is not added.
    pub fn link(&mut self, from: &str, to: &str, kind: ConnectionKind) -> Option<&mut Connection> {
        if from == to {
            return None;
        }
        let found = self.doc.connections.iter().position(|c| c.from == from && c.to == to && c.kind == kind);
        let i = found.unwrap_or_else(|| {
            self.doc.connections.push(connection(from, to, kind));
            self.doc.connections.len() - 1
        });
        Some(&mut self.doc.connections[i])
    }
}

/// A connection with nothing but its ends and kind.
pub fn connection(from: &str, to: &str, kind: ConnectionKind) -> Connection {
    Connection {
        id: None,
        from: from.into(),
        to: to.into(),
        kind,
        direction: None,
        via: None,
        protocol: None,
        initiates: None,
        mode: None,
        carries: Vec::new(),
        max_rounds: None,
        when: None,
        pattern: None,
        extensions: Default::default(),
    }
}

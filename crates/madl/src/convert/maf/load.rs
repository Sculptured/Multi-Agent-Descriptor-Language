//! Reading Agent Framework declarative files: a file, or every `.yaml` and `.yml` file directly in a
//! folder. A folder may hold one workflow and any number of `kind: Prompt` agents.
use crate::convert::ImportError;
use serde_json::{Map, Value};
use std::path::Path;

/// One declarative file: its name, relative to the folder imported, and its contents.
pub(super) struct File {
    pub rel: String,
    pub value: Map<String, Value>,
}

impl File {
    pub fn text(&self, key: &str) -> Option<&str> {
        self.value.get(key).and_then(Value::as_str).filter(|s| !s.trim().is_empty())
    }
}

/// What was read: the workflow, the agents, and the files passed over with the reason.
pub(super) struct Sources {
    /// The file's stem, or the folder's name.
    pub stem: String,
    pub workflow: Option<File>,
    pub agents: Vec<File>,
    pub skipped: Vec<(String, String)>,
}

enum Kind {
    Workflow,
    Agent,
}

/// A workflow has `kind: Workflow`, or, as the Python loader accepts, no `kind` and top-level
/// `actions` or `trigger`. An agent has `kind: Prompt` (or `Agent`). Kinds ignore case.
fn kind_of(value: &Map<String, Value>) -> Option<Kind> {
    match value.get("kind").and_then(Value::as_str).map(str::to_ascii_lowercase).as_deref() {
        Some("workflow") => Some(Kind::Workflow),
        Some("prompt" | "agent") => Some(Kind::Agent),
        None if value.contains_key("actions") || value.contains_key("trigger") => Some(Kind::Workflow),
        _ => None,
    }
}

pub(super) fn read(path: &Path) -> Result<Sources, ImportError> {
    let fail = |why: String| ImportError(format!("{}: {why}", path.display()));
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "workflow".into());
    let mut out = Sources { stem, workflow: None, agents: Vec::new(), skipped: Vec::new() };
    if !path.is_dir() {
        let file = parse(path, &name_of(path)).map_err(fail)?;
        let kind = kind_of(&file.value).ok_or_else(|| fail("is not a workflow or a `kind: Prompt` agent".into()))?;
        out.add(kind, file).map_err(fail)?;
        return Ok(out);
    }
    for entry in yaml_files(path).map_err(|e| fail(e.to_string()))? {
        let rel = name_of(&entry);
        match parse(&entry, &rel) {
            Ok(file) => match kind_of(&file.value) {
                Some(kind) => out.add(kind, file).map_err(fail)?,
                None => out.skipped.push((rel, "is not a workflow or a `kind: Prompt` agent; skipped".into())),
            },
            Err(why) => out.skipped.push((rel, format!("could not be read ({why}); skipped"))),
        }
    }
    if out.workflow.is_none() && out.agents.is_empty() {
        return Err(fail("holds no workflow and no `kind: Prompt` agent".into()));
    }
    Ok(out)
}

impl Sources {
    fn add(&mut self, kind: Kind, file: File) -> Result<(), String> {
        match kind {
            Kind::Agent => self.agents.push(file),
            Kind::Workflow => match &self.workflow {
                Some(first) => return Err(format!("holds two workflows, {} and {}", first.rel, file.rel)),
                None => self.workflow = Some(file),
            },
        }
        Ok(())
    }
}

fn parse(path: &Path, rel: &str) -> Result<File, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let value: Value = serde_yaml_ng::from_str(&text).map_err(|e| e.to_string())?;
    let Value::Object(value) = value else { return Err("is not a mapping".into()) };
    Ok(File { rel: rel.to_string(), value })
}

fn yaml_files(dir: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let p = entry?.path();
        let yaml = p.extension().is_some_and(|e| e == "yaml" || e == "yml");
        if yaml && p.is_file() {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}

fn name_of(path: &Path) -> String {
    path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

//! Reading an ADK agent tree: `root_agent.yaml` and every file its `sub_agents` and `AgentTool`s
//! name. Like ADK, a `config_path` may not lead outside the root file's folder.
use crate::convert::ImportError;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

/// What an ADK config's `agent_class` makes the agent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Class {
    Llm,
    Sequential,
    Parallel,
    Loop,
    /// Another class, by its Python name: its behaviour is code.
    Custom(String),
    /// An agent defined in Python (`sub_agents: [{ code: … }]`), by its import path.
    Code(String),
}

/// One agent of the tree, and where it came from.
pub(super) struct Node {
    /// The file, relative to the root file's folder, and the JSON Pointer of this agent in it.
    pub at: String,
    pub class: Class,
    pub config: Map<String, Value>,
    pub children: Vec<Node>,
    /// The `AgentTool`s in `tools`, with the agents they call.
    pub agent_tools: Vec<Node>,
}

impl Node {
    pub fn name(&self) -> &str {
        match &self.class {
            Class::Code(path) => path.rsplit('.').nth(1).unwrap_or(path),
            _ => self.text("name").unwrap_or("agent"),
        }
    }

    pub fn text(&self, key: &str) -> Option<&str> {
        self.config.get(key).and_then(Value::as_str).filter(|s| !s.trim().is_empty())
    }

    pub fn flag(&self, key: &str) -> bool {
        self.config.get(key).and_then(Value::as_bool).unwrap_or(false)
    }
}

pub(super) struct Loader {
    root: PathBuf,
    open: Vec<PathBuf>,
}

impl Loader {
    pub fn new(file: &Path) -> Result<Self, ImportError> {
        let file = file.canonicalize().map_err(|e| ImportError(format!("{}: {e}", file.display())))?;
        let root = file.parent().map(Path::to_path_buf).unwrap_or_default();
        Ok(Loader { root, open: Vec::new() })
    }

    /// Reads one config file, and the files it names.
    pub fn file(&mut self, path: &Path) -> Result<Node, ImportError> {
        let fail = |why: String| ImportError(format!("{}: {why}", path.display()));
        let path = path.canonicalize().map_err(|e| fail(e.to_string()))?;
        if !path.starts_with(&self.root) {
            return Err(fail(format!("is outside {}", self.root.display())));
        }
        if self.open.contains(&path) {
            return Err(fail("names itself through its sub-agents".into()));
        }
        let text = std::fs::read_to_string(&path).map_err(|e| fail(e.to_string()))?;
        let value: Value = serde_yaml_ng::from_str(&text).map_err(|e| fail(e.to_string()))?;
        let Value::Object(config) = value else { return Err(fail("is not an ADK agent config".into())) };
        let rel = path.strip_prefix(&self.root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        self.open.push(path.clone());
        let node = self.node(&path, format!("{rel}#"), config);
        self.open.pop();
        node
    }

    fn node(&mut self, file: &Path, at: String, config: Map<String, Value>) -> Result<Node, ImportError> {
        let dir = file.parent().unwrap_or(&self.root).to_path_buf();
        let list = |key: &str| config.get(key).and_then(Value::as_array).cloned().unwrap_or_default();
        let mut children = Vec::new();
        for (i, r) in list("sub_agents").iter().enumerate() {
            children.push(self.reference(&dir, r, &format!("{at}/sub_agents/{i}"))?);
        }
        let mut agent_tools = Vec::new();
        for (i, t) in list("tools").iter().enumerate() {
            if t.get("name").and_then(Value::as_str) == Some("AgentTool") {
                let target = t.pointer("/args/agent").unwrap_or(&Value::Null);
                agent_tools.push(self.reference(&dir, target, &format!("{at}/tools/{i}/args/agent"))?);
            }
        }
        Ok(Node { at, class: class_of(&config), config, children, agent_tools })
    }

    /// `{ config_path }` (or a bare path) reads that file; `{ code }` names an agent defined in Python.
    fn reference(&mut self, dir: &Path, r: &Value, at: &str) -> Result<Node, ImportError> {
        if let Some(code) = r.get("code").and_then(Value::as_str) {
            let class = Class::Code(code.to_string());
            return Ok(Node {
                at: at.to_string(),
                class,
                config: Map::new(),
                children: Vec::new(),
                agent_tools: Vec::new(),
            });
        }
        let path = r.get("config_path").and_then(Value::as_str).or(r.as_str());
        let path =
            path.ok_or_else(|| ImportError(format!("{at}: names no agent: expected `config_path` or `code`")))?;
        if Path::new(path).is_absolute() {
            return Err(ImportError(format!("{at}: `{path}` is absolute; ADK takes paths relative to the file")));
        }
        self.file(&dir.join(path))
    }
}

fn class_of(config: &Map<String, Value>) -> Class {
    let class = config.get("agent_class").and_then(Value::as_str).unwrap_or("LlmAgent");
    let short = class.strip_prefix("google.adk.agents.").unwrap_or(class);
    match short {
        "LlmAgent" | "Agent" => Class::Llm,
        "SequentialAgent" => Class::Sequential,
        "ParallelAgent" => Class::Parallel,
        "LoopAgent" => Class::Loop,
        _ => Class::Custom(class.to_string()),
    }
}

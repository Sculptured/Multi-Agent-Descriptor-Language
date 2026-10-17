//! Agents and what they are given: tools, workspace, checks, subagents.
use super::{Extensions, ModelRef, Resume, Settings};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

/// One participant: a model-driven agent, a remote agent or a person.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Agent {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
    /// The agent's job, in plain text (often a `$ref` to a Markdown file).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub instructions: Option<String>,
    /// Shorthand for `settings.preferences.runtime`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub runtime: Option<String>,
    /// Shorthand for `settings.preferences.model`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub model: Option<ModelRef>,
    /// Shorthand for `settings.preferences.resume`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub resume: Option<Resume>,
    /// For `runtime: remote-a2a`: the agent's A2A endpoint or Agent Card URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub endpoint: Option<String>,
    /// The groups the agent belongs to, by id. An agent may be in no group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub groups: Vec<String>,
    /// Tools the agent is given. Restrictions decide which of them it may use.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Tool>>", optional)]
    pub tools: Vec<Tool>,
    /// Skills: a path or name, or the skill itself embedded (`{ name, description, body }`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<SkillRef>>", optional)]
    pub skills: Vec<SkillRef>,
    /// Files given to the agent as context, relative to the document.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub context: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub workspace: Option<Workspace>,
    /// Where the agent runs: a selector over machine properties such as `os` or `name`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub placement: Option<Placement>,
    /// What the agent produces.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Output>>", optional)]
    pub outputs: Vec<Output>,
    /// What the agent reads and writes, and what else it changes: its data contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub data_access: Option<DataAccess>,
    /// The decisions this agent owns: within these domains the others defer to it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub authority: Option<Authority>,
    /// Commands run to check the agent's work.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Check>>", optional)]
    pub checks: Vec<Check>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub settings: Option<Settings>,
    /// What agents this one spawns start as, and the settings they get.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub subagents: Option<Subagents>,
    #[serde(default, skip_serializing_if = "Extensions::is_empty")]
    #[ts(as = "Option<Extensions>", optional)]
    pub extensions: Extensions,
}

/// A tool: an MCP server, or one of the runtime's built-in tools.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Tool {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub mcp: Option<McpServer>,
    /// A runtime's own tool, as `<runtime>:<name>`, e.g. `claude-code:Bash`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub builtin: Option<String>,
}

/// How to reach an MCP server: a command to start (stdio) or a URL (HTTP).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct McpServer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(as = "Option<BTreeMap<String, EnvValue>>", optional)]
    pub env: BTreeMap<String, EnvValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub url: Option<String>,
}

/// A value given as text, or a secret the runner resolves. Secrets never appear in documents.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum EnvValue {
    Text(String),
    Secret(SecretRef),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct SecretRef {
    /// The secret's name in the runner's secret store.
    pub secret: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Workspace {
    /// The agent's working folder, relative to the document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub folder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub copy: Option<CopyMode>,
}

/// `own`: the agent works in its own git copy. `shared`: in the folder itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CopyMode {
    Own,
    Shared,
    /// One copy per item of a fan-out.
    PerItem,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Placement {
    /// Every key must match the machine's property of the same name.
    pub machine: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Output {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    /// Commands that decide whether this output is done: every one exits 0. A test, not a reviewer's
    /// opinion, is the judge.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Check>>", optional)]
    pub acceptance: Vec<Check>,
}

/// An agent's data contract: the paths it reads and writes (relative to the document; a trailing
/// `/` or `/**` covers a folder), the effects it has beyond files, and why. A Runner SHOULD refuse a
/// write outside `writes` where it can, and MUST say where it cannot. `restrictions.files` stays the
/// hard limit; `data_access` declares what the agent is for.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct DataAccess {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub reads: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub writes: Vec<String>,
    /// Effects beyond its files, in plain words: "sends email", "pushes to main".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub side_effects: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub rationale: Option<String>,
}

/// The decisions an agent owns, as domains in plain words ("wing geometry", "release dates").
/// Runners SHOULD tell every agent who decides what, and send a question within a domain to its owner.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Authority {
    pub domains: Vec<String>,
    /// The group (or `system`) whose boundary this agent's decisions may cross: a decision belongs to
    /// the agent whose scope contains its consequences. Absent: decisions within the agent's own work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub rationale: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Check {
    /// A shell command run in the agent's folder; exit 0 means passed.
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub after: Option<CheckWhen>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub timeout_s: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CheckWhen {
    EachTurn,
    Finish,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Subagents {
    /// What a spawned agent starts as. Its `id` is assigned at run time and is not given here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub template: Option<SubagentTemplate>,
    /// Settings for spawned agents only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub settings: Option<Settings>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct SubagentTemplate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub instructions: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub runtime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub model: Option<ModelRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Tool>>", optional)]
    pub tools: Vec<Tool>,
}

/// A skill given to an agent: by path or name, or embedded so the document carries it (MEP 0004).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum SkillRef {
    /// A path to a skill folder or `SKILL.md`, relative to the document, or a skill's name.
    Ref(String),
    Embedded(EmbeddedSkill),
}

/// A skill in the Agent Skills format: `name` (lowercase letters, digits and hyphens, at most 64),
/// `description` (when to use it, at most 1024 characters) and `body` (the `SKILL.md` instructions).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct EmbeddedSkill {
    pub name: String,
    pub description: String,
    pub body: String,
}

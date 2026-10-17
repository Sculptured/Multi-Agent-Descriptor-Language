//! Settings blocks: preferences (override) and restrictions (only tighten), and how far each
//! reaches.
use super::Communication;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use ts_rs::TS;

/// The same shape at the root (`defaults`), on a group, on an agent, and for subagents.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Settings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub preferences: Option<Preferences>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub restrictions: Option<Restrictions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub applies_to: Option<AppliesTo>,
}

/// How far this block's preferences and restrictions reach. Defaults: preferences `self`,
/// restrictions `descendants`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct AppliesTo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub preferences: Option<Reach>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub restrictions: Option<Reach>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum Reach {
    /// The agents the block covers, not what they spawn.
    #[serde(rename = "self")]
    Itself,
    /// Only what those agents spawn, and what that spawns.
    Subagents,
    /// The agents and everything they spawn.
    Descendants,
}

/// Overridable choices. The nearest value wins: a scalar or list replaces, `options` merges key
/// by key.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Preferences {
    /// `claude-code`, `codex`, `camel`, `crewai`, `langgraph`, `openai-agents`, `autogen`,
    /// `google-adk`, `microsoft-agent-framework`, `remote-a2a`, `human`, or `x-<vendor>-<name>`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub runtime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub model: Option<ModelRef>,
    /// The default protocol for communication: `native`, `a2a@<version>`, `mcp@<version>`, or
    /// `x-<vendor>-<name>[@<version>]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub protocol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub resume: Option<Resume>,
    /// The language agents write in, as a BCP 47 tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub questions: Option<Questions>,
    /// Runtime-specific options, merged key by key down the chain.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[ts(as = "Option<BTreeMap<String, Value>>", optional)]
    pub options: BTreeMap<String, Value>,
}

/// A model as `provider/model`, optionally with fallbacks tried in order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum ModelRef {
    Id(String),
    WithFallback {
        id: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        #[ts(as = "Option<Vec<String>>", optional)]
        fallback: Vec<String>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum Resume {
    ResumeIfPossible,
    RelaunchFresh,
    CheckpointOnly,
}

/// What happens when an agent asks a person something.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Questions {
    /// How long to wait for an answer, e.g. `30m`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub timeout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub on_timeout: Option<OnTimeout>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum OnTimeout {
    /// The agent continues with its own best judgment.
    Continue,
    /// The agent waits until someone answers.
    Wait,
    Stop,
}

/// Limits. Down the chain they can only tighten; loosening one is an error.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Restrictions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tools: Option<ToolRule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub files: Option<FileRule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub network: Option<Network>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub budget: Option<Budget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub spawn: Option<Spawn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub approvals: Option<Approvals>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub communication: Option<Communication>,
}

/// `allow`: only these tools (absent: every tool the agent is given). `deny`: never these.
/// Tools are named by tool id, `<tool id>/<tool name>` for one MCP tool, or `<runtime>:<name>`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct ToolRule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub allow: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub deny: Vec<String>,
}

/// Paths the agent may read and write, relative to the document. Absent: no limit.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct FileRule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub read: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub write: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Network {
    pub egress: Egress,
}

/// `deny` (no network), `allow` (any host), or a list of hosts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum Egress {
    Rule(EgressRule),
    Hosts(Vec<String>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum EgressRule {
    Deny,
    Allow,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Budget {
    /// Tokens of every kind. A bare number is the hard value; `{ soft, hard }` adds a soft one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "super::Limit<number>")]
    pub tokens: Option<super::Limit<u64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cost_usd: Option<f64>,
    /// A duration such as `90s`, `30m`, `2h` or `1h30m`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub wall_clock: Option<super::Limit<String>>,
    /// Turns (model replies to a message) in one run (MEP-0002).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "super::Limit<number>")]
    pub turns: Option<super::Limit<u64>>,
    /// Time spent working (in turns), as a duration (MEP-0002).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub working_time: Option<super::Limit<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub scope: Option<BudgetScope>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum BudgetScope {
    /// The agent and everything it spawns draw from one budget.
    Shared,
    /// Each agent has a budget of its own.
    Own,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Spawn {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub allowed: Option<bool>,
    /// Most agents one agent may spawn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_children: Option<u32>,
    /// Most levels below the agent: 1 lets it spawn agents that cannot spawn their own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_depth: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Approvals {
    /// Actions that need a person's approval first, e.g. `shell`, `network`,
    /// `write-outside-workspace`, or a tool name.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub required_for: Vec<String>,
}

//! The top of a MADL document: the system, its inputs and its groups.
use super::{Agent, Channel, Communication, Connection, Extensions, Layout, Policies, Settings, Step, Topology};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A MADL document after `$ref` resolution: one multi-agent system.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Document {
    /// The spec revision this document is written against: `draft` or a dated revision.
    pub madl: String,
    pub system: System,
    /// Global settings: they apply to every agent unless a group or the agent overrides them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub defaults: Option<Settings>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Input>>", optional)]
    pub inputs: Vec<Input>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Group>>", optional)]
    pub groups: Vec<Group>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Agent>>", optional)]
    pub agents: Vec<Agent>,
    /// Nodes that are not agents: checks, people's decisions, merges, fan-out and fan-in.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Step>>", optional)]
    pub steps: Vec<Step>,
    /// Where messages and artifacts travel: chats, folders, shared memory, buses and more.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Channel>>", optional)]
    pub channels: Vec<Channel>,
    /// Directed, committed lines of communication between agents, groups and steps.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<Connection>>", optional)]
    pub connections: Vec<Connection>,
    /// System-wide communication mode and bans.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub communication: Option<Communication>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub topology: Option<Topology>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub policies: Option<Policies>,
    /// How to draw the system. It has no meaning for execution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub layout: Option<Layout>,
    #[serde(default, skip_serializing_if = "Extensions::is_empty")]
    #[ts(as = "Option<Extensions>", optional)]
    pub extensions: Extensions,
}

/// What the system is.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct System {
    /// Stable identifier: letters, digits, `-` and `_`, starting with a letter or digit.
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    /// What one run of the whole system may use (MEP-0002).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub budget: Option<super::RunBudget>,
}

/// Something the system is given when it starts: a goal, links, notes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Input {
    pub id: String,
    /// The input itself. Empty or absent: it is given when the run starts.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[ts(as = "Option<String>", optional)]
    pub text: String,
    /// The agents that receive it. Empty: the starting agents.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[ts(as = "Option<Vec<String>>", optional)]
    pub to: Vec<String>,
}

/// A named set of agents that share a purpose and settings. Groups hold only the agents that
/// list them, and may nest.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct Group {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
    /// What the members share. Passed to every member and to agents connected to the group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub purpose: Option<String>,
    /// The enclosing group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub parent: Option<String>,
    /// Settings for the group's members (and, by reach, their subagents).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub settings: Option<Settings>,
    #[serde(default, skip_serializing_if = "Extensions::is_empty")]
    #[ts(as = "Option<Extensions>", optional)]
    pub extensions: Extensions,
}

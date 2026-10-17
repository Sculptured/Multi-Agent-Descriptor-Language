# Interoperability

These mappings are informative.

## Sculptured

[Sculptured](https://github.com/Alexander-Aghili/Sculptured) exports teams as MADL and imports MADL
documents as teams; an exported team imported again is unchanged.

| Sculptured team file | MADL |
|---|---|
| team `id`, `name` | `system.id`, `system.name` |
| agent `kind`: `claude`, `codex` | `runtime: claude-code`, `codex` |
| agent `job` | `instructions` |
| agent `folder` | `workspace.folder` |
| models: agent, group, team (per kind) | each agent's effective `model`, as `provider/model`; group and team defaults in `com.usesculptured` |
| `token_limit { tokens, include_subagents }` | `settings.restrictions.budget { tokens, scope: shared \| own }` |
| `check` | `checks: [ { command, after: each-turn } ]` |
| `resume` | `resume` |
| groups with their boxes | `groups`, and `layout.groups` |
| `hands-work-to`, `reports-to`, `watches` | `handoff`, `report`, `watch` |
| the Input | `inputs` |
| `limits.max_child_levels` | `defaults.restrictions.spawn.max_depth` |
| schedules | `policies.schedules` |
| workflow | `com.usesculptured.workflow`; its stages are also drawn as steps and handoffs with outcome conditions |
| posture, own copies, question timeout, decision log | `com.usesculptured`, `workspace.copy`, `preferences.questions.timeout` |

## CAMEL

[CAMEL](https://github.com/camel-ai/camel)'s Workforce maps onto MADL as follows:

| CAMEL | MADL |
|---|---|
| `Workforce` | a group with a `supervisor` pattern, `routing: dynamic` |
| nested `Workforce` | a child group (`parent`) |
| `coordinator_agent` | the supervisor |
| `task_agent` (decomposition) | `policies.on_failure.strategy: decompose`, or an agent that delegates to the coordinator |
| `SingleAgentWorker` | an agent with `runtime: camel`, `tools`, `instructions` |
| `RolePlayingWorker` (`chat_turn_limit`) | a `role-play` pattern with `max_rounds` |
| `new_worker_agent` | `subagents.template` |
| `share_memory` | a `shared-memory` channel among the workers |
| `HumanToolkit` | a `human` participant with `ask` and `approve` connections |
| recovery: retry, replan, decompose | `policies.on_failure` |
| `task_timeout_seconds` | `restrictions.budget.wall_clock` |

What CAMEL defines in Python code (custom toolkits, callables) is referenced from
`extensions.ai.camel` by import path.

## A2A

An agent with `runtime: remote-a2a` has an `endpoint`: its A2A endpoint or Agent Card URL.
Connections that use `a2a@<version>` travel over A2A. A tool can publish an A2A Agent Card for each
agent from its name, its `description` (else its instructions' first paragraph) and its tools.

## Google ADK

`madl import adk <root_agent.yaml or its folder>` builds a document from an
[ADK](https://adk.dev) agent config tree, reading the YAML and never running Python. It lists every
part that did not carry over exactly as `approximated` or `unsupported`, with the source file and
JSON Pointer; what it does not list was kept.

| ADK | MADL |
|---|---|
| `LlmAgent`: `name`, `description`, `instruction`, `model` | an agent: `id`, `description`, `instructions`, `model: google/<model>`; `runtime: google-adk` in `defaults`. An agent without a model gets the nearest enclosing `LlmAgent`'s, as in ADK |
| `SequentialAgent` | a group (`purpose` from `description`); `handoff` from each sub-agent to the next |
| `LoopAgent` (`max_iterations`) | the same, and a `handoff` from the last back to the first with `max_rounds`. Leaving early when an agent escalates (`exit_loop`) is approximated |
| `ParallelAgent` | a group; the work goes to every sub-agent. ADK's wait for every branch is approximated: the next agent receives each branch's handoff |
| `sub_agents` of an `LlmAgent` (transfer) | `delegate` to each, `report` back unless `disallow_transfer_to_parent`, `handoff` to peers unless `disallow_transfer_to_peers` (the supervisor pattern); `communication.mode: closed` |
| `AgentTool` | `delegate` with `mode: sync` to the called agent |
| `output_key`, `{key}` in instructions | an output, and a `shared-memory` channel `session-state` with each agent's `read`/`write` access |
| built-in tools; Python functions | `builtin: google-adk:<name>`; `builtin: google-adk:<import path>` (only an ADK runner can call them) |
| `McpToolset` | an `mcp` tool (command or URL); `tool_filter` becomes `restrictions.tools.allow`; environment values become `{ secret: NAME }` |
| `generate_content_config`, `include_contents` | `preferences.options` |
| agents defined in Python (`code:`), custom agent classes, callbacks, schemas, tool `args` | kept under `extensions.dev.adk`, with a note |
| the user's message | an input without `text` to the root's first agents |

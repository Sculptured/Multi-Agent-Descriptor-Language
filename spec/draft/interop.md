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

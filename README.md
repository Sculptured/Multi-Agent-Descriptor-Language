# Multi-Agent Descriptor Language (MADL)

[![CI](https://github.com/Alexander-Aghili/Multi-Agent-Descriptor-Language/actions/workflows/ci.yml/badge.svg)](https://github.com/Alexander-Aghili/Multi-Agent-Descriptor-Language/actions/workflows/ci.yml)
[![Spec: draft](https://img.shields.io/badge/spec-draft-orange)](spec/draft/index.md)
[![License: Apache-2.0](https://img.shields.io/badge/code-Apache--2.0-blue)](LICENSE)
[![Spec license: CC BY 4.0](https://img.shields.io/badge/spec-CC%20BY%204.0-lightgrey)](LICENSE-SPEC)

MADL is an open, vendor-neutral file format for describing a **multi-agent system**: its agents,
how they are grouped, who may talk to whom and how, the shape of the work, the settings each agent
runs with, and the restrictions it runs under. One MADL file can be validated, drawn, compared,
shared and run by any tool that implements it.

MADL describes systems; it does not move messages. Agents talk over protocols such as
[A2A](https://a2a-protocol.org) and reach tools over [MCP](https://modelcontextprotocol.io); a MADL
file says which protocol each line of communication uses.

```yaml
madl: draft
system: { id: cad-review, name: CAD review team }

defaults:                                   # global settings, for every agent
  preferences: { runtime: claude-code, model: anthropic/claude-sonnet-5-5 }
  restrictions:
    tools: { deny: [web-fetch] }            # never these tools
    budget: { tokens: 4000000, scope: shared }

agents:
  - id: cad
    runtime: codex                          # overrides the default
    instructions: Model the bracket and export a STEP file.
  - id: reviewer
    instructions: Check the model; send it back with changes or approve it.
    settings:
      restrictions:
        tools: { allow: [cad-kernel] }      # only these tools; can only tighten

connections:                                # committed, directional communication
  - { from: cad, to: reviewer, kind: handoff, via: shared-folder, carries: [ { artifact: model } ] }
  - { from: reviewer, to: cad, kind: handoff, max_rounds: 3 }   # a loop, with a way in

channels:
  - { id: shared-folder, medium: filesystem, path: ./cad }

communication:
  mode: closed                              # only declared connections are allowed
  deny: [ { from: reviewer, to: planner } ] # and bans always win
```

## What a MADL file says

| | |
|---|---|
| **Participants** | Agents on any runtime (Claude Code, Codex, CAMEL, CrewAI, LangGraph, OpenAI Agents, remote A2A agents), people, and steps such as checks, approvals, merges, fan-out and fan-in |
| **Groups** | Named sets of agents with a shared purpose and shared settings; groups nest |
| **Contracts** | What each agent reads, writes and otherwise changes (`data_access`), the decisions it owns (`authority`), and acceptance checks that decide when an output is done. Overlapping writes or decisions are flagged |
| **Settings** | Global `defaults`, overridden per group and per agent. **Preferences** (model, runtime, protocol…) override; **restrictions** (tools, files, network, budgets, spawning, approvals, communication) only tighten. Each block says whether it reaches the agents an agent spawns |
| **Communication** | Channels (direct chat, shared folder, shared memory, blackboard, bus, git, tracker, human) and connections with their kind, direction, protocol, initiator, sync mode, contents, round limit and condition; open or closed mode with bans |
| **Topology** | An explicit graph, or patterns (pipeline, supervisor, committee, debate, role-play, auction, swarm, blackboard) that expand into one; loops are allowed when work can reach them |
| **Policies** | Failure handling (retry, replan, decompose, escalate), escalation, schedules, guards |
| **Layout** | Optional positions so every tool draws the system the same way |
| **Extensions** | Reverse-DNS namespaces for what one tool needs and others may ignore |

## Use it

```sh
cargo install --git https://github.com/Alexander-Aghili/Multi-Agent-Descriptor-Language madl-cli

madl validate team.madl.yaml      # schema + rules, with stable error codes and JSON Pointer paths
madl resolve  team.madl.yaml      # every agent's effective settings, and where each value came from
madl expand   team.madl.yaml      # $refs resolved, patterns expanded
madl graph    team.madl.yaml      # a Mermaid flowchart
madl import adk ./my_agent        # a Google ADK agent config tree as MADL, with what did not carry over
madl schema                       # the JSON Schema
```

From Rust:

```rust
let doc = madl::load(std::path::Path::new("team.madl.yaml"))?;
let report = madl::validate(&doc);
let settings = madl::resolve(&madl::expand(&doc));
```

Editors can validate and complete MADL files with the published schema:
[`schema/draft/schema.json`](schema/draft/schema.json).

## Repository

| Path | |
|---|---|
| [`spec/draft/`](spec/draft/index.md) | The specification (normative) |
| [`schema/draft/schema.json`](schema/draft/schema.json) | JSON Schema, generated from the Rust types |
| [`examples/`](examples) | Pipeline, supervisor workforce, debate, CAD review, textbook fan-out, open team with bans, a document split across files |
| [`conformance/`](conformance) | Valid and invalid documents with expected error codes, and expected resolver and expander output; every implementation runs them |
| [`crates/madl`](crates/madl) | The reference implementation: types, loader, validator, resolver, expander, Mermaid, importers (Google ADK) |
| [`crates/madl-cli`](crates/madl-cli) | The `madl` command |
| [`bindings/typescript`](bindings/typescript) | Generated TypeScript types |
| [`docs/design/`](docs/design) | Design records |

## Implementations

- **[Sculptured](https://github.com/Alexander-Aghili/Sculptured)** exports any team as MADL and imports MADL files as teams.

Python and TypeScript packages and a CAMEL adapter are planned; they will call the Rust core so
their behaviour cannot drift from the spec's fixtures.

## Status

The spec is a **draft**: documents say `madl: draft` and may need changes before the first dated
revision. Proposals and feedback are welcome; see [CONTRIBUTING.md](CONTRIBUTING.md) and
[GOVERNANCE.md](GOVERNANCE.md).

## License

Code: [Apache-2.0](LICENSE). Specification text: [CC BY 4.0](LICENSE-SPEC).

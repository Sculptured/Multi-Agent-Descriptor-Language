# MADL specification (draft)

**Revision:** `draft` · **Status:** working draft; it may change before the first dated revision.

The Multi-Agent Descriptor Language (MADL) describes a multi-agent system: its participants, how
they are grouped, how they communicate, the shape of the work, the settings each participant runs
with, and the restrictions it runs under. This specification defines the document format and the
rules every reader applies to it.

MADL describes systems. It does not define how messages travel: connections name a protocol
(for example [A2A](https://a2a-protocol.org)) and tools are reached over protocols such as
[MCP](https://modelcontextprotocol.io).

## Conventions

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **MAY** and
**OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14)
([RFC 2119](https://www.rfc-editor.org/rfc/rfc2119), [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174))
when, and only when, they appear in all capitals.

Locations in a document are written as [JSON Pointers](https://www.rfc-editor.org/rfc/rfc6901)
into the document after `$ref` resolution, for example `/agents/2/groups/0`.

## Terms

| Term | Meaning |
|---|---|
| **Document** | One MADL file (or several joined by `$ref`), describing one system |
| **Agent** | A participant that acts: a model-driven agent, a remote agent, or a person |
| **Step** | A node that is not an agent: a check, a person's decision, a merge, a fan-out or a fan-in |
| **Node** | An agent or a step |
| **Group** | A named set of agents with a shared purpose and settings |
| **Settings block** | Preferences, restrictions and their reach, at the root, on a group, on an agent or for an agent's subagents |
| **Subagent** | An agent another agent starts while the system runs; it is not listed in the document |
| **Reader** | Software that loads and validates documents (see [Conformance](conformance.md)) |
| **Runner** | Software that runs the system a document describes |

## Contents

1. [Document](document.md): format, ids, `$ref`, the system, inputs, groups, agents, steps, layout, extensions
2. [Settings](settings.md): preferences, restrictions, reach, subagents, resolution
3. [Communication](communication.md): protocols, channels, connections, open and closed mode, bans
4. [Topology](topology.md): the graph, patterns, starting points, loops, conditions, policies
5. [Conformance](conformance.md): levels, diagnostics, fixtures, security considerations
6. [Interoperability](interop.md): Sculptured, CAMEL, A2A, Google ADK, Microsoft Agent Framework

The JSON Schema [`schema/draft/schema.json`](../../schema/draft/schema.json) describes a document
after `$ref` resolution. Where the schema and this text disagree, this text governs.

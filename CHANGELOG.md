# Changelog

## draft (unreleased)

The first public draft.

- `madl import adk`: builds a document from a Google ADK agent config tree (`root_agent.yaml` and
  the files it names) without running Python, and lists every part that was approximated or not
  supported, by source file and JSON Pointer.

- MEP 0005: agents take a `description` (how the others see them); an input without `text` is given
  when the run starts; `google-adk` and `microsoft-agent-framework` are known runtimes.

- MEP 0004: `skills` entries may embed the skill itself (`{ name, description, body }`, the Agent
  Skills format), so a document carries its skills. New errors `skill-shape` and `duplicate-skill`.

- MEP 0003: agents may declare `data_access` (reads, writes, side effects, rationale) and `authority`
  (the decisions they own, and with `scope` how far across groups they reach); outputs take `acceptance` checks that decide when they are done. New
  warnings `write-overlap` and `authority-overlap`. Ideas from Pact (github.com/wandercom/pact).

- MEP 0002: limits take `{ soft, hard }` (wrap up, then stop); per-agent budgets gain `turns` and
  `working_time`; `system.budget` sets a budget for the whole run (plan-window shares, tokens, cost,
  wall clock, most agents working at once). New codes `budget-soft-above-hard`,
  `budget-percent-out-of-range`, `budget-no-room`.

- Document: system, defaults, inputs, groups (nesting), agents, steps, channels, connections,
  communication, topology, policies, layout, extensions; `$ref` across files.
- Settings: preferences override, restrictions only tighten; `applies_to` reach (`self`,
  `subagents`, `descendants`); subagent templates and settings; a stated resolution order.
- Communication: protocols (`native`, `a2a@…`, `mcp@…`, `x-…`), channels with mediums and access,
  connections with kind, direction, initiator, mode, contents, round limits and conditions; open or
  closed mode; bans that always win.
- Topology: patterns that expand into connections; starting points; loops allowed when work can
  reach them.
- Reference implementation (`madl` crate and CLI), JSON Schema, TypeScript bindings, examples and
  conformance fixtures.

# Conformance

## Levels

| Level | MUST |
|---|---|
| **Reader** | Parse YAML and JSON; resolve `$ref`; check the document against the schema and every rule in this specification; report problems with the codes below |
| **Resolver** | Be a Reader, and expand patterns and compute effective settings exactly as `conformance/expand/` and `conformance/resolve/` say |
| **Runner** | Be a Resolver, and run the system, enforcing every restriction it accepts |
| **Renderer** | Be a Reader, and draw the graph, using `layout` when present |

### Runner

A runner that cannot enforce a restriction in a document (for example, network egress on a
platform where it cannot limit network access) MUST either refuse the document or tell the person
which restriction it cannot enforce and run only if they accept. It MUST NOT silently ignore one.

## Diagnostics

A reader reports each problem with a stable code, a severity, the JSON Pointer of the value at
fault, and a sentence. Tools and fixtures match on the code and pointer; the sentence is for people.

| Code | Severity | Meaning |
|---|---|---|
| `parse` | error | Not YAML or JSON |
| `shape` | error | Does not match the schema: a missing, unknown or wrongly typed member |
| `ref-missing`, `ref-outside`, `ref-cycle`, `ref-pointer` | error | A `$ref` that can't be followed |
| `unknown-revision` | error | `madl` names a revision the reader does not implement |
| `bad-id`, `duplicate-id` | error | An id that is malformed or used twice |
| `unknown-reference`, `wrong-kind` | error | A reference to nothing, or to the wrong kind of thing |
| `group-cycle` | error | A group inside itself |
| `self-connection` | error | A connection from a node to itself |
| `unknown-runtime`, `unknown-medium`, `unknown-protocol` | error | A name that is neither known nor an `x-` extension |
| `bad-duration`, `bad-extension-key` | error | A malformed duration or extension namespace |
| `shorthand-conflict` | error | A value set both as agent shorthand and in its preferences |
| `remote-needs-endpoint` | error | `runtime: remote-a2a` without `endpoint` |
| `step-field`, `pattern-shape`, `deny-shape` | error | A step, pattern or ban without what its kind needs, or with what it doesn't use |
| `restriction-loosened` | error | A block loosening an inherited restriction |
| `budget-soft-above-hard`, `budget-percent-out-of-range`, `budget-no-room` | error | A [limit](settings.md#limits) whose soft value is above its hard one, a plan share outside 0–100%, or a run with room for no agent |
| `connection-denied` | error | A declared connection that a ban matches |
| `no-starting-point` | error | No agent starts the work |
| `loop-without-start` | error | A loop no work can reach |
| `unreachable` | warning | A step no work reaches |
| `write-overlap` | warning | Two agents' `data_access.writes` cover the same path |
| `authority-overlap` | warning | Two agents' `authority` names the same domain |
| `skill-shape` | error | An embedded skill's name, description or body does not fit the Agent Skills format |
| `duplicate-skill` | error | An agent has two embedded skills with the same name |

## Fixtures

`conformance/` holds:

- `valid/`: documents with no errors;
- `invalid/`: each document with an `*.expected.json` naming the code and pointer it must produce;
- `resolve/`: documents with the exact effective settings (`*.resolved.json`);
- `expand/`: documents with the exact expanded connections and channels (`*.expanded.json`).

Every implementation runs every fixture.

## Security considerations

1. **Documents can run things.** Checks and guards are shell commands, and instructions steer
   agents that have tools. A runner MUST show a document's commands, tools, network access and
   restrictions to the person and ask before the first run of a document from elsewhere.
2. **Secrets.** Documents hold references (`{ secret: NAME }`) that the runner resolves, never
   secret values.
3. **References.** `$ref` stays inside the document's folder and never fetches over a network
   unless the person opts in.
4. **Restrictions are honest.** Restrictions only tighten, so a group or agent setting can't
   undo a system-wide limit, and spawning can't escape one. A runner that can't enforce a
   restriction says so.
5. **Signing** of documents is reserved for a later revision.

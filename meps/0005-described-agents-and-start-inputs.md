# MEP 0005: Agent descriptions, inputs given at start, and two more runtimes

- **Status:** Accepted
- **Issue:** none (proposed with the reference implementation)
- **Revision:** draft

## Problem

Importing systems built with Google's Agent Development Kit (ADK) and Microsoft Agent Framework
showed three things a document could not say:

1. **How an agent is seen by the others.** ADK routes work between agents by each agent's
   `description`; A2A Agent Cards carry one; Sculptured briefs an agent about its teammates. MADL
   had only `instructions`, which is what the agent itself is told, often pages long.
2. **An input that arrives when the run starts.** In both frameworks the system's input is the
   message a person opens the conversation with. `inputs[].text` was required, so an importer had to
   invent placeholder text.
3. **Their runtimes.** Neither had a name; importers had to use `x-` names for two widely used
   frameworks.

## Proposal

```yaml
inputs:
  - { id: user-message, to: [coordinator] }       # no text: given when the run starts
agents:
  - id: coordinator
    runtime: google-adk
    description: Greets the user and sends each request to the agent that handles it.
    instructions: { $ref: ./prompts/coordinator.md }
```

- Agents take an optional `description`: what the agent does, in a sentence or two. A Runner SHOULD
  give it to the agents connected to this one. A2A Agent Cards use it before falling back to the
  first paragraph of `instructions`.
- An input's `text` is optional. Without it (or with empty text) the input is given when the run
  starts: a Runner SHOULD ask the person starting the run, and MUST NOT start the agents it goes to
  with an empty input in its place.
- `google-adk` and `microsoft-agent-framework` join the known runtimes.

## Validation

No new codes. `description` is free text; an input's `text` is no longer required.

## Compatibility

Additive: every existing document stays valid and means the same. Writers that construct an
`Agent` value in code gain one optional field.

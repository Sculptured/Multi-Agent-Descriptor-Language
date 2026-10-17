# MEP 0004: Embedded skills

- **Status:** Accepted
- **Issue:** none (proposed with the reference implementation)
- **Revision:** draft

## Problem

`skills` names skills by path or name. A document that is meant to travel on its own (sent to
someone, or run as one file) then depends on skill files that may not travel with it, or on skills
the receiver happens to have installed under the same name.

## Proposal

A `skills` entry may also be the skill itself, in the Agent Skills format:

```yaml
agents:
  - id: keeper
    skills:
      - skills/house-style            # a path or a name, as before
      - name: money-rules             # embedded
        description: Use when handling amounts of money.
        body: |
          Amounts are Decimal, never float. Round half-even to cents.
```

- `name`: 1–64 lowercase letters, digits and single hyphens.
- `description`: when to use the skill, 1–1024 characters. A Runner SHOULD keep only names and
  descriptions in the agent's context until the skill is used.
- `body`: the `SKILL.md` instructions, in Markdown.

An agent's skills have different names. A skill belongs to the agent that lists it; copying the
agent copies its skills.

## Validation

- `skill-shape` (error): an embedded skill's name, description or body does not fit the format.
- `duplicate-skill` (error): an agent has two embedded skills with the same name.

## Compatibility

Additive: every existing document stays valid. A Reader that only understands strings reports the
embedded entries as unsupported rather than dropping them silently.

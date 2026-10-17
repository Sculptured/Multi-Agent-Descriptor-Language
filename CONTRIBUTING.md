# Contributing to MADL

Thank you for helping. MADL is a specification first and code second: changes to what a document
means start as a proposal, then land in the spec, the schema, the fixtures and the reference
implementation together.

## Small changes

Typos, clearer wording, new examples, more conformance fixtures and bug fixes in the crates: open a
pull request.

## Changes to the language

New fields, new kinds, changed rules or anything that changes what a valid document is or means:

1. Open an issue titled `MEP: <short name>` describing the problem, the proposal, and what existing
   documents it affects.
2. Once there is rough agreement, add `meps/NNNN-<short-name>.md` (see [GOVERNANCE.md](GOVERNANCE.md)).
3. The pull request that implements it updates, together:
   - `spec/draft/` (normative text),
   - the Rust types (and regenerates `schema/draft/schema.json` and `bindings/typescript` with
     `UPDATE_SCHEMAS=1 cargo test`),
   - `conformance/` fixtures (one valid or invalid case per new rule),
   - `CHANGELOG.md`.

## Working on the code

```sh
cargo test --workspace                 # unit tests, examples, conformance fixtures, schema freshness
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
UPDATE_SCHEMAS=1 cargo test            # after changing types
UPDATE_FIXTURES=1 cargo test           # after an intended change to resolve/expand or importer output; review the diff
```

Functions stay at 30 lines or fewer, files under 300. Tests never call a model or a network
service.

## Conduct

Everyone taking part follows the [Code of Conduct](CODE_OF_CONDUCT.md).

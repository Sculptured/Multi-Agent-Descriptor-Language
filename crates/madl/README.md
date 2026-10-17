# madl

The reference implementation of the [Multi-Agent Descriptor Language](https://github.com/Alexander-Aghili/Multi-Agent-Descriptor-Language):
document types, a loader (YAML, JSON, `$ref`), a validator with stable error codes, pattern
expansion, settings resolution, Mermaid output, and importers for Google ADK agent configs and
Microsoft Agent Framework declarative files.

```rust
let doc = madl::load(std::path::Path::new("team.madl.yaml"))?;
let report = madl::validate(&doc);
for d in &report.diagnostics {
    eprintln!("{d}"); // error[restriction-loosened] /agents/1/settings/restrictions/budget/tokens: …
}
let expanded = madl::expand(&doc);
let settings = madl::resolve(&expanded);           // every agent's effective settings, with sources
let chart = madl::mermaid::flowchart(&expanded);   // a Mermaid flowchart

let imported = madl::convert::maf::import(std::path::Path::new("workflows/support"))?;
for note in &imported.notes {
    eprintln!("{note}"); // approximated Support.yaml#/trigger/actions/2: …
}
let doc = imported.document;                       // or madl::convert::adk::import(…)
```

The types generate the published JSON Schema (`schemars`) and TypeScript types (`ts-rs`).

License: Apache-2.0.

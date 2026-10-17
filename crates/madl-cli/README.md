# madl-cli

The `madl` command for [MADL](https://github.com/Alexander-Aghili/Multi-Agent-Descriptor-Language) documents.

```sh
cargo install --git https://github.com/Alexander-Aghili/Multi-Agent-Descriptor-Language madl-cli

madl validate team.madl.yaml        # exit 1 on errors; --json for JSON lines
madl resolve  team.madl.yaml        # effective settings per agent, with sources
madl expand   team.madl.yaml        # $refs resolved, patterns expanded (--json for JSON)
madl graph    team.madl.yaml        # Mermaid flowchart
madl import adk ./my_agent          # a Google ADK agent config tree as MADL
madl import maf ./workflow          # Microsoft Agent Framework declarative files as MADL
madl schema                         # the JSON Schema
```

`import` prints the document on stdout, and on stderr every part of the source that was
`approximated` or `unsupported`, with its file and JSON Pointer; `--json` prints both as one object.
It fails if the source can't be read or the result is not valid MADL.

`--allow-outside` lets `$ref` reach files outside the document's folder.

License: Apache-2.0.

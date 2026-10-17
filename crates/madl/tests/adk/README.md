# ADK importer fixtures

Each folder is an ADK agent config tree, with the document (`expected.madl.yaml`) and notes
(`expected.notes`) that `madl import adk` must produce for it.

All folders except `research_team` are copied unchanged from
[google/adk-python](https://github.com/google/adk-python) `contributing/samples` at commit
`4dbb55d0a7e02f6d654244e42506bc220ba285ba` (Apache-2.0, copyright Google LLC). `research_team` is
ours: it covers what the samples don't (parallel branches, refused transfers, an inherited model,
an MCP tool filter, a loop around one agent).

# Agent Framework importer fixtures

Each folder holds Microsoft Agent Framework declarative files in `src`, with the document
(`expected.madl.yaml`) and notes (`expected.notes`) that `madl import maf <folder>/src` must
produce for them.

All folders except `edge_cases` are copied unchanged from
[microsoft/agent-framework](https://github.com/microsoft/agent-framework) at commit
`44fe223da7ea6853eefb7053e25c2c213a2e5c15` (MIT, copyright Microsoft Corporation; the license is in
`LICENSE.agent-framework`): `declarative-agents/workflow-samples`, `declarative-agents/agent-samples`
and `dotnet/samples/03-workflows/Declarative/InputArguments`. `edge_cases` is ours: it covers what
the samples don't (a workflow in the Python loader's shape, `If` and `Foreach` with `BreakLoop`, a
`GotoAction` to a missing action, secrets written in a file, MCP approvals and allowed tools, an
unknown action, a Prompt agent the workflow never invokes, a second definition of a name, and files
that are not Agent Framework files).

# Ports and Adapters

Ports define the runtime boundary for external data. They live in
`lib/ports/phenome-ports/` and use domain types only.

Adapters implement ports for concrete systems. The Bootstrappo adapter
lives in `lib/adapters/phenome-adapter-primer/` and translates external types into
normalized domain types. Additional adapters live in:
- `lib/adapters/rotappo-adapter-analytics/`
- `lib/adapters/rotappo-adapter-ml/`
- `lib/adapters/rotappo-adapter-notification/`

Rules:
- UI/CLI do not import adapters directly, except `rotappo-ui-terminal` calling
  bootstrappo adapter command handlers for the bootstrappo CLI surface.
- Adapters do not import UI/CLI.
- Ports stay free of adapter-specific types.

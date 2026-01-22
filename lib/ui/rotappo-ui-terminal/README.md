# rotappo-ui-terminal

Layer: interfaces

Purpose:
- CLI formatting and dispatch wiring for the bootstrappo surface.

Dependencies:
- rotappo-ui-presentation
- rotappo-application
- phenome-ports

Boundaries:
- No ratatui dependencies.
- CLI dispatch may call bootstrappo adapter handlers.

# phenome-ui-terminal

Layer: interfaces

Purpose:
- CLI formatting and dispatch wiring for the primer surface.

Dependencies:
- phenome-ui-presentation
- phenome-application
- phenome-ports

Boundaries:
- No ratatui dependencies.
- CLI dispatch may call primer adapter handlers.

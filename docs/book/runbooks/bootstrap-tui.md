# Bootstrap TUI (Bootstrappo)

## Purpose

Document how the Bootstrappo bootstrap TUI is wired and how to add new panels
safely without breaking the event-driven UI contract.

## Current layout

- Entry point: `lib/ui/rotappo-ui-tui/src/bootstrap/app.rs`
- Terminal loop: `lib/ui/rotappo-ui-tui/src/bootstrap/terminal.rs`
- Shared loop helper: `lib/ui/rotappo-ui-tui/src/terminal.rs`
- Panels: `lib/ui/rotappo-ui-tui/src/bootstrap/panels/`
- State: `lib/ui/rotappo-ui-tui/src/bootstrap/state.rs`

The bootstrap UI renders:
- Header panel
- Dependency tree panel
- Status panel
- Summary overlay (completion view)
- Control menu overlay
- Logs overlay (from the control menu)

## Adding a new panel

1) Create a new panel module in
   `lib/ui/rotappo-ui-tui/src/bootstrap/panels/`.
2) Add it to `lib/ui/rotappo-ui-tui/src/bootstrap/panels/mod.rs`.
3) Update `BootstrapApp::render` to include the panel in the layout.
4) If the panel needs input handling, add keys to
   `BootstrapApp::handle_key_event`.
5) If the panel depends on Bootstrap state, extend
   `lib/ui/rotappo-ui-tui/src/bootstrap/state.rs`.

## Data flow

- `phenome-ports` defines `BootstrapPort` and component state shapes.
- `phenome-adapter-primer` subscribes to bootstrappo events and updates
  `ComponentState`.
- The TUI reads state via `PortSet::bootstrap` every tick (200ms).

## Keyboard shortcuts

- `m` opens the control menu.
- `e` toggles component detail expansion.
- `c` collapses/expands the selected layer in the tree.
- `q` quits the TUI (or closes the log overlay).
- `Esc` closes overlays.

The control menu exposes context-aware actions such as View Logs, Skip, Retry,
and Adjust Timeout based on the selected component state.
Adjust Timeout opens a seconds input; press Enter to apply or Esc to cancel.

## Safety and UX notes

- Panels must be read-only; they should never mutate bootstrap state directly.
- Use existing keyboard shortcuts and avoid conflicts (m, e, c, q, arrows).
- Keep rendering O(n) in number of visible rows to avoid lag on 30+ components.

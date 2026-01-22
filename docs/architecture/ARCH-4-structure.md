# ARCH-4 Structure Canon + Module Index

## Purpose
Define the canonical repository layout for rotappo, including module
boundaries, ownership, and where shared macros/helpers live. This document
anchors ARCH-4 and aligns the distributed TUI adapter with the hex app
adapter model (bootstrappo today).

## Layout Principles
1) Layers are explicit: domain -> ports -> application -> adapters -> interfaces.
2) Public APIs are narrow and documented at the crate root.
3) Interfaces do not import adapters (except CLI dispatch in ui-terminal).
4) Shared helpers live in a single, intentional home.
5) Each crate has a clear owner and responsibility.

## Canonical Directory Layout
```
lib/
  core/
    phenome-domain/
    phenome-ports/
    rotappo-application/
    phenome-adapter-primer/
    rotappo-adapter-analytics/
    rotappo-adapter-ml/
    rotappo-adapter-notification/
    rotappo-ml/
  ui/
    rotappo-ui-presentation/
    rotappo-ui-core/
    rotappo-ui-terminal/
    rotappo-ui-tui/
src/
  bin/
    cli.rs
    tui.rs
    analytics-service.rs
    ml-service.rs
  lib.rs
docs/
  architecture/
  book/
```

## Module Index (Crates)
| Path | Layer | Responsibility | Depends on | Owner |
| --- | --- | --- | --- | --- |
| `lib/domain/phenome-domain` | domain | Core models + invariants | none | core |
| `lib/ports/phenome-ports` | ports | Port traits + contracts | domain | core |
| `lib/runtime/rotappo-application` | application | Runtime orchestration | domain, ports | core |
| `lib/adapters/phenome-adapter-primer` | adapters | Bootstrappo integration | domain, ports | integrations |
| `lib/adapters/rotappo-adapter-analytics` | adapters | Analytics service adapter | domain, ports | integrations |
| `lib/adapters/rotappo-adapter-ml` | adapters | ML service adapter | domain, ports | integrations |
| `lib/adapters/rotappo-adapter-notification` | adapters | Notification adapter | domain, ports | integrations |
| `lib/runtime/rotappo-ml` | core | ML models + inference helpers | domain | ml |
| `lib/ui/rotappo-ui-presentation` | presentation | Shared formatting + logging | domain, ports | interfaces |
| `lib/ui/rotappo-ui-core` | ui-core | Framework-agnostic UI contracts | domain, ports | interfaces |
| `lib/ui/rotappo-ui-terminal` | interfaces | CLI rendering + dispatch | presentation, application, ports | interfaces |
| `lib/ui/rotappo-ui-tui` | interfaces | Ratatui adapter + TUI UI | presentation, application, ports | interfaces |

## TUI Module Index (Adapter)
`lib/ui/rotappo-ui-tui/src/` is split into:
- `app/`: input, navigation, and state transitions.
- `layout/`: grid specs + resolver helpers.
- `panels/`: renderers for active views and overlays.
- `bootstrap/`: bootstrappo-specific bootstrap TUI panels + state.
- `state/`: UI state types shared across panels and handlers.
- `util/`: rendering helpers that stay within the TUI adapter.

## Macro and Helper Location Rules
- Layer-specific macros live in the layer that owns them.
- Cross-layer macros must be proposed in ARCH-4B and documented.
- Helpers that are UI/CLI agnostic belong in `rotappo-ui-presentation`.
- Ratatui or crossterm helpers stay in `rotappo-ui-tui`.
- Macro inventory and usage guidance lives in `docs/architecture/ARCH-4-macros.md`.

## Ownership and Boundaries
- Each crate must document its responsibility and allowed imports at its root.
- Adapters implement ports and do not import UI crates.
- Interfaces depend on ports/application/presentation only.
- The distributed TUI design principles live in
  `docs/architecture/ARCH-4-distributed-tui-design.md`.

## Documentation Requirements
- New crates must add or update README content at the crate root.
- Architecture changes require a short ADR in `docs/architecture`.
- Module responsibility map: `docs/architecture/ARCH-4-module-responsibilities.md`.
- Architecture index: `docs/architecture/README.md`.

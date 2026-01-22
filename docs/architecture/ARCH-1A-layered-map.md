# ARCH-1A Layered Module Map + Target Tree

This document defines the target layered architecture for rotappo and
the module tree that supports it. It is intentionally a map, not a
move action.

## Scope
- Separate domain logic from UI/CLI presentation.
- Make ports/adapters explicit and keep interfaces thin.
- Identify shared presentation helpers that are UI/CLI agnostic.

## Current workspace root (summary)
- lib/domain/
- lib/ports/
- lib/runtime/
- lib/adapters/
- lib/ui/
- src/bin/
- src/lib.rs

## Target layers (rules of use)
1) domain: pure models and invariants; no UI/CLI/adapters
2) application: orchestration and runtime; depends on domain + ports
3) ports: interfaces defined in domain terms; no adapters/UI/CLI
4) adapters: external system integration; implements ports
5) presentation: shared formatting/view-model helpers (UI/CLI agnostic)
6) interfaces: CLI/TUI; only render + I/O + wiring into application

Dependency rules:
- interfaces -> presentation -> application -> domain
- application -> ports
- adapters -> ports + domain
- interfaces do not import adapters directly (use wiring/composition), except
  the bootstrappo CLI dispatch in `rotappo-ui-terminal`.
- domain does not import adapters, interfaces, or presentation
- CLI dispatch in `rotappo-ui-terminal` may call adapter handlers.

ASCII dependency map:

    [interfaces: cli-formatting/tui]
             |
             v
      [presentation]
             |
             v
      [application/runtime] ---> [ports]
             ^                      ^
             |                      |
          [domain] <------------ [adapters]

Composition roots:
- src/bin/cli.rs (bootstrappo CLI entrypoint)
- src/bin/tui.rs (TUI)

## Target module tree (current)

lib/
  domain/
    phenome-domain/
  ports/
    phenome-ports/
  runtime/
    rotappo-application/
    rotappo-ml/
  adapters/
    phenome-adapter-primer/
    rotappo-adapter-analytics/
    rotappo-adapter-ml/
    rotappo-adapter-notification/
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

Notes:
- Interfaces are nested under `lib/ui/`.
- Core adapters live under `lib/adapters/rotappo-adapter-*`.
- The canonical structure map lives in `docs/architecture/ARCH-4-structure.md`.
- `rotappo-ui-presentation` collects UI/CLI-agnostic helpers (formatting,
  log config, view models). UI/CLI should not own these helpers.
- Use `rotappo_ui_terminal` and `rotappo_ui_tui` for
  public imports (no compatibility re-exports).

## Completed moves
- Runtime models -> `lib/domain/phenome-domain/`
- Orchestration -> `lib/runtime/rotappo-application/`
- Formatting/logging -> `lib/ui/rotappo-ui-presentation/`
- Ports -> `lib/ports/phenome-ports/`
- Interfaces -> `lib/ui/rotappo-ui-*`
- Bootstrappo adapter -> `lib/adapters/phenome-adapter-primer/`
- Analytics adapter -> `lib/adapters/rotappo-adapter-analytics/`
- ML adapter -> `lib/adapters/rotappo-adapter-ml/`
- Notification adapter -> `lib/adapters/rotappo-adapter-notification/`
- ML models -> `lib/runtime/rotappo-ml/`

## Known coupling to verify
1) Ports must stay domain-only; avoid adapter types.
2) Presentation must not import adapters.
3) Application must not depend on interface-layer types.

## Next actions (aligned to ARCH-1A/B)
- ARCH-003: split domain vs application modules.
- ARCH-004: move shared presentation helpers into presentation/ and
  update UI/CLI imports.
- ARCH-005: normalize ports to domain types and move adapter-specific
  types behind adapters.

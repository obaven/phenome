# Architecture Overview

Rotappo follows a layered design:

1) domain: pure models and invariants
2) application: orchestration and runtime flows
3) ports: interfaces defined in domain terms
4) adapters: external systems implementing ports
5) presentation: UI/CLI-agnostic formatting and view helpers
6) interfaces: CLI/TUI rendering and I/O

Dependencies flow inward only:

ui-* -> ui-presentation -> application -> domain
application -> ports
adapters -> ports + domain
ui-terminal -> adapter (bootstrappo CLI handlers)

Crate layout (in progress):
- `lib/domain/phenome-domain` (domain types)
- `lib/ports/phenome-ports` (ports)
- `lib/runtime/rotappo-application` (runtime orchestration)
- `lib/adapters/phenome-adapter-primer` (bootstrappo adapter)
- `lib/adapters/rotappo-adapter-analytics` (analytics adapter)
- `lib/adapters/rotappo-adapter-ml` (ML adapter)
- `lib/adapters/rotappo-adapter-notification` (notification adapter)
- `lib/runtime/rotappo-ml` (ML models and inference helpers)
- `lib/ui/rotappo-ui-presentation` (formatting/logging helpers)
- `lib/ui/rotappo-ui-core` (framework-agnostic UI contracts)
- `lib/ui/rotappo-ui-terminal` (CLI formatting + dispatch)
- `lib/ui/rotappo-ui-tui` (ratatui adapter)

Canonical layout details live in `docs/architecture/ARCH-4-structure.md`.

Directional rules are enforced by `tests/crate_boundaries.rs` and
interface checks in `tests/interface_boundaries.rs`.

Composition roots:
- `src/bin/cli.rs` (bootstrappo CLI entrypoint)
- `src/bin/tui.rs` (TUI)

Decision records:
- `docs/architecture/adr/` (ADRs)

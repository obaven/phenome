# Architecture Overview

Phenome follows a layered design:

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
ui-terminal -> adapter (primer CLI handlers)

Crate layout (in progress):
- `lib/domain/phenome-domain` (domain types)
- `lib/ports/phenome-ports` (ports)
- `lib/runtime/phenome-application` (runtime orchestration)
- `lib/adapters/phenome-adapter-primer` (primer adapter)
- `lib/adapters/phenome-adapter-analytics` (analytics adapter)
- `lib/adapters/phenome-adapter-ml` (ML adapter)
- `lib/adapters/phenome-adapter-notification` (notification adapter)
- `lib/runtime/phenome-ml` (ML models and inference helpers)
- `lib/ui/phenome-ui-presentation` (formatting/logging helpers)
- `lib/ui/phenome-ui-core` (framework-agnostic UI contracts)
- `lib/ui/phenome-ui-terminal` (CLI formatting + dispatch)
- `lib/ui/phenome-ui-tui` (ratatui adapter)

Canonical layout details live in `docs/architecture/ARCH-4-structure.md`.

Directional rules are enforced by `tests/crate_boundaries.rs` and
interface checks in `tests/interface_boundaries.rs`.

Composition roots:
- `src/bin/cli.rs` (primer CLI entrypoint)
- `src/bin/tui.rs` (TUI)

Decision records:
- `docs/architecture/adr/` (ADRs)

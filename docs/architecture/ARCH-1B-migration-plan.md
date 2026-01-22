# ARCH-1B Migration Action

This action sequences the layered-architecture refactor while keeping
CLI/TUI behavior stable while updating public paths directly.

## Principles
- Keep domain and application boundaries strict.
- Each stage compiles independently.
- Update call sites directly when module paths move.

## Stage action
1) Presentation extraction
   - Move formatting/logging into `lib/ui/rotappo-ui-presentation/`.
   - Checkpoints: build, CLI output parity smoke.
   - Rollback: restore previous modules.

2) Domain/application split
   - Move runtime models into `lib/domain/phenome-domain/`.
   - Move orchestration into `lib/runtime/rotappo-application/`.
   - Checkpoints: unit tests, TUI smoke.
   - Rollback: collapse modules back into a single runtime module.

3) Port normalization + adapter boundary
   - Ports return domain types only.
   - Adapters translate external types behind ports.
   - UI/CLI depend on runtime + ports, not adapters.
   - Checkpoints: snapshot consistency, CLI output parity.
   - Rollback: restore adapter-facing types in ports.

4) Interface rename (completed)
   - Move interface modules into `lib/ui/`.
   - Update bin imports and internal paths to use `rotappo-ui-*` crates.
   - Checkpoints: compile and CLI/TUI smoke.
   - Rollback: restore original module paths.

## Validation checklist
- `cargo test -p rotappo` (or workspace equivalent) per stage.
- CLI: confirm output parity for action/problem formatting.
- TUI: smoke check layout + key panels render.
- Ensure no domain imports from `interfaces` or `adapters`.

## Rollback strategy
- Revert the latest stage only; update call sites alongside moves.
- Restore port signatures first if adapters/UI/CLI fail to compile.
- Re-run the validation checklist.

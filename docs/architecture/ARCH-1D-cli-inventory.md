# ARCH-1D CLI Surface Inventory (MIG-1D)

This document inventories the current CLI surfaces for rotappo and
bootstrappo. It is the single source of truth for CLI command lists,
flags, output modes, and parity notes during migration.

## Rotappo CLI formatting (no standalone terminal binary)

Source:
- `lib/ui/rotappo-ui-terminal`

Note: The historical `terminal` CLI binary was removed. Rotappo keeps the
formatting helpers in `rotappo-ui-terminal` and hosts the bootstrappo CLI
execution (see below).

Output modes (formatting helpers):
- `plain`
- `json`
- `ndjson`

## Bootstrappo CLI (rotappo)

Rotappo CLI entrypoint:
- `cargo run --features cli,module-bootstrappo --bin cli -- --help`
- Source: `src/bin/cli.rs`
- CLI parsing/dispatch: `lib/ui/rotappo-ui-terminal/src/cli/bootstrappo.rs`
- Command handlers: `lib/adapters/phenome-adapter-primer/src/controller/`
- Source of truth: CLI behavior is defined here; bootstrappo no longer ships CLI logic.

Commands (from MIG-1 scope; verify in bootstrappo):
- `assembly`
- `reconcile`
- `rotate`
- `debug`
- `nuke`
- `cache`
- `generate`
- `visualize`

Flags and output modes:
- Capture from `bootstrappo --help` and `bootstrappo <cmd> --help`
- Document any output format flags (json, yaml, etc.) here

## Mapping (Bootstrappo -> Rotappo)

| Bootstrappo command | Rotappo surface | Notes |
| --- | --- | --- |
| `assembly` | `assembly` | Rotappo assembly renders from snapshot; keep parity with bootstrappo assembly output. |
| `reconcile` | `actions` | Action registry should list reconcile; execution remains in bootstrappo. |
| `rotate` | `actions` | Action registry should list rotate. |
| `debug` | `logs` | Closest surface is logs; map any debug output fields to events. |
| `nuke` | `actions` | Destructive action; ensure it is clearly labeled. |
| `cache` | `actions` | If cache emits state output, consider mapping to `snapshot`/`assembly`. |
| `generate` | `actions` | Action registry should list generate. |
| `visualize` | `snapshot` | Closest surface is snapshot/assembly; confirm expected output. |

## Update checklist

1) Update this inventory when any CLI command or flag changes.
2) Update CLI snapshots/golden tests for intentional output changes.
3) Re-verify mapping table for any renamed or new commands.
4) Keep guardrail checks and CI in sync with the CLI surface.
5) When migrating bootstrappo CLI logic, keep the rotappo CLI in sync.

## META-11 Bootstrappo self-serve migration notes

- Capture CLI help snapshots in the bootstrappo repo:
  - `bootstrappo --help`
  - `bootstrappo <cmd> --help`
  - If a command has subcommands, also capture
    `bootstrappo <cmd> <subcmd> --help`.
- Store snapshots under `primer/tests/fixtures/cli/` with clear
  names (example: `help.top.txt`, `help.assembly.txt`, `help.generate.list.txt`).
- Add a `primer/tests/cli_golden.rs` harness that runs the binary
  via `std::process::Command` and compares stdout to the fixtures.
  Use `env!("CARGO_BIN_EXE_cli")` to locate the test binary.
- Support an update flow (example env var: `UPDATE_CLI_SNAPSHOTS=1`) to
  rewrite fixtures intentionally.
- For optional output snapshots, only use local fixture inputs (assembly,
  config, cache) and avoid live cluster access.
- Update the Bootstrappo section of this doc after any CLI change.
- Keep bootstrappo CLI modules isolated (no rotappo CLI imports).

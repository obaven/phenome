# CLI Runbook

This runbook covers the bootstrappo CLI hosted in rotappo. It is the sole
source of truth for CLI behavior; the bootstrappo repo no longer ships CLI
logic.

## Architecture map
- Entry point: `src/bin/cli.rs`
- CLI parse/dispatch: `lib/ui/rotappo-ui-terminal/src/cli/bootstrappo.rs`
- Command handlers: `lib/adapters/phenome-adapter-primer/src/controller/`
- CLI formatting helpers: `lib/ui/rotappo-ui-terminal/src/format.rs`

Feature flags:
- `cli` enables CLI formatters and the `cli` binary
- `module-bootstrappo` selects the bootstrappo CLI surface

## Setup (rotappo-ui-terminal)
- `rotappo-ui-terminal` provides CLI formatting + dispatch.
- Enable `cli` + `module-bootstrappo` to compile the bootstrappo CLI entrypoint in rotappo.
- Ensure the bootstrappo repo is present at `../primer` for adapter calls.

## Run the bootstrappo CLI
From the rotappo repo root:

```bash
cargo run --features cli,module-bootstrappo --bin cli -- --help
```

Optional GitOps directory override:

```bash
BOOTSTRAPPO_GITOPS_DIR=/path/to/gitops \
  cargo run --features cli,module-bootstrappo --bin cli -- --help
```

Optional config path override:

```bash
BOOTSTRAPPO_CONFIG_PATH=/path/to/bootstrap-config.yaml \
  cargo run --features cli,module-bootstrappo --bin cli -- --help
```

Or via flag:

```bash
cargo run --features cli,module-bootstrappo --bin cli -- --config /path/to/bootstrap-config.yaml
```

## Command reference
All commands run from the rotappo repo root and assume the bootstrappo repo
exists at `../primer`.

Assembly:

```bash
# Validate config/assembly
cargo run --features cli,module-bootstrappo --bin cli -- \
  assembly validate ../primer/data/configs/bootstrap-config.yaml

# Visualize assembly
cargo run --features cli,module-bootstrappo --bin cli -- \
  assembly visualize \
  ../primer/data/configs/bootstrap-config.yaml \
  --view full \
  --format svg \
  --layout dot \
  --output /tmp/bootstrappo-assembly.svg
```

Status / Diff / Explain / Catalog:

```bash
# Status overview (validation + observed signals)
cargo run --features cli,module-bootstrappo --bin cli -- status

# Diff expected vs observed signals
cargo run --features cli,module-bootstrappo --bin cli -- diff

# Explain a single module
cargo run --features cli,module-bootstrappo --bin cli -- explain k3s

# Print catalog YAML to stdout
cargo run --features cli,module-bootstrappo --bin cli -- catalog

# Write catalog YAML to a file
cargo run --features cli,module-bootstrappo --bin cli -- \
  catalog --output /tmp/bootstrappo-catalog.yaml
```

Reconcile:

```bash
# One-shot converge
cargo run --features cli,module-bootstrappo --bin cli -- \
  reconcile --assembly ../primer/data/configs/bootstrap-config.yaml

# Watch mode with caching + parallelism
cargo run --features cli,module-bootstrappo --bin cli -- \
  reconcile \
  --assembly ../primer/data/configs/bootstrap-config.yaml \
  --watch \
  --cache \
  --parallel \
  --concurrency 4

# Override assembly overlay
cargo run --features cli,module-bootstrappo --bin cli -- \
  reconcile \
  --assembly ../primer/data/configs/bootstrap-config.yaml \
  --overlay dev
```

Rotate:

```bash
# Dry-run rotation
cargo run --features cli,module-bootstrappo --bin cli -- \
  rotate ingress \
  --assembly ../primer/data/configs/bootstrap-config.yaml \
  --dry-run
```

Nuke:

```bash
# Dry-run deletion order
cargo run --features cli,module-bootstrappo --bin cli -- \
  nuke --assembly ../primer/data/configs/bootstrap-config.yaml --dry-run

# Aggressive mode (skips confirmation)
cargo run --features cli,module-bootstrappo --bin cli -- \
  nuke --assembly ../primer/data/configs/bootstrap-config.yaml --aggressive
```

Debug:

```bash
# Registry inventory
cargo run --features cli,module-bootstrappo --bin cli -- debug registry

# Assembly execution order
cargo run --features cli,module-bootstrappo --bin cli -- \
  debug assembly-order --assembly ../primer/data/configs/bootstrap-config.yaml
```

Cache:

```bash
# Cache stats
cargo run --features cli,module-bootstrappo --bin cli -- cache status

# Purge cache without prompt
cargo run --features cli,module-bootstrappo --bin cli -- cache purge --force
```

Generate:

```bash
# Generate storage config from local devices
cargo run --features cli,module-bootstrappo --bin cli -- \
  generate storage --min-size 50
```

Visualize:

```bash
# Visualize discovered storage devices
cargo run --features cli,module-bootstrappo --bin cli -- \
  visualize generate storage \
  --min-size 50 \
  --format dot \
  --layout dot \
  --output /tmp/bootstrappo-storage.dot
```

## Assembly visualize (smoke runbook)
1) Choose a config path. From the rotappo root, use the bootstrappo config file:
   `../primer/data/configs/bootstrap-config.yaml`
2) Generate a DOT graph:

```bash
cargo run --features cli,module-bootstrappo --bin cli -- \
  assembly visualize \
  ../primer/data/configs/bootstrap-config.yaml \
  --format dot \
  --layout dot \
  --output /tmp/bootstrappo-assembly.dot
```

Notes:
- `--format svg` or `--format png` require Graphviz installed on PATH.
- If config warnings appear, verify `BOOTSTRAPPO_CONFIG_PATH` or run from the
  bootstrappo repo so relative config paths resolve.

## Change workflow
1) Update CLI args/subcommands in
   `lib/ui/rotappo-ui-terminal/src/cli/bootstrappo.rs`.
2) Implement or update handlers in
   `lib/adapters/phenome-adapter-primer/src/controller/`.
3) Keep adapter runtime ports (`health`, `assembly`, `mapping`) private.
4) Update the inventory doc:
   `docs/architecture/ARCH-1D-cli-inventory.md`.
5) Update snapshots/tests if output changes.

## Validation
- `cargo test --test cli_golden`
- `cargo test --test cli_boundaries`
- `cargo test --features cli,module-bootstrappo --test cli_bootstrappo_visualize`
- `cargo make check-cli-stability`
- `cargo make check-cli-bootstrappo-surface`

## Make tasks (CLI commands)
- `cargo make cli-bootstrappo-help`
- `cargo make cli-bootstrappo-assembly-validate`
- `cargo make cli-bootstrappo-assembly-visualize-dot`
- `cargo make cli-bootstrappo-debug-registry`
- `cargo make cli-bootstrappo-cache-status`
- `cargo make cli-bootstrappo-rotate-dry-run`
- `cargo make cli-bootstrappo-smoke`

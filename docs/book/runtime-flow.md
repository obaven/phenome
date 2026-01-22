# Runtime Data Flow

Runtime orchestration lives in `lib/runtime/rotappo-application/src/runtime.rs`.
It owns the loop that refreshes ports, aggregates domain snapshots,
and exposes state to the interfaces.

At a high level:
1) Ports provide the latest action, health, and events.
2) Application builds a domain `Snapshot`.
3) Presentation formats the snapshot for CLI/TUI.

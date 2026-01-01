# v0.0.3 Kanban Boards (Compact View)

## A. v0.0.3 Release Board (Program)
### Backlog
- BSP-001 — [Epic] Layer 0: Security & Identity Cohesion
- BSP-002 — [Epic] Layer 1: Datastores Cohesion
- BSP-003 — [Epic] Layer 2: Observability Cohesion
- BSP-004 — [Epic] Layer 3: Analytics Cohesion
- BSP-005 — [Epic] Layer 4: Entertainment & Productivity Cohesion
- BSP-006 — [Epic] Layer 5: Infrastructure & GitOps Cohesion
### Ready
### In Progress
### Review
### Blocked
### Done

## B. Layer 0: Security & Identity
### Backlog
- BSP-012 — [Story] Implement Vaultwarden + ESO for Secrets Injection
- BSP-013 — [Story] Implement CrowdSec IPS (Agent + Bouncer)
- BSP-014 — [Story] Implement Kyverno Governance Policies
### Ready
### In Progress
- BSP-011 — [Story] Implement Zero-Trust Access Control (Authelia + Ingress)
### Review
### Blocked
### Done
- BSP-010 — [Story] Implement Sealed Secrets for Cloudflare Credentials (Core)

## C. Layer 5: Infrastructure & GitOps
### Backlog
- BSP-020 — [Story] Implement Hierarchical Namespaces (HNC)
- BSP-021 — [Story] Implement Velero Backups to MinIO
- BSP-022 — [Story] Implement Harbor Registry + Trivy Scanning
- BSP-023 — [Story] Implement KEDA Autoscaling (Prometheus + Kafka)
- BSP-024 — [Story] Implement Cilium Service Mesh + Hubble UI
### Ready
### In Progress
### Review
### Blocked
### Done

## D. Layer 1: Datastores
### Backlog
- BSP-030 — [Story] Deploy CloudNativePG (Postgres + TimescaleDB)
- BSP-031 — [Story] Deploy Redis HA Cluster
- BSP-032 — [Story] Deploy Qdrant Vector Database
- BSP-033 — [Story] Deploy DuckDB / Mongo / Cassandra (GitOps)
### Ready
### In Progress
### Review
### Blocked
### Done

## E. Layer 2: Observability
### Backlog
- BSP-040 — [Story] Implement Vector Observability Pipeline
- BSP-041 — [Story] Deploy Prometheus + Loki (Retention & Storage)
- BSP-042 — [Story] Provision Grafana Dashboards-as-Code
- BSP-043 — [Story] Implement Uptime Kuma Monitoring
### Ready
### In Progress
### Review
### Blocked
### Done

## F. Layer 3: Analytics
### Backlog
- BSP-050 — [Story] Deploy Redpanda Streaming (Tiered Storage)
- BSP-051 — [Story] Deploy Trino Federation Engine
- BSP-052 — [Story] Implement DBT Transformation Pipelines
- BSP-053 — [Story] Deploy OpenMetadata Governance
### Ready
### In Progress
### Review
### Blocked
### Done

## G. Layer 4: Entertainment & Productivity
### Backlog
- BSP-060 — [Story] Deploy the "Arr" Suite (Radarr, Sonarr, etc.)
- BSP-061 — [Story] Deploy Tdarr + GPU Transcoding
- BSP-062 — [Story] Deploy Jellyfin + Jellyseerr + Homepage
- BSP-063 — [Story] Deploy AppFlowy + Stalwart Mail
- BSP-064 — [Story] Implement Notification Bridge (ntfy + Apprise)
### Ready
### In Progress
### Review
### Blocked
### Done

## H. Meta 0: Composer Upgrade (Event-driven Reconcile + Rotations)

**Goal:** Replace hardcoded bootstrap sequencing with a **watchers-first, event-driven reconciler**:
- **Event-driven requeue loop** (watchers trigger reconcile work).
- **Converge mode** exits when satisfied; **daemon mode** via `--watch` continues for drift + late arrivals.
- **Hybrid execution:** kube-rs for state/capabilities; kubectl/helm subprocess for apply + verification fallback.
- **Rotations:** deterministic managed-only backfills when providers arrive late (ingress/tls/dns/policy/lb).
- **Safety:** only apps with `expose.enabled=true` (and managed_by_bootstrappo=true) are rotated.

### Backlog


### Ready
### In Progress
### Review
### Blocked
### Done
- ✅ BSP-065 — [Epic] Event-driven Reconciler: watch → scaffold/apply → rotate
- ✅ BSP-066 — [Story] Declarative Bootstrap Plan (DAG + gates + capabilities + rotations)
- ✅ BSP-067 — [Story] Reconcile Engine + CLI (one-shot + optional watch)
- ✅ BSP-068 — [Story] Exoskeleton Layout Contract (deterministic generation targets)
- ✅ BSP-069 — [Story] Rotation Engine (ingress + tls + dns + policy + lb)
- ✅ BSP-070 — [Story] Capability Detection (hybrid kube-rs + subprocess verify)
- ✅ BSP-071 — [Task] Integration scenarios + docs
- ✅ BSP-072 — [Task] Migrate Components to Plan-Driven Execution
- ✅ BSP-073 — [Task] Deprecate Legacy Bootstrap Command
- ✅ BSP-074 — [Task] Remove Legacy Ops Modules
- ✅ BSP-075 — [Task] Remove Layer Trait
- ✅ BSP-076 — [Task] Clean Up Composer Patterns (consolidate utilities)
- ✅ BSP-077 — [Task] Modernize CLI Commands
- ✅ BSP-078 — [Task] Plan Validation
- ✅ BSP-079 — [Task] Clean Up CLI Commands
- ✅ BSP-080 — [Story] Dynamic Plan Visualization Engine

## I. Meta 2: Driver Consolidation
### Backlog
- BSP-092 — [Story] Split Main into Distributed Commands
- BSP-093 — [Story] Segregate Ops from Drivers (Context Separation)
- BSP-094 — [Story] Implement Driver Registry (Inventory)
- BSP-095 — [Story] Centralize Config Loading (Schema + Validation)
- BSP-098 — [Task] Migrate Storage Layer to Drivers
### Ready
### In Progress
### Review
### Blocked
### Done
- BSP-096 — [Story] Composable Driver Base + Macro (branch:v0-0-3/meta-2/BSP-096)
- BSP-097 — [Task] Migrate Networking Layer to Drivers (branch:v0-0-3/meta-2/BSP-097)

## J. Meta 3: MCP Plugin Integration
### Backlog
- BSP-100 — [Epic] Implement Bootstrappo MCP Server
### Ready
### In Progress
### Review
### Blocked
### Done

## K. Meta 4: Cohesion and Alignment
### Backlog
- BSP-113 — [Task] Kanban Hygiene and Dependency Map Refresh
- BSP-114 — [Story] Layer Plan Scaffolding (No Implementations)
- BSP-115 — [Task] Driver Authoring Kit (Docs + Templates)
- BSP-116 — [Task] Config Schema Placeholders for Upcoming Layers
- BSP-117 — [Task] Pre-layer Verification Guardrails
- BSP-118 — [Task] Operational Runbook Completion (Rotate + Nuke)
### Ready
### In Progress
- BSP-112 — [Epic] Meta-4: Layer Readiness Hardening
### Review
### Blocked
### Done
- ✅ BSP-101 — [Epic] Meta-4: Cohesion and Alignment
- ✅ BSP-102 — [Story] Migrate Remaining Legacy Drivers to Recipes
- ✅ BSP-103 — [Task] Expose DriverSpec Metadata in Debug Output
- ✅ BSP-104 — [Story] Wire Reconcile --watch to Event-driven Loop
- ✅ BSP-105 — [Story] Implement TLS/DNS Rotations with Managed Outputs
- ✅ BSP-106 — [Story] Align GitOps Pathing with Exoskeleton Layout
- ✅ BSP-107 — [Task] Normalize Driver Dependencies and Registry Validation
- ✅ BSP-108 — [Task] Update Layer Kanban CLI Surface to Plan-driven Commands
- ✅ BSP-109 — [Task] Plan Validation Completeness
- ✅ BSP-110 — [Story] Registry Graph + Dependency Validation
- ✅ BSP-111 — [Task] Reconcile Workflow Documentation Alignment

## L. Meta 5: Kro Helm Compatibility
### Backlog
- BSP-119 — [Epic] Meta-5: Kro Helm Template Compatibility
- BSP-120 — [Story] Render-Mode Sanitization for Kro Template Generation
- BSP-121 — [Task] Reusable Kro Sanitization Utility
- BSP-122 — [Task] Kro Render Regression Tests for Helm Bash Scripts
- BSP-123 — [Task] Audit CEL/Bash Collisions and Document Guidance
### Ready
### In Progress
### Review
### Blocked
### Done

## M. Meta 7: Storage Designation Automation
### Backlog
- BSP-129 — [Epic] Meta-7: Automated Storage Designation + Propagation
- BSP-130 — [Story] Storage Provider Inventory + Capability Scan
- BSP-131 — [Task] Storage Designation Schema + Policy Defaults
- BSP-132 — [Story] Storage Designation Propagation Engine
- BSP-133 — [Task] Integrate Storage Designations into Apps
### Ready
### In Progress
### Review
### Blocked
### Done

## N. Meta 8: Resource Designation + Provider System
### Backlog
- BSP-134 — [Epic] Meta-8: Automated Resource Designation + Provider System
- BSP-135 — [Story] Cluster Bootstrap + Provider Detection
- BSP-136 — [Task] Node Role Designation from Cluster Topology
- BSP-137 — [Story] Resource Designation Schema + Provider Mapping
- BSP-138 — [Task] Resource Designation Propagation to Apps
### Ready
### In Progress
### Review
### Blocked
### Done

## O. Meta 9: Bootstrap Acceleration + Free Lunch Wins
### Backlog
- BSP-139 — [Epic] Meta-9: Bootstrap Acceleration + Free Lunch Wins
- BSP-140 — [Story] Bootstrap Timing Baseline + Hotspot Report
- BSP-141 — [Story] Fast-path Plan Execution (Skip No-ops + Multi-step)
- BSP-142 — [Task] Plan Step Change Detection + Render Cache
- BSP-143 — [Story] Parallelize Independent Steps (DAG Batches)
### Ready
### In Progress
### Review
### Blocked
### Done

## P. UI-1: Rotappo Multi-Surface UX Platform
### Backlog
- BSP-144 — [Epic] UI-1: Rotappo Multi-Surface UX Platform
- BSP-145 — [Epic] UI-1A: Rotappo Runtime + State API
- BSP-146 — [Epic] UI-1B: Rotappo CLI Parity + Action Registry
- BSP-147 — [Epic] UI-1C: Rotappo Terminal UI (Fullstack Terminal App)
- BSP-148 — [Epic] UI-1D: Rotappo Web UI + API
- BSP-149 — [Epic] UI-1E: Rotappo Cross-Platform Desktop App
- BSP-150 — [Story] Rotappo Runtime State Snapshot + Aggregation
- BSP-151 — [Story] Rotappo Runtime Event Bus + Log Stream
- BSP-152 — [Story] Rotappo Runtime Action Router + Safety Gate
- BSP-153 — [Task] Rotappo Config Schema + Persistence
- BSP-154 — [Story] Rotappo CLI Action Registry + UI Mapping
- BSP-155 — [Story] Rotappo CLI Structured Output Modes
- BSP-156 — [Task] Rotappo CLI Prompt/Confirm + Dry-run Guardrails
- BSP-157 — [Story] Rotappo TUI Shell + Layout Scaffolding
- BSP-158 — [Story] Rotappo TUI Plan + Gate Status View
- BSP-159 — [Story] Rotappo TUI Action Panel + Confirmations
- BSP-160 — [Task] Rotappo TUI Log + Event Stream Panel
- BSP-161 — [Story] Rotappo Web API Server + Event Streaming
- BSP-162 — [Story] Rotappo Web UI Shell + Routing
- BSP-163 — [Story] Rotappo Web Dashboard: Plan, Capabilities, Storage
- BSP-164 — [Task] Rotappo Web Actions + Access Guardrails
- BSP-165 — [Story] Rotappo Desktop Shell + Webview Integration
- BSP-166 — [Task] Rotappo Desktop Local Runtime Bridge
- BSP-167 — [Task] Rotappo Desktop Packaging + Updates + Notifications
### Ready
### In Progress
### Review
### Blocked
### Done

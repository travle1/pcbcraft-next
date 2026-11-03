# Architecture decisions and review record

## Current product decision — November 3, 2026

The user explicitly clarified that **PCBcraft is a standalone application**.
[Accepted decision](adr/0001-standalone-application.md), [framework](framework.md),
[implementation plan](implementation-plan.md). This supersedes the earlier
compiled-in SolidCraft workspace and mechanical Session reuse recommendation.
PCBcraft owns its executable, settings, Electronics session and native project
lifecycle. Family code is reused through independently consumable libraries.
This correction has not received a new Astra review; do not claim otherwise.

## Historical review and experiment

On November 2, Astra reviewed the architecture packet and amended the route
after PR #11360 source/lockfile/CI evidence. It recommended direct migrated-host
reuse and a typed host Session extension, superseding an earlier extraction-first
proposal. Those deployment/ownership recommendations assumed the wrong product
boundary and are no longer instructions. The reviewed domain and correctness
contracts below remain applicable to PCBcraft's own session.

Astra's initial packet review did not inspect the user's image pixels or compile
an app. Its later review inspected PR evidence, not the local startup experiment.
On November 3 a separate pinned SolidCraft checkout built and rendered locally;
its CLI failed E0432. The [reference record](host-baseline.md) preserves those
results. That screenshot was SolidCraft, not PCBcraft. Neither the successful
launch nor the CLI failure is a standalone PCBcraft acceptance result.

## Retained contracts under standalone ownership

| Concern | Current requirement |
| --- | --- |
| Reuse | Maximum independent family-library reuse; qualified APIs/licenses/pins/closures, focused extraction requests; no app/mechanical-engine prerequisite |
| Geometry | Direct pinned SolidCore/shared layers, explicit required capabilities, no Truck |
| Session | One PCBcraft Electronics authority for aggregate/operations/revisions/undo/save and GUI/CLI/MCP |
| First complete workflow | Own window, three views, one original component, browser/inspector/layers, cross-selection, edit/undo/redo, .pc reopen and operation parity |
| Identity | Occurrence-qualified sheets/components/units/terminals/pads; non-bijective mappings; explicit remaps and net split/merge lineage or unresolved correspondence |
| Transactions | New incarnation on reopen, monotonic revisions through undo/redo, bounded idempotency, plan-local IDs, owned handles/full revalidation and atomic candidate checks |
| Interaction | Canonical selection, no re-emitted projected intent, separate hover/highlight and scene-generation guards |
| Jobs/outputs | Fingerprinted current/stale results; immutable exports may finish at older revision; cancellation before publish versus AlreadyCompleted; qualified reproducibility |
| Geometry/variants | Narrow capability matrix and asymmetric transforms; role-specific DNP policy may retain copper/paste |
| Libraries | Reproducible offline definitions, durable identities/pinned revisions, explicit mappings/overrides and reviewable upgrades |

## Remaining proof gates

1. W0: create PCBcraft's own executable boundary and qualify independent shared
   components; build/run without a SolidCraft app or sibling checkout. No false
   claim of an extracted SDK. The upstream experiment does not satisfy this gate.
2. F1: settle IDs, quantities/overflow/tolerances and operation/retry/snapshot
   contracts. Original fixtures cover repeated occurrences, multiunit one physical
   part, non-bijective mappings, stale ECO and jobs after undo/reopen.
3. W1/H0: actual PCBcraft UI and single-session edit/undo/redo/save/reopen,
   cross-selection and GUI/headless parity, matching the user's FigCraft designs.
4. Later geometry/check/output profiles need specific capability and independent
   fixture evidence. All-upstream-tests success is not a blanket prerequisite.

No host app changes, dependency additions, remote, agent queue, commits, pushes,
PRs or upstream messages were created by the standalone correction. The starter
remains a foundation library; an executable is next work, not an existing feature.

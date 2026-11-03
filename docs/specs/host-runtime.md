# Standalone runtime and shared-library reuse

Status: deployment corrected November 3, 2026; runtime contracts retain applicable
Astra review findings. [Standalone decision](../adr/0001-standalone-application.md),
[framework](../framework.md), [review record](../review-record.md).
The filename is retained for existing links; this spec does not require a host app.

## Application and dependency boundary

PCBcraft owns its executable, window, settings, project tabs and desktop lifecycle.
It consumes verified independent family libraries and SolidCore; it neither runs
inside SolidCraft nor launches it as a backend. The SolidCraft checkout is source
reference only. Existing ui-egui public exports still pull in engine dependencies;
public visibility alone does not make a component suitable for this boundary.

Reuse icons/themes/controls and generic services through real independent APIs.
For private components, specify the smallest useful library extraction or a
licensed bounded temporary adapter with provenance/replacement plan. Do not copy
the whole application or wait for a general plugin platform. PCBcraft owns the
application composition even when it shares most of its visual building blocks.

**One PCBcraft Electronics session** owns the whole project aggregate, semantic
operations, transactions, revisions, undo/redo, dirty state and save/recovery
coordination. GUI/CLI/MCP and all views use this authority. Independent history,
job or IO helpers may support it without importing a mechanical Session/Document
or evaluating geometry for pure electrical edits. No parallel per-view authority.

No initial cross-document ECAD/MCAD transaction is included. Mechanical links
are explicit versioned references/snapshots; .sc export remains a later adapter.
See [file UX](persistence.md#project-files-and-user-facing-type).

## Semantic operation boundary

UI gestures and MCP/CLI decode to the same typed request. PCBcraft's Electronics session
validates domain changes and commits once. PCBcraft's UI presents commands,
results and lifecycle state through adapters to that session. Simple edits need not
create a public two-stage plan object; ECO/library upgrades and reviewed batches
benefit from explicit prepare/apply. Hover/camera actions need no transaction plan.
Interactive previews are transient; Escape cancels without modifying the aggregate.

The [AI authoring contract](ai-authoring.md) makes in-app Assistant/external MCP an
early consumer of this boundary, with versioned reference guidance and structured
context. A model proposes intent; deterministic schematic arrangement/routing
produces editable geometry under locks and readability constraints. Inspect,
prepare and apply are distinct. Neither prompting nor MCP transport provides
electrical correctness, atomicity or readable arrangement by itself.

Illustrative contracts, not production Rust or shipped third-party interfaces:

```text
SnapshotRef = { project_id, session_incarnation, revision, content_hash }
ChangeRequest = { request_id, operation_version, operation,
                  typed_parameters, expected_snapshot }
ChangePlan = { base, dependency_hashes, provisional_ids, affected_refs,
               changes, diagnostics, required_capabilities,
               candidate_poststate_hash }
ApplyRequest = { request_id, expected_snapshot, server_plan_handle }
CommitResult = { request_id, previous_revision, revision,
                 created_refs, changed_refs, removed_refs,
                 findings, synchronization_state }
prepare(snapshot, resolved_inputs, request) -> ChangePlan | DomainError
session.apply(apply_request) -> CommitResult | Conflict | Unsupported | Invalid
```

Preparation makes no persistent allocation or mutation. Plan-local provisional
IDs are stable inside that plan; accepted IDs are mapped/validated at commit, with
no global allocator consumption on rejection. The service owns a bounded plan
handle tied to session/revision/dependency context. If a protocol accepts a supplied
plan instead, revalidate the entire operation, dependencies and candidate state;
a caller's hash is never authority to apply arbitrary changes.

Commit validates the current context/revision, capabilities and dependencies,
constructs and checks the complete candidate post-state, then replaces it
atomically with one undo entry and one revision event. Failure preserves document,
allocation state and history. Resolve external inputs to immutable hashes before
commit. Runtime session incarnation changes on reopen; revisions advance for every
successful commit, undo and redo, even when a prior content hash reappears.
Persisted project identity and runtime incarnation serve different purposes.

Request IDs provide bounded idempotency: within the advertised session/retry
window, the same ID and identical normalized request returns the original result
without another mutation; reuse with different content rejects. Inspect the
operation result before retrying an uncertain apply. Reopen starts a new
incarnation, so a client must re-query/reconcile rather than blindly replay a
previous-session request. Exact retention limits are part of F1's protocol contract.

Operations distinguish inspection, authored edits, transient interaction, analysis
and artifact generation. Proposed `eda.*` IDs belong to PCBcraft's semantic command registry, shared by
its GUI/CLI/MCP adapters. Schemas expose units, limits, required
capabilities, stable references and supported versions. Undo restores the whole
aggregate and correspondence; derived results are invalidated or reused only by
matching fingerprints. Undo is not persistent audit/replay. Future crash journals
must record versioned operations and pinned inputs, not mutable catalog paths.

## Canonical interaction state and cross-probing

The Electronics interaction service owns one canonical selection store. Only a
new user/API action submits a selection intent and creates a selection event.
Views map this state into their own visuals without re-emitting a selection
intent. Separate hover/highlight stores avoid conflating transient emphasis with
persistent selection. Selection is not electrical document data or undo history.

| Event | Owner | Contract |
| --- | --- | --- |
| DocumentCommitted | Chosen authoritative session | One new revision plus changed refs, invalidation categories and reconciliation state |
| SelectionChanged | Canonical interaction store | Semantic refs, occurrence context, origin and propagation token; consumers project only |
| HighlightChanged / HoverChanged | Interaction store | Separate visual states, no document dirtying |
| AnalysisProgress / AnalysisReady | Job service | Job/input fingerprints, coverage and freshness |
| SynchronizationChanged | Reconciliation service | Why representations agree, differ, conflict or are not evaluable |
| DependencyChanged | Resolver/source watcher | Availability notification, never automatic design upgrade |

A semantic selection identifies component occurrence/terminal, symbol unit and
sheet occurrence, board footprint/pad/track/net, or an explicitly linked model.
Revisioned forward/reverse maps handle several graphics/pads per terminal,
repeated sheet occurrences and missing/unlinked counterparts. Net highlighting
separates intended membership from disconnected copper islands. Never pick an
arbitrary occurrence because names match. Auto-reveal/focus is a preference; it
does not force tab switching or reveal hidden layers without policy.

A scene pick includes scene generation, source snapshot and mapped semantic ref;
validate them against current interaction/document context before accepting it.
Remove deleted targets from selection/highlight. Propagation tokens and the
single-store rule prevent schematic/PCB/3D echo loops. Local live cross-probing
does not promise network collaboration or cross-document atomic editing.

## Workspace composition and shared controls

PCBcraft's desktop composition presents document tabs, workspace/ribbon, panels,
command search, themes/icons, navigation and lifecycle state. Schematic/Layout/
Render views share one Electronics session and correspondence. Sheet navigation
uses occurrence IDs; views do not own separately saved documents.

Use independently consumable family controls. App-private helpers need a bounded
extraction or licensed adapter before use; do not import the full ui-egui/engine
chain to obtain them. Keep 2D navigation planar and reuse independent 3D camera/
render services where verified. Semantic icon IDs, masters, palette aliases and
small variants retain source provenance. The local icon catalog has no upstream
consumer API yet. The first slice uses a [narrow local SVG adapter](../../assets/family/README.md)
with unmodified licensed resources. Inspect actual output under [UI acceptance](workspace-ui.md).

## Jobs, concurrency and freshness

Imports, indexing, compilation, checks, fills, geometry and output jobs read
immutable snapshots plus bounded resolved inputs. Shared progress/cancel utilities
can be reused only if independently separable; workers cannot mutate the session.
Serialize commits through Electronics authority and bound queue/memory/parallelism.

A fingerprint includes project and session incarnation, authored revision/hash,
variant, dependencies, rules/profile and algorithm versions. Distinguish:

- Current analysis: publish as current only if matching; otherwise discard or show
  explicitly stale, never green/current merely because no findings were returned.
- Mutation proposals: reject stale bases; a completed computation does not authorize
  application to a changed session or a reopened document.
- Immutable snapshot exports: may complete for revision R while the editor is at
  R+1; all files/checks use captured R, and the package is labeled with R.

Cancellation before atomic publication prevents publication. If publication won
the race, return AlreadyCompleted with the artifact/result identity rather than
claiming cancellation removed a completed package. Failed jobs publish no partial
mutation or complete-package marker. Progress describes a phase, not validity.
Reproducibility is qualified by profile/tool/algorithm versions and environment
assumptions; no universal cross-platform byte identity is promised.

Error categories: InvalidInput, MissingDependency, Conflict, Unsupported,
Cancelled, IOFailure and InternalFailure with bounded detail/affected refs. Tests
control completion/cancel signals and cover edit/undo/reopen/document-switch races,
not wall-clock sleeps. Keep native thread/filesystem assumptions out of the core;
browser scheduling support requires its own verified adapter and measurements.

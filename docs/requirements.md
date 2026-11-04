# Requirements and first decisions

These express Travis's direction for a new project, without inheriting the old
fork's implementation model or contribution policy.

## Standalone application and shared libraries

PCBcraft is a standalone application with its own executable, window, settings,
document lifecycle and packaging. The user correction supersedes the compiled-in
host proposal; [accepted decision](adr/0001-standalone-application.md). The SolidCraft
app, its mechanical engine and its source checkout are not PCBcraft prerequisites.

Maximize reuse of verified family icons, themes, controls and independently useful
services. App-private facilities need focused library extraction or a reviewed
licensed adapter, not embedding the whole app. PCBcraft composes its own egui
workspace from these building blocks and the user's FigCraft layouts. Audit each
selected dependency closure, API and license; an app-coupled public symbol is not
a standalone library. Do not invent future packages or a generic plugin platform.

One PCBcraft Electronics session owns the project aggregate, transactions,
revisions, undo/redo, dirty state and save/recovery coordination. All views and
GUI/CLI/MCP use it. Reuse independent utilities under that ownership without
depending on the mechanical Session, Document, feature tree or command registry.
No per-view histories or initial cross-document ECAD/MCAD transactions.

Use direct pinned SolidCore/shared-layer libraries for qualified geometry.
Only specific dependencies/capabilities gate a slice; the whole SolidCraft build,
migration and CLI are not gates. Missing reusable planar/3D operations need a
narrow shared-library request and original fixtures, never Truck or invented APIs.
Preserve EDA ownership of logical components/terminals/nets, schematic intent,
board realization, rules and mappings. Geometry handles/caches cannot own them.

## First-class AI-assisted authoring

AI/MCP is an early authoring path, not a late chat add-on. In-app Assistant and
external agents inspect and edit through the same Electronics session as manual
tools. Supply versioned EDA guidance, resolved part/pin data, selected context,
capability discovery, atomic proposals, preview/apply/discard and one-step undo.
The Assistant follows the user's FigCraft entry/review layouts and verified
SolidCraft interaction patterns; no account is required for manual design.

Readable circuits require a deterministic schematic arranger/router alongside
the model: aligned functional rows/columns, visible local interconnections,
obstacle-aware orthogonal paths, explicit junctions and locked positions. Do not
ask a model to guess all coordinates or silently change connectivity for neatness.
Arrangement and electrical changes are separate operations with separate checks.
See the [AI authoring contract](specs/ai-authoring.md), including actual SolidCraft
prompt/skill/MCP findings and the bounded next implementation sequence. These
broader facilities remain goals; the current [MCP slice](mcp.md) implements the
live bridge, atomic proposals, review and bounded placement/routing helpers.

## KiCad support

Support KiCad through format adapters independent of the internal domain model.
Track read, edit, preserve and write support separately by version/object kind.
Report unsupported data; preserve it when proven safe and block unsafe edits.
Use public specifications/manuals and black-box tests only, never implementation.

S-expression syntax is distinct from schema, identity, stable serialization and
version compatibility. Do not select JSON/binary storage merely to replace the
syntax. Select final native encoding and compatibility guarantees after independent
IO and no-op/single-edit spikes using original fixtures and a named KiCad version.
The [current schematic draft](adr/0002-schematic-draft.md) has an experimental
strict JSON envelope; it does not settle full-project or KiCad encoding. Use
a distinct electronics project extension; `.pc` is provisional. Container-code
reuse does not mandate `.sc` or mechanical semantics. Possible later `.sc` export
needs a separate adapter and is not promised.

## Library management is a core contract

Distinguish logical component, symbol drawing, package/footprint, placed instance,
and optional manufacturer/part and supplier information. Generic parts must work
without purchasing records. Preserve explicit terminal-to-symbol-pin-to-pad
mappings; matching names/numbers alone cannot establish full compatibility.

Use durable identity separate from names/paths, pinned revision/content identity,
and a project-resolved snapshot of used definitions and required assets. Designs
must reopen and use supported parts offline without silently changing them to
installed versions. Report missing required mappings/geometry; optional 3D loss
should not invalidate electrical checks.

Keep local overrides and assembly variants explicit. Upgrades propose affected
instances, mapping/geometry changes and conflicts for review, then apply atomically
with revision checks and undo. Source deletion/rename must not erase captured
definitions. Record provenance/licenses and unknown metadata without inventing
manufacturer or upstream revision information.

Local, team and public sources need predictable resolution without mandatory
cloud accounts or enterprise PLM. Reuse independently consumable parts-library or mechanical-link utilities where
verified, with PCBcraft owning EDA identity/mapping semantics. Do not pull in the
SolidCraft application merely to access its library panel.

## First acceptance path

Begin with the [base workspace plan](implementation-plan.md), resolving actual
standalone desktop and shared-library boundaries, not the whole SolidCraft build.
W0 and F1 minimal contracts can start together; contracts precede persistence
implementation. The first complete application milestone has PCBcraft's own window/three views, one original editable component, browser/inspector/layers,
selection/cross-selection, one edit/undo/redo, provisional `.pc` save/reopen and
GUI/API parity. Routing/full checks/KiCad/manufacturing are later gates. Original
headless identity/mapping/concurrency fixtures run alongside the visible work.

The user's FigCraft schematic/layout/render designs are visual authority, not
implemented behavior. Compose verified shared components in the single window;
[UI acceptance](specs/workspace-ui.md) is a gate for every visible slice.

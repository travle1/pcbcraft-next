# Flat and hierarchical sheets

Implemented November 3, 2026 in the standalone application. This extends the
single Electronics session; sheets do not have separate project files, histories or
save authorities at runtime. Layout and cross-view rendering remain future work.

## Use the editor

- **Manage → Sheets → New sheet** creates a flat sheet at project level.
- **Manage → Sheets → Subsheet** creates a child of the active sheet and a
  corresponding sheet symbol in its parent. Creation opens the child.
- The **Project / Sheets / sheet context menus** expose the same creation actions;
  a sheet context menu also offers rename, interface ports and reviewed deletion.
- In either creation dialog, choose a new empty definition or reuse an existing
  definition. Shared drawing edits affect every occurrence; the physical parts
  remain distinct. Cycles are rejected without changing the project.
- Click a browser sheet name to navigate; its chevron only expands/collapses.
  Symbols and Nets expand sheet-local lists; selecting an item navigates to its
  occurrence and selects it directly. Root-level sheets are
  siblings; children nest beneath their parent. Camera and component selection
  are retained per occurrence during the current session. The breadcrumb and
  sheet count track the active occurrence.
- **Capture → Place → Port** adds a named interface port at the last canvas
  position. Add its input/output/bidirectional/passive direction, then wire the
  port anchor using **W**, just like a component pin. Direction is recorded;
  it does not imply ERC has run.
- Parent sheet symbols expose matching port anchors automatically. Wire their
  pins in the parent to connect parent and child circuits. Adding, renaming,
  moving or removing a port updates every parent binding atomically.
- Select and drag a sheet symbol or port to move it. Attached authored wires
  stretch in the same transaction. Double-click a sheet symbol to enter the
  child, or use **Open subsheet** in Properties/context menu. Double-click a
  port to edit it. Properties also offers rename and reviewed deletion. **Delete**
  on a selected sheet symbol/port opens that same removal review.
- **S** command search includes New flat sheet, New hierarchical subsheet and
  Add interface port. **Ctrl+Z / Ctrl+Shift+Z** undo/redo the whole project.
- Save writes every definition, occurrence, connection and text override to one
  experimental `.pc` project. Open starts at the primary sheet (initially Main). Existing v1–5 drafts migrate
  to a Main occurrence while preserving their existing object IDs and contents.

The original fan-controller example remains an ordinary single-sheet circuit;
its generic controller/power symbols are components, not disguised subsheets.

## Identity, connectivity and ownership

`project::Project` owns the main definition, additional definitions and the
occurrence tree. Each existing `Document` is now sheet-definition content;
`SheetId` identifies it. `SheetOccurrenceId` identifies one use of that definition.
A parent `SheetSymbolId` references a child definition. Its corresponding child
occurrence records the parent occurrence and sheet-symbol ID explicitly.

A repeated parent definition creates distinct nested occurrences beneath each
parent occurrence. Existing occurrence IDs survive definition edits, rename,
undo and save/reopen. No ID is derived from a name, reference, tree path or position.
The current definition model still has single-unit symbols; multiunit components
and more general component mapping remain future work.

`PortId` is the authoritative interface identity. A child `Endpoint::Port` and a
parent `Endpoint::SheetPin` are distinct local anchors joined by an explicit
occurrence mapping during whole-project connectivity compilation. Local net names
on different sheets never join circuits. Hierarchy is not inferred from labels,
coordinates, overlap or equal reference designators. Different local aliases on
one explicitly bridged net remain scoped aliases; the report does not pick a
silently authoritative name. Global labels and buses are not implemented.

`Session::apply_on(revision, occurrence, command)` edits the requested definition;
`Session::project_command` handles sheet/interface operations. Both validate a
candidate whole project before one commit. A failure leaves history, revision and
content unchanged. Navigation is view state and never increments the revision.
Undo/redo increment revisions. GUI scope changes cancel transient wire/drag/text/
inspector state so an old draft cannot be applied to another occurrence.

Deleting a sheet removes its parent symbol and incident parent connections, then
its occurrence subtree. If the parent definition is shared, that definition edit
affects all its occurrences. Definitions still used elsewhere remain. Removing a
port removes its attached child/parent wires everywhere. The review displays the
counts of removed sheet/component/wire occurrences, and undo restores the entire
aggregate. The primary Main occurrence is retained; it can be renamed.

## Hooks for later PCB cross-probing

`project::ComponentTarget` is `(ProjectId, SheetOccurrenceId, ComponentId)`.
`TerminalTarget` adds the logical string terminal ID. The same definition component
in two sheet occurrences therefore resolves to two distinct physical targets.
A future footprint map must use these targets; matching `R1`, net names or screen
coordinates is insufficient. `resolve_component` maps the target back to the
schematic symbol; deleted/unresolved targets return no match.

`Session::probe_selection` produces a renderer-independent snapshot of selected
components and the terminal membership of selected wires/nets, including explicit
hierarchical connections. It carries project, runtime incarnation, revision and
occurrence. `validate_probe` rejects stale, reopened or missing targets. Net groups
are derived at a revision; this does not promise a persistent net ID through splits
and merges. Tests exercise repeated sheets, rename, undo and reopening.

The future layout adapter must map these targets to footprints/pads, represent
missing or one-to-many matches, and consume selection without re-emitting it as
new user intent. GUI selection projection, loop-suppression/origin tokens, board
association/ECO and actual layout highlighting are not implemented by these hooks.
`project::report` exposes the complete occurrence-qualified connectivity snapshot
through **Search → Nets → Netlist… → Copy JSON**. This is an inspection format,
not KiCad/SPICE interchange or an electrical validation report.

## Persistence, limits and acceptance

The desktop now saves `pcbcraft-project-draft/6`, with a complete project envelope.
The bounded v1–5 single-sheet reader migrates old files; legacy single-sheet APIs
reject hierarchy/multiple-sheet files instead of silently dropping content. Strict
validation rejects dangling mappings, missing occurrences, cycles, duplicate branch
names/IDs, unsupported fields and invalid coordinates. Limits are 64 definitions,
256 occurrences, 16 hierarchy edges, 32 ports and 64 child symbols per definition;
the native file remains bounded to 4 MiB. Existing per-sheet item limits still apply.

Headless acceptance covers isolated flat nets, repeated nested definitions,
physical/terminal targets, port propagation, movement, deletion/undo, stale/cyclic
rejection, complete persistence and external-change protection. egui pointer tests
cover real creation dialogs, browser navigation, camera/selection restoration,
scoped edits, port wiring, sheet-block dragging, save/reopen and stale drafts.
Native screenshot review covers the visible hierarchy and parent blocks at full
and compact sizes, using the user's FigCraft export as composition reference.

Not implemented: reparenting, detaching/copying an existing shared definition,
port reordering/side customization, sheet-symbol resizing/rotation, mixed box
selection of sheet symbols/ports, pinned/preview sheet tabs, project-wide reference
annotation, global labels, buses, ERC, KiCad hierarchy interchange or board layout.
Sheet/port moves currently use their own dedicated selection/drag controls; they
are not included in component/wire/text group moves. These bounds are intentional,
not hidden claims of full hierarchical EDA compatibility.

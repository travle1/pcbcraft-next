# First runnable schematic slice

November 3, 2026. This records implemented behavior, distinct from the complete
[framework plan](framework.md). See [README](../README.md) for launch and controls.

## What works

PCBcraft owns an egui/eframe 0.36.2 executable and window. The composition follows
the user's `01-schematic-overview.png` and `02-schematic-properties.png` in
`../../designs/exports/workspaces-2026-10-30/`, inspected as actual pixels. The
workspace has a grouped command ribbon, floating browser/properties/display panels, a
subtle line-grid sheet, navigation controls, parts browser, command search, status bar
and a tabbed schematic-search/messages/ERC availability drawer. The launcher hides
while its drawer is open; navigation and tool guidance move above it, while the
right filter rail stays separate. Collapsed filters remain reopenable. Ribbon
command search uses icon/path rows and keyboard navigation; unavailable history
commands are disabled. Neither search surface adds a second command authority. Its expanded workspace picker follows the
actual SolidCraft card component, with Schematic Current and PCB layout/3D render/Simulate
Planned. Following user review, the
[theme foundation](theme-foundation.md) uses the verified SolidCraft dark palette,
compact controls and theme-resolved icons. Future themes are presentation state.
Family resources have pinned
[provenance](../assets/family/README.md); designs are unchanged.

Capture now follows the FigCraft command grouping, including Power/Port, Mirror,
Annotate and Sync, with group dropdowns. Unimplemented actions and ribbon customization remain disabled. Capture, Analyze,
Libraries, Manage and View select their own command groups; tab selection never
opens a panel or edits a document. The canvas uses finer strokes/labels and bounded
fit magnification. Fit reserves the actual panel widths, including compact windows;
values and pin details hide below 25% zoom while references/connectivity remain. Symbols, Pins, Wires, Labels and Text can be filtered; other selection types are
planned. View controls grid/snap and pin/wire visibility.

The parts browser replaces the old centered symbol dialog. It has searchable
thumbnails, Small/Medium/Large/List views and click-to-place or drag-to-sheet
placement. It stays open without replacing the project Browser. Its title grip
floats or snaps left/right, with a position menu for keyboard access. Short docks
use a compact list, and panels can collapse to retain space. Previews
reuse the canvas symbol renderer. Catalog browsing never edits the document;
valid drops use the session's Place/undo path, while stale or outside drops and
cancelled drags do not insert anything.

Eight original generic definitions—resistor, capacitor, LED, two-pin connector,
controller module, four-wire fan connector, regulator module and temperature sensor—
support place/select/drag/rotate/delete and reference/value editing. Pin-to-pin
links follow component movement. One Electronics session owns edits, revision,
undo/redo, dirty state and save snapshots. Drag previews and navigation are
disposable UI state. The same commands work without the desktop feature.

The compact Properties inspector follows the user's FigCraft layout: actual symbol
thumbnail, reference/value, aligned fields, pin count, unavailable Datasheet when
unassigned, and More properties. It closes, floats and docks without editing the
project. The detailed view exposes real data with unavailable categories disabled;
reference/value/rotation apply atomically, with revision/incarnation checks and undo.
Browser rows keep fixed insets/height; selection changes highlight and font weight.
Canvas selection uses symbol/reference ink rather than a filled bounding rectangle.

Browser uses the FigCraft project/sheet tree; Symbols/Nets entries open scoped
Search result tables. The tree scrolls when needed and retains a bottom height grip;
the grip cannot consume the last rows. Nets are derived from actual terminal
links and cached by incarnation/revision. Click a row or wire to highlight the net,
edit its name, inspect terminals and review connection removal. A split preview
explains name retention/clearing before one undoable transaction. Net Properties
scrolls when necessary. Netlist… generates a copyable JSON snapshot, separate from
native persistence or external interchange. [Details and planned layout UX](net-workflows.md).

File operations run off the UI thread and disable edits until resolved. New/open/
close prompt on committed unsaved edits. Save uses a same-directory temporary
file, file sync and replacement; existing loaded files are compared against their
last-read bytes before replacement. Failed loads/saves retain the editor. `.pc`
uses a bounded, strictly versioned draft envelope described in
[ADR 0002](adr/0002-schematic-draft.md). Built-in parts work offline.

## Deliberate limits

This slice has one sheet and single-unit components. Box selection supports symbols, wires and sheet notes; mixed group movement/deletion commits atomically. See [selection and text](selection-and-text.md). Legacy links have derived orthogonal paths with pin/body avoidance. New wires
support free endpoints, authored bends and explicit branches, with direct deletion
and right-click actions. Select mode drags segments, corners and free junctions;
previews commit once on release. Manual routes do not automatically avoid bodies
or unrelated nets. General label placement and buses remain planned.
See [wire editing](wire-editing.md). The compiler covers only explicit links
in this single-sheet draft, not full hierarchical schematic semantics.
The [original fan demo](fan-controller-demo.md) uses stored section annotations
and display aliases for explicit links. Names do not create connectivity;
validation rejects conflicting or disconnected alias groups. Incidental
crossings do not establish connectivity. Explicit endpoint drops and edited
collinear overlaps join compatible runs and heal redundant geometry; conflicting
names reject atomically. Delete a component to remove
its incident links; undo restores the aggregate. This is a starting interaction
model, not production schematic capture or an electrical reference design.

ERC/DRC, simulation, PCB/layout/3D and split view, footprints, hierarchy/multiunit support, external libraries,
ECO/cross-probing, variants, manufacturing and KiCad adapters remain planned.
No checks are reported as passed. No SolidCore API is needed by this 2D slice;
the future geometry boundary remains SolidCore-first, with no Truck fallback.
Assistant/MCP/CLI edit transports, atomic multi-command proposals, retry idempotency
and general schematic arrangement are not implemented yet. Bounded manual wire
authoring and repositioning are now available. The demo
has curated coordinates with bounded display routing and supply aliases. First-class AI authoring is now
an explicit early milestone in [its spec](specs/ai-authoring.md), not inferred from
the existence of a headless command enum.

Undo is limited to 32 document snapshots and is not persisted. Autosave, crash
recovery, cross-process file locking and general migrations remain work. The version 5 draft reader accepts
versions 1–4, preserving symbol definitions and path/alias presentation while assigning
legacy notes durable identities. Version 5 stores text appearance and web links. External-edit comparison is not a lock against a simultaneous
writer between comparison and replacement. Saved drafts pin the built-in catalog
version; they do not yet embed general library definitions/assets. Settings and
panel layouts are currently session-local. Dark theme is implemented; full
accessibility, high-DPI/multi-platform and large-design qualification remain open.

## Verification and next work

Core tests exercise stale/invalid edits without mutation, revision-monotonic
undo/redo, identity preservation, deletion of incident links, connection rejection,
coordinate limits, strict decoding, round-trip/reopen and external-edit protection.
egui tests drive pointer placement/drag/pin connections and cancellation, and check
file-operation edit blocking, async save/reopen/failure retention, display-control
visibility and the unsaved-document guard. Ribbon checks cover text/caption
clipping, shared card bounds, larger text/pixel scaling and pointer dispatch.
Workspace-picker tests also cover disabled entries, outside-click/Escape dismissal,
keyboard activation, document/selection preservation and scaled card geometry.
Dropdown edits are checked against session undo, command order against the
reference, and grid/fit/filter behavior against document preservation.
Parts-browser tests exercise view changes/search/empty results, click and drag
placement with undo, cancellation, outside drops and stale-revision rejection.
Workspace tests additionally exercise floating/left/right docking, drag cancellation,
actual panel bounds at 1600×1000 and 1000×700, unchanged camera when opening a
drawer, and command-search keyboard dispatch/dismissal without click-through.
Net checks cover deterministic compilation, name/path preservation, split reviews,
atomic undo, stale edits/reopen and version compatibility. UI checks also cover
Browser's last row and height grip, net naming/report opening and canvas wire picking.
Display-route tests also cover straight/perpendicular pin connections, body avoidance,
blocked routes and branch dots versus ordinary endpoints/bends. The current suite passes; formatting,
Clippy with warnings denied and the standalone build also pass. Native screenshots
in local ignored `work/proof/` record actual desktop rendering.

W0 has a real standalone consumer/build proof. W1/D0/F0/H0 have a schematic-only
vertical slice; they are **not complete** against the broader three-view milestone.
F1 identity/transaction/library contracts still need expansion before hierarchy,
external definitions, job proposals or MCP are built against them.

Next bounded slice: AI0 atomic prepare/apply batches and request guards, followed
by a live MCP bridge and Assistant proposal UI. Authored paths/junctions now have a bounded manual editing slice;
a deterministic arranger remains accompanying capture work; these support manual
and AI authoring together. External netlist adapters remain a separate task. Introduce
electrical pin types and a small, explicitly covered ERC profile. In parallel
planning, qualify a reusable component-library boundary and independent SolidCore
geometry/render capabilities for layout/3D. Settle shared interfaces before
dispatching agents. The initial standalone work is recorded in sectional commits
in the private [travle1/pcbcraft-next](https://github.com/travle1/pcbcraft-next)
repository. No agent queue, hosted CI or PR has been created.


The browser hierarchy correction supersedes the earlier Symbols/Nets tabbed list.
The loose wire-edge count and permanent scroll instruction footer are removed.
Only the existing Main sheet is real: **New sheet / New subsheet** are disabled
with explanations. No multi-sheet schema or hierarchical connectivity is claimed.
See [sheet workflow prerequisites](sheet-workflows.md).

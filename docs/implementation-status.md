# First runnable schematic slice

November 3, 2026. This records implemented behavior, distinct from the complete
[framework plan](framework.md). See [README](../README.md) for launch and controls.

## Live AI / MCP authoring (November 4, 2026)

Implemented directly in PCBcraft: atomic candidate prepare/apply/discard, bounded
idempotent receipts, live app-session bridge and a standalone stdio MCP executable.
Ten tools expose catalog/pin data, selection, sheets and nets plus schematic edit
proposals. Assistant reviews Before/Proposed, operations and checks;
user Apply shares normal GUI Undo. External apply is an explicit session setting.
Ordered grid placement and bounded orthogonal connections are initial helpers,
not a full automatic arranger. [Setup, verification and exact limits](mcp.md).
An initial [ACP v1 conversation adapter](assistant.md) now supports Send, streaming,
Stop, bounded history and review-only tools. It is tested with a scripted protocol
peer; real agent/provider qualification, ERC and large-project work remain ahead.

Assistant now opens from a bottom-right launcher to the full workspace height,
with a collapse chevron and bounded horizontal resize. It uses the existing shared
card overlaying the right rail. Properties/filters/right-docked library keep their
positions beneath it and return unchanged on collapse; they are not pushed into
the sheet. The panel shows actual part/connection changes, Before/Proposed and
pinned Apply/Discard, with operations/checks in Change details. Its current-sheet
and selection context accompanies a bounded three-line input labeled Your request
and suggested requests. The input stays visible during proposal review.
Copy request copies a brief to paste into an external AI app; it does not send a message.
Connection setup and diagnostics are secondary dialogs. The camera, project and
pending review stay unchanged when the panel opens or collapses. A configured ACP
agent enables Send; Conversation/Changes switches between replies and proposals.
No process starts until Send. The in-app bridge always denies agent Apply/Discard,
even if external apply is enabled. New/open clears the transcript and cancels the
turn; manual edits invalidate active work. There is no title-bar launcher.

The connected-grid follow-up adds proposal-local fixed symbols and region limits,
with deterministic replanning of internal direct pin connections. It preserves
terminal membership, wire IDs and outside geometry, and supports repeat no-op.
Branch/outside endpoint movement and unqualified text/hierarchy clearance reject
explicitly. This is a bounded 20-symbol catalog-only profile; the full SA1 arranger,
persisted locks and exact text clearance remain open. [Profile limits](mcp.md#placement-routes-and-remaining-work).

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
values, net names and pin details hide below 25% zoom while references/connectivity remain. Symbols, Pins, Wires, Labels and Text can be filtered; other selection types are
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

Browser uses the FigCraft project/sheet tree; Symbols/Nets expand in-place lists,
with direct selection and the ribbon’s Symbol icon. Search tables remain separate.
The bottom drawer has a vertical resize grip with session-local retained height.
The tree scrolls when needed and retains a bottom height grip;
the grip cannot consume the last rows. Nets are derived from actual terminal
links and cached by incarnation/revision, with caches reset on occurrence switches. Click a row or wire to highlight the net,
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

The editor now has real sheet creation, navigation, parent blocks and interface ports.
The entire project shares history and persistence; revision-bound cross-probe
snapshots expose future layout targets without implementing a layout view.

## Deliberate limits

This slice supports flat and hierarchical sheets, repeated definitions, explicit ports and single-unit components; see [sheet workflows](sheet-workflows.md). Box selection supports symbols, wires and sheet notes; mixed group movement/deletion commits atomically. See [selection and text](selection-and-text.md). Legacy links have derived orthogonal paths with pin/body avoidance. New wires
support free endpoints, authored bends and explicit branches, with direct deletion
and right-click actions. Select mode drags segments, corners and free junctions;
previews commit once on release. Manual routes do not automatically avoid bodies
or unrelated nets. General label placement and buses remain planned.
See [wire editing](wire-editing.md). The compiler covers explicit local links and occurrence-qualified port bridges. Global labels, buses and full hierarchical ERC remain future work.
The [original 31-part fan demo](fan-controller-demo.md) uses stored section annotations,
a 24-pin generic controller, authored orthogonal routes and display aliases for explicit links. Names do not create connectivity;
validation rejects conflicting or disconnected alias groups. Incidental
crossings do not establish connectivity. Explicit endpoint drops and edited
collinear overlaps join compatible runs and heal redundant geometry; conflicting
names reject atomically. Delete a component to remove
its incident links; undo restores the aggregate. This is a starting interaction
model, not production schematic capture or an electrical reference design.

ERC/DRC, simulation, PCB/layout/3D and split view, footprints, multiunit support, advanced hierarchy editing, external libraries,
ECO/cross-probing, variants, manufacturing and KiCad adapters remain planned.
No checks are reported as passed. No SolidCore API is needed by this 2D slice;
the future geometry boundary remains SolidCore-first, with no Truck fallback.
MCP transport, atomic multi-command proposals and retry idempotency now work.
Real agent/provider qualification and general schematic arrangement remain open. Bounded manual wire
authoring and repositioning are now available. The demo
has curated coordinates/routes and supply aliases. Catalog version 3 adds four
original symbol types while preserving all version 1/2 geometry. First-class AI authoring is now
an explicit early milestone in [its spec](specs/ai-authoring.md), not inferred from
the existence of a headless command enum.

Undo is limited to 32 whole-project snapshots and is not persisted. Autosave, crash
recovery, cross-process file locking and general migrations remain work. The version 7 project reader migrates
versions 1–5, preserving symbol definitions and path/alias presentation while assigning
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
F1 identity/library contracts still need expansion for external definitions,
advanced hierarchy and background jobs. The bounded MCP profile uses current
sheet/identity contracts and the new session-owned transaction path.

Next bounded slices: expand AI arrangement and schematic tools, qualify an
real in-app agent/provider workflow, and add background preparation for large projects.
Atomic proposals, request guards, live MCP and the first review UI now work.
Authored paths/junctions have a bounded manual editing slice;
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

## Page, placement and grid follow-up

Capture exposes New sheet/Subsheet in Sheets, and Port/Text in Place. Symbol, port,
subsheet and text tools preview under the cursor and repeat until right-click/Esc.
Subsheet blocks stay in the parent while placing; flat sheets still open directly.
Text supports dragging out a box, saved wrap width/minimum height, and undoable
creation/resizing. Preview movement/cancellation is not a document mutation.

View contains a dedicated Grid group with spacing presets, visibility and Snap,
plus saved page size/orientation and session-local dashed guide modes. Grid/snap
readouts were removed from the status corner; zoom is beside navigation. Navigation
uses immediate raw wheel deltas anchored at the pointer. Authored wire paths skip
unneeded obstacle collection. These are targeted improvements, not a benchmark or
a general schematic arranger. The original demo now uses more space within A3.

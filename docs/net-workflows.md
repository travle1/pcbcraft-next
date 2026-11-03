# Nets, workspaces and layout interaction

November 3, 2026. This distinguishes the current schematic draft from planned
layout, interchange and simulation work. PCBcraft remains a standalone app.

## Available now

- **Browser → Sheets → sheet → Symbols / Nets** expands sheet-local lists.
  Click a symbol or net to select it in that sheet without opening a drawer.
  Branch disclosure does not change selection. Search is a separate filtered table
  in the bottom drawer; drag the drawer's top grip to resize it vertically.
- Click a net row or a visible wire/alias stub to highlight its connected pins
  and links. The Wires selection filter controls canvas net picking. A net result selects the full explicit group; box selection separately supports
  symbols, wires and notes as described in [selection and text](selection-and-text.md).
- Net Properties lists the terminals and connections. Apply name renames the
  connected group; a blank name clears it. Naming a normal wire keeps its path
  visible. Existing remote aliases retain their presentation.
- Each connection's remove control opens a review. Removal can split the group:
  the largest remaining linked fragment keeps its name (stable terminal-ID tie
  break); other linked fragments become unnamed with visible paths. Isolated pins
  are reported separately, and a name disappears if no linked fragment remains.
  Apply validates the captured session incarnation/revision and commits once;
  Cancel does nothing and Undo restores the entire change.
- **W** draws from a pin, wire/junction or empty sheet. Click for bends, connect
  to a pin/wire, or Enter/double-click for a free end. Space flips the next bend;
  Backspace removes the last corner; Esc/right-click cancels uncommitted work.
  Branch creation splits the target wire and commits all changes together.
- Click a wire then **Delete** removes that wire. Search net selection then
  Delete, the Properties **Delete net** button, or **Delete whole net** in the
  context menu removes the connected graph while keeping components. These direct
  edits are immediately undoable; the existing per-connection review remains.
- Select mode supports dragging a segment perpendicular to itself, dragging a
  corner, and repositioning free endpoints/junctions. Fixed pin anchors and graph
  membership is preserved during ordinary reshaping. Explicit endpoint drops and
  edited collinear overlaps can join runs; duplicate strokes/backtracking and
  redundant junctions heal together. Preview is disposable; release is one undo entry.
  Right-click menus distinguish wire, symbol and empty-sheet actions and retain
  their target while the pointer moves into the menu.
- Joining differently named groups rejects rather than choosing a name silently.
  Clear or reconcile names first. Buses and general label placement remain planned.
  [Wire controls, persistence and current limits](wire-editing.md).
- **Search → Nets → Netlist… → Copy JSON** produces a connectivity snapshot with the project,
  session incarnation, revision, occurrence-qualified component/terminal targets,
  sheet paths, local aliases and complete connected node groups. Isolated component
  pins appear as singleton groups. This is a report, not an
  importable native project or a KiCad/SPICE file. Native `.pc` still saves the
  editable document, including its drawing.

Connectivity is compiled from explicit pin/junction edges, never coordinates,
crossings or matching names. Results are deterministic for a fixed document and
cached per incarnation/revision and invalidated on occurrence switches. The derived net anchor is a selection key for
that revision, not a durable electrical identity across arbitrary splits/merges.
The draft now supports flat and hierarchical sheets with explicit port bridges and single-unit symbols. The project report qualifies connectivity by occurrence; see [sheet workflows](sheet-workflows.md). No ERC or PCB connectivity is implied.

## Reading external netlists: next adapter, not implemented

Use an explicit import command with a named format/version, bounded input and
review before mutation. A file suffix cannot select semantics safely. Parse into
an intermediate connectivity description; resolve components and terminal IDs
against available definitions; report unresolved parts/pins and unsupported
records. Never fabricate a footprint mapping, merge by reference text alone, or
discard unsupported content without a report. Applying a validated import is one
revision-checked transaction with undo. Original fixtures should cover duplicate
references/pins, missing definitions, disconnected groups and stable round trips.

Start with one pinned KiCad connectivity profile using public documentation and
original data. KiCad native netlist and its intermediate XML generator format are
different formats. SPICE requires a separate model/pin-mapping and analysis
contract. A connectivity netlist alone cannot reconstruct schematic placement,
authored wires, PCB routing, rules or a complete reusable project.

The public [KiCad schematic manual](https://docs.kicad.org/master/en/eeschema/eeschema.html)
describes netlist exports and simulation models; the
[PCB manual](https://docs.kicad.org/8.0/en/pcbnew/pcbnew.html) describes direct
schematic-to-board updates. These are behavior references, not implementation
sources. No KiCad code or supplied example was imported. A format profile must
pin its documentation/version before implementation.

## Layout: one project, optional split view

Recommended first layout interaction, still planned:

1. One PCBcraft window with a project tab; Schematic, PCB layout and 3D render
   are workspaces of that project. Sheet navigation stays inside Schematic.
2. The workspace selector switches the main view. A later explicit **Split
   schematic / layout** command displays both beside each other, with a resizable
   divider. Each retains its camera; focusing a viewport chooses its ribbon,
   inspector and local tools. A narrow window can return to a single view without
   losing document or selection state. Splitting is optional, not the default.
3. Both use one Electronics session, undo history and save lifecycle. Selection
   maps through explicit component/terminal/footprint/pad correspondence; highlight
   the matching objects without emitting a second selection intent. Missing or
   ambiguous targets are visible, and cross-probing does not force camera motion.
4. Selection is live; electrical changes use a separate reviewed ECO. Preview
   added/removed/replaced parts, net splits/merges and affected pad mappings,
   preserve board-local intent, reject stale previews and apply atomically with
   undo. No invisible rerouting, deletion or automatic shorting from a click.
   PCB-only projects remain supported by the broader architecture.

The present derived netlist is useful input, not the board model or an ECO engine.
Before layout, settle persistent component/pad mapping and split/merge lineage,
then implement one component/footprint cross-selection and a reviewed update.
Do not create a second document by exporting/reimporting netlists between views.
Detached secondary windows may follow later with the same session, after focus,
selection and lifecycle behavior is proven. They are not required for first layout.

## Simulate: visible, planned

The workspace picker now includes **Simulate — Planned**, disabled. Its eventual
scope is circuit-model setup, analysis configuration, probes and waveform results.
No solver, model library, waveform UI or simulator export is implemented. Original
generic fan-controller modules are illustrative, not qualified simulation models.

A later simulator adapter consumes a pinned connectivity/model/analysis snapshot.
Missing models and terminal mappings must be explicit. Jobs need cancellation,
revision/variant/configuration provenance and stale-result handling; a result
must never imply the current edited document was simulated. Solver choice and
license/dependency qualification are separate decisions. See the
[framework](framework.md), [UI contract](specs/workspace-ui.md) and
[current implementation limits](implementation-status.md).

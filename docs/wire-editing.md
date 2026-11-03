# Wire editing

Implemented November 3, 2026. Wires are editable schematic geometry and explicit
connectivity, including incomplete wires without attached parts.

## Draw, branch and finish

Press **W**, or use Capture → Wire / Browser → Nets → + Wire. Click a component pin,
a visible wire, a free endpoint/junction, or empty sheet to begin. Clicking empty
sheet adds an orthogonal leg/corner. **Space** flips the next elbow orientation;
**Backspace** removes the last preview corner. Clicking a pin or wire finishes and
connects. **Enter** or double-click finishes with a free endpoint. **Esc** cancels
the unfinished wire and returns to Select; right-click cancels the preview while
leaving the wire tool available. Nothing is saved until the wire is finished.

Ending on a segment splits that edge and inserts an explicit junction. Starting
there creates a branch when finished, not when the first click happens. Ordinary crossings do not join nets. Deliberately drawing or dragging a
positive-length collinear overlap joins compatible runs and removes duplicate
strokes. Differently named nets reject that edit. Open circles distinguish
free ends from connected junctions. A free wire appears in the Nets browser even
when it has zero component pins. Existing name conflicts reject the complete
candidate rather than partially connecting it.

## Select, reposition and delete

In Select mode (**V**):

- Click a wire to select it and show its endpoint/corner handles; Properties shows
  the containing net and says that Delete removes the selected wire.
- Drag a segment to slide it perpendicular to its direction. End anchors remain
  fixed; the editor adds orthogonal end legs as needed.
- Drag an interior corner to reshape its adjoining legs. Drag a free endpoint or
  junction to reposition it; all attached routes follow. Drop an endpoint on a
  pin or wire to attach it, creating a junction when needed. Component pins remain
  attached to their components and move with those components.
- A drag is a disposable preview; release commits one undoable geometry edit.
  Esc cancels. Releasing over an obstructing panel cancels rather than dropping
  an invisible edit. Stale document revisions/reopens reject the edit.
- **Delete** removes the selected wire between its graph endpoints, including all
  of its bends. It does not erase other branches in the same net. **Ctrl+Z** undoes.
- Select a net in Browser, then Delete, to remove all its wires. Components remain.
  The Properties **Delete net** button and right-click **Delete whole net** do the
  same. The existing per-connection review action is also available.

Right-click menus act on the object under the pointer: wire actions include
Start wire here, Select net, Delete wire and Delete whole net; symbols offer
Properties, Rotate and Delete; empty sheet offers wire/symbol creation and Fit.
Menus also expose Undo/Redo. Their captured target and revision remain fixed while
the pointer moves into the popup; opening a menu does not mutate the document.

## Saved data and limits

Native draft **v4** saves junction IDs, wire endpoints and authored bends. It reads
v1–v3 drafts without silently changing their stored layout/alias presentation.
Editing or branching an older automatic route saves that route explicitly. Keep
using the new binary for v4 files; older binaries cannot read the new schema.
The connectivity report includes free wire points and explicit edge membership.

One Electronics session owns edits, naming, splits, deletion, history and save.
Route reshaping preserves component/pin identities. An explicit endpoint drop or
collinear overlap can merge nets; mere proximity and ordinary crossings cannot.
Healing removes doubled-back legs, duplicate overlapping runs and redundant
intermediate junctions in the touched geometry. New junction/edge IDs can be
created or removed during that normalization; component/pin identities remain.
Legacy display routes are revision-bound inputs to cleanup: affected geometry
becomes authored while unrelated automatic routes remain unchanged. Loading a
file does not itself perform healing. Structural
errors and conflicting names reject the whole transaction. Disconnecting a named
net retains the name on one largest remaining linked fragment; other fragments
become unnamed. No unnamed orphan junctions accumulate after deletion.

This is bounded manual editing, not a schematic autorouter or ERC. Authored paths
may cross symbols or other nets; there is no automatic obstacle routing or general
self-intersection solver. Overlap cleanup has a bounded comparison/merge budget;
a too-complex edit rejects atomically. Coincident distinct pins and unsupported
parallel-edge loops reject instead of losing terminal identity. Connecting two interior points of the same original edge
in one new wire is currently rejected; create the intermediate branch first.
No buses, general label-placement tool, multi-selection route move or PCB routing
is implemented. Wire deletion removes the selected graph edge rather than cutting
only one visible leg; drag its corners/segments to reshape it instead.

Tests cover free creation, branches, crossing isolation, incompatible names,
cancellation, stale edits, direct/context deletion, dragging, netlist preservation,
undo/redo, v4 round trips and rejection of new data in old envelopes. The separate
[AI authoring plan](specs/ai-authoring.md) still needs atomic multi-command proposals,
MCP/Assistant transports and deterministic arrangement.

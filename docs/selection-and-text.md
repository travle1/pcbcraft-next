# Selection and editable schematic text

Implemented November 3, 2026 in the standalone application. These controls use the
existing family menu/card/field styling and the single Electronics session.

## Mouse and keyboard

In **Select** mode (`V`), left-drag empty sheet space or right-drag to draw a box.
Left-to-right is a solid **Window** box: whole symbol bodies, complete drawn wire
paths and whole note bounds must fit. Right-to-left is a dashed **Crossing** box:
touching one of those objects includes it. The box shows the hit count. Selection
itself changes neither the document revision nor undo history.

- Shift adds hits; Ctrl toggles them. Shift/Ctrl-click also adds/toggles individual items.
- Drag a member of an existing group to move the group; Delete removes it.
- A single symbol still moves normally. A single wire still supports segment/corner/endpoint editing.
- Selected wire groups move their route geometry; pins on unselected components stay attached by orthogonal end legs.
- Esc cancels the drag. Edits/undo/reopen during a preview invalidate its captured revision/incarnation.
- A stationary right-click opens the styled context menu. Right-drag selection consumes its release and does not open that menu.
- Middle-drag pans; scrolling zooms. The explicit Pan tool retains its navigation behavior.
- Symbols, Wires and Text filters determine box hits. Labels filter controls direct attached-label selection. Pin hit testing remains available to wiring.

Only one atomic command is committed on release. Invalid or out-of-bounds group
moves leave the whole document unchanged. Deleting components also deletes their
incident wires. Existing topology/name validation and overlap healing still apply.
Box selection includes visible drawn wire paths, not the invisible spans of
remote aliases. Clicking their displayed labels still edits the explicit net.

## Text and links

Press **T**, use **Capture → Place → Text**, command search **Add text**, or
right-click empty sheet space → **Place text**. These enter placement mode, with
a text cursor and dashed preview. Drag to set the box size or click to place a
48 × 18 mm box. Release commits one note and its dimensions atomically and opens
Properties. Repeat placement until right-click or Esc; cancelling creates nothing.
The inspector exposes box width and minimum height. Text wraps to width and grows
vertically when needed, so it is never silently clipped. Existing unboxed notes
retain their prior presentation. Box size/formatting use the same save/undo authority.

Click a sheet note, component reference/value, or displayed net label to edit it.
The component inspector also offers **Reference style / Value style**; net Properties
has **Text appearance…**. These are attached labels: editing reference/value updates
the component, and editing a net name goes through explicit connectivity validation.
There are no string-based connections and no standalone electrical label-placement tool yet.

The text inspector provides content, IBM Plex Sans/Semibold or bundled Monospace,
physical height in millimetres, italic, underline, X/Y offset and an optional web
link. Notes support multiple lines. **Apply** commits one undoable edit; **Reset**
reloads current content/appearance. Controls scroll at compact sizes while Apply
stays visible. The Properties card retains float/dock/close controls.

Drag a note to move it; drag an attached label to offset its presentation while its
component/connectivity stays in place. Ctrl-click linked text or choose **Open link**
in Properties/context menu to open an HTTP(S) address. Loading or painting never
opens/fetches links. The URL field is optional, separate from the displayed text.
Ctrl-click on a saved web link takes precedence over Ctrl-toggle selection.

Reference uniqueness, name/connectivity consistency, bounded content, physical size
and coordinate limits are validated before commit. Text inspector drafts reject
stale revisions or reopened sessions. Native project draft 7 / single-sheet draft 6
save box dimensions as well as text IDs, appearance and links. Project draft 6
and older single-sheet drafts remain readable. See [ADR 0002](adr/0002-schematic-draft.md).

## Limits and verification

This is plain text with one style and optional URL per target, not rich text with
multiple inline hyperlink spans. Pin names/numbers come from built-in definitions
and remain read-only. Arbitrary system font loading, text rotation/alignment tools,
font embedding, standalone electrical label objects and typography export are future work.
Net style uses current wire carriers; topology cleanup can remove a carrier, as
recorded in ADR 0002. Equal names never establish electrical identity.

Tests exercise both buttons, window/crossing semantics, modifiers/filters, group
preview/commit/undo/delete, cancellation, fixed external pins, text dragging and
explicit URL dispatch. Core tests cover mixed atomic edits, invalid/stale requests,
text styling/links round trips and legacy migration. The compact inspector test
checks visible Apply, one-step commits and stale/reopened rejection. Native captures
under ignored `work/proof/selection-*.png` verify the rendered inspector and marquee;
these are local review artifacts, not published design assets.

# Using the schematic workspace

Run `cargo run --locked -- --demo` from the repository for the original
fan-controller illustration. Omit `--demo` for an empty project or pass a saved
`.pc` path. **File → Open example** loads the demo later. Its generic components
illustrate editing; the example is not an electrically validated design.

## Place and edit parts

Press **A** to open the Parts library. Search by name/value and choose Small,
Medium, Large or List view. Click a thumbnail then the sheet, or drag a part onto
the sheet. Clicking a thumbnail starts a cursor preview that repeats after each
canvas click; right-click/Esc finishes. Drag-and-drop remains a single drop.
The library stays open. Twelve built-in generic definitions work offline; external catalog libraries are not implemented.

Drag placed symbols to move them, **R** to rotate and **Delete** to remove them.
Edit reference/value in Properties and click **Apply changes**. **More properties**
opens the General / Pin functions / Library view; unavailable fields remain disabled.
**Done** applies reference, value and rotation together as one undoable edit;
**Cancel** leaves the document unchanged.

## Wire and inspect nets

Press **W** to draw a wire from a pin, existing wire or empty sheet. Click to add
bends; click a pin/wire to connect, or **Enter** / double-click for a free end.
**Space** flips the next bend and **Backspace** removes the last corner.

In Select mode (**V**), drag a segment to slide it, a corner to reshape it, or a
free endpoint/junction to reposition it. Attached pins stay with their components.
Endpoint drops attach to wires/pins, and edited collinear overlaps heal into clean
runs; conflicting net names reject the edit. Crossings alone do not connect.
Select a wire then **Delete** to remove it. Right-click for contextual actions.
See [wire editing](wire-editing.md) for exact behavior and limits.

Expand **Nets** beneath a sheet in Browser to select its nets directly. The Search drawer offers a separate searchable result table.
Click a result or wire to highlight its connections and edit its name in Properties.
Connection removal offers a review before committing; Undo restores the change.
**Search → Nets → Netlist… → Copy JSON** generates a connectivity report.
External netlist import is not implemented. See [net workflows](net-workflows.md).

## Selection, text and navigation

In Select mode, left-drag empty space or right-drag to box-select. Left-to-right
selects enclosed items; right-to-left includes touched items. Shift adds; Ctrl
toggles. Drag a selected group to move it, or Delete to remove it, as one undoable
edit. A stationary right-click opens the context menu.

**T** or **Capture → Place → Text** starts a text cursor with a dashed frame.
Drag to size a text box, or click for a 48 × 18 mm box. Nothing is created until
release/click. Edit its contents in Properties; box width wraps text and minimum
height grows to keep all text visible. The tool remains active for repeated placement.
Right-click or Esc finishes without creating another object.
Click a note, component reference/value or net label to edit its content, font,
physical height, italic/underline, offset and optional web link in Properties.
Drag text to reposition it; Ctrl-click opens a saved HTTP(S) link. **Apply** commits
one undoable edit; **Reset** reloads current text. See [selection and text](selection-and-text.md).

**Esc** or right-click cancels placement. **F** fits circuit content; **Shift+F**
or the navigation strip's **Fit page** frames the complete page. Wheel zoom responds
immediately around the pointer; middle-drag pans. Zoom percentage is beside navigation.

**View → Grid** contains visibility, Snap and the current spacing button. Choose
0.254, 0.5, 0.635, 1, 1.27, 2.54 or 5.08 mm. Placement/movement uses the chosen
spacing; changing it does not move existing circuitry. Dense grid lines thin out
visually at overview zoom without changing snap spacing. These are session-local
view settings, not electronics edits.

**View → Page → Page setup** selects A4/A3/A2/Letter and orientation. Page size is
saved per sheet definition and undoable; shared occurrences use the same page.
The Page group menu selects **Always / Automatic / Off** for the dashed guide.
Automatic shows nearby edge segments during placement/dragging. Content can remain
outside the page; no clipping, repositioning or printing/export is implied.

 **Ctrl+Z / Ctrl+Shift+Z** undo/redo; **Ctrl+S / Ctrl+O** save/open.
**S** opens command search, with arrows to choose, Enter to run, Tab to fill and
Esc to dismiss. Shortcuts defer to focused text fields.

## Panels and sheets

Ribbon tabs switch the command groups beneath them. **View → Panels** reopens
closed panels. Drag the Parts library or Properties grip/title to float it, or
release near the left/right edge to dock. Their **…** menus offer placement and
collapse controls; **×** closes the panel. Short library docks use a compact list;
floating restores the chosen thumbnail size. Panel placement is session-local.

Browser follows **Project → Sheets → sheet → Symbols / Nets**, with children
nested beneath parents. Selection highlights/bolds rows without moving them.
Disclosure arrows expand/collapse; sheet names navigate. Symbols and Nets expand into their own sheet-local lists; clicking an item selects
it directly without opening a drawer. Symbols use the Capture ribbon’s Symbol icon.
The browser has a bottom height grip and scrolls when needed. Search remains a
separate searchable table. Search/ERC/Messages share the bottom drawer; drag its
top grip vertically to resize it, and use its chevron to collapse it. The chosen
height is retained when reopening or changing tabs during the current session. ERC currently
shows availability, not electrical-check results.

Use **Capture → Sheets → New sheet / Subsheet** (also under Manage), or browser context menus, to create
flat or hierarchical sheets. Choose a new definition or reuse an existing one.
**Capture → Place → Port** adds interface ports; parent pins update automatically. Wire
those anchors with **W**, drag sheet symbols/ports to move them, and double-click
a sheet symbol to enter its child. Properties and context menus offer rename/edit
and reviewed deletion. All sheets share undo/redo and one `.pc` save. Existing
single-sheet files migrate on open. See [sheet workflows and future cross-probing hooks](sheet-workflows.md).

## Current limits

The workspace picker marks PCB layout, 3D render and Simulate as **Planned**.
Assistant/MCP, ERC/DRC, simulation, KiCad, external part libraries and manufacturing
are not implemented. Native `.pc` is experimental; autosave and crash recovery
remain future work. Read [implementation status](implementation-status.md) before
relying on saved drafts and the [AI authoring plan](specs/ai-authoring.md) for the
next integration milestone.

At fit-sheet overview scales below 25%, net names, component values and pin details
are hidden to avoid crowded labels. Zoom in to inspect them; Browser and Properties
keep the underlying names available. The example’s authored text offsets are saved
with its editable data. Existing saved projects keep their own composition.

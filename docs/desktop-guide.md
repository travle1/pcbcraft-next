# Using the schematic workspace

Run `cargo run --locked -- --demo` from the repository for the original
fan-controller illustration. Omit `--demo` for an empty project or pass a saved
`.pc` path. **File → Open example** loads the demo later. Its generic components
illustrate editing; the example is not an electrically validated design.

## Place and edit parts

Press **A** to open the Parts library. Search by name/value and choose Small,
Medium, Large or List view. Click a thumbnail then the sheet, or drag a part onto
the sheet. The library stays open for further placement. Eight built-in generic
definitions work offline; external catalog libraries are not implemented.

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

Click **Nets** beneath a sheet in Browser to open its searchable result table.
Click a result or wire to highlight its connections and edit its name in Properties.
Connection removal offers a review before committing; Undo restores the change.
**Search → Nets → Netlist… → Copy JSON** generates a connectivity report.
External netlist import is not implemented. See [net workflows](net-workflows.md).

## Selection, text and navigation

In Select mode, left-drag empty space or right-drag to box-select. Left-to-right
selects enclosed items; right-to-left includes touched items. Shift adds; Ctrl
toggles. Drag a selected group to move it, or Delete to remove it, as one undoable
edit. A stationary right-click opens the context menu.

**T** or **Capture → Place → Text** adds a sheet note at the last canvas position.
Click a note, component reference/value or net label to edit its content, font,
physical height, italic/underline, offset and optional web link in Properties.
Drag text to reposition it; Ctrl-click opens a saved HTTP(S) link. **Apply** commits
one undoable edit; **Reset** reloads current text. See [selection and text](selection-and-text.md).

**Esc** cancels the active tool, **F** fits the drawing, scroll zooms and middle-drag
pans. **Ctrl+Z / Ctrl+Shift+Z** undo/redo; **Ctrl+S / Ctrl+O** save/open.
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
Disclosure arrows expand/collapse; sheet names navigate. Symbols and Nets open
sheet-scoped Search tables. The browser has a height grip and scrolls when needed.
Search/ERC/Messages share the bottom drawer; its chevron collapses it. ERC currently
shows availability, not electrical-check results.

Use **Manage → Sheets → New sheet / Subsheet**, or browser context menus, to create
flat or hierarchical sheets. Choose a new definition or reuse an existing one.
**Capture → Port** adds interface ports; parent pins update automatically. Wire
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

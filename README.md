# PCBcraft — standalone EDA application

Repository: [travle1/pcbcraft-next](https://github.com/travle1/pcbcraft-next)
(private standalone repository, separate from the earlier `pcbcraft` fork).

PCBcraft is its own desktop application in the SolidCraft family. It owns its
executable, electronics model, session, commands, undo and native projects.
It targets reuse of qualified family UI/icon/theme libraries and SolidCore geometry through
explicit library boundaries. It does not require the SolidCraft app, its
mechanical engine, an installed SolidCraft copy or a sibling source checkout.

This [standalone decision](docs/adr/0001-standalone-application.md) supersedes the
earlier proposal to build an Electronics workspace inside SolidCraft. Schematic,
Layout and Render are views within PCBcraft. The user's FigCraft renderings remain
the visual authority; maximizing family reuse remains a requirement.

Original contributions use **MIT OR Apache-2.0**. The first runnable slice is a
standalone egui schematic workspace: eight original generic symbol definitions, placement,
selection, dragging, rotation, properties, authored wires/branches, undo/redo and
experimental local `.pc` save/open. It includes an original 16-component fan-controller illustration.
No implementation, fixtures or Git history from the preserved GPL fork have been
imported. Family artwork and font retain their [notices](assets/family/README.md).

## Run the app

From the repository directory (locally `/home/travisl/projects/PCBcraft/pcbcraft-next`):

```sh
cargo run --locked -- --demo
```

Omit `--demo` for an empty sheet, or pass a saved `.pc` path. After dependencies
are fetched, add `--offline`. The built executable is `target/debug/pcbcraft`.
Rust 1.95+ and a desktop graphics environment are required; this slice was built
and launched on the local Linux desktop with Rust 1.98.1.

Press **A** to open the Parts library. Search by name/value and choose Small,
Medium, Large or List view. Click a thumbnail then the sheet, or drag a part onto
the sheet; the library stays open for further placement. Drag its grip/title to
float it or release near the left/right edge to dock; the **…** menu offers the
same positions. The project Browser remains available. Short docks use a compact
list; floating restores the chosen thumbnail size. Close the library with **×**. Drag placed symbols to move, **R** to rotate and
**W** to draw a wire from a pin, existing wire or empty sheet. Click to add bends;
click a pin/wire to connect, or press **Enter** / double-click for a free end.
**Space** flips the next bend and **Backspace** removes the last corner.
Click a wire then **Delete** to remove it. In Select mode, drag a segment to slide
it, a corner to reshape it, or a free endpoint/junction to reposition it. Attached
pins stay with their components. Endpoint drops attach to wires/pins, and edited
collinear overlaps heal into clean runs; conflicting net names reject the edit.
Right-click a wire, symbol or the sheet for its
context menu. See [wire editing](docs/wire-editing.md) for exact behavior. **V** selects, **Esc** cancels the active tool,
**F** fits the drawing, scroll zooms and middle-drag pans. Use **Ctrl+Z / Ctrl+Shift+Z**
for undo/redo, **Ctrl+S** to save, **Ctrl+O** to open and **S** to search commands.
In Select mode, drag empty space with the left button, or right-drag, to box-select.
Left-to-right selects enclosed items; right-to-left includes touched items.
Shift adds; Ctrl toggles. Drag a selected group to move it, or Delete to remove it,
as one undoable edit. A stationary right-click opens the context menu.

**T** or **Capture → Place → Text** adds a sheet note at the last canvas position.
Click a note, component reference/value or net label to edit its content, font,
physical height, italic/underline, position offset and optional web link in Properties.
Drag text to reposition it; Ctrl-click opens a saved HTTP(S) link. **Apply** saves
one undoable edit; **Reset** reloads the current text. See [selection and text](docs/selection-and-text.md).
Edit component reference/value in its normal Properties and click **Apply changes**. File → Open example
loads the demo. Ribbon tabs switch the command groups below them; commands within
**View → Panels** reopen closed panels. Properties has a close button and a drag
grip to float or snap left/right; its **…** menu also sets position or collapse.
**More properties** opens the FigCraft-style General / Pin functions / Library view.
Done applies reference, value and rotation together as one undoable edit; Cancel
leaves the document unchanged. Browser selection highlights/bolds without moving rows.
Browser follows **Project → Sheets → Main → Symbols / Nets**. Disclosure arrows
only expand/collapse branches; the sheet remains active independently of object selection.
Click Symbols or Nets under Main to open that sheet's searchable result table in
the bottom Search drawer. The browser has a resize grip and scrolls when needed.
Click a net result or wire to highlight its connections and edit its name in Properties. Each connection has a review-before-remove control; Undo restores the
whole change. **Search → Nets → Netlist… → Copy JSON** generates a connectivity report.
Flat sheets and hierarchical subsheets cannot be added yet. Their creation actions
are disabled in the Project/Sheets/Main context menus; see [sheet workflows](docs/sheet-workflows.md).
External netlist import is not implemented. See [net and layout workflows](docs/net-workflows.md).
Search/ERC/Messages share the bottom drawer; its chevron collapses it. Search
finds placed symbols, while **S** opens ribbon command search (arrows, Enter, Tab,
Esc). Shortcuts defer to focused text fields.

This is a bounded schematic draft, not the full EDA framework. Crossings do not
create junctions; deliberately ending a wire on a segment inserts an explicit
junction. Free wires and their bends are saved along with pin connections. **Simulate — Planned**
appears in the workspace picker. ERC, simulation, PCB layout, 3D,
KiCad, external part libraries and manufacturing are not implemented. The UI marks
these tools accordingly. Read [implementation status and limits](docs/implementation-status.md)
before relying on saved drafts; `.pc` is provisional and this schema is experimental.

The [theme foundation](docs/theme-foundation.md) now uses verified SolidCraft dark
colors, semantic UI/canvas/icon roles and compact grouped controls. Your FigCraft
layouts remain the composition reference; theme selection is future work.
The [UI audit](docs/ui-audit.md) records the shared menu, action-button and checkbox
corrections and the reference/review requirements for further UI work.

**AI/MCP is a first-class planned authoring path.** The current app has no Assistant
connection or MCP server yet. The [AI authoring spec](docs/specs/ai-authoring.md)
defines selected context, component/connection proposals, review and one-step undo,
plus deterministic schematic arrangement so the model does not guess coordinates.
It records SolidCraft's actual prompt/skill/MCP approach and the next bounded tasks.

Read [AGENTS.md](AGENTS.md), [requirements](docs/requirements.md),
[approved sources](docs/sources.md) and [developer flow](docs/development-flow.md).
The [framework](docs/framework.md) covers domain/libraries, schematic/PCB/3D,
ERC/DRC, cross-probing, persistence, manufacturing and first-class UI.
The [implementation plan](docs/implementation-plan.md) starts with PCBcraft's own
desktop/library boundary and minimal Electronics contracts, then the visible
workspace and one editable component with save/reopen/undo. No crate per concept
or general plugin framework is required.

Many SolidCraft controls are still app-coupled. The [reuse register](docs/upstream-reuse-requests.md)
records focused library extraction requests and temporary-adapter criteria;
these libraries are not assumed available. SolidCore is a separate pinned
geometry dependency to qualify when needed; no Truck. Native `.pc` is provisional,
with an independent schema; `.sc` export is a possible later semantic adapter.

The separate `../integrations/solidcraft/` checkout and its
[build/startup record](docs/host-baseline.md) are reference evidence only. That
successful upstream launch was not PCBcraft. Its CLI error does not block this app.
The package has no SolidCraft app/engine, Truck or sibling-checkout dependency.
It bundles a small licensed family SVG/font set through a temporary adapter;
independent geometry/render libraries will be qualified when a slice needs them.

## Local checks

```sh
cargo fmt --all -- --check
cargo check --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
cargo test --offline --locked
cargo test --offline --locked --no-default-features
```

The tests cover core atomic edits/history/persistence and egui pointer interaction.
The no-default-features run checks the same core without desktop dependencies.
`origin` is `https://github.com/travle1/pcbcraft-next.git`; `main` holds the initial
sectional commits. No hosted CI, agent launcher or PR queue is configured.
Generated builds and local verification captures under `target/` and `work/` are
excluded from Git. The sibling FigCraft designs and SolidCraft reference checkout
are local reference material, not required build dependencies or repository contents.

With access to the private repository, a new checkout starts with:

```sh
git clone https://github.com/travle1/pcbcraft-next.git
cd pcbcraft-next
cargo run --locked -- --demo
```

The initial publication was checked from staged snapshots: the core commit passed
22 headless tests; the desktop commit passed all 59 tests and Clippy. The current
UI audit and feature limits are documented; this is not a manufacturing-ready release.

The active project path is `/home/travisl/projects/PCBcraft/pcbcraft-next`.
Open it through normal Codex project controls. Future implementation tasks should
start fresh from this guidance and approved sources without importing GPL-fork
review material as implementation context.

# Theme foundation and visual correction

November 3, 2026: the user rejected the first prototype's washed-out palette and
requested the supplied SolidCraft screenshot as a better palette reference.
The user's FigCraft layouts remain the composition reference. The earlier teal
approximation, overly large headings/padding and uniform monochrome icon treatment
are superseded. This is an implemented visual correction, not final UI approval.

## Current visual basis

The dark UI maps verified SolidCraft palette data at
`c553365a2197a689e749389a6fbf707a38df9b61`:
[palette values](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/assets/styles/ui/palettes.rs),
[theme roles](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/ui-egui/src/theme.rs),
[icon roles](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/assets/icons/ui/design/palette.toml).
These are data/reference mappings with original PCBcraft composition and adapters;
no app/engine dependency or whole theme engine was imported. Retained family
[notices and resource provenance](../assets/family/README.md) apply.

| Role | Current dark value |
| --- | --- |
| Application chrome | `#1c1f26` |
| Panel / toolbar surface | `#272b34` |
| Raised control / panel header | `#2f343e` |
| Input field | `#1e2229` |
| Border | `#3f4552` |
| Primary / secondary text | `#dee2e8` / `#949ba8` |
| Accent / selected surface | `#3a88ee` / `#2a466e` |
| Schematic canvas / minor grid / major grid | `#2e3440` / `#333944` / `#383e49` |

Original schematic roles distinguish symbol outlines, electrical links, reference
labels, values and selected geometry. A value label does not reuse a warning role.
The ribbon has grouped tools with icons above labels, following the reference
interaction structure. Panel radius is 6 points, body type 12, heading 14, with
compact control spacing. Shared workspace/move artwork retains multiple semantic
colors; navigation remains monochrome. Original PCBcraft app/symbol/wire glyphs
cover EDA concepts without displaying mechanical cubes as electrical components.

The user's subsequent close-up SolidCraft ribbon reference governs its proportions:
a narrow workspace card, quiet tabs/search row, aligned icon/label groups and a
separate caption row with full-height group dividers. The workspace and command
cards share their top/bottom edges. `Metrics::ribbon()` derives the height from
explicit row, inset and scroll-gutter budgets; larger type grows the ribbon instead
of clipping captions. `src/desktop/ribbon.rs` owns this composition and uses the
existing application actions. Dropdown chevrons are drawn paths, independent of
font glyph coverage. Horizontal overflow scrolls without changing tool order.

The subsequent FigCraft detail supplied by the user defines Capture command
content independently from the SolidCraft chrome/palette reference:

| Group | Promoted commands |
| --- | --- |
| Place | Symbol, Power, Port |
| Connect | Wire, Bus, Label |
| Modify | Move, Rotate, Mirror |
| Inspect | Rules, ERC, Annotate, BOM |
| Design | Sync |

Each group caption opens its menu. Both menu rows and ribbon buttons use the same
action/availability definitions and existing session commands. Delete is retained
in Modify's menu and the keyboard shortcut. Manage selects Project/History commands;
all five header tabs switch command groups. The customization `+` stays disabled.
This does not implement the planned commands: ERC opens availability/status only.

The schematic canvas now uses the user's detailed rendering for presentation:
subtle adaptive line grid, fine electrical strokes, smaller reference/value/pin
type, quiet selection, `1 / 1` sheet indicator and a wider icon/fit navigation
strip. `SchematicMetrics` and canvas color roles own the paint settings. Fit caps
magnification of small drafts; zoom remains available. None of these changes
alter electrical coordinates, snap spacing, identity or native document data.
Symbols, Pins and Wires are functional selection filters; unsupported selection classes
are disabled. Grid/snap/pin/wire visibility remain in View. The editable original
fan-controller illustration is retained; no Fan Controller bitmap or unimplemented electrical
content is substituted for the model. Current evidence: `work/proof/capture-workspace.png`.

The expanded selector now follows the user's open-workspace screenshot and
SolidCraft's actual `ui_kit::workspace_menu` component at the same pin. The
[licensed adaptation and replacement boundary](workspace-picker-provenance.md)
retain its 450-point rows, icon tiles, status pills, accent rails, real semibold
titles, description/tab hierarchy and footer. The whole workspace card is the
opener. Schematic is Current; PCB layout, 3D render and Simulate are Planned, disabled rows.
Do not mark a workspace Available until its destination actually works.

Escape, outside click and selecting Current dismiss the picker without changing
the project or its active editing tool. Planned rows stay open. Menu keystrokes
and the outside dismissal click do not reach the canvas. The popup stays below
the workspace card and scrolls its rows when vertical space is constrained.
Native evidence: `work/proof/workspace-picker.png`. Appearance remains contextual
theme state; the picker adds no document/session model or host dependency.

The user subsequently replaced the old centered Place symbol dialog with a
compact parts-browser reference. Following further review, `src/desktop/library.rs`
now opens a floating Parts library beside the Browser, with a draggable grip/title,
left/right edge snap preview and explicit Float/Dock left/Dock right menu. It
keeps the project Browser available. Dock stacks collapse neighboring panels when
space is limited; expanding them floats the library again. The custom header
has a close button (no window-collapse arrow), source/count row, search, four view
modes, symbol thumbnail tiles and a short placement footer. Medium uses two
columns; Small uses three; Large and List use one. Titles/values, empty search,
hover/focus/selected states and bounded scrolling use the contextual theme.
SolidCraft dark roles and actual schematic symbol previews remain the basis.
Short docks use a compact list and restore the preferred thumbnail mode when
space returns. Native evidence: `work/proof/figcraft-library.png` and
`work/proof/figcraft-compact.png`.

Clicking/keyboard-activating a tile starts a placement preview; dragging a tile
onto the sheet commits one normal Place command. Outside drops and Escape cancel
without a document edit. Drag payloads bind the runtime session incarnation and
revision, and stale drops are rejected. The panel stays open after placement;
search/view/selection are session-local UI state. The original built-in parts remain
the entire catalog; external library management and 3D previews are future work.

## FigCraft interaction and SolidCraft control pass

The actual `.figcraft` archive and its bottom-tools prototype were inspected,
including the paired drawer-open/closed navigation nodes and filter states.
The bottom launcher is now replaced by the open Search/ERC/Messages drawer.
Navigation and quiet tool guidance sit above it; the right filter rail has its
own bounds. No camera movement or electrical edit occurs merely from opening
these panels. Explicit Fit uses the visible drawing area when the drawer is open.
Search in the drawer finds real placed symbols; command search stays in the ribbon.
Panel headings use the reference's left-aligned title and trailing chevron.

SolidCraft's actual `ui_kit/panel.rs`, `fields.rs`, `layout.rs`, `search.rs`,
`preferences_layout.rs` and `assistant.rs` were read at c553365. The bounded card,
centered field and pinned catalog footer adaptations are documented in
[panel/control provenance](panel-controls-provenance.md). The card allocates its
outer rectangle once; content scrolls inside, so it cannot grow into the drawer.
Properties use the verified 70-point label column, 28-point controls and centered
field text. A visible grip indicates library mobility. Shared token colors remain
contextual. No whole UI crate, mechanical engine or new dependency was introduced.

Command search opens at the ribbon field with icons, command paths, selected and
disabled rows, shortcuts, empty results and a keyboard footer. Arrow keys select,
Enter runs, Tab fills the name, and Escape/outside click dismiss without editing
the canvas. The menu appearance timing is 120 ms, matching FigCraft's File/Connect
transitions; drawer/filter changes remain instant as authored. egui supplies the
fade easing. This is not a claim that every FigCraft prototype interaction exists.
Preferences, Assistant, the other workspaces and full EDA features remain planned.

Native review captures: `work/proof/figcraft-workspace.png`,
`figcraft-library.png`, `figcraft-compact.png`, and `solidcraft-command-search.png`.
These cover closed/open drawer, floating/docked library, a wire tool in progress,
compact layout and command search. Tests use actual egui area bounds and pointer
input, including snap/cancel, resize, preserved revisions and dismissal routing.
Settings/positions currently persist only for the application session.

## Theme boundary

`src/desktop/theme.rs` owns `Theme`, composed of `UiColors`, `CanvasColors`,
`IconColors` and `Metrics`. Widgets and painters read roles from the egui context.
Do not add palette literals to canvas code or individual panels. Transparent
paint and opacity operations are structural, not new palette colors. Layout
geometry may remain local; typography and common density metrics belong here.

`theme::set(context, theme)` publishes a complete theme and egui style between
frames. It requests redraw and affects subsequent widget/canvas rendering. State
is per context; there is no process-global mutable palette. Theme changes never
edit the electronics document, create undo entries or change its revision/dirty
state. Future saved appearance preferences belong to application settings, not
the `.pc` engineering payload. Future viewport/PCB layer palettes can extend this
presentation boundary without changing electrical identity or layer meaning.

The SVG adapter resolves stable source roles in memory. It retains original
masters and keys image caches by resolved icon palette, preventing old-theme icons
from surviving a theme change. Two-stage substitution prevents role collisions.
Only qualified resources are consumed; this is not a generic SVG/knockout engine.
Replace it with a qualified independent family icon package when available.

## Future themes and gates

Only the refined dark preset is shipped. A theme picker, persisted preferences,
external theme files, light/high-contrast/color-vision presets and appearance
transitions are future work. Do not expose unreviewed presets as finished features.
External files will need a versioned schema, bounded validation, explicit fallback
rules and migration; there is no external theme parser yet.

New presets must cover application chrome, fields, hover/pressed/focus/selected/
disabled states, schematic canvas, overlays and icon roles together. Validate text
and control contrast, avoid color-only state communication, preserve explicit
warnings/unavailable-check labels, and inspect actual empty/populated/dialog/error
states at representative sizes and display scales. Color meaning must remain
stable when a palette changes. Density must not change electrical units, snap or
geometry. No full accessibility/high-DPI qualification is claimed by this slice.

Tests cover per-context publication, unchanged electronics state, style/canvas/icon
agreement and cache invalidation including a source/destination color collision.
Ribbon tests also check unclipped text and aligned cards/captions at 1000/1600-point
widths, larger text and 150% pixel scaling, plus pointer dispatch for the existing
commands and workspace menu. These are focused egui checks, not full platform
accessibility qualification. Native rendered evidence is in local
`work/proof/ribbon-reference-pass.png` (and the earlier
`work/proof/refined-schematic.png`); it is a
review artifact, not a claim of pixel parity or a final polished product.

## Ribbon and inspector behavior correction

Capture, Analyze, Libraries, Manage and View are selected tabs with distinct command
groups, following SolidCraft's `toolbar.rs` tab-state handler at the pinned source.
They do not launch panels. The commands below them can launch panels or edits.
Selected View toggles show their state; unavailable actions remain disabled.

Properties now follows the actual `02-schematic-properties.png` compact inspector
and More properties composition, retaining the SolidCraft dark roles. The thumbnail
uses the selected definition, with tiny internal labels suppressed. Its grip and
position menu float/dock the card; × closes it and View → Properties reopens it.
The detailed modal has Component / Placement columns and truthful metadata.
Browser row geometry is constant across selection, which changes fill/font weight;
canvas selection changes symbol/reference ink without a large filled rectangle.

## Shared menu and action requirement

The [UI consistency audit](ui-audit.md) replaces the remaining plain command menus
with the verified family row/frame treatment, and centralizes primary/secondary/
destructive actions and checkboxes. `UiColors::danger` maps the upstream dark error
role (`#f06460`) independently from warning and electrical states. Menu radius,
spacing, typography, columns and icons share one implementation; no space-padded
shortcut labels. New UI work must use these helpers and rendered reference review.

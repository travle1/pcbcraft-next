# UI consistency audit — November 3, 2026

This pass corrects the user's rejected plain context menu and audits the other
implemented surfaces for the same default-widget shortcuts. It is a local
implementation and visual review, not a claim of final visual approval.

## Reference and scope

The user's actual `../designs/exports/workspaces-2026-10-30/01-schematic-overview.png`
was inspected again: it governs composition, grouped commands, restrained canvas,
Browser/Properties and bottom tools. The earlier inspected properties export and
`.figcraft` interactions remain authoritative. Designs are unchanged. The latest
supplied SolidCraft context-menu screenshot governs menu density and hierarchy.

The actual SolidCraft source at `c553365a2197a689e749389a6fbf707a38df9b61` was read:
`ui_kit/menu.rs`, `container.rs`, `actions.rs`, `checks.rs`, `context_menu.rs`,
`icons.rs` and `assets/styles/ui/palettes.rs`. Bounded licensed adaptations live in
[controls.rs](../src/desktop/controls.rs), with [provenance](panel-controls-provenance.md)
and retained notices. This reuses the verified control patterns without a dependency
on the SolidCraft application or its mechanical engine.

## Findings and corrections

| Surface | Problem found | Implemented correction |
| --- | --- | --- |
| Canvas right-click | Plain text buttons; no icon/shortcut columns; delete looked like a normal command; instructional paragraph inside menu | Shared 250-point popup, 30-point rows, 16-point icons, aligned shortcut column, grouped separators and red destructive actions. Wire/symbol/sheet actions retain the same session commands. |
| File | Spaces embedded in labels to imitate columns; ordinary default rows | Same menu component, real shortcut column, family icons, grouped lifecycle commands and busy-state gating. |
| Ribbon group menus | Separate image-button treatment and inconsistent widths | Same rows/frame; shortcuts and actual toggle states; unavailable commands remain disabled with explanations. Delete uses the actual family trash master. |
| Panel position menus | Default selectable labels and plain collapse button | Shared checked menu rows and separators; float/dock/collapse behavior retained. |
| Properties rotation | Default dropdown rows | Same checked menu rows within the existing rotation field. |
| Form actions | Save/apply/done and destructive actions had little hierarchy | Shared primary, secondary and destructive roles; accent commit buttons, quiet outlined secondary actions, red destructive text, consistent minimum target size and interaction shades. |
| Selection filters | Default gray checkbox treatment | Adapted family checkboxes: pixel-aligned boxes, accent checked state, focus outline, stable text bounds and native accessibility metadata. Unsupported filters stay disabled. |
| Net Browser toolbar | Count plus two larger actions exceeded the narrow panel | Count occupies its own line; both actions stay fully visible at compact width. |
| General control finish | Default 3-point corners mixed with family controls | Shared 6-point control corners, rounded popup frame, subdued border/shadow. All colors resolve from contextual theme roles. |
| Removal affordances | Close glyph used to delete a connection | Actual family trash glyph with semantic danger palette; close glyphs remain for closing panels. |

Browser selection, floating/docked cards, parts thumbnails, command search,
workspace picker, ribbon layout and canvas composition were reviewed as existing
custom surfaces. They retain their current layouts. The drawer and navigation
continue to follow the user's FigCraft composition. Native OS file dialogs remain
native. Text inputs, tabs and sliders still use egui interaction semantics under
the family theme; custom paint is used where a specific shared pattern requires it.

The menu's disabled states do not activate commands. Hover/focus uses the selected
surface and accent ink. Labels and shortcuts have independent bounds; long labels
truncate inside their row instead of moving neighboring columns. Semantic `danger`
is separate from warning, selection and electrical colors. Icon cache keys include
the resolved palette, including the danger ramp; original masters stay unchanged.

## Verification and remaining boundaries

Native screenshots in ignored `work/proof/` are review evidence:
`ui-audit-context.png`, `ui-audit-properties.png` and `ui-audit-compact.png`.
The first native review exposed truncated filter labels; intrinsic grid sizing
was corrected before the final captures. The compact capture then exposed an
overflowing Net Browser toolbar; its count/actions were separated and checked
again. The menu capture also waits for its
actual popup and freezes the injected pointer across press/release; normal launches
never inject input. No pre-existing user window is operated on.

Focused egui tests exercise menu dispatch, disabled ribbon commands, retained
context targets, exact-wire deletion/undo, opening-click isolation, distinct label
and shortcut columns, and complete filter labels at compact and larger-text sizes.
Existing panel/docking/property/filter tests remain in the suite. Build, formatting
and Clippy are checked in addition to rendered review.

This does not implement Preferences, Assistant/MCP, ERC, simulation, layout or
manufacturing. Those controls retain honest planned/unavailable states. Theme
selection, persistent panel placement, full keyboard/screen-reader qualification
and all display-scale combinations remain future work. This is not pixel-perfect
parity with every SolidCraft control or a completed FigCraft interaction prototype.

## Rule for subsequent UI work

Use shared menu rows, action tones, checkboxes, fields and card helpers. Do not
introduce a one-off plain menu, space-padded shortcut label, close-as-delete glyph,
new widget color literal or invisible mobility control. For a missing family
pattern, inspect its actual pinned source, record a narrow licensed adaptation or
qualified library use, and compare the rendered result with the user's design.
Inspect empty, selected, disabled, hover/focus and compact states; a successful
build alone does not qualify a visual change. Preserve one command authority and
the existing standalone boundary throughout.


## Selection and text follow-up

The November 3 selection/text pass retains shared menu, card, checkbox, field and
action roles. Window/crossing marquees use contextual selection/accent colors,
solid/dashed outlines and a hit count. Text Properties uses the existing movable
card, labelled fields and primary Apply/secondary Reset; Apply remains outside the
scrolling fields at compact sizes. Component Properties now allows room for the
reference/value style controls. Notes, attached labels and URLs are document data;
theme colors and panel placement remain presentation state.

Native captures inspected at 1600×1000 and 1000×700:
`work/proof/selection-text-inspector.png`, `selection-text-compact.png` and
`selection-crossing.png`. egui pointer tests cover both selection buttons and
existing right-click menus, while a compact inspector test checks visible Apply
and stale/reopened draft rejection. The full suite passes 69 tests (28 headless)
and warning-free Clippy. See [selection and text](selection-and-text.md) for scope.


## Browser hierarchy correction

The prior Symbols/Nets toggle and flat part list did not follow the user's FigCraft
navigation. The wire-edge total below that list was neither a net count nor a useful
navigation entry. Implementation-derived guidance that endorsed those controls is
superseded by the final schematic export and its sheet-navigation interaction recipe.

The Browser now shows Project → Project settings / Sheets → Main → Symbols / Nets.
Symbols/Nets now expand in-place sheet-local lists; item clicks select the object
without opening the bottom drawer. The Symbol glyph is the Capture ribbon glyph.
Search remains independently available. Disclosure hit targets have stable IDs
and do not alter object selection.
The active sheet and the selected result are separate states. The loose connection
count and permanent scrolling instructions are removed; a quiet resize grip remains.
The card sizes to its rows unless manually resized and scrolls when needed.

At that checkpoint only Main existed. Flat-sheet and hierarchical-sheet creation
were still disabled; the subsequent sheet implementation below supersedes that
limitation. Project settings remain unavailable. See [sheet workflows](sheet-workflows.md) for the implementation
sequence and acceptance criteria. No fictitious child sheets were added to the demo.

Reviewed the user's actual final schematic export and read-only navigation recipes,
plus SolidCraft `ui_kit/tree.rs` at c553365 for independent disclosure/selection and
stable row behavior. Navigation/result rows and the sheet glyph are original PCBcraft
code using the existing family theme/controls, without an app/engine dependency.
Native captures: `work/proof/browser-hierarchy.png`, `browser-net-results-compact.png`
and `browser-empty-compact.png`. Compact review caught clipped tree/result rows;
spacing and drawer height were corrected and recaptured. UI tests cover hierarchy,
fixed result positions on selection, collapse without navigation, disabled creation,
scrolling to the final item, real net edits/report access and compact/scaled bounds.


## Flat and hierarchical sheet controls (November 3, 2026)

Real sheet actions now use the existing menu/action/field tokens: Manage → Sheets,
browser context menus, creation and destructive-review dialogs, and contextual
Properties for sheet symbols and ports. The tree follows the user's final
`01-schematic-overview.png`: Main expanded, inactive children/siblings folded on
open, stable selection rows, nested children and sheet-local Symbols/Nets categories.
The browser sizes to visible rows and scrolls within the available viewport.

Native 1600×1000 and 1000×700 captures were reviewed at
`work/proof/sheets-main.png` and `work/proof/sheets-compact.png`. Compact review
caught title/pin collisions: titles now sit above blocks, matching the rendering;
port rows have more separation, long titles truncate, and tiny pin labels hide
below 35% zoom. A pointer/render test guards title/pin separation and zoom behavior.
Other tests cover live dialogs, scoped navigation, camera/selection restoration,
port wiring, draggable blocks, Delete review and save/reopen. All 80 tests and
Clippy pass. Local proof files are excluded from version control.

This is a schematic hierarchy slice, not a layout view, a complete FigCraft
animation implementation or an accessibility/performance certification.

## Demo, identity and panel follow-up

The user rejected the sparse example, overlapping labels, generic app icon and
Browser-to-drawer shortcuts. The demo is now an original 31-part study based on
`designs/exports/workspaces-2026-10-30/01-schematic-overview.png` for composition:
central controller, compact support circuits and surrounding interfaces. The
reference design file remains unchanged. All geometry/text is editable, with
explicit wires/junctions, supply aliases and stored text offsets. No screenshot
or external electrical design is embedded in the canvas.

Measured tests cover reference/value clearance from authored wires and full-size
text-to-text clearance. Native captures at 1600×1000 and 1000×700 are reviewed;
small overview detail still requires zoom. This is curated presentation, not a
general-purpose label arranger or an electrical validation result.

Symbols and Nets are independent collapsible Browser lists, initially folded to
keep sheets discoverable. Click a list item to select it; only sheet navigation or
item selection changes the active sheet. The drawer’s top grip resizes vertically,
clamped to the workspace, without changing camera, document or undo. Its height
persists across tabs/collapse within the session. Existing panel layering and
shared theme/control roles remain in use.

An original layered-board/P mark replaces the chip in the title bar and supplies
the native OS window icon. See [original asset provenance](../assets/original/README.md).

# Electronics workspace: visual and interaction contract

Status: standalone composition corrected after user clarification; applicable Astra
interaction contracts retained. The [first schematic slice](../implementation-status.md)
implements part of this contract; other views and full qualification remain planned.
[Framework](../framework.md).

## Visual authority and reusable foundation

Latest palette direction (November 3): the user prefers the supplied SolidCraft
dark screenshot over the first prototype's washed-out approximation. The
[implemented theme boundary and verified palette](../theme-foundation.md) govern
current colors, density and future themes. Keep the FigCraft layout/composition
below; do not treat its earlier teal palette as overriding this later direction.

Use Travis's own editable [FigCraft design](../../../designs/PCBcraft-main-window.figcraft)
and the six final exports below. All six were inspected as actual pixels for this
draft. Preserve their files. Generic FigCraft examples and an invented replacement
theme are not authorities. The Fan Controller artwork is a visual reference, not
an imported electrical model, licensed fixture or proof of application behavior.

| Reference | Observed design to retain |
| --- | --- |
| [01 Schematic overview](../../../designs/exports/workspaces-2026-10-30/01-schematic-overview.png) | Unified top document bar, blue capture accent, workspace selector/ribbon, sheet browser, compact inspector, separate selection filters, bottom check/search/messages and navigation |
| [02 Symbol properties](../../../designs/exports/workspaces-2026-10-30/02-schematic-properties.png) | Large focused properties dialog, tabbed sections, two-column fields, component and placement groups, provenance/files access, quiet Done action |
| [03 Layout top](../../../designs/exports/workspaces-2026-10-30/03-layout-top.png) | Same chrome with teal layout accent, grouped layer manager, copper/fill visibility controls, dark precise canvas, footprint inspector and selection filters |
| [04 Layout bottom](../../../designs/exports/workspaces-2026-10-30/04-layout-bottom.png) | Flip-view state and mirrored board presentation; selected physical layer remains independent from camera side |
| [05 Render isometric](../../../designs/exports/workspaces-2026-10-30/05-render-isometric.png) | Board assembly browser, missing-model count, large quiet 3D scene, Scene/Camera/Output settings, bottom camera presets |
| [06 Render top](../../../designs/exports/workspaces-2026-10-30/06-render-top.png) | Same scene/settings with top camera, board materials, SMT/through-hole visibility, stable framing |

Actual SolidCraft `ui-egui` uses egui 0.36 at the inspected revision. Its workspace
tables refer to concrete engine commands; the crate depends on engine and GPU.
Many `ui_kit` helpers are `pub(crate)`, not an externally available widget SDK.
Use these as references and qualified shared-library candidates. A public UI
symbol is not independent when its crate brings the mechanical engine. PCBcraft
owns its egui app/window and consumes narrow verified UI/icon/theme libraries;
needed private controls require focused extraction or a licensed temporary adapter.
The [reference experiment](../host-baseline.md) is not a PCBcraft launch.
The local icon project is a catalog, not an integrated SDK. Do not invent craft-ui
availability. [Standalone decision](../adr/0001-standalone-application.md),
[sources](../sources.md), [first milestone](../implementation-plan.md).

## Base workspace milestone

Compose PCBcraft's own window using verified family UI building blocks: one application menu, document tabs, workspace
selector, ribbon/command search, resizable panels, canvas, navigation and status.
Offer Schematic, PCB layout and Render views of one electronics project. Browser
entries reveal sheet occurrences/boards/assembly; no mechanical feature tree is
imposed. Keep PCBcraft project tabs distinct from sheet occurrence navigation.
The selector also lists **Simulate — Planned**, disabled until a real destination
exists. Simulation is not required to complete the first three-view milestone.

For the expanded workspace selector, the user's later SolidCraft screenshot and
the [bounded source adaptation](../workspace-picker-provenance.md) are the concrete
authority: full-card opener, icon tiles, Current/Planned badges, descriptions,
tab summaries and footer. The implemented selector exposes only Schematic as
usable; planned destinations cannot activate or change the document. Popup
dismissal must consume the click/escape before the editing canvas handles it.

The user's later ribbon/canvas close-ups confirm the Capture group membership
and finer schematic presentation recorded in the
[theme foundation](../theme-foundation.md). Preserve that command order and
caption dropdown behavior while retaining SolidCraft chrome colors. Header
customization and unimplemented EDA tools remain explicitly unavailable;
displaying an icon does not establish implementation. Keep the original editable
demo distinct from the dense Fan Controller design reference.

First complete application slice: one original editable component in PCBcraft's
standalone window; switch all three views, resize/collapse panels, use browser/inspector/
layers, cross-select the component, perform one semantic edit and undo/redo,
save/reopen provisional `.pc`, and verify GUI/API operation parity. Tools not implemented remain disabled with
a reason/capability tooltip; no fake ERC-pass, fake saved indicator, routing or
render result. The sample can use neutral placeholders for unavailable 3D rather
than importing the Fan Controller design or inventing a second rendering engine.
Use one PCBcraft Electronics session for edit/history/save; no separate
authoritative per-view state or SolidCraft mechanical Session dependency. Routing, full checks, KiCad
and manufacturing are not prerequisites for this workspace milestone.

### Browser, nets and coordinated layout views

The current Browser reserves separate header, list, footer and resize-grip space.
Use a persistent scrollbar and allow height adjustment within workspace bounds;
the last row must be reachable even at compact window sizes. Symbols/Nets changes
only the list. Net selection highlights explicit connected terminals/links and
opens its movable Properties inspector; editing and reviewed removal use the
Electronics session. Both Browser and overflowing Properties retain visible
scroll affordances. [Current net interactions](../net-workflows.md).

Planned layout interaction starts in the same project window. Workspace selection
switches the main viewport; an optional schematic/layout split later gives two
independent cameras and one shared document, selection authority, undo and save.
The focused viewport owns the ribbon, inspector and local tools. Project tabs
remain distinct from workspace views and sheet navigation. Do not create duplicate
documents or use exported netlists to keep internal views synchronized.

Cross-probing uses explicit component/terminal/footprint/pad correspondence,
reports missing/ambiguous targets and avoids selection feedback loops. Live
selection does not silently apply electrical changes: reviewed, revision-checked
ECO handles those changes and preserves board-local intent. Detached windows may
follow after single-window/split focus and lifecycle contracts are proven. Neither
layout nor split view is implemented by the current schematic slice.

## Common visual language

Retain the reference's restrained dark slate/teal surfaces, rounded floating
panels, narrow dividers, muted text/icon hierarchy, generous main canvas and small
workspace accent changes. Reuse verified family semantic theme tokens, typography and
icon sizing rather than extracting approximate screenshot colors into a separate
theme. Define reviewed aliases for canvas/grid, panel/surface/input, normal/muted
text, focus, selection, hover, error/warning, copper/layer roles and stale analysis.
Electrical colors and selected layers are semantic and user configurable.

Use the verified family spacing/type scales with clear title/body/caption/value hierarchy,
consistent field rows, aligned units and predictable icon baselines. Numeric
properties retain precision independent of display rounding. Status is conveyed
by text and shape as well as color. No gradients, decoration, added dashboard
cards or oversized controls absent from the user's design. Light theme uses the
shared equivalent roles and retains hierarchy; the provided dark exports are its
layout authority, not proof that light-theme values have been designed.

Use logical points and egui/platform DPI handling in PCBcraft. At 1440×900 logical size preserve the
reference composition; at smaller sizes collapse secondary panels in a deliberate
order and permit command overflow/scroll rather than overlapping the canvas.
Inspector/dialog content scrolls, minimum canvas remains usable, and panels can
be restored by keyboard. At 1×/1.5×/2× icons/text/lines remain crisp, fields do not
clip, and physical-coordinate accuracy never depends on display scale. Exact
minimum window size and density presets are review choices in the host spike.

## Editing and navigation behavior

Selection, hover, highlight, active tool and electrical connectivity are separate
states. Use one canonical interaction selection store and semantic mappings for schematic/
PCB/3D cross-probing. Only new user/API actions emit selection intents; projecting
views never re-emit them. Keep occurrence context/propagation tokens and validate
scene generation on picks. Show
one-to-many matches, unlinked targets and stale projections explicitly. Preserve
context when moving between views. Auto-reveal/tab-switch is opt-in, no echo loops.
Layer visibility, active edit layer, fill visibility and camera side are separate
controls; flipping the view does not move components or change their layer.

Tools expose start/preview/commit/cancel, snapping/grid and active layer. Escape
cancels the preview before exiting a tool; a drag commits one authoritative session undo operation.
Selection filters and locked/hidden objects govern picks predictably, with a
disambiguation list for overlapping entities. Pan/zoom/fit reuse host planar
navigation; 3D orbit/presets reuse host camera behavior. Text-field shortcuts do
not leak into canvas commands; right-click menus show the same command labels.

Inspector edits validate units/ranges and show pending/error states without
silently normalizing an invalid design. Multi-selection distinguishes mixed values.
Full properties edits form a reviewable atomic change; closing/cancelling cannot
leave partial edits. Library browser supports symbol/package preview, mapping
status, pinned source/license, offline availability and generic parts without
manufacturer data. Upgrades/ECOs show added/removed/changed/conflicted objects,
route impact and stale-preview rejection before Apply.

The user's later parts-library reference supersedes the initial centered symbol
picker presentation: use a movable parts browser with thumbnail size/list modes,
search and clear count/source context. It floats beside the Browser or snaps into
a left/right stack; it must not replace the Browser. The position menu provides
the same choices without dragging. Short stacks use a compact list. Keep the
library grip, collapse controls on other panels and close action visually distinct. The current
built-in implementation supports click/drag placement and retains the panel while
placing. The open bottom drawer replaces its launcher, shifts navigation upward
and stays clear of the filter rail. See the [verified interaction/control pass](../theme-foundation.md#figcraft-interaction-and-solidcraft-control-pass).
General component metadata, external catalogs and 3D thumbnails remain
separate library-management work; see [implemented status](../implementation-status.md).

## Checks, outputs and complete states

ERC/DRC panels show not-run/running/current/stale/partial/failed/cancelled states,
coverage profile, counts by severity, explanation/witness, rule source and waiver.
Selecting a finding highlights its objects and reveals context without losing
the list. A green check never means unimplemented checks passed. Status bar shows
tool, coordinates/units, grid/snap, layer, save state and check freshness without
turning into an implementation log. Developer details belong in a diagnostics view.

3D shows missing/failed model placeholders and counts, declared variant/DNP/view
policy (including optional ghosted unfitted parts), loading
and stale badges. Render settings distinguish appearance from actual stackup and
fabrication data. Output review selects board, assembly variant, profile and
pinned revision; presents checks/unsupported data, progress/cancel, manifest and
destination. Export from an older snapshot remains labeled with its revision.

Empty states provide a single useful next action. Offline/missing libraries,
unknown format, save conflict, recovery choice and unsupported host capabilities
have actionable explanations. Preserve the user's content on all failures. Never
hide missing dependencies behind an endless spinner.

### First-class Assistant

Assistant is a core authoring surface, available across workspaces as they ship.
The user's final schematic overview has its bottom-right entry; the earlier
[AI review rendering](../../../designs/ai-review.png) supplies selected-context,
proposal summary and Preview/Apply/Discard behavior. Apply current SolidCraft dark
tokens to that interaction hierarchy, not the older rendering's light palette.
The panel floats/docks, closes/reopens and keeps Browser available; opening it
does not move the camera. Show actual connection state, scope/context chips,
optional view attachment, prompt composer, tool activity and Stop.

Present electrical changes separately from presentation-only arrangement, with
highlighted before/after geometry, unresolved assumptions, check coverage and the
proposal's base revision. Apply creates one undo entry; Discard changes nothing;
stale previews require regeneration. No fake connection or green electrical check.
Manual Arrange uses the same deterministic service without an agent. An AI/cloud
account is optional for basic design. These are planned controls, not current UI;
the [AI authoring spec](ai-authoring.md) defines the implementation sequence.

## Accessibility and visual acceptance

Reuse host focus traversal, accessible labels, command palette and platform
shortcut bindings. All essential panel/tool/property actions work by keyboard;
visible focus survives view switches, dialogs trap/restore focus and Escape is
predictable. Tooltips supplement labels; icon-only controls have accessible names.
Color-independent highlights and adjustable contrast/line weight are required.
Screen-reader coverage must be tested through the chosen egui/host accessibility
path; a canvas alternative exposes selected entities and findings as structured
lists rather than claiming geometric screen-reader support for free.

UI-1 acceptance captures the six corresponding view states using original
synthetic electrical data, deterministic viewport/camera, fonts, theme and DPI.
Review composition against these exact user exports: chrome/panel hierarchy,
tool density, selection/inspector, layer/camera independence and board framing.
The electrical content need not reproduce Fan Controller. Add representative
empty/loading/error/stale/disabled-tool, small-window and keyboard-focus captures.
Mask only genuinely nondeterministic timestamps/cursors; approve meaningful
pixel differences manually. Record host/source revisions and screenshot settings.
Run interaction checks in addition to screenshot comparison. UI polish is a gate
for every visible slice, not a final cosmetic phase.

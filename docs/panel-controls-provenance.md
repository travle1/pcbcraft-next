# Panel and control adaptation

The November 3 polish pass inspected SolidCraft at
`c553365a2197a689e749389a6fbf707a38df9b61`, including these actual components:

- `crates/ui-egui/src/ui_kit/panel.rs`: bounded workspace cards, shared header,
  drag grip, side-change menu and independent scroll area.
- `crates/ui-egui/src/ui_kit/fields.rs`: vertically centered text inputs.
- `crates/ui-egui/src/ui_kit/layout.rs`: catalog body/footer allocation and
  aligned form controls.
- `crates/ui-egui/src/ui_kit/search.rs`: expanding ribbon search, result rows,
  command paths and keyboard footer.
- `crates/ui-egui/src/ui_kit/preferences_layout.rs` and `assistant.rs`: responsive
  control packing and fixed header/body/footer boundaries.
- `assets/styles/ui/`: default form metrics (28-point controls, 70-point label
  column, 6-point control radius, 9-point field inset, 40-point header band).

`src/desktop/controls.rs` is a narrow adaptation of the bounded card, text-input
alignment and catalog footer patterns. SolidCraft contributors, copyright 2026,
MIT OR Apache-2.0; retained notices are in `assets/family/SOLIDCRAFT-LICENSE-MIT`,
`SOLIDCRAFT-LICENSE-APACHE` and `SOLIDCRAFT-NOTICE`. It maps upstream tokens to PCBcraft's contextual
theme and omits mechanical engine state, scenario infrastructure, global tokens
and translation services. No reference checkout is modified or linked.

PCBcraft owns the surrounding layout, library docking state and EDA commands.
The command palette follows the upstream composition with PCBcraft's own command
catalog/dispatch. These private upstream components are not a published SDK.
Replace the narrow adaptations with a qualified independent upstream UI package
when available; preserve keyboard behavior and layout tests. No upstream request
has been sent.

The user's `designs/PCBcraft-main-window.figcraft` remains the composition and
interaction authority. The archive was read without modification. Nodes 6878,
6888, 4847/6862 and 7017/7022 define drawer/launcher, navigation and filter states;
File/Connect interactions use 120 ms ease-out dissolves. Drawer/filter transitions
are explicitly instant, despite a serialized default duration of 300 ms.
No additional animation was inferred from that unused duration.

The subsequent correction also read `crates/ui-egui/src/toolbar.rs` at the same pin:
`ribbon_tabs` sets `app.ui.tab` on click. PCBcraft's original typed ribbon-tab state
and group tables follow that interaction contract, without importing the app's
command registry. The user's compact inspector and detailed General / Pin functions /
Library composition were checked against `02-schematic-properties.png` and the
read-only `designs/compact_properties.py`. Current inspection/docking composition
is original PCBcraft code using the existing licensed control adapter.

## Menu, action and checkbox audit

The subsequent app-wide consistency pass read and narrowly adapted the same pin's
`ui_kit/menu.rs` (`MenuItem`, menu wrapper, separator), `container.rs` (popup frame),
`actions.rs` (semantic action tones and interaction shades) and `checks.rs`
(pixel-aligned checkbox, check glyph, focus/accessibility states). The shared
adapter in `controls.rs` retains source attribution. PCBcraft omits upstream
translations, preference search, scenario handles, submenu/platform infrastructure
and global token access. Menu command dispatch, drawing operations and session
ownership remain PCBcraft's. Normal/primary/danger form treatment is a bounded
adaptation; it does not claim every upstream bevel/contrast/theme mode is present.

`model/delete.svg` and `model/measure.svg` were copied unchanged from
`assets/icons/ui/design/svg/` at c553365, with exact hashes in the resource register.
Delete shades its faces using the semantic danger role while preserving the
master's outline and white slot strokes. No generic mask/knockout processing is
claimed. Replace these controls/resources with a qualified independent family
package when available. See the [audit and verification scope](ui-audit.md).

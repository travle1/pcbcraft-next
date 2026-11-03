# Workspace picker component

This is a narrow licensed adaptation of SolidCraft's workspace row composition,
not a copy of its application shell or mechanical workspace dispatcher.

- Source: `tangency-labs/solidcraft`, revision
  `c553365a2197a689e749389a6fbf707a38df9b61`.
- [Row painter and popup layout](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/ui-egui/src/ui_kit/menu.rs),
  `workspace_menu`; [whole-card opener](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/ui-egui/src/toolbar.rs).
- Author/license: SolidCraft contributors, MIT OR Apache-2.0. Retain
  [MIT](../assets/family/SOLIDCRAFT-LICENSE-MIT),
  [Apache](../assets/family/SOLIDCRAFT-LICENSE-APACHE) and
  [NOTICE](../assets/family/SOLIDCRAFT-NOTICE).
- Consumer boundary: `src/desktop/workspace.rs` uses only egui, PCBcraft theme
  roles and the existing icon adapter. No upstream `SolveApp`, `Item`, engine,
  session, command registry or UI crate dependency.
- Adaptations: three electronics entries, honest Current/Planned status,
  current-project-preserving dismissal, bounded scrolling, real font weight,
  keyboard focus and themed paints. Planned rows cannot activate or dismiss the
  picker. Popup lifetime belongs to egui, not the electronics document.
- Replacement: consume a qualified standalone family workspace-picker widget
  with entry data, capability/status, theme/icon inputs and selection output
  when available. Preserve behavior and tests; no upstream request has been sent.

The expanded SolidCraft screenshot supplied by Travis is the visual reference.
The current app supports Schematic only; this component does not implement PCB
layout, 3D rendering or shared selection across those future views.

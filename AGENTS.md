# Agent instructions

This is the active new project. Read README.md, docs/requirements.md,
docs/sources.md and docs/development-flow.md before work. Do not use the sibling
`pcbcraft/` checkout, its source-derived notes/fixtures or another GPL EDA
implementation as an implementation reference. Historical material in the
parent workspace is not an instruction to inspect or import it.

Read docs/framework.md, docs/adr/0001-standalone-application.md and task-linked
specs before implementation. PCBcraft is a STANDALONE APPLICATION with its own
executable, window, settings, native file lifecycle and Electronics session.
The user explicitly corrected the earlier compiled-in SolidCraft proposal.
Do not implement PCBcraft inside SolidCraft or make its app/mechanical engine a
build/runtime prerequisite. The parent integrations/solidcraft checkout is a
reference only, not a Cargo member, sibling path dependency or executable to launch
as PCBcraft. Its successful startup is not a PCBcraft acceptance test.

The excluded mechanical engine is the stateful application/document engine
(`solidcraft-engine`), not independent geometric computation or rendering.
Qualified `solidcraft-kernel` (SolidCore-backed), `solidcraft-render`,
`solidcraft-gpu`, geometry/record and STEP utilities are legitimate library
candidates. See docs/specs/geometry-outputs.md for pinned dependency evidence.

Maximize verified shared-library reuse: icons, themes, controls and independently
separable utilities from the SolidCraft family; SolidCore for geometry. Public
symbols inside solidcraft-ui-egui are not an independent SDK when their dependency
closure pulls in the mechanical engine. Qualify actual package APIs, pins,
licenses, features and dependencies. Propose narrow upstream extractions for
needed private components; do not invent crates or copy the full app shell.
PCBcraft owns its egui application composition and uses the user's FigCraft work.

Read docs/theme-foundation.md and docs/ui-audit.md for presentation work.
Use the shared controls for menus, action roles, checkboxes, fields and cards.
Inspect the actual pinned SolidCraft implementation for missing patterns; do not
fall back to default text menus, space-padded shortcuts or one-off colors.
Validate rendered empty/selected/hover/disabled/compact states against the user's
references, not only compilation. The user's latest palette
reference is the supplied SolidCraft dark screenshot; FigCraft remains the layout
reference. Use contextual semantic UI/canvas/icon roles and shared metrics, with
palette-aware icon caches. Do not scatter literal colors through widgets or put
appearance settings in the electronics document. The theme picker and external
preset format are future work, not existing facilities.

One PCBcraft Electronics session owns aggregate/operations/revisions/undo/save;
GUI/CLI/MCP use it. Independently reusable history/job/container utilities may
support it without imposing a mechanical Document or Session. No per-view or
transport-specific commit authorities, and no mechanical evaluation for pure
electrical edits. No initial cross-document ECAD/MCAD transactions.

Native electronics uses an independent schema and provisional .pc project suffix;
.sc export is only a possible later feature. Direct pinned SolidCore APIs and
shared UI utilities are separate dependencies. Only selected capabilities gate a
slice; do not require the full SolidCraft build/migration or its CLI to pass.
No Truck in selected builds; unrelated lockfile membership alone is not failure.

Original project contributions use MIT OR Apache-2.0. Record exact source,
revision, license and use before importing reusable components or data, and
preserve their notices. No inherited implementation is included; this is not a
certification of prior exposure or future upstream eligibility.

- Maximize actual family-library reuse within the standalone boundary. Each
  extraction/temporary adapter needs provenance, a narrow API and replacement plan.
- New geometry targets SolidCore's shared layers. No Truck dependencies, new
  Truck code or fallback. Do not invent dependencies on unshipped APIs.
- Keep electrical identity, connectivity, PCB rules and pin/pad mappings explicit.
  Geometry handles and render caches cannot replace electrical identity.
- PCBcraft's single Electronics session is the transaction/undo authority for every electronics
  view and UI/CLI/MCP. No cross-document ECAD/MCAD transactions initially. Reopen
  changes runtime incarnation; undo/redo advance revisions; retries need idempotency.
- Implement KiCad support independently from published format specifications,
  user manuals and black-box tests. Do not inspect, copy, adapt or translate
  KiCad implementation source, headers, patches or annotated-source/Doxygen
  implementation pages. Stop reading a reference if it contains implementation.
- Licensed symbol/footprint data may be used with recorded provenance/notices.
  Do not copy the old fork's fixtures; data retains its own license.
- Bound and validate inputs and return structured errors. No unsafe or production
  unwrap/expect/panic/todo/unimplemented/unreachable. Keep slow work off the editor.

Keep each task small, with allowed files and acceptance conditions. One
implementing agent per isolated checkout/branch; settle shared interface ownership
before parallel work. Do not launch agents just because a backlog exists.

Run formatting, Cargo checks and relevant behavior tests. Add tests when there
is a meaningful contract, not merely to give an empty starter a test count.
Inspect actual rendered output for UI changes.

The standalone repository is https://github.com/travle1/pcbcraft-next;
`origin` points there. It was created private on November 3, 2026 under the user's
explicit authorization to commit the existing work in sections and push it.
Do not point this repository at the preserved fork. Future tasks should state their
commit/push/PR scope; new destinations, visibility changes, upstream contributions,
dependency/security configuration changes and messages need appropriate explicit
authority. No agent launcher, PR queue or hosted CI is configured.

# 0001: PCBcraft is a standalone application

Accepted by explicit user correction, November 3, 2026. This supersedes the
earlier compiled-in SolidCraft workspace and mechanical Session reuse proposals.
The earlier Astra review does not constitute review of this corrected deployment
plan. Its domain, transaction, library, geometry and validation contracts remain
useful where independent of deployment.

## Product and ownership

PCBcraft has its own executable, application identity, window, settings, project
open/save lifecycle and packaging. It runs without SolidCraft installed, running,
or checked out beside it. Schematic, Layout and Render are views within PCBcraft.
The active repository is `pcbcraft-next/`. The original dependency-free foundation
was retained and extended with the [first standalone schematic slice](../implementation-status.md).

One PCBcraft Electronics session owns the complete project aggregate, commands,
revisions, undo/redo, dirty state and save/recovery coordination. GUI, CLI and MCP
invoke that same authority. Reuse proven independent utilities underneath it;
do not depend on SolidCraft's mechanical `Session`, `Document`, feature tree or
command registry. No separate per-view histories or authoritative documents.

## Reuse boundary

```text
PCBcraft executable / egui desktop composition
  ├─ PCBcraft Electronics session and domain
  │    ├─ PCBcraft native persistence, libraries and format adapters
  │    └─ analysis/jobs; geometry through qualified SolidCore adapters
  └─ verified shared UI/icons/theme/render utilities

SolidCraft application / mechanical engine: no dependency edge
```

The excluded engine is application/document orchestration (`solidcraft-engine`),
not every library capable of mechanical operations. The independent
`solidcraft-kernel` wrapper over SolidCore and `solidcraft-render`/`solidcraft-gpu`
are valid candidates after capability and external-consumer qualification.
PCBcraft may reuse extrusion, booleans, STEP, cameras, tessellation and rendering
without inheriting SolidCraft's mechanical document, command history or app.
See [the verified crate map](../specs/geometry-outputs.md#verified-geometry-and-rendering-library-candidates).

Maximize real family-library reuse while respecting this boundary. Shared crates
may be published packages or pinned packages from an upstream repository; their
repository location is not the issue. Their selected dependency closure and API
must work independently of the SolidCraft application/mechanical engine.
Cargo may fetch a monorepo to resolve a pinned Git package; this does not mean the
whole application is compiled or shipped. Audit the selected normal/build graph,
features and assets. Upstream crate test dependencies are a separate graph.

At the inspected pin, importing all of `solidcraft-ui-egui` brings engine coupling
even for some public widgets. A public symbol is not an independent library.
App-private icons/themes/controls are candidates for focused upstream extraction,
not permission to embed the whole app. Record source, revision, license, API,
feature/dependency closure and a minimal external-consumer check per component.
Do not invent an available `craft-ui`, icon SDK or generic workflow framework.

Use actual reusable packages where verified. For needed unpublished pieces,
define a narrow extraction or licensed temporary component/resource adapter
with provenance, tests and an upstream replacement plan before importing it.
PCBcraft owns its application composition; it need not wait for a universal shell
SDK or every upstream extraction. Avoid a private copy of the full SolidCraft
shell and avoid rewriting reusable widgets or geometry unnecessarily. Upstream
requests remain drafts until explicitly authorized to send.

SolidCore is a separate direct library concern. Pin and test the specific APIs
required by a slice; electrical identity and undo remain in PCBcraft. No Truck.
The full SolidCraft build or its CLI failure is not a PCBcraft build gate.

## Files and visual authority

The provisional `.pc` project belongs to PCBcraft, with its own schema/type marker,
file associations, versioning and migrations. Independent container/atomic-write
utilities may be reused, without requiring SolidCraft `FileKind`, mechanical JSON
or metadata extensions. Possible later `.sc` export is an explicit semantic adapter.

The user's FigCraft renderings remain the visual authority. Compose that workspace
in PCBcraft using verified family design tokens, icons and controls. A standalone
window does not imply a new visual language.

## Evidence and next gates

`../integrations/solidcraft/` relative to the active repository is a separate
reference checkout. The [upstream startup experiment](../host-baseline.md) remains
valid evidence about SolidCraft, not proof that PCBcraft starts or that shared
libraries have been extracted. Do not add this checkout to PCBcraft's Cargo
workspace, use sibling path dependencies, launch SolidCraft as PCBcraft, or make
PCBcraft tests invoke its executable.

W0 now maps the independently consumable dependencies and establishes PCBcraft's
own desktop build boundary. F1 defines the Electronics contracts. W1 starts the
PCBcraft window with the three workspace views; H0 proves one original component,
edit/undo/redo, `.pc` save/reopen and cross-selection inside that application.
See the [implementation plan](../implementation-plan.md) for ordered tasks.

Acceptance includes building from a clean PCBcraft checkout with only its declared
pinned dependencies, no sibling SolidCraft checkout or installed SolidCraft app;
isolated PCBcraft settings; GUI/headless parity through one Electronics session;
and the user's visual requirements. Claims of independence require a selected
dependency audit, not just a different executable name.

The preserved GPL fork, no-KiCad-implementation-source rule, original licenses,
FigCraft designs and publication restrictions remain unchanged.

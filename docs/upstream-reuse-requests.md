# Upstream reuse and library request register

Upstream requests are planning only; revised for standalone PCBcraft November 3, 2026. No requests,
issues, messages or PRs sent. [Decision](adr/0001-standalone-application.md),
[framework](framework.md), [reference evidence](host-baseline.md).

PCBcraft is an external application consumer. Maximize actual reuse through
small independent libraries, without a dependency on SolidCraft's app/mechanical
engine. Do not require a generalized platform extraction. Where a facility is
unavailable, record the needed boundary and a bounded temporary-adapter plan;
do not silently embed the full application or claim a package already exists.

| Verified surface | External consumer need / dependency boundary | Proposed upstream request | Temporary approach and replacement |
| --- | --- | --- | --- |
| Semantic SVG masters, palettes, aliases and small variants; host runtime exists but no standalone SDK verified | Licensed artwork and stable semantic lookup without ui-egui/engine | Focused catalog package and minimal egui adapter, preserving notices and theme/variant contract | Small pinned licensed resource adapter now implemented with theme-resolved SVG roles and palette-aware texture keys; replace with maintained independent package when available |
| Theme tokens and ui_kit; some public exports, many private helpers | Same spacing/type/focus/property/panel behavior without mechanical Session/commands | Independently consumable egui-compatible token/control package for specifically used widgets | Original contextual theme adapter now maps a bounded subset of verified palette/metric data; replace this mapping with a qualified shared token package when available, preserving PCBcraft canvas roles. No full ui-egui dependency |
| Workspace/ribbon presentation tables bind mechanical commands; workspace card painter is private | Compose PCBcraft menus/tabs/ribbon from its own semantic command descriptions | Extract a workspace picker taking entry/status, theme/icon inputs and returning selection, without host command types | A bounded [licensed card-picker adaptation](workspace-picker-provenance.md) is now used; replace it with the independent widget when qualified. PCBcraft retains window/view and command ownership |
| IO/container/atomic-write, lifecycle/job utilities are app-coupled to varying degrees | PCBcraft schema, settings and one Electronics authority using generic utilities | Extract only proven generic IO/history/job helpers with explicit data/bounds/cancellation contracts | Qualify existing independent utility crates or narrowly extracted helpers; retain PCBcraft state ownership and replace adapters without migrating electrical IDs |
| Existing `solidcraft-render`, `solidcraft-gpu` and SolidCore-backed `solidcraft-kernel`; selected normal/build graphs exclude engine/app | Reuse cameras, scenes, GPU pipelines/capture/tessellation and geometry for PCBcraft's scene adapter | First qualify these actual libraries externally; request only missing operations or app-private render controls, not extraction of already separate backends | Pinned direct library consumption when validated; PCBcraft owns electrical IDs, scene construction and UI. No dependency added yet |

Evidence rechecked November 3 at the pinned baseline:
[UI-kit exports](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/ui-egui/src/ui_kit/mod.rs),
[workspace tables](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/ui-egui/src/workspace.rs)
and the [source register](sources.md). Public controls include `Button`, `Container`,
`Surface`, `NameList` and `Window`; menu, tree, metrics and many panel helpers remain
crate-private. `icons::paint_icon` is public within the UI crate, not a standalone
icon package. Public visibility inside an engine-dependent crate does not satisfy
standalone consumption. Each candidate needs an external-consumer build and
selected dependency audit; crate packaging/compatibility are not assumed.
The [geometry/rendering crate map](specs/geometry-outputs.md#verified-geometry-and-rendering-library-candidates)
records already separate backend libraries; keep them distinct from the coupled
render workspace UI.

The separate `/home/travisl/projects/solidcraft-icons` catalog/review workspace
supplied the first [pinned SVG resource bundle and temporary egui adapter](../assets/family/README.md),
not an installed SDK dependency. Preserve its provenance lock,
per-resource licenses/notices, protected artwork and pinned revision; leave it
unchanged. Eventual upstream catalog consumption should replace a temporary
adapter, not erase provenance or create a permanently forked icon set.

Each future request records exact source revision/private symbol, consumer use
case, minimal inputs/outputs, semantic/versioning ownership, selected dependency
constraints, original acceptance fixture and migration/removal condition. Confirm
the ask is still needed in current upstream before sending it. This register is
not authority to contact developers, file issues, publish packages or open PRs.

AI follow-up candidate at c553365: `agent_chat/backend.rs` defines a useful provider
seam and `ui_kit/assistant.rs` has conversation/composer/activity/review controls.
They are still inside the app-coupled UI crate; context gathering imports the
mechanical Session. A focused future request could expose provider events,
attachment/context inputs and a caller-supplied tool dispatcher, plus themed
Assistant controls, without SolidCraft command/document/engine types. PCBcraft
would supply EDA context, guidance, permissions and its one session. Until a usable
library is qualified, implement only the bounded PCBcraft adapter needed by
[AI0–AI3](specs/ai-authoring.md), with an explicit replacement boundary; no whole
agent-chat copy or independent provider framework. This request has not been sent.


The later [bounded panel/control adaptation](panel-controls-provenance.md) provides
a concrete extraction candidate: an engine-independent fixed outer card, typed
header/body/footer layout, centered fields and catalog footer. The standalone
consumer needs theme/geometry inputs and interaction outputs, with no global
Tokens, Scenario, mechanical Session or command types. This replaces the temporary
adapter when independently packaged; PCBcraft retains EDA state and composition.

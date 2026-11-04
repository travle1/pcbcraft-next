# Source register

## Required source boundary

Read the [independent implementation and source policy](source-policy.md) before
implementation or source research. PCBcraft is starting over with independently
written code, not relicensing or porting the preserved GPL fork. KiCad implementation
and the fork's code, source-derived notes/fixtures, translations and AI rewrites
are excluded, including when supplied through searches, tools or other agents.
Historical entries below record provenance and prior exposure; they do not authorize
new use of excluded material. Record qualified reuse with its exact source,
revision/version, license option, scope, modifications and retained notices.

## Assistant conversation and ACP (November 4, 2026)

The bounded conversation state, worker seam, snapshot-bound review-only dispatch,
ACP stdio adapter and scripted peer are original PCBcraft implementation/test code.
They use existing std, serde/serde_json and tempfile facilities; no dependency,
provider SDK or new asset was imported. Read the official ACP v1
[initialization](https://agentclientprotocol.com/protocol/v1/initialization),
[session setup](https://agentclientprotocol.com/protocol/v1/session-setup) and
[prompt-turn](https://agentclientprotocol.com/protocol/v1/prompt-turn) contracts.
The latest published v1 is the selected bounded protocol; v2 draft and
provider-specific extensions are not claimed as supported.

Read SolidCraft's `agent_chat/backend.rs` and selected ACP connection/bridge
functions at c553365 as architectural references for caller-owned context,
private live tools and a provider boundary. No whole backend/chat/UI implementation
was copied. Existing licensed card/action/theme adapters remain in use, with the
same replacement boundary; the upstream extraction is deferred. Normal launches
start no agent. Qualification uses an original scripted process, not a real model
or account; [behavior and remaining limits](assistant.md) distinguish these.
No KiCad/old-fork implementation, derived notes or fixtures were consulted.

## Assistant placement correction (November 4, 2026)

Inspected the user's actual schematic-overview and AI-review images plus the
read-only `assistant_drawer_behavior.py` recipe. FigCraft specifies the bottom-right
launcher and vertical expansion. Read pinned SolidCraft `ui_kit/assistant_entry.rs`
and `agent_chat/panel.rs` at c553365 for right-edge full-height bounds, adjacent
tool space, card clipping and resize behavior. The narrow adaptation uses the
existing shared card/controls; [provenance and replacement boundary](panel-controls-provenance.md)
cover the imported unchanged Assistant SVG and retained licenses. User designs,
reference checkout and the preserved fork are unchanged; no app/engine dependency
or provider backend was introduced.

The user's subsequent review supersedes tool-space reservation: Assistant overlays
the right rail and leaves Properties/filters in place. Re-read the actual FigCraft
images and `assistant_drawer_behavior.py` for current-sheet/selection context,
draft/suggested-request behavior and review hierarchy. The new contextual clipboard
brief, candidate summary and secondary configuration/diagnostics are original
PCBcraft work; direct in-app model conversation is still unavailable.

## Connected-grid arrangement (November 4, 2026)

Original PCBcraft operation, constraints, route filtering and synthetic fixtures,
extending the existing candidate/session and catalog routing code. No external
implementation, asset or dependency was imported. The native file schema is
unchanged; locks and region limits belong to a proposal. Existing licensed UI
controls and symbol painting provide review, with the current theme roles.
The preserved fork, KiCad implementation, user designs and reference checkout
were not used as implementation sources or modified. [Behavior and limits](mcp.md#placement-routes-and-remaining-work).

## Direct MCP implementation (November 4, 2026)

Original PCBcraft session/proposal, protocol, local bridge and review code. No
SolidCraft engine/source was imported and no dependency was added. Existing
licensed theme/controls and the original symbol painter are reused. Read the
user's `designs/ai-review.png` pixels again for proposal/preview/Apply interactions.
The design and reference checkouts remain unchanged.

Protocol behavior was checked against official MCP 2025-06-18
[lifecycle](https://modelcontextprotocol.io/specification/2025-06-18/basic/lifecycle),
[stdio transport](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports)
and [tools](https://modelcontextprotocol.io/specification/2025-06-18/server/tools)
specifications. The installed Python MCP SDK 1.27.1 was used as a real external
client, not added as a project dependency. [Implementation and evidence](mcp.md).
No KiCad implementation or preserved GPL EDA source/notes/fixtures were inspected.


## Roadmap, community and credit conventions (November 3, 2026)

Reviewed the [Tangency Labs repository index](https://github.com/orgs/tangency-labs/repositories?type=all)
and representative approved project documentation:

- [SolidCraft README](https://github.com/tangency-labs/solidcraft/blob/main/README.md),
  [roadmap](https://github.com/tangency-labs/solidcraft/blob/main/ROADMAP.md),
  [contributing](https://github.com/tangency-labs/solidcraft/blob/main/CONTRIBUTING.md)
  and [security](https://github.com/tangency-labs/solidcraft/blob/main/SECURITY.md):
  product overview, a short roadmap with detailed engineering links, setup/review
  guidance and a separate sensitive-report route.
- [FieldCraft README](https://github.com/tangency-labs/fieldcraft/blob/develop/README.md),
  [roadmap](https://github.com/tangency-labs/fieldcraft/blob/develop/ROADMAP.md),
  [contributing](https://github.com/tangency-labs/fieldcraft/blob/develop/CONTRIBUTING.md)
  and [conduct policy](https://github.com/tangency-labs/fieldcraft/blob/develop/CODE_OF_CONDUCT.md):
  evidence-backed feature status, workflow milestones, scoped contributions and
  concise conduct/reporting expectations. Its claim/triage automation is not installed here.
- [SolidCore README](https://github.com/tangency-labs/solidcore/blob/main/README.md):
  explicit scope, experimental status, validation evidence and bounded work claims.

These informed structure only. The new roadmap, contribution/conduct/security/support
docs and issue/PR templates are original PCBcraft text. Upstream release promises,
milestone counts, source rules, bots and reporting contacts were not imported.
No EDA implementation, old-fork source/fixtures or KiCad implementation was inspected.

A read-only [repository metadata check](https://api.github.com/repos/travle1/pcbcraft-next)
reports `main`, public visibility, Issues enabled and Discussions disabled. The user
selected their GitHub account for contact; docs use the verified owner `travle1`
and distinguish public contact requests from confidential reports. Private vulnerability
reporting availability was not established; no repository settings were changed.

For [commit credit](../ATTRIBUTION.md#commit-credit), reviewed official
[GitHub trailer/account guidance](https://docs.github.com/en/pull-requests/how-tos/commit-changes/creating-a-commit-with-multiple-authors)
and [RFC 2606](https://www.rfc-editor.org/rfc/rfc2606.html#section-2).
The user requested tool/model attribution; the project-local `.invalid` identifiers
explicitly do not represent provider/GitHub accounts. Only actual participants are
credited, with generic tool names when model identity is unknown. No history was rewritten.

## README and media refresh (November 3, 2026)

Read SolidCraft's README at the existing `c553365` pin for product/media structure;
no upstream screenshot, animation or new implementation was imported. The README
now uses real PCBcraft captures at `5bb0a4d`, with a labeled still-capture GIF tour.
[Media provenance and reproduction](screenshots/README.md) retain existing asset
notices. Source inspection of PCBcraft's manifest/session/desktop confirmed that
the editing core exists but MCP, its live-session bridge and Assistant connection
are still unimplemented. Documentation distinguishes those contracts from working
features and corrects the AI spec's obsolete description of manual wire geometry.
No application code, dependencies, user designs or reference checkout were changed.

## Sheet implementation (November 3, 2026)

The sheet aggregate, hierarchy/port endpoints, occurrence-qualified connectivity,
cross-probe seam and tests are original PCBcraft work. The user's actual
`designs/exports/workspaces-2026-10-30/01-schematic-overview.png` pixels and
`designs/sheet_navigation.py` were reviewed read-only. SolidCraft's existing
`ui_kit/tree.rs` at the pinned c553365 reference informed stable selection/disclosure
behavior. This work extends the existing licensed shared controls and theme;
no new source/assets/dependencies were imported. The designs, reference checkout
and preserved fork are unchanged. No KiCad implementation or inherited fork data
was consulted. See [implemented sheet behavior](sheet-workflows.md).


## Browser hierarchy correction (November 3, 2026)

Read actual pixels of the user's final `01-schematic-overview.png`, plus their
read-only `designs/clean_browser.py` and `designs/sheet_navigation.py` interaction
recipes: project/sheets tree and sheet-local Symbols/Nets links to the Search drawer.
These original design recipes were inspected, not executed. Designs remain unchanged.
SolidCraft `ui_kit/tree.rs` and relevant `browser.rs` declarations at c553365 supplied
stable-row/disclosure/selection behavior. The small navigation/result-row code and
sheet SVG are original PCBcraft composition using existing licensed controls/tokens;
no mechanical Browser model or upstream engine was imported. This corrects the earlier
implementation-derived tab/list guidance. See [browser review](ui-audit.md#browser-hierarchy-correction).

## UI consistency audit (November 3, 2026)

Actual SolidCraft menu/container/actions/checks and icon sources at c553365 were
inspected and narrowly adapted with notices. The user's schematic export was
visually reviewed again. See [audit](ui-audit.md) and
[control provenance](panel-controls-provenance.md) for exact scope and limitations.
Delete/measure SVG masters remain byte-for-byte copies with registered hashes.
No new dependency or reference checkout change.

## AI authoring investigation (November 3, 2026)

Read-only investigation at SolidCraft `c553365a2197a689e749389a6fbf707a38df9b61`:
`agent_chat/{mod,acp,context,backend,api_client,gate}.rs`, `ui_kit/assistant.rs`,
MCP `server.rs`, `tools.rs`, `skill_reference.rs`, selected test definitions,
`skills/solidcraft-modeling/SKILL.md`, and agent-chat docs. The
[AI authoring source map](specs/ai-authoring.md#what-solidcraft-actually-does)
distinguishes system notes, MCP initialization guidance, bundled skills, context
helpers and enforced tool/session behavior. `prompts/list` is empty; external
ACP capabilities affect prompt injection, and direct API chat supplies its own
system note. No source was copied or upstream code/tests executed for this review.

Inspected actual pixels of the user's `designs/ai-review.png` and final schematic
overview export. The former supplies proposal review interactions, the latter
the Assistant entry; current SolidCraft dark tokens still govern palette. Designs
remain unchanged. Official MCP 2026-07-28 tools/resources documentation informs
the distinction between context/tools and an agent connection; no protocol version
or provider dependency has been selected. This task changes guidance only, not the app.

## Net workflow follow-up (November 3, 2026)

The explicit-link compiler, naming/disconnect behavior, report schema, fixtures and
Simulate glyph are original PCBcraft work. The public
[KiCad schematic manual](https://docs.kicad.org/master/en/eeschema/eeschema.html)
and [PCB manual](https://docs.kicad.org/8.0/en/pcbnew/pcbnew.html) informed the
distinction between connectivity interchange, simulation models and direct
schematic/board updates in [net workflows](net-workflows.md). No implementation
source or supplied data/example was imported; external import is not implemented.
Cached egui 0.36.2 public API/source verified scroll style and input-event fields.
No dependency or reference checkout was changed.

## Product boundary takes precedence

The user clarified that PCBcraft is a standalone app. The
[accepted decision](adr/0001-standalone-application.md) supersedes earlier
recommendations to extend SolidCraft's application or mechanical Session.
Upstream evidence below establishes source behavior and reuse opportunities,
not a required host dependency. Independently consumable libraries need their
own package/API/dependency/license qualification.

## Upstream reference baseline

On November 3, 2026, PRs #11351 and #11360 were verified merged. The separate
local checkout pins main at `c553365a2197a689e749389a6fbf707a38df9b61`.
[Baseline evidence and API map](host-baseline.md) record actual local Cargo
resolution/build/startup status and the current SolidCore/record pins. Source:
[pinned workspace](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/Cargo.toml),
[desktop](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/apps/solidcraft/Cargo.toml),
[wrapper](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/smlib/Cargo.toml).
`solidcore-record` is now an actual consumed crate; the wrapper no longer declares
a native/C++ feature or build script. These changes supersede the old PR snapshot
below. No upstream source was copied during that original qualification and no host
source was modified. The later resource imports are recorded below.

## Historical migrated-PR evidence

[PR #11360 assessment](migrated-host-assessment.md) records head
`7ad1d3e12540fee1ded8391283567ab7fa24f588`, its open/unmerged dependencies, default
SolidCore/optional native features and CI limits. All 22 non-vendored manifests
and the full lockfile at that head were inspected: no Truck declaration/package.
No local Cargo graph/build/launch was run at that earlier head. Direct pinned SolidCore geometry use
is distinct from reuse of SolidCraft application services. The old main findings
below remain historical evidence. The later user clarification decides standalone
deployment and PCBcraft session ownership; app/Session reuse is superseded. Upstream
library requests are [planned only](upstream-reuse-requests.md), not sent.

## Framework evidence (November 2, 2026)

SolidCraft source was inspected at `28b7ae007d34874e01cc66eb8dbbd07a3f186a01`:
[workspace manifest](https://github.com/tangency-labs/solidcraft/blob/28b7ae007d34874e01cc66eb8dbbd07a3f186a01/Cargo.toml),
[UI manifest](https://github.com/tangency-labs/solidcraft/blob/28b7ae007d34874e01cc66eb8dbbd07a3f186a01/crates/ui-egui/Cargo.toml),
[engine manifest](https://github.com/tangency-labs/solidcraft/blob/28b7ae007d34874e01cc66eb8dbbd07a3f186a01/crates/engine/Cargo.toml),
[kernel manifest](https://github.com/tangency-labs/solidcraft/blob/28b7ae007d34874e01cc66eb8dbbd07a3f186a01/crates/kernel/Cargo.toml).
These prove egui 0.36 and the unconditional UI→engine→kernel→Truck dependency.
Do not mistake geometry not being called for geometry not being linked. A Truck
entry elsewhere in a workspace lockfile alone does not prove selected-build linkage.
That was the earlier main-branch assessment. The newer reference shows the
Truck dependency changed; it does not make the UI crate independent of its engine
or determine PCBcraft session ownership.

Persistence/lifecycle evidence: [design IO](https://github.com/tangency-labs/solidcraft/blob/28b7ae007d34874e01cc66eb8dbbd07a3f186a01/crates/io/src/design.rs),
[drawing IO](https://github.com/tangency-labs/solidcraft/blob/28b7ae007d34874e01cc66eb8dbbd07a3f186a01/crates/io/src/design/drawing.rs),
[Document](https://github.com/tangency-labs/solidcraft/blob/28b7ae007d34874e01cc66eb8dbbd07a3f186a01/crates/doc/src/document.rs),
[metadata](https://github.com/tangency-labs/solidcraft/blob/28b7ae007d34874e01cc66eb8dbbd07a3f186a01/crates/doc/src/metadata.rs),
[format](https://github.com/tangency-labs/solidcraft/blob/28b7ae007d34874e01cc66eb8dbbd07a3f186a01/crates/doc/src/format.rs),
[Session](https://github.com/tangency-labs/solidcraft/blob/28b7ae007d34874e01cc66eb8dbbd07a3f186a01/crates/engine/src/lib.rs),
[commands](https://github.com/tangency-labs/solidcraft/blob/28b7ae007d34874e01cc66eb8dbbd07a3f186a01/crates/engine/src/cmd/mod.rs).
Typed mechanical Document/Session and informational metadata do not supply a
generic electronics/plugin API. Snapshot undo does not promise saved undo history.

UI reuse evidence: [workspace tables](https://github.com/tangency-labs/solidcraft/blob/28b7ae007d34874e01cc66eb8dbbd07a3f186a01/crates/ui-egui/src/workspace.rs)
and [UI kit](https://github.com/tangency-labs/solidcraft/blob/28b7ae007d34874e01cc66eb8dbbd07a3f186a01/crates/ui-egui/src/ui_kit/mod.rs).
Many controls are crate-private. The user's six final exports were inspected;
their [exact local paths](specs/workspace-ui.md#visual-authority-and-reusable-foundation)
are visual authority. The separate `/home/travisl/projects/solidcraft-icons`
catalog/review workspace was read only; its provenance lock pins host artwork at
`60488a0837affcee79832bdb7bf6e860e84b10f2`. At that review, no consumer dependency/assets had been imported; see the later resource record below.

SolidCore [layers](https://github.com/tangency-labs/solidcore/blob/09de60b3e81bfda525cd6d0368a227760770d77e/docs/layers.md)
and [workspace](https://github.com/tangency-labs/solidcore/blob/09de60b3e81bfda525cd6d0368a227760770d77e/Cargo.toml)
separate architecture direction from actual available crates. Planned shared
record/ops/tess/view/kit/link names must not become invented dependencies.

Manufacturing: [Ucamco's official Gerber/XNC downloads](https://www.ucamco.com/en/gerber/downloads)
are the primary source; a named revision/profile and original fixtures must be
pinned per exporter before implementation. No output conformance was tested.
File-suffix evidence: [pkg-config guide](https://people.freedesktop.org/~dbn/pkg-config-guide.html)
and [Oracle Pro*C manual](https://docs.oracle.com/cd/B13789_01/win.101/a96111/use.htm)
confirm `.pc` collisions; this is not an exhaustive association survey. The guide
was accessible via search excerpt; direct fetch returned 403. Extension remains
provisional, not unique or a semantic interoperability guarantee.

## Original starter sources and policy

This project contains original guidance, a schematic model/session and egui app. No code,
fixtures or history from the older GPL PCBcraft fork have been imported. Do not
use its implementation or source-derived notes as references for new work.
This register is not a formal clean-room or upstream-eligibility certification.
This conversation previously reviewed the fork, so future implementation tasks
should start fresh with approved sources and the target repository's rules.

| Public reference | Use and limit |
| --- | --- |
| [SolidCraft AGENTS](https://github.com/tangency-labs/solidcraft/blob/4a680d17bae766b41d3a91ea7cb166ef08586846/AGENTS.md) and [CONTRIBUTING](https://github.com/tangency-labs/solidcraft/blob/4a680d17bae766b41d3a91ea7cb166ef08586846/CONTRIBUTING.md) | Host source policy/developer flow; recheck before contributing |
| [SolidCore layers](https://github.com/tangency-labs/solidcore/blob/967005ac2561ae0b6928c47d8584c5bf0e7d2a9c/docs/layers.md) | Shared ownership direction; planned layer names are not proof of shipped APIs |
| [SolidCore exports](https://github.com/tangency-labs/solidcore/blob/967005ac2561ae0b6928c47d8584c5bf0e7d2a9c/crates/solidcore/src/lib.rs) | Verify actual consumer interfaces/limitations before choosing a revision |
| [SolidCraft PR #8206](https://github.com/tangency-labs/solidcraft/pull/8206) | Parts-library reuse; last checked open/unmerged at head `401f89c3b011d678473e5eb379f34cfc85b140e3`; recheck availability |
| [KiCad format specifications](https://dev-docs.kicad.org/en/file-formats/) and [manuals](https://docs.kicad.org/) | Independent compatibility behavior; do not follow implementation-source links |

At the initial guidance-only stage, no dependencies or external fixtures were
bundled. Later qualified imports are recorded above and in the asset register.
When adding one, record exact source/revision/license/use and preserve notices. The older fork's historical
attribution is not copied into this project. Dependencies' public licenses do
not supersede their contribution policies.

`LICENSE-MIT` uses the standard grant with this project's copyright line.
`LICENSE-APACHE` is the standard Apache 2.0 terms, taken from the local craft-libs
license text at revision `1aa81a773aa9198a0e4c0daeda954d622475af38`. No craft-libs
implementation or assets were imported.

## First schematic implementation sources (November 3, 2026)

Original Rust document/session/persistence, symbol drawings, fixtures and UI code
were authored in this project. No preserved GPL fork, derived notes/fixtures or
KiCad implementation was consulted for this slice. This describes the work in
this slice, not a historical clean-room certification.

The user's own schematic-overview and schematic-properties exports in
`../../designs/exports/workspaces-2026-10-30/` were visually inspected. SolidCraft
UI-kit/theme/font source at `c553365a2197a689e749389a6fbf707a38df9b61` and the local
`solidcraft-icons` catalog informed independent family-resource use. Imported
SVGs/font, exact paths/hashes, authors and retained notices are in the
[resource register](../assets/family/README.md). UI/app/engine source was not copied.

Pinned egui/eframe/egui_extras 0.36.2 cached upstream source supplied actual widget,
App::ui, panel, image loader and screenshot API behavior. Cargo.lock records all
resolved dependencies. The selected normal/build graph is saved locally at
`../work/proof/dependencies.txt`: no SolidCraft app/engine, SolidCore or Truck
package is selected by this schematic-only implementation. Geometry comes later.

## Visual correction after user review

The user supplied a SolidCraft dark screenshot and rejected the first approximate
washed-out palette. `assets/styles/ui/palettes.rs`, `theme.rs` and the icon-role
TOML at the pinned c553365 reference supplied verified color/metric data and
semantic-role behavior. PCBcraft's theme publication, cache adapter and composition
are original; no host theme engine was imported. Additional shared SVG/palette
resources retain exact provenance and notices. See [theme foundation](theme-foundation.md).
Original EDA chip/symbol/wire glyphs were authored here under the root license.

## Expanded workspace picker

After the user supplied the expanded-selector screenshot, a narrow licensed
adaptation of SolidCraft's `ui_kit/menu.rs::workspace_menu` was added as
`src/desktop/workspace.rs`. `toolbar.rs` supplied the whole-card opener behavior;
`fonts.rs` confirmed real IBM Plex variable weight selection. Exact pin, license,
scope, changes and replacement plan are in
[workspace-picker-provenance.md](workspace-picker-provenance.md). This supersedes
the earlier no-UI-source-copy statement for this component only. The render
workspace SVG was added to the resource register; the PCB-layout glyph is original.
No mechanical dispatcher, host Session or whole shell was imported.

The later Capture/canvas correction follows the user's own close-ups and the
unchanged `01-schematic-overview.png` export, viewed as pixels. Four more unmodified
family icons (mirror, list, configuration table, select) use the same c553365 pin
and retained notices. Their hashes are in the resource register. Original EDA
glyphs fill missing concepts; no external EDA implementation or artwork was used.
The ribbon action table, caption menus, line grid and camera/presentation changes
are original PCBcraft composition, with no new package dependencies.

The subsequent parts-browser change uses the user's compact catalog screenshot as
layout reference and retains the existing SolidCraft theme/resources. Pinned
SolidCraft `dialogs_parts.rs` was inspected: its thumbnail path creates a mechanical
Session, so it was not imported. PCBcraft's new panel reuses its original canvas
symbol painter and the original built-in catalog. Cached egui 0.36.2 Response /
DragAndDrop APIs supplied the drag transport; the Electronics session owns each
placement. No new external assets, dependencies or library data were imported.


## Panel, catalog and search control pass

The user's actual FigCraft archive and six workspace exports remain unchanged.
The latest panel work inspected its saved transition/variable states, plus the
SolidCraft source behind the user's Preferences, Assistant and command-search
screenshots. [Panel/control provenance](panel-controls-provenance.md) records the
exact c553365 source modules, narrow adaptations, licenses and replacement plan.
`controls.rs` adapts the bounded host, field alignment and catalog-footer patterns;
PCBcraft owns the electronics command list, state and workspace composition.
This adds another explicitly attributed UI-source adaptation, not a whole-app
copy or a new dependency. No KiCad implementation or preserved GPL fork was read.

## Page, placement and navigation follow-up

Reviewed the user's `designs/schematic-proximity-guide.png` and
`designs/schematic-sheet-boundary.png` pixels, and their `proximity_guide.py` /
`sheet_boundary_view.py` source recipes read-only. The quiet dashed guide, optional
proximity mode and independent drawing page inform original PCBcraft implementation.
No user design was edited or executed.

Read SolidCraft's [navigation.rs at c553365](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/ui-egui/src/navigation.rs)
for immediate raw wheel input/units and viewport navigation references. PCBcraft's
2D pointer-anchor math, bounded page model, placement drafts and text-box editing
are original code using existing egui APIs; no SolidCraft application dependency
or new crate was added. Grid uses the already registered/licensed `grid.svg` master;
text and subsheet glyphs are original. Existing font/control notices still apply.
The preserved fork and KiCad implementation were not consulted.

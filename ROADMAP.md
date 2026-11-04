# PCBcraft roadmap

PCBcraft is a standalone electronics-design application: manual editing and AI
agents working on the same project, from schematic intent to a checked board and
manufacturing package. It follows the SolidCraft family's visual language and
reuses qualified libraries without depending on the SolidCraft application.

This is the short product roadmap, reviewed **November 4, 2026**. It describes this
development checkout, including work not yet merged or published. Implemented does
not mean released or manufacturing-qualified. Priorities can change; these are
acceptance milestones, not delivery dates or completion percentages.

[Detailed status](docs/implementation-status.md) ·
[Engineering tasks](docs/implementation-plan.md) ·
[Architecture](docs/framework.md) · [Contributing](CONTRIBUTING.md)

## Working foundation

The current schematic editor has placement, selection, group edits, properties,
authored wires and branches, editable text/links, net inspection and a JSON
connectivity report. Flat/hierarchical sheets, reused sheet definitions and explicit
ports share one project-wide undo history and experimental `.pc` save/open.
The desktop has a grouped ribbon, command search and movable Parts/Properties panels.
Twelve original generic symbols work offline; the fan-controller example is an
editing illustration, not an electrically validated design.

Live MCP now supports inspection, atomic proposals, a review panel and shared Undo.
This is a foundation, not completed schematic capture. An initial ACP v1 conversation
adapter now has scripted-peer qualification; real agent/provider qualification, ERC/DRC,
board editing, 3D, simulation, KiCad interchange and manufacturing remain ahead. Cross-probe identity hooks exist; layout highlighting
does not. See [current limits](docs/implementation-status.md#deliberate-limits).

## Next: AI-assisted capture and dependable schematic editing

| Priority | Outcome | Completion evidence |
| --- | --- | --- |
| Atomic proposals (**AI0**, implemented bounded profile) | Prepare and preview a coherent set of component/connection edits, then apply it once | Stale/invalid proposals leave zero changes; retried requests do not duplicate edits; one Undo restores the whole project |
| Live MCP and Assistant (**AI1 implemented; AI2/AI3 partial**) | An external agent and the in-app review panel inspect and edit the live Electronics session | One qualified client completes inspect → propose → review → apply → save/reopen → undo; GUI and agent see the same revision; unsupported checks remain explicit |
| Readable arrangement (**SA0–SA1**) | Arrange a selected circuit into aligned functional groups with orthogonal routes and clear labels | Original 8–20-part fixtures preserve terminal membership, locks and outside geometry; impossible constraints return findings instead of silently changing the circuit |
| Capture reliability and polish | Better manual wiring, selection, hierarchy/text interaction, keyboard access and persistent workspace settings; autosave/recovery | Reproduction tests for edits and recovery; reviewed full/compact/scaled UI; no loss of connections, IDs or user work |

Manual paths, ordered grid placement and bounded orthogonal connection routing
are implemented. A connected-grid profile adds proposal-local locks/region limits
and replans internal direct connections. Exact text clearance, saved locks,
functional groups and branched-network arrangement remain ahead of full SA1 acceptance.
See [MCP implementation and limits](docs/mcp.md). The [AI authoring contract](docs/specs/ai-authoring.md) owns
those task definitions. AI work starts here, not after the board toolset is complete.
Readable manual design must remain useful without an account or agent.

The [Assistant conversation slice](docs/assistant.md) adds Send, streaming, Stop,
bounded history and snapshot-bound review-only tools. A named real agent/version
must still complete the full acceptance workflow before AI3 is considered qualified.

## Following milestones

These are dependency-related workstreams, not a requirement to finish every item
in one row before investigating the next. Library/identity contracts and check
coverage precede features that depend on them. The library-management draft now
sets out a useful starter, portable exact used definitions and configurable
local/team/public workflows; it remains a proposal for review. Narrow geometry
and interchange qualification can proceed independently when their interfaces are settled.

| Area | Planned capability | First useful acceptance gate |
| --- | --- | --- |
| [Libraries and component selection](docs/specs/library-management.md) | Durable component/symbol/footprint identities, explicit terminal-to-pad mappings, pinned project snapshots, offline assets, local/team/public sources and reviewable upgrades | Reopen a project with its used definitions offline; preview a library update, diagnose pin/pad incompatibility and undo it; retain provenance and local overrides |
| Electrical checking | Pin electrical types, power/drive intent, intentional NC, hierarchical connectivity, then buses and multiunit breadth | Original fixtures explain known ERC violations and unknown coverage; derived results are revision-bound; structural validation is never presented as a passed electrical check |
| First board workflow | Stackup/layers/outline, footprint placement, tracks/vias, ratsnest, basic rules/DRC and reviewed schematic-to-board changes | A small original two-layer board can be edited, checked, saved/reopened and reconciled without losing associations; schematic/layout selections cross-probe through stable occurrence identities |
| 3D and mechanical review | SolidCore-backed board/layer geometry, component models and aligned selection across schematic/layout/3D | Asymmetric top/bottom fixtures verify origins, rotation, mirroring and layer heights; absent optional models are reported; geometry never owns electrical identity |
| KiCad interoperability | A declared import/export/round-trip profile based on public formats and original black-box fixtures | Named KiCad versions and object coverage; no-op and single-edit round trips; unsupported or lossy cases reported/refused explicitly |
| Manufacturing | Gerber/drill, BOM, placement and assembly outputs with variant/DNP policies | One immutable project revision, library snapshot and output profile produce a manifest and coherent output set; required checks gate publication; independent output inspection |

The board workflow must accommodate PCB-only design as well as schematic-driven
work. The preferred UI keeps one project window, with optional schematic/layout
split views and shared selection/history. Reproducible library management and
explainable ERC/DRC are core requirements, not later cosmetic features.

## Broader scope

Richer rules, zones, multiunit/bus coverage, assembly variants, differential pairs,
length tuning, advanced stackups/vias, further interchange profiles and ECAD/MCAD
exchange follow bounded foundations. Their presence in a spec is not a feature claim.

**Simulate** is already shown as Planned. Simulation models, solver integration,
results and cross-probing need their own qualified slice. Advanced SI/PI, general
autorouting, real-time collaboration and enterprise lifecycle systems are deferred;
none requires a cloud account for ordinary local design. Possible `.sc` export is
a future semantic adapter; native `.pc` remains provisional.

## Quality and release readiness

Across every milestone: shared manual/agent operations, atomic failure, explicit
units/identity, stale-job handling, original fixtures, rendered UI review and
license provenance. Theme support should extend semantic tokens rather than add
widget-specific colors. Accessibility, high-DPI behavior, performance budgets and
multi-platform packaging need measured qualification.

Before a packaged release, document supported platforms and format versions,
recovery/migration behavior, checks actually run and remaining limitations.
There is no release cadence, hosted CI gate or automatic agent queue configured by
this roadmap. The [development flow](docs/development-flow.md) explains bounded
work and review; [issues](https://github.com/travle1/pcbcraft-next/issues) track
reported work. Update this page and implementation status when capabilities land.


The initial [browser target](web/README.md) shares schematic editing and `.pc`
codecs, with upload/download and a manual GitHub Pages workflow. Browser storage,
Assistant transport and broader browser/mobile qualification remain open. This
adds a distribution target without changing Electronics session ownership.

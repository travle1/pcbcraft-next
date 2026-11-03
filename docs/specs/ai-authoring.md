# AI-assisted authoring and readable schematic arrangement

Status: first-class product requirement, November 3, 2026. This is an implementation
contract and next-slice plan, not an available Assistant, MCP server or arranger.
The current app has none of those three. It has a headless Electronics session,
single-command validation/undo, flat/hierarchical sheets and occurrence-qualified
connectivity reports to build on. The [README](../../README.md#ai-and-mcp) exposes
this implementation gap prominently; a headless command core is not an MCP server.
Read [runtime contracts](host-runtime.md), [domain](domain.md),
[UI](workspace-ui.md) and [implementation status](../implementation-status.md).

## Product outcome

The Assistant can inspect a selected circuit, explain it, propose components and
terminal connections, and produce a readable, editable schematic. It is present
across Schematic/Layout/Render/Simulate as those workspaces become available, not
a separate document or a chat-only add-on. External MCP clients and in-app chat
reach the same semantic operations. Manual design and automatic arrangement work
without an AI account. No model, solver or remote service starts merely on launch.

There are two separate reviewable operations:

- **Design/connect:** change electrical intent using resolved part definitions,
  terminal IDs, supplied requirements and explicit assumptions. Show every added,
  removed or merged connection; missing pin/voltage/model data stays unresolved.
- **Arrange schematic:** change presentation while preserving exact component IDs,
  terminal membership, values and electrical intent. It must not silently invent
  connections, swap pins, add components or change the PCB to improve appearance.

The model proposes function, parts, net membership and constraints. PCBcraft's
deterministic arrangement/router computes positions and paths. A prompt saying
"make it neat" is guidance, not a layout algorithm or an electrical check.
The present demo uses curated coordinates, bounded orthogonal display routing and
supply aliases. [Manual authored wiring](../wire-editing.md) now supports free
endpoints, branches, bends, dragging and deletion with undo. This is not the
planned deterministic arranger or an AI connection.

## What SolidCraft actually does

Inspected reference checkout `c553365a2197a689e749389a6fbf707a38df9b61`, read-only.
These are actual upstream mechanisms, not independently consumable SDK promises:

| Mechanism | Source and observed behavior |
| --- | --- |
| In-app agent connection | [agent_chat/acp.rs](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/ui-egui/src/agent_chat/acp.rs): `session_new_params` supplies the live-window MCP bridge and `SYSTEM_NOTE`; standard system-prompt support is negotiated, with adapter-specific metadata fallbacks. Agents that accept neither may not receive the note. |
| Session guidance | The same `SYSTEM_NOTE` tells the assistant to use design tools, read the modeling skill, plan numerical expectations, batch edits and verify results. The direct API backend also injects this note. |
| MCP initialization guidance | [mcp/server.rs](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/mcp/src/server.rs): `INSTRUCTIONS` adds plan/build/verify guidance and discovery hints to initialization. `prompts/list` returns an empty list; this is not a catalog of MCP prompt templates. |
| Versioned skill documents | [skill_reference.rs](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/mcp/src/skill_reference.rs) embeds modeling, editing, checking and mechanisms guidance. The tool serves named bundled files, including workflow, examples and release-specific limitations. |
| Context helpers | [agent_chat/context.rs](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/ui-egui/src/agent_chat/context.rs): selection descriptions include tool-usable identities and geometry; `with_context` appends them to the message, with omitted counts. Attachments/view images are separate prompt parts. |
| Provider seam | [backend.rs](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/ui-egui/src/agent_chat/backend.rs) separates chat from provider. [api_client.rs](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/ui-egui/src/agent_chat/api_client.rs) maps discovered MCP tools into provider function tools and routes calls back through MCP. Its startup currently consumes the tool catalog but does not forward the MCP initialization instructions; it supplies the session note separately. |
| Enforcement and verification | [gate.rs](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/ui-egui/src/agent_chat/gate.rs) and chat dispatch enforce the selected permission mode. [mcp/tools.rs](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/mcp/src/tools.rs) exposes schemas, batches, inspection and numerical expectation checks. Prompt text alone is not enforcement. |

SolidCraft tests also check that instruction-referenced commands exist and exercise
a modeling recipe. Those tests were read, not run in this task. No general EDA
schematic arrangement capability is established by this CAD integration. The chat
context currently imports its mechanical Session; use the architecture and seek
narrow library extraction, not a dependency on its whole UI/engine. No upstream
implementation was copied for this specification.

## PCBcraft agent guidance and connection boundary

Use the same layered approach with EDA-specific content:

1. A short, versioned session note: active project/instance, scope, supported tools,
   inspect before changes, explicit pin IDs/units, plan/preview/apply, preserve locks,
   report unsupported checks. Keep tool names generated/checked against the catalog.
2. Bundled reference guides for capture, circuit editing and verification. Include
   original examples, current capabilities and failure recovery. Guides are not
   electrical-library truth or permission grants. A release change updates both
   capability schemas and references; tests reject stale/nonexistent tool names.
3. Structured context: selected occurrence-qualified parts/nets/pins, resolved
   definition revisions, current connectivity, relevant electrical constraints,
   intended functions, locked positions, sheet bounds, check coverage and snapshot
   stamp. Paginate/query large projects; disclose truncation. An optional rendered
   view helps visual review but is not the source of electrical identity.
4. A backend seam for an external MCP client or an in-app agent/provider adapter.
   MCP supplies tools/context, not a model or conversation protocol. Qualify one
   initial adapter and protocol revision before claiming any provider works.

Proposed tool responsibilities (names provisional, none currently exposed):

| Responsibility | Contract |
| --- | --- |
| Inspect/capabilities/reference | Bounded project/selection/net queries, schemas, supported profiles and bundled guidance; read-only |
| Resolve components | Pin catalog revision, part/symbol/terminal data and optional datasheet evidence; unavailable facts stay unknown |
| Prepare circuit edit | Explicit create/connect/disconnect/value operations, assumptions and expected net memberships; plan-local IDs; no mutation |
| Arrange selected circuit | Scope, locked refs, layout profile, functional groups and connection-display constraints; presentation-only proposal |
| Validate/render preview | Structural, connectivity and readability findings, separately from named ERC coverage; revision-stamped image/diff |
| Apply/discard/status | Server-owned plan handle, request ID and expected snapshot; exactly one commit/undo on apply, no mutation on discard |

Follow runtime incarnation/revision, bounded idempotency and candidate validation
contracts. Retrying a lost apply response cannot duplicate parts. A proposed
multi-command batch must be atomic; looping today's `Session::apply` would create
partial designs and separate undo entries, and is not the required implementation.
Concurrent user edits/undo/reopen invalidate a preview. Recompute for a new base;
do not silently overwrite intervening edits. Stop cancels preparation; an already
committed result remains identified and undoable.

The live connection must address the app's actual Electronics session through a
bounded local bridge, not edit `.pc` on disk or create an invisible second session.
An explicitly chosen headless session is separate and advertised as such. The first
transport should be local, with instance-scoped access; no public listener or
silent client-configuration edits. Model connectivity/credentials remain outside
the electronics document. Document text and imported metadata are design data,
not instructions to execute tools or broaden context access.

## Making the schematic look intentional

Treat nets as connections among terminal sets, not all-pairs lines or a force-based
graph that spreads parts randomly. Functional groups are proposed metadata that
the user can adjust; a high-fanout ground net must not collapse the whole circuit
into one group. Component bounds include measured symbol, pin and label geometry.

The first profile is a compact orthogonal matrix with variable-size cells:

- Align functional blocks in rows/columns, with inputs, processing and outputs
  following a chosen reading direction. Reserve routing channels between them.
  Feedback and bidirectional interfaces are explicit exceptions, not broken DAGs.
- Place dependent passives beside their associated circuit, with a common spacing
  rhythm. Use a power/ground orientation convention where permitted by the symbol.
  Drawing proximity is not evidence of adequate physical PCB decoupling/layout.
- Preserve locked positions/orientations, authored paths and groups. Optimize only
  the selected region by default, using fixed boundary anchors. Prefer small local
  changes; arranging one resistor must not shuffle the rest of the sheet.
- Route orthogonal paths around symbol/pin-text/value/reference obstacles. Prefer
  short paths with few bends/crossings. A shared trunk belongs to one net with
  explicit branches/junctions; unrelated nets cannot share a segment. A crossing
  never becomes a connection merely because the arranger put lines together.
- Default to visible wiring within and between nearby blocks, matching the user's
  interconnected presentation. Use explicit reviewed power/ground/long-distance
  label policies where continuous lines would obscure the circuit. Do not improve
  a score by replacing every wire with an isolated alias or shrinking everything.
- Keep identity and terminal geometry intact. Do not reorder symbol pins, swap
  gates or mirror semantic pins unless a separately supported operation permits it.

Pipeline: build a constraint graph from the pinned snapshot; assign functional
regions and ranks (handle feedback cycles explicitly); order rows/columns using
actual pin sides; compact with bounds/locks; route net-owned branches in reserved
channels; place labels; check topology and readability; produce an editable preview.
Start with deterministic bounded heuristics and stable tie breaks. No claim of
globally optimal layout. If constraints conflict or routing exceeds its budget,
return affected objects and an incomplete proposal; do not apply broken routes.

Hard gates and measured objectives must be separate:

| Hard gate | Measured preference |
| --- | --- |
| Same terminal-set connectivity for arrange-only | Fewer crossings and bends, shorter wire length |
| Locked/out-of-scope items unchanged | Less displacement from the user's current drawing |
| No symbol/text overlaps or wires through unrelated pins/bodies in the supported profile | Consistent rows, columns, spacing and signal direction |
| Correct endpoints/junctions and explicit unsupported cases | Compactness within a readable scale, fewer remote aliases |
| Grid/unit/bounds and library definitions preserved | Balanced block density and labels near their objects |

The hard gates win over any weighted score. Compare topology as terminal partitions,
not mutable net names or revision-local anchors. Check two *different* concerns:
electrical coverage (known drivers, voltage assumptions, missing/NC pins, etc.)
and presentation quality. Until ERC exists, report **structural connectivity
checked; electrical checks not implemented**, never a green "circuit valid" badge.

Authored bends, free endpoints and explicit junctions are now persisted and editable;
manual wire segments/corners/branches can be moved or deleted with undo. The current
project envelope carries these paths across sheet occurrences. Legacy links may
still use derived display routes. Locks, functional groups, layout profiles and
prepared arrangement constraints still need bounded schema/version contracts.
Build the arranger on the authored geometry instead of a parallel display-only
model. This is 2D schematic geometry, independent of PCB copper routing or a
mechanical CAD kernel. See [wire editing](../wire-editing.md) and
[current project persistence](../sheet-workflows.md#persistence-limits-and-acceptance).

## Assistant and preview UI

Use the user's [final schematic overview](../../../designs/exports/workspaces-2026-10-30/01-schematic-overview.png)
for the bottom-right Assistant entry and [AI review design](../../../designs/ai-review.png)
for context chips, proposed-change summary, visual preview and Apply/Discard.
The latter is an older light rendering: reuse its interaction hierarchy, with the
current SolidCraft dark tokens. Both actual images were inspected, unchanged.

Provide a movable/dockable Assistant panel with connection/model state, explicit
selected-context chips, optional view attachment, prompt composer, Stop and tool
activity. It must not replace Browser or reset the camera. Distinguish Inspect,
Propose and an eventual explicitly enabled scoped auto-apply mode. Default design
generation presents a proposal; the user need not approve each micro-operation.
Unavailable connection controls clearly state their status.

Preview overlays use separate proposed/removed/changed geometry and a Before/After
control. Show added parts/connections, topology changes, unresolved assumptions,
readability findings, electrical-check coverage, base revision and locked scope.
Let the user refine constraints or accept a coherent complete proposal. Partial
acceptance must re-plan/revalidate dependencies; arbitrary checkbox subsets cannot
leave dangling nets. Apply is one undoable transaction; Discard has no mutation.
Model text, a tool returning success, and a check actually passing are distinct.

Example request: "Connect the temperature sensor to the controller and arrange
that signal path left-to-right. Keep the supply block fixed." The result must show
resolved terminals, proposed connections and an aligned preview, identify any
unknown pin capability, preserve the fixed supply and undo in one step.

## Next bounded implementation sequence and acceptance

| Task | Scope / dependencies | Exit gate |
| --- | --- | --- |
| AI0 | Extend current session with prepare/apply, stable provisional IDs, request idempotency and atomic batches; domain/session owner, no transport dependency | Connect/place a small original circuit as one candidate; invalid step, stale base and repeated request cause zero duplicate/partial mutation; one undo restores all |
| AI1 | Read/prepare/apply MCP adapter plus live app-session bridge; depends on AI0 for mutations; inspect/reference reads may start earlier | External client and GUI inspect the same revision; edits appear in the live window; capability/readability/unsupported status honest; no disk-based second authority |
| SA0 (partial) | Manual paths/junctions, dragging and v4 migration are implemented; prepared layout constraints still build on AI0; changes domain/canvas/persistence and original fixtures together | Manual and prepared path edits preserve intended net membership, endpoints and labels across save/reopen/undo |
| SA1 | One arrange-selection profile for bounded original 8–20 component circuits; depends on SA0, no model/provider dependency | Aligned connected blocks, no supported overlap/endpoint violations, fixed locks/boundary, deterministic output and repeat no-op for same input/profile; unsatisfiable case rejects |
| AI2 | Assistant context/proposal/diff UI with a deterministic test adapter first; live backend after AI1 qualification | FigCraft-style review, usable keyboard/focus/compact window, Apply/Discard/Stop and stale preview; no fake connected state |
| AI3 | Bundle PCBcraft guides and qualify one real agent/provider through AI1+AI2, then consume SA1 | Inspect → propose connections → arrange → report checks → preview → apply → save/reopen → undo; no unsupported ERC/PCB/simulation promises |

This is an early capture milestone, before broad layout/manufacturing/simulation;
it is not postponed until the rest of the app is complete. AI0 is the first code
slice. No dependency addition, provider configuration, agent launch or code work
is performed merely by this document. No new crate per task is required.

Original fixtures include fan-controller-style signal paths, high-fanout power,
feedback, multiple terminal branches, long labels, a locked part, selected-region
boundary pins, impossible spacing and stale completion after undo/reopen. Use
geometry/topology assertions plus reviewed screenshots at declared scales; pixel
similarity alone cannot prove a connected circuit. Test prompts/tool references
against capabilities, and report supported check coverage independently.

Protocol reference: official [MCP tools](https://modelcontextprotocol.io/specification/2026-07-28/server/tools)
and [resources](https://modelcontextprotocol.io/specification/2026-07-28/server/resources).
The inspected SolidCraft direct backend requests protocol 2025-06-18; that is
observed upstream behavior, not a PCBcraft version choice. Qualify the selected
client/server versions and transport before adding a dependency or claiming parity.

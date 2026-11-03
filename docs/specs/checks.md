# Electrical, physical and framework validation

Status: revised after Astra packet review; coverage is a plan, not implemented support.
[Framework](../framework.md), [domain](domain.md), [jobs](host-runtime.md#jobs-concurrency-and-freshness).

## Invariants, findings and rule resolution

Reject commits with broken references, ambiguous occurrence identity, invalid
quantities/layers/mappings or corrupt schema. Permit structurally valid work with
open connections, temporary shorts/clearance violations or incomplete placement.
Interactive routing may choose blocking/shove/walk-around modes only when those
capabilities exist; export policy chooses what findings block release. Do not
make every DRC violation an impossible-to-save document.

Checks read an immutable snapshot plus resolved library/variant/rule/algorithm
inputs. Separate schematic connectivity compilation, electrical intent checking,
physical copper connectivity, geometric DRC and manufacturing policy. A useful
ERC result does not prove the board matches the schematic or that it is routed.

Illustrative result contract:

```text
CheckReport = { job_context, input_fingerprint, profile, coverage,
                completed_checks, skipped_checks_with_reasons, findings }
Finding = { stable_kind, severity, semantic_refs, location_or_witness,
            rule_id_and_revision, actual, required, explanation,
            source_provenance, waiver_state }
Waiver = { id, finding_kind, object_scope, rule_fingerprint,
           witness_fingerprint, rationale, author_label, expiry_or_review }
```

Rule evaluation uses typed units and explicit scopes: project defaults, board
fabrication profile, layer/region, class/net/pair and object-specific override.
Resolve each rule family using a documented total precedence order; tied
contradictory rules are configuration errors, never hash-map iteration wins.
Geometry minimum constraints may combine as the most restrictive where defined;
do not apply that shortcut to incompatible allowed-value or topology rules.
Expose all candidates and why the winner applied. Imported rule origins and
unsupported expressions remain visible. The precise per-family precedence table
must be reviewed before its implementation, not hidden in a general expression DSL.

Waivers are persisted/reviewable, scoped to stable IDs and a witness/rule hash.
Changed geometry, context or rule marks them for review; rename alone does not
break identity. Suppression does not erase the original result. A waiver cannot
turn an unexecuted/unsupported check into a pass. Project severity overrides are
explicit and included in the release profile. No automatic waiver generation.

## ERC and schematic connectivity

| Concern | Required model/check behavior | Phase |
| --- | --- | --- |
| Electrical pin types | Passive, input/output, bidirectional, tri-state, open collector/emitter, power in/out and explicit unspecified types; versioned compatibility table and explanations | First workflow basic passive/power; full table next |
| Drivers and power intent | Conflicting outputs, undriven required inputs/power, open-collector pull-up policy, explicit power-source assertions; unknown library types cannot be assumed safe | First workflow bounded profile; richer policy next |
| Connection geometry | Endpoints/junctions versus crossing wires, dangling wires/labels, unconnected pins and intentional NC; contact tolerance explicit and independent of zoom | First workflow |
| Hierarchy | Local/global/hierarchical labels, port directions, repeated-sheet occurrence-qualified nets, absent/mismatched ports and recursive hierarchy rejection | Hierarchy phase |
| Buses | Typed members/ranges/groups and bus-entry mapping, mismatch diagnostics; graphical bus line alone does not join every net | Hierarchy phase |
| Multiunit/stacked/hidden pins | One component across units, required-unit presence, common terminals, stacked drawing pins, hidden power connections explicit; NC and electrically absent differ | Model foundation; checking expands next |
| Labels/net identity | Conflicting names/aliases, duplicate references, case/name profile, explicit split/merge lineage; names never silently remap a board net | First workflow bounded naming; hierarchy expands |
| Net ties | Explicit allowed logical-net bridge through defined physical pad/region mapping; unrelated shorts still fail | Advanced layout phase |
| Variants | DNP does not remove pads/nets by itself; electrically different variants need explicit topology, checked separately | Basic DNP first; topology variants later |

Pin numbers are strings, including alphanumeric names. Duplicate-number pads may
represent one terminal intentionally; mechanical/NPTH pads may have no terminal.
Do not force a one-pin-one-pad bijection. Diagnose missing/ambiguous mappings,
connected NC terminals, missing footprint terminals and explicitly permitted
omissions. Manufacturer pin-function alternatives are selected configurations,
not silent renaming. Repeated sheet definitions require distinct placed component
occurrences, BOM identities and cross-probe targets. Multiple symbol units of one
occurrence produce one physical component and assembly item.

## DRC and physical connectivity

| Concern | Coverage and failure witness | Phase |
| --- | --- | --- |
| Outline/stackup | Closed valid board region/cutouts, layer order/thickness, copper roles, legal dielectric and via spans | First workflow simple rigid two-layer; advanced stacks later |
| Connectivity | Copper islands through pads/tracks/vias/fills, unconnected intended terminals, shorts versus allowed net ties, schematic-board discrepancies and PCB-only intent | First workflow basic items; fill/net ties expand |
| Copper geometry | Width, spacing, pad/track/via/zone clearance, copper-edge/cutout distance, keepouts and layer/region scope | First workflow bounded primitives |
| Holes/vias | Drill/slot size, plated versus NPTH, annular ring, hole-hole and hole-copper/edge spacing, via type/span and allowed stack; blind/buried/microvia constraints | Through-hole first; advanced types later |
| Placement | Courtyard overlap, component/board-edge/keepout rules, side/height constraints, locked items and missing/mismatched footprints | Basic placement next; 3D interference later |
| Mask/paste/silk | Mask openings/bridges/expansion, paste aperture policy, silk-pad/mask/edge clearance and readability checks; distinguish source shape from generated aperture | Basic output policy first; detailed DFM next |
| Zones | Boundary/priority/clearance/thermal/island policy, deterministic fill, stale/failed fill and disconnected thermal witnesses | Zone phase, prerequisite for any zone release |
| Routing | Segment/via legalities, return-path/length/skew/neckdown/differential-pair constraints with declared measurement policy | Basic routes first; tuning/pairs later |
| Assembly variants | DNP, alternates, fiducials/test points, mechanical pads and BOM/placement inclusion and 3D/paste behavior follow declared role-specific variant policies | Basic assembly first; richer variants later |

DRC uses exact/robust predicates within declared tolerances and a deterministic
spatial index; drawn/tessellated pixels are not a clearance oracle. Compare edge
cases just below/equal/above thresholds, holes and mirrored/rotated shapes.
Connectivity must define whether boundary contact counts, plated span joining
layers and fill ownership. Advanced unsupported pad primitives cannot be silently
reduced to bounding boxes and declared clear. A missing required geometric
capability yields partial/unsupported coverage and blocks that release profile.

Physical topology and intended net associations are distinct. Net split/merge
offers an ECO with old/new identity membership and explicit correspondence choices;
it cannot silently follow renamed net labels. Deleting/replacing a component
previews impact on routing and associations, with conflicts for board-local edits.
Accepting a stale preview rejects; failure mutates nothing; one authoritative session undo restores
the previous authored data, associations and valid geometry/cache invalidation.

## Tests and acceptance evidence

Use original synthetic examples with asymmetric pin/pad/model landmarks. Keep
licensed library fixtures separate with notices. No old-fork or KiCad source.

| Gate | Evidence |
| --- | --- |
| CORE-1 | IDs/occurrence paths survive labels, net split/merge is explicit; quantities round-trip without display rounding; bounded invalid input rejects |
| TX-1 | UI and headless request use the same Electronics authority/post-state; stale/invalid apply leaves zero mutation/history allocation; one undo/redo restores associations with a new monotonic revision; duplicate request IDs cannot repeat mutations |
| LIB-1 | Offline reopen of used definitions/models, missing optional versus required data, rename/delete source, pin-map-incompatible upgrade and conflicting override preview |
| ECO-1 | Logical-led and PCB-only fixtures, component deletion/replacement, board-local divergence, stale plan rejection and full one-step undo |
| CHECK-1 | Positive/negative boundary witnesses for each supported check; deterministic ordering, rule provenance, partial/stale coverage and changed-waiver review |
| JOB-1 | Edit, undo, close/reopen and document-switch before controlled job completion; no cross-document or stale mutation, cancellation/publication races and queue bounds tested without sleeps |
| IO-1 | Save/reopen with assets/IDs, wrong magic/type/version, migration, interrupted save/recovery and externally changed destination; prior file preserved |
| UI-1 | [Visual/interaction acceptance](workspace-ui.md#accessibility-and-visual-acceptance) against the user's six views plus degraded states |
| GEO-1 | Top/bottom placement and asymmetric model orientation; holes/outline/stackup and selection mapping; missing model degrades, missing required geometry blocks |
| OUT-1 | Gerber/drill/BOM/placement/assembly set independently inspected against supported spec/profile, qualified tool/profile reproducibility, concurrent edit never mixes revisions, pre-publication cancel/failure never marks complete; post-publication cancel reports AlreadyCompleted |
| KICAD-1 | Named version/profile no-op and supported edit, foreign IDs/unknown nodes preserved where promised, unsafe conversion refused and black-box reopen |

Check properties across randomized valid layouts and adversarial bounded files
where useful; compare independent semantic expectations, not just serializer
output with itself. Test pure domain without display/GPU/filesystem; test real
PCBcraft Electronics persistence/undo in its own app, and real shared rendering
when available. Standalone packaging must build without the SolidCraft app or
sibling checkout. Reused-library checks come from the selected source; current starter checks prove
no application behavior. Do not add test counts as a substitute for these gates.

## Proposed performance and observability targets

These are planning targets, not measurements or guarantees. Establish a named
reference machine and original stress corpus before enforcing them: small board
100 components/1,000 pads; medium 1,000 components/10,000 pads/100,000 route items;
hierarchy 100 sheet occurrences. Bound inputs separately from these benchmarks.
Target p95 interactive frame ≤16.7 ms on the agreed desktop, selection/highlight
≤50 ms, local commit feedback ≤100 ms, and visible progress/cancel acknowledgement
≤100 ms for long work. Initial bounded-board check target ≤2 s and offline reopen
≤2 s exclude optional high-quality ray tracing; measure before accepting budgets.
Full checks/import/fill/export may exceed them and stay asynchronous. No browser
performance claim follows from desktop results; budgets require profiling there.

Record document/session token, revision/fingerprints, job/operation IDs, duration,
capability coverage and structured failure category. Provide a local diagnostics
export without credentials, private absolute paths or full board data by default.
No required telemetry. Log bounded summaries, keep deterministic repro fixtures
on explicit user action, and distinguish product messages from developer traces.

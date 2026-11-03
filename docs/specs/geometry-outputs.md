# Geometry, 3D and manufacturing

Status: phased contracts revised after Astra packet review; not capability proof. [Framework index](../framework.md).

## Geometry dependency versus UI/service dependency

PCBcraft is standalone. Use direct pinned SolidCore/shared-layer libraries for
geometry, independently of reusable UI libraries. The SolidCraft application,
mechanical engine, its CLI build and complete migration are not prerequisites.
Only selected library APIs/capabilities gate a slice.

The [reference experiment](../host-baseline.md) identifies actual SolidCore and
shared-record pins consumed upstream. It does not make the full host kernel/UI
chain a PCBcraft dependency. A narrow reusable wrapper is a candidate only if its
API/license/dependency closure is independently qualified. Otherwise use verified
SolidCore interfaces directly; do not invent future crates or duplicate geometry.
Electrical graphs and identity must not depend on B-rep internals. See the
[standalone decision](../adr/0001-standalone-application.md).

## SolidCore-first geometry boundary

### Verified geometry and rendering library candidates

Read-only verification at SolidCraft `c553365a2197a689e749389a6fbf707a38df9b61`,
November 3, 2026. `cargo tree --offline --locked -p <package> --target
x86_64-unknown-linux-gnu -e normal,build` confirms the following family dependency
sets; these are library candidates, not dependencies already added to PCBcraft.

| Package | Existing exported capability | Family normal/build dependencies |
| --- | --- | --- |
| `solidcraft-render` | `Camera`, `Scene`/`SceneMesh`/`SceneLine`, `ProductScene`/`ProductMesh`, appearances/materials, picture/job descriptions | `solidcraft-geom`, `solidcore-record`; no app/engine/document |
| `solidcraft-gpu` | GPU viewport pipelines, offscreen capture, GPU tessellation; public `view`, `capture`, `TessGpu` APIs | Render/geom/record above, plus wgpu and other external crates; no app/engine/document |
| `solidcraft-kernel` | SolidCore-backed geometric operations including `extrude_planar_regions`, `planar_regions_boolean`, booleans, transforms, measures and STEP-related facilities | `solidcore`, `solidcore-record`, `solidcraft-geom`, `solidcraft-profile`, `solidcraft-smlib`, `solidcraft-step`; no app/engine/document |

None of those selected graphs contains Truck. GPU's own dev dependencies include
engine/kernel/IO for upstream tests; this does not add them to a consuming app's
normal build. The root workspace's patches are not automatically a promise of an
identical external-consumer resolution: pin and test actual packages/features in
a small independent consumer before adopting them. This check did not compile a
new external consumer or establish PCB-specific capability coverage.

PCBcraft's scene adapter owns the translation from stackup, outline, component
placements, copper, pads/vias, mask/paste and silkscreen into layer-specific meshes,
lines, appearances and qualified model geometry. Shared libraries render that
scene; they do not supply an EDA model or automatically understand layers/nets.
Keep semantic IDs and layer/selection mapping outside render handles. Not every
visible copper or silk element needs a full B-rep. Capability tests must cover thin
layer depth/visibility, top/bottom transforms, model alignment and picking.

The existing `ui-egui::render_workspace` imports `SolveApp`, engine selections
and mechanical render settings. Reusing its entire UI is a separate extraction
problem. Prefer the already available backend libraries, then request specific
camera/material/control UI components if needed. Extrusion or measurement
operations can be reused without the original mechanical toolbar/command shell.

Sources at the same pin:
[render manifest](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/render/Cargo.toml),
[render exports](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/render/src/lib.rs),
[GPU manifest](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/gpu/Cargo.toml),
[GPU exports](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/gpu/src/lib.rs),
[kernel manifest](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/kernel/Cargo.toml),
[product scene](https://github.com/tangency-labs/solidcraft/blob/c553365a2197a689e749389a6fbf707a38df9b61/crates/render/src/product.rs).

### Shared-layer consumption

Use verified shared geometric computation/record/operations/tessellation/view
layers. SolidCore's current layer plan assigns geometry computation to the
kernel and workflows to ops; apps should call shared operations rather than
carrying their own distance/intersection/meshing/import algorithms. Several
planned shared crates were absent from earlier inspected manifests; `solidcore-record`
is now present at the pinned host baseline. Do not generalize that to other layers. Their
availability/operation coverage is an explicit prerequisite, not a placeholder
dependency or permission to add Truck.

EDA owns semantic geometry inputs: board outline/holes, pad and track parameters,
zone intent, placement side and physical configuration. The shared geometry
service constructs/evaluates shapes, intersections and measurable distances.
EDA rules compare returned measurements with effective constraints and explain
electrical/manufacturing meaning. Record exact/approximate status, algorithm
version and error bounds. Contact, clearance and release decisions cannot rely
on a screen mesh or an unbounded display approximation.

For a missing shared planar operation needed by DRC/zone fill, specify the exact
required computation and original fixtures for the shared layer; do not duplicate
a general polygon/boolean kernel in the app. Simple intent/identity/structural
tests can proceed while this dependency is unresolved. Existing host Truck paths
must not become new project dependencies or a hidden compatibility fallback.

## Narrow capability matrix and coordinate contract

Do not represent every electrical feature as a B-rep. Logical graphs, symbol
paths, authored pad/track parameters and simple 2D display primitives are EDA
records. Use verified shared geometry for the computations that actually need
it. Missing advanced booleans/meshing must not block the shell, basic model,
editing or structural fixtures; they block only affected projections/checks or
release profiles. No alternate kernel/Truck fallback is permitted.

| Capability | Evidence / required adapter proof | Original acceptance fixture |
| --- | --- | --- |
| Units, points and transforms | Raw SolidCore Point2/Point3/Transform exports were observed; selected shared operation/provider API and behavior still require G0 verification | Asymmetric landmarks, unit conversion and inverse transform within declared tolerance |
| Planar contact/distance/intersection | Exact shared operation API/primitive coverage unverified; required before relevant connectivity/clearance coverage | Contact and clearance just below/equal/above threshold, rotated pads and holes |
| Planar offset/union/difference | PR #11360 kernel exports `planar_regions_boolean`; specific primitive/API/robustness qualification remains required for corresponding zones/apertures | Concave region, cutout, touching edges, thermal and island cases |
| Board body/extrusion and holes | PR #11360 exports `extrude_planar_regions`; direct SolidCore versus existing boundary and tolerance coverage need task-specific qualification | Original non-symmetric outline/slot with known dimensions and stack thickness |
| Display primitives/tessellation | Separate disposable 2D primitive display from verified shared tessellation; planned layer names are not shipped API proof | Zoom/DPI, pick mapping and rebuild without authored-state change |
| Model loading/alignment | STEP-related workspace crates observed; exact allowed shared import/view consumer API not established | Licensed/original asymmetric model, top/bottom and rotation/offset landmarks |
| Cancellation/robustness | Raw CancelToken/Tolerances exports observed; per-operation guarantees still need tests | Deterministic cancellation, invalid geometry, bounded failure without session mutation |

G0 records exact selected SHA, signature, supported primitive/operation, units,
error/cancel behavior, tolerance/approximation status and test result for each row.
Exported raw types do not prove a complete supported application API. Tests above
are required evidence, not tests already executed by this specification task.

Proposed EDA frame: board XY is right-handed, +Z toward the top side; origin is an
explicit authored board datum, distinct from fabrication/assembly output datums.
F1/G0 fixes the exact Z reference plane and layer-Z offsets from ordered stackup,
units and planar rotation convention before implementation. Each adapter records
an explicit source-to-target matrix, including model units/local datum, side and
handedness. Bottom-side placement has a defined mirror/orientation composition;
view flipping is a separate camera transform and never changes authored side.
Test top/bottom asymmetric pad numbering/model landmarks, 90-degree rotation,
nonzero origin and manufacturing datum conversion. Do not infer host/kernel axes
or a vendor centroid convention by appearance. Reject/report lossy conversion.

## 2D/3D projections and component models

Render adapters consume an immutable, revisioned scene description mapped back
to domain refs. The board scene derives outline, thickness/stackup, mask/copper/
silkscreen and fitted component models from the same authored board/variant.
Scene caches are keyed by geometry, library/model hash, display settings and
algorithm version. Deleting the cache cannot alter design or analysis results.

Every model attachment declares asset hash, units, local origin/axes, scale,
package datum, rotation/translation and mapping to its footprint instance.
Convert board-side mirroring and assembly transforms in the shared geometry
adapter; do not guess from a mesh's centre or mutate pad identity to fit a model.
Record uncertain vendor-model alignment and offer an explicit reviewed override.
Missing model geometry shows an honest proxy or absence with component identity,
without replacing exact electrical geometry or corrupting BOM/placement data.

Cross-probing uses footprint/pad↔component/terminal↔explicit model/link occurrence mapping, not
triangle numbers. A rendered pick carries the source revision; stale picks are
resolved/rejected before selection. Rendering-only visibility, copper-fill hiding,
exploded view or camera choice never changes conductive topology. Refill/rebuild
is a separate command/job. Board-body appearance belongs to 3D/render; layout
layer visibility remains an inspection choice.

Reuse independently consumable family projection/camera/navigation, capture and
material controls where qualified. PCBcraft owns its Render view; app-private
helpers need a narrow library boundary, not a full host dependency. Keep 2D
navigation planar.
Electrical layers and copper identities remain distinct from display materials.
Mechanical enclosure checks are analysis of explicitly linked bodies with named
coverage, not proof of full manufacturing fitness.

## Routing, zones and analysis stages

The complete first workflow supports a bounded two-layer route with explicit
width/layer and collision findings, not shove routing. Later operations add
through/blind/buried via spans, arc/45-degree routing, walkaround/shove, layer
transitions and track dragging. Each route plan must preserve net identity and
respect the effective rule policy at its base revision.

Zone fill consumes authoritative boundaries, priority, net/layers, keepouts and
thermal/island settings plus effective rules. The filled result is derived and
revision-tagged. Refill can be cancellable, and stale fills cannot pass a release
gate. Global/per-layer fill visibility is separate from calculation and retains
user choices. Thermal relief, island removal and net ties need independent
electrical/geometry fixtures before coverage is claimed.

Differential pairs and tuning store intended membership/topology, widths/gaps,
length/delay/skew constraints and reference stackup assumptions. Solver outputs
are reviewed route proposals. Distinguish geometric length from electrical delay
and modeled impedance; unavailable material/frequency data is a missing-input
diagnostic. Signal/power integrity simulation is a later service boundary, not
implied by basic DRC or a pair-routing button.

## One manufacturing snapshot

All output adapters consume one validated immutable release context:

```text
ReleaseContext = { project_id, authored_content_hash, electronics_revision,
                   board_id, variant_id, resolved_dependency_hashes,
                   effective_rules_hash, geometry_and_fill_fingerprints,
                   check_coverage_and_accepted_findings, output_profile,
                   tool_and_algorithm_versions }
```

A current matching connectivity/DRC/fill result is required for the scope the
profile needs. Policy specifies blockers, warnings and acknowledged limitations.
Unrouted/shorted copper, missing required assets/mappings or unsupported geometry
cannot become an unlabeled production-ready package. An inspection export can
be allowed separately, carrying its limitations prominently in the manifest.

| Output | Contract |
| --- | --- |
| Gerber | Supported copper/mask/paste/silkscreen/outline layers with declared units, origin, polarity and attributes; layer functions and stack order are explicit |
| Drill/route | Profiled NC/Excellon-compatible output with units/precision/zero policy, separate plated/nonplated classification and explicit slots/routing/tool mapping |
| BOM | Resolved component/part identity, values/package, declared variant and BOM inclusion policy, quantities and approved alternates; supplier observations remain separate |
| Pick-and-place | Components included by assembly/variant placement policy, stable references, side, datum/rotation/centroid and units; profile records mirrored-bottom conventions |
| Assembly/fabrication drawings | Shared drawing/vector/export facilities with dimensions, references, polarity, stackup and revision/variant information |
| Mechanical exchange | Verified shared export of board/body/component placement, with units and model-coverage report |
| Package manifest | Source/profile/tool fingerprints, artifact hashes, coverage/waivers, units/origin/layer/tool maps and validation results |

Use [Ucamco's public specifications and downloads](https://www.ucamco.com/en/gerber/downloads)
for Gerber targets, with an explicit supported revision. Exact drill and assembly
conventions require a named profile and acceptance corpus; do not guess a
manufacturer's rotations, slot support or filename requirements. IPC-2581 and
other formats can later implement the same release-context/artifact contract
after their specification and licensing have been reviewed. No universal exporter
plugin framework is needed for the first two actual output adapters.

Outputs aim for deterministic results within a qualified profile, pinned tool/
algorithm versions and recorded environment assumptions. Universal cross-platform
byte identity is not promised.
Normalize ordering and numeric formatting; volatile audit timestamps belong in a
separate manifest field and do not change manufacturing geometry. A release job
stages artifacts, validates them and publishes a complete set/manifest atomically
where supported. Cancellation accepted before that publication prevents publishing;
if atomic publication wins the race, report AlreadyCompleted with package identity.
Disk failure/invalid output leaves no set marked complete; previous packages are
not partially overwritten. Editing during
export does not change the captured source, and the completed package labels its
original revision rather than the latest editor state.

BOM, placement, 3D and paste read the same declared variant with role-specific
policies; they need not omit the same objects. DNP may retain footprint/copper/
paste, while 3D can show DNP as a ghost. View hiding is independent of population
and release policy. These choices are captured in the output profile/manifest.

Validation uses original geometric cases, format parsers/independent viewers and
black-box tools where appropriate. Check dimensions, polarity, drill/tool counts,
netlist correspondence, variant population and known rotations against independently
specified expected values. Reusing the exporter as its sole test oracle is
insufficient. Publishing/uploading manufacturing data is a separate user action.

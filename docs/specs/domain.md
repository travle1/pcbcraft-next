# EDA domain contract

Status: revised after Astra packet review; proposed data model, not production code. [Framework index](../framework.md).
All types and module names in this specification are original illustrative
contracts. Exact schema names and bounds are finalized by tasks F0/F1 before
dependent implementation; no host or dependency type is implied to exist.

## Identity, quantities and state

Each EDA payload has a persisted workspace identity. Use distinct typed IDs for
sheet definitions/instances, component definitions/instances, logical terminals,
symbol placements, boards, footprint instances, physical pads, nets and rules.
Do not identify objects by reference designator, library path, vector index or
render face index. IDs survive save/edit/undo; deleted IDs are not reassigned to
unrelated objects. A cloned independent design receives a new workspace identity
and an explicit remap. Save As and clone are distinct operations.

F1 selects opaque typed durable IDs, namespace representation, generation and
collision policy before dependent implementation. Host feature/occurrence IDs and imported KiCad UUIDs
are external mappings, not the EDA allocator. A semantic reference includes
workspace, object kind/ID, optional sheet-instance path and optional terminal/pad
subreference. Runtime session incarnations and monotonically advancing authoritative revisions are distinct
from persisted document identity and from content hashes. Undo does not rewind
the authoritative revision or silently recycle identity.

Proposed quantity baseline: signed integer nanometres for authored EDA lengths
and positions, bounded integer microdegrees for authored planar orientation,
explicit units for electrical/physical constraints. Bounds, overflow behavior and
conversion tolerances must be fixed with original fixtures before F1 lands.
Grid snapping is an editor decision, not a loss of saved precision. Convert to
SolidCore/host units at one adapter boundary, check finite values/ranges and
report quantization; never round repeatedly on load/save or mix schematic and
board coordinate conventions. Derived analytical geometry can use the shared
kernel's representations without becoming a second authored coordinate truth.

Tolerance values are named and versioned: import quantization, geometric contact,
curve approximation, rule comparison and display tessellation are separate.
A display tolerance cannot decide electrical connectivity or manufacturing edges.
No silent epsilon makes a failing rule pass. Precision and threshold cases are
part of acceptance data.

## Identity relationships and remapping

| Identity | Relationship / invariant |
| --- | --- |
| Sheet definition | Reusable authored sheet content; not a placed occurrence |
| Sheet occurrence | Stable opaque identity referring to a definition and parent occurrence; paths use IDs, never display names |
| Component occurrence | One physical part within an occurrence context; owns values/variant identity and maps to at most one selected physical realization per board configuration |
| Symbol unit placement | Presentation belonging to that component occurrence; multiple units do not create extra BOM parts |
| Logical terminal | Electrical identity independent of pin graphics; several visible/hidden/stacked graphics may map to one terminal |
| Footprint instance | Physical placement identity; may link to a component occurrence or be explicitly board-local |
| Pad instance | Distinct physical identity even when pad numbers repeat; several pads may map to one terminal |
| Pin-to-pad correspondence | Explicit relation, not name equality; several terminals sharing a pad require a supported declared electrical equivalence or reject |
| Mechanical/NC state | Mechanical pads may lack terminals; NC is explicit intent, not a missing mapping or deleted physical pad |
| Logical net result | Derived membership with source witnesses; authored associations and split/merge lineage are tracked explicitly |

A hierarchy move preserves occurrence/component identity where explicitly valid;
otherwise the move produces a reviewable remap with affected references, variants,
BOM and board associations. Do not silently rebuild IDs from renamed hierarchy
paths. Repeated definitions instantiate distinct physical occurrences. Net splits
or merges create lineage/mapping decisions or unresolved correspondence; arbitrary
merges cannot promise the same persistent computed-net ID. Neither net labels nor
matching member counts silently decide board association.

## Logical model, schematic and hierarchy

| Record | Facts it owns |
| --- | --- |
| Electrical component definition | Logical terminals/functions and supported package mappings; a generic resistor is valid without a part number |
| Electrical component instance | Stable physical-device identity, values, resolved definitions and explicit instance overrides |
| Symbol representation | Drawing and unit/body styles, pin anchors and terminal correspondence |
| Sheet definition | Authored symbol placements, wire paths, junction intent, labels, ports, buses and graphics |
| Sheet instance | Definition reference, parent instance, instance parameters and path; repeated channels produce distinct physical instances |
| Logical connection intent | Explicit terminal connections and authored schematic/netlist/PCB-only connection sources |
| Resolved logical graph | Derived net membership and diagnostics for a specific source revision/configuration |

A multi-unit symbol has several independently placed presentations of one
physical component. Shared/common pins are one terminal with explicit repeated
representations, not accidental duplicate devices. Unit/gate and pin swaps are
explicit allowed mappings with electrical compatibility, history and downstream
impact; initially unsupported swaps reject rather than renumber silently.

Hierarchy separates reusable definitions from instance identities. A sheet can
be placed multiple times; instance paths scope local nets and references.
Global/hierarchical/local labels, ports and bus members have explicit scoping
rules and diagnostics. A bus is a collection/mapping of signals, not one net.
No-connect markers express intent; power symbols and hidden/stacked pins need
documented semantics, not geometry guesses.

The connectivity compiler consumes authored intent and produces one deterministic
logical graph with source witnesses. Crossing lines connect only under the
specified junction/endpoint rules. It never takes screen hit-testing, visibility
or wire artwork as electrical authority. A direct agent/netlist workflow writes
the same intent. Do not maintain an independently editable compiled-net table.

ERC consumes that graph plus terminal electrical types, configuration and rules.
Findings identify rule/code/severity, affected semantic references, source
witnesses, revision and coverage. Begin with assignment and incompatible-driver
cases; later include hierarchy, power/intent and package mapping completeness.
Unsupported electrical types or unresolved labels make coverage incomplete.

## Component selection, libraries and assemblies

The [library management draft](library-management.md) develops the user workflows,
source resolution, starter-content expectations and L0–L4 acceptance sequence
under these ownership/identity contracts. It is proposed work, not an implemented
external library or approval to import data.

A component selection resolves logical definition, symbol representation,
package/footprint and terminal-to-pin-to-pad mapping as separate facts. Physical
package and footprint are related but not interchangeable: package characteristics
can have multiple land patterns. Manufacturer part identity and supplier offers
are optional and carry their own source/revision/observed-time information.

A resolved dependency records durable identity, exact revision/content hash,
embedded used definition, dependency hashes, source locator, license/attribution,
capability coverage and verification status. Capture only what the project needs.
Source path/nickname and search-index location are not immutable identity. A
changed installed source cannot rewrite a project's resolved dependency.

Local project definitions work without a registry. Team folders/Git and public
catalogs can provide artifacts through the same resolution semantics, with no
mandatory cloud login. Missing optional supplier/3D data does not block electrical
editing. Missing required terminal/pad geometry or mapping blocks affected work
with an exact dependency diagnostic. Unknown revisions/licenses remain unknown.

Preserve a base revision and explicit local/instance overrides. An upgrade plans
old-base/new-base/override differences, detects conflicts, and reports changed
terminals, pad geometry/origin, routes, checks, BOM and model alignment. Matching
names/numbers is insufficient to approve a replacement. Many pads per terminal,
mechanical pads, exposed pads and pin aliases require explicit mappings and tests.
Apply a reviewed upgrade as one transaction; source removal leaves captured parts
intact. Never use a fresh catalog hit as a silent substitute.

Assembly variants are overlays on stable design identity: fitted/not-fitted,
parameter changes and explicitly compatible alternate part choices. They select
their own BOM/placement/assembly outputs. DNP policy may retain footprint, copper
and paste; view hiding is independent. BOM, placement, 3D and paste consume the
same declared variant and role-specific policies, not necessarily equal omissions. A fabrication-changing variant is an
explicit physical configuration, not an accidental side effect of a BOM option.
Keep the active variant in analysis/export input fingerprints.

## Board model and physical realization

The initial workflow has one board; schema ownership allows board identity
without promising a full multi-board system. A board may be schematic-linked,
netlist-led or PCB-only. Board-local electrical intent and independent footprint
instances are valid; no fake schematic component is required. Later linking
requires a reviewed correspondence plan.

| Area | Authoritative model | Derived work / later behavior |
| --- | --- | --- |
| Stackup/layers | Ordered stable layer IDs, function/material/thickness, copper and reference information, via spans | Electrical/impedance analysis with explicit assumptions; never infer from display order |
| Outline | Boundary/holes/slots and mechanical constraints using the shared geometry record seam | Validated closed regions, extents and 3D body projection |
| Placement | Footprint identity, transform/side, locked state and source correspondence; pad numbers and geometry | Courtyard/assembly intersections and component-model alignment |
| Routing | Net-owned tracks/arcs with width/layer, vias with drill/plating/span, junction/connection anchors | Physical connectivity, clearance and route/tuning previews |
| Zones | Boundary, net/layers, priority, keepouts, thermal/island policy and settings | Revision-tagged filled copper result; visibility is independent of fill computation |
| Manufacturing geometry | Mask/paste overrides, silk/fab/courtyard/mechanical items, dimensions and notes | Derived output layers according to an explicit output profile |

Copper connectivity is computed from actual conductive regions per layer and
valid plated connections. Logical intended nets and physical copper islands are
distinct. Compare them for shorts and unconnected obligations. Drills, slots,
voids, net ties and deliberate connections have explicit semantics; graphics,
hidden layers or model meshes do not create connectivity.

Routing supports an explicit policy: free-with-findings, walkaround and later
shove are different capabilities. Planned route previews carry source revision,
layer, effective rules and touched-object witnesses. Differential pairs, topology,
length/delay/skew tuning and impedance constraints build on the same rule and
connectivity services; they are later phases, not claimed in the first route tool.

## Rules, constraints and validity

Commit invariants protect bounded quantities, unique identity, referential
integrity, valid layer ownership and complete required mappings. Design quality
rules can report a structurally valid but unrouted or violating board. Export
profiles separately define required check coverage and accepted findings.

Rules have stable identity, kind, physical units, scope, source/provenance,
priority and optional named overrides/waivers. Initial scopes are board default,
net class, net and object/pair. Resolve by explicit per-kind precedence; ties that
contradict diagnose ambiguity, while combinable restrictions use a documented
combination rule. Never rely on map iteration. Each finding returns its effective
rule and resolution trace. Waivers carry reason, bounded scope and a witness that
can become stale after relevant edits.

DRC covers physical connectivity, clearance, width, edges/holes, via/drill and
annular-ring constraints, mask/paste/courtyard and manufacturing capabilities in
stages. Results declare implemented checks, skipped unsupported geometry,
approximations and input revision. A zero finding count with missing coverage is
not a clean manufacturing gate.

## Schematic/board reconciliation

Cross-probing uses stable correspondence once its supported mapping exists; electrical changes
do not silently regenerate the other editor. An ECO compares linked intent and
realization against their last agreed baseline and produces adds/removes/updates,
ambiguous matches, route consequences and a proposed post-state.

The user chooses direction and scope: schematic→board, board→schematic where
supported, or explicit selective reconciliation. Conflicting edits on both sides
require a decision, not last-writer-wins. Keep footprint placement, board-only
objects and unchanged routes by identity. A changed pad/net mapping reports
orphaned/shorted copper; it must not move or delete it invisibly. Plans include
base revision, source/library hashes, affected IDs and unsupported operations;
apply atomically only while current. Store the agreed mapping/baseline so the
next comparison can explain new divergence. The MVP implements one named direction
while preserving the model's PCB-only and future reverse-update path.

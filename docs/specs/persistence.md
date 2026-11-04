# PCBcraft persistence and format compatibility

Status: revised after Astra packet review; proposed Electronics persistence.
[Framework index](../framework.md).

## Existing SolidCraft contracts

At the inspected revisions, new designs use `.sc`. The container is ZIP with
`container.json`, authoritative `document.json`, content-addressed embedded blobs,
and optional preview/model caches. Plain JSON design IO also exists. A document
has a `solidcraft/N` format, migration/pending-change machinery and refusal of
unsupported newer versions. The design reader reconstructs the authoritative
Document; display caches are disposable and never proof of model correctness.

Session owns an `Arc<Document>`, commit revision, dirty/saved comparison and
undo/redo snapshots. Newly opened sessions initialize empty undo/redo stacks;
reopening a saved file does not promise persistent edit history. These are
reference behaviors, not the session or schema PCBcraft must depend on. Reuse
independent utilities where qualified; PCBcraft owns its Electronics session and
versioned native persistence. See the [standalone decision](../adr/0001-standalone-application.md).

Host metadata preserves namespaced objects, but its contract says informational,
never geometry. Metadata writes have 64 KiB per namespaced value and bounded
total/depth. It is suitable for descriptive integration properties, not a
production home for authoritative board/net/library data. Unknown top-level
fields also do not establish a supported extension slot. See [sources](../sources.md).

## Proposed authoritative EDA payload

PCBcraft's Electronics session owns the entire native project aggregate, with
an independent schema/content marker and provisional .pc suffix. PCBcraft handles
type detection, open/save, migrations and recovery itself. No SolidCraft Session,
FileKind, mechanical document or external application is required.

Reuse independently consumable container/asset/atomic-write utilities when their
APIs, bounds, licenses and dependency closure are verified. Needed private IO
helpers require a narrow extraction/adapter; do not import the mechanical engine
to save electronics. Native encoding and exact API remain open until F1 contracts
and F0's format decision. Contracts precede persistence implementation.

Illustrative envelope only, not a selected external schema:

```json
{
  "schema": "pcbcraft-eda/1",
  "workspace_id": "persisted-domain-identity",
  "design_intent": {},
  "schematics": [],
  "boards": [],
  "resolved_library": {},
  "correspondence": {},
  "variants": [],
  "rule_sets": [],
  "assets": [],
  "compatibility_sources": []
}
```

These sections correspond to [domain ownership](domain.md), not duplicate stores
for the same fact. Keep caches, selection, active jobs, prices fetched at runtime
and GPU handles out of authoritative electrical data. The authoritative session snapshot must
contain the whole aggregate so save/recovery/undo see the same state as all views.
EDA changes participate in dirty tracking and invalidate affected analysis and
EDA geometry projections. Mechanical links are explicit versioned references;
initial scope excludes cross-document ECAD/MCAD transactions and kernel evaluation
for pure electrical metadata changes.

Version the container/envelope and EDA schema independently of editor version.
Native readers must refuse unsupported saved semantic versions instead of
silently dropping them. Independently reused migration helpers must not require
mechanical format stamps or a kernel dependency. EDA migrations are ordered, deterministic,
bounded and tested from original prior-version fixtures; validate before replacing
the current design. Unknown authoritative variants/fields must be preserved by an
explicit safe policy or rejected/read-only, not accepted then discarded on save.
Do not downgrade a schema stamp to bypass a reader's refusal.

PCBcraft needs explicit electronics type detection and open/save dispatch.
SolidCraft Drawing support is a reference precedent, not a required host API. Appending arbitrary
ZIP entries is not a verified preservation or plugin contract.

## Project files and user-facing type

The user chose a distinct electronics extension and suggested **`.pc`**. Treat it
as provisional for the entire electronics project: schematics, board(s), resolved
used libraries, variants and settings. Sheet and board tabs are child views, not
necessarily separately saved files. A PCB-only project remains valid. The exact
suffix, multi-board MVP limit and encoding need the integration ADR.

`.pc` is short but already used by [pkg-config metadata](https://people.freedesktop.org/~dbn/pkg-config-guide.html)
and [Oracle Pro*C source](https://docs.oracle.com/cd/B13789_01/win.101/a96111/use.htm).
These verified collisions do not forbid it; they make globally claiming every
`.pc` file undesirable. Decide OS associations explicitly and test an existing
pkg-config/Pro*C file is never opened as an editable electronics project. A longer
product-specific suffix remains an option before registration, not a decision
already made. Do not infer exclusive ownership from an unused-looking suffix.

Separate these contracts:

| Layer | Required behavior |
| --- | --- |
| Project container | Bounded package/asset ownership and complete snapshot; may reuse host ZIP implementation after review |
| Electronics schema | Typed authoritative EDA facts with independent version/migrations |
| Suffix and visible type | Provisional `.pc`, displayed as Electronics project with distinct icon; no native `.sc` default |
| Sniff and dispatch | Bounded content magic/container marker, document-kind and version validation; ZIP signature alone is insufficient |
| MIME/OS association | Product-specific candidate identifier only after packaging decision; no claim of an assigned standard MIME type |
| Editor route | Open project in Electronics workspace, restore/reveal selected sheet/board by stable IDs |
| Interchange | KiCad, manufacturing and possible future `.sc` export are explicit adapters with coverage reports |

Open rejects content/suffix mismatches clearly and offers inspection/import where
safe. Double-click routes by recognized content and association, not arbitrary
text parsing; unknown versions cannot be silently rewritten. Save and Save As
keep the native electronics type, required assets and identity/copy semantics
explicit. Export chooses a destination/profile and never renames a native file
to pretend it converted. Multi-document product containers containing MCAD and
electronics child documents remain a possible later family feature, not the MVP.
Possible `.sc` export later must describe which board/model/link semantics it
preserves; shared ZIP bytes alone establish no semantic compatibility.

## Dependencies, assets and path safety

The [library management draft](library-management.md#project-snapshots-caching-and-sharing)
defines proposed used-definition capture, portability exceptions and source/cache
ownership. Its first slice must respect the current draft limits until a
versioned persistence change is qualified.

Embed used definitions and required resolved assets through the native project
resource manifest, potentially reusing independently extracted container utilities. Reuse content addressing, deduplication, bounded reads and
package handling; avoid base64-duplicating large assets into every component.
Asset records include content hash, byte length/media type, role, provenance,
license/notice and optional external source locator/revision. Keep external path
location separate from content identity. New EDA resource kinds need reader/writer
round-trip support; current host blob support is not automatically generic.

Preserve project-relative references and original import provenance. No path
traversal, device-file access, arbitrary extraction, implicit network fetch or
script execution is allowed from a project file. Normalize and validate paths,
respect PCBcraft access rules and approved dependency locations, and cap decompressed
bytes, nesting and counts before allocating. A missing hash or inconsistent blob
is an exact dependency error. Optional missing model assets can degrade preview;
required electrical data cannot be silently substituted.

Save stages a complete valid container beside the target, then uses verified independently separable atomic
replacement/backup utilities under Electronics lifecycle ownership. A failure preserves the previous file and current
dirty document. Detect an externally changed target before overwrite. Autosave
and recovery use the same payload/version/asset rules; recovery cannot resurrect
a stale job as current. Multi-file interchange output uses a staging directory
and manifest, not a sequence that presents half an export as complete.

## KiCad interoperability is explicit

Implement from [public formats](https://dev-docs.kicad.org/en/file-formats/), manuals
and black-box tests with original or independently licensed fixtures. Never read
KiCad implementation source, patches, headers or annotated-source pages. Do not
reuse preserved-fork implementation/fixtures. KiCad syntax does not dictate EDA
domain identity or storage and does not by itself determine round-trip quality.

Maintain a matrix per supported major/format version for project settings,
schematics/hierarchy, symbols, footprints, boards, stackup, rules, models and
manufacturing metadata. Track read/represent/edit/preserve/write independently.
The initial implementation supports one named profile; broader parity is deferred.

An import produces source-preserving syntax/provenance data, external-ID↔domain
maps, represented facts and a coverage report. Unknown sections can remain opaque
only when edits cannot invalidate their semantics. Preserve UUIDs, instance paths,
source coordinates/units and original data ownership. Do not create ungrounded
manufacturer or library revision metadata during migration.

No-op save should preserve source exactly where the contract promises it. A
supported edit patches only owned data or performs an explicit reported conversion.
Schema/version conversion, renumbering, unsupported features or lost information
must be visible before export. Unsupported authoritative electrical/rule/geometry
content blocks destructive edits or limits the document to inspection. Arbitrary
retention without understanding dependencies is not a losslessness guarantee.

Round-trip acceptance includes no-op, one supported field edit, hierarchy/ID
mapping, unknown-node preservation and unsafe-edit refusal, reopen in the target
KiCad version, and independent semantic comparisons. UI appearance or a successful
parse alone is insufficient. Export never overwrites original source by default.

The native electronics project is authoritative for this proposed mode. KiCad
import/export remains required, with its own preservation guarantees. A future
direct-KiCad authoritative editing mode needs an explicit ownership/save decision;
do not maintain two freely writable conflicting canonical projects.

## First integrated persistence proof

Use one original tiny EDA component and its pinned definition in PCBcraft's own
desktop application. Electronics save/reopen retains IDs, pin/pad mapping and dependency data;
a value/placement edit dirties the session, undoes once through that same session,
redoes and survives save. GUI and MCP/headless use the same operation.
Invalid payload/newer schema refuses safely; interrupted save preserves the old
file. Verify the running PCBcraft app as well as its headless codec/operation harness. No renderer
or new geometry dependency is needed for this proof.

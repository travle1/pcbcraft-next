# 0002: Bounded schematic draft contracts

Implemented November 3, 2026 for the requested base app/schematic slice. These
contracts enable a real editing loop without pretending the full framework is
complete. They do not settle final electronics-project encoding or interoperability.

## Ownership and identity

`document.rs` contains original schematic data; `session.rs` is the sole commit
and undo authority. Project, sheet, component, symbol, junction and wire IDs are opaque
random 128-bit typed values. Reference designators are mutable labels. One component
has one symbol in this slice; unsupported multiunit/hierarchy forms reject rather
than silently flatten. Pin IDs are definition-owned strings, separate from drawing coordinates. The first
four definitions retain `"1"`/`"2"`; the new generic modules have up to eight terminals.

Positions use signed integer nanometres, bounded to ±1 metre per axis; grid is
1.27 mm, rotation is an integer quarter-turn. UI transforms use floats only for
display/hit testing, then snap and validate authored positions. Snapping clamps
to the last valid grid cell. Wire links store pin or junction endpoint identities, not a
computed-net identity. Crossings have no semantic effect.

Commands carry an expected revision. A candidate aggregate is cloned, changed
and fully validated before one commit/history entry. Invalid or stale edits leave
zero mutation. No-op edits create no revision/history entry. Undo/redo restore
snapshots but advance revision; reopen starts a new random session incarnation.
Typed error categories distinguish stale revision, invalid data, unsupported
format, IO and external-file conflict. GUI invokes the same headless authority.
Future transports must add incarnation checks and request idempotency before
remote retry support; revision alone is insufficient across reopen.

## Experimental persistence

Provisional `.pc` files are UTF-8 JSON with an explicit marker:

```json
{"format":"pcbcraft-schematic-draft/4","document":{"…":"typed draft payload"}}
```

The snippet is illustrative, not an accepted fixture. Unknown fields/variants,
unsupported format/catalog versions, duplicate identities/references, dangling
endpoints, invalid coordinates and duplicate terminal pairs reject. A file is at
most 4 MiB; documents contain at most 4096 components and 4096 links. This limits
the current snapshot-based editor. The suffix alone never establishes file type.
General project containers, embedded libraries/assets, general migrations and `.sc`/KiCad
adapters remain separate design work. Do not silently load richer documents here.

The four original definitions are pinned by `builtin_definitions_version = 1`
and retained unchanged in the executable. Definition set 2 adds four original
multi-pin generic modules. Placing one into a version 1 draft upgrades the set in
the same transaction; undo restores the prior aggregate. The version 5 envelope
reader accepts version 1–4 files. Version 1 envelopes cannot contain version 2
module/annotation/link-alias data; saves emit version 5. Version 1 drawings/terminal semantics must not
silently change. A future general library snapshot format must migrate these
definitions explicitly. No manufacturer identity, package or footprint is implied.

Version 2 adds bounded sheet annotations (256 maximum, 128 printable bytes each)
and optional link display aliases (32 printable bytes). Aliases abbreviate explicit
terminal-to-terminal links, never connect by string equality. Validation requires
one name per connected link group and one connected group per name. This is not
a general net-label placement model. The renderer draws aliases at their
actual terminal anchors; endpoint identities still drive edits and persistence.

Version 3 adds `Wire.draw_path`, default false for older files and omitted when
false. A named link with true retains its normal drawn path; an unnamed link always
draws its path. Versions 1/2 reject true rather than misrepresenting named paths.
The original aliases remain unchanged on load. Renaming a connected group gives
all its links the same name; newly named normal links set `draw_path = true`.
An unchanged name is a no-op. Clearing a name restores visible paths.

Version 4 adds typed, bounded junctions (up to 8192) and wire endpoints that are
pins or junction references. Optional authored interior bends (up to 256 per wire)
form nonzero orthogonal paths. No bend data retains the prior automatic display
route; splitting or repositioning that route makes its geometry explicit. Older
format envelopes reject these new fields/endpoint forms. Branch creation splits
an edge and inserts a durable junction in the same candidate as the new wire.
Free degree-one endpoints are valid incomplete design, shown with open circles.
Unused junctions are removed when their last incident edge is deleted.

Drag previews use a validated disposable candidate. Segment slides keep endpoint
identities/positions fixed and add end legs; corner edits preserve other anchors;
junction movement stretches every incident authored route. Ordinary reshaping
preserves pin membership. Deliberate endpoint drops and positive-length collinear
overlaps join compatible runs; conflicting net names reject atomically. Cleanup
removes duplicate/backtracking runs and redundant junctions in the touched area;
normalization may replace wire/junction IDs while preserving component/pin IDs.
Pure crossings stay disconnected. Supplied legacy projections are validated and
revision-bound; unrelated automatic routes remain unchanged. Releasing applies through the session with captured revision
and incarnation; cancellation or failed/stale validation makes no edit. Moving
components stretches attached authored end legs in that same transaction.
See [wire editing](../wire-editing.md). None of this implies ERC, body avoidance for
manual paths, automatic rerouting of all neighbours, or PCB copper routing.

The derived compiler groups explicit pin/junction links and lists unconnected pins.
It never joins crossings or equal strings. Results are ordered by typed IDs;
anchors are revision-local selection keys, not persistent net identities.
`RenameNet` and `Disconnect` use the same candidate validation and history as all
other edits. A disconnect preview reports fragments and name consequences; the
largest remaining linked fragment by node count retains the name (terminal-ID tie break), other
linked fragments become unnamed, and isolated pins have no group name. A reviewed
GUI apply binds incarnation/revision and rejects stale/reopened changes. Undo
restores the entire aggregate. Durable net correspondence/ECO remains future work.

`pcbcraft-connectivity-report/1` is a separate JSON report with components,
terminal membership, names, link IDs, unconnected pins and runtime snapshot stamp.
It is neither a native project reader nor KiCad/SPICE interchange. No import or
schematic/board synchronization is implied. See [net workflows](../net-workflows.md).

Detailed Properties applies reference/value/rotation as one validated candidate
and one undo entry. Its draft carries session incarnation and revision, rejecting
stale/reopened edits without mutation. Panel position and ribbon tab state remain
presentation-only and are not serialized into `.pc`.

Save captures a validated document, writes/syncs a temporary sibling and replaces
the target. Load/save are worker operations while authoring is disabled. Keep
the prior document and dirty state on failure; mark saved only the successfully
written snapshot. Existing-file comparison detects prior external edits, but
does not provide cross-process locking or power-loss/directory durability guarantees.
Undo history and runtime revision/incarnation are not persisted. Recovery/autosave
are deferred and must be added before making stronger durability promises.


## Version 5: selection and editable text

The version 5 envelope adds a durable `AnnotationId` to each note and a bounded
`text_overrides` list addressed by typed note/reference/value/net targets.
Notes allow up to 4096 bytes, including newlines and tabs.
A text override contains physical height (0.25–12 mm), font enum, italic/underline,
bounded position offset and optional HTTP(S) URL. No external fonts or URL fetching
happen during load/render. A user action opens a URL through the desktop integration.
Labels retain their component or explicit wire identity; strings do not create nets.
A net appearance edit applies to all current links of that connected group.
The current draft has no durable net-level styling entity: topology edits may
remove a carrier; surviving connected overrides are inherited, while overrides for
deleted carriers are pruned. Undo restores the prior aggregate.

The reader migrates v1–4 notes by assigning IDs; saving v5 makes those IDs durable.
Old envelopes reject new text IDs/appearance data. Existing bounded decoding,
unknown-field rejection and atomic save contracts remain. This is still an
experimental schematic schema, not a finalized electronics-project format.

Selection is runtime presentation state. `MoveItems`, `DeleteItems`, `AddNote` and
`SetText` go through the same candidate validation, revision check and history as
other commands. Group/text drag previews bind session incarnation and revision,
commit once on release, and cancel without mutation. Group motion retains external
pin anchors, repairs incident authored legs and uses existing overlap normalization.
No second document/history authority or new dependencies are introduced.

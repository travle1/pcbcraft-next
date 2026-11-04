# Live schematic MCP

PCBcraft now implements a local MCP adapter directly against its own Electronics
session. SolidCraft is neither a dependency nor a process to launch. This is an
initial, bounded schematic-authoring implementation, not a connected model/chat UI.

## Start and connect

Build **both** executables, then launch the editor:

```sh
cargo build --offline --locked --bins
./target/debug/pcbcraft --demo
```

Open **AI / MCP** in the top bar, choose **Enable local MCP**, then **Copy client
config**. Paste that configuration into your MCP client's server settings. The
configuration uses `pcbcraft-mcp --connect /tmp/pcbcraft-mcp-…/connection.json` and
points to this exact window. It is regenerated after disabling/re-enabling or
restarting the app. No client settings or credentials are modified automatically.
The app may also be launched with `--mcp` to enable its local bridge explicitly.

The bridge is off on normal startup. Each enabled window has a random loopback
port and a 256-bit token in a private temporary directory. The connection file is
mode 0600 inside a 0700 directory; it is removed when the bridge stops. This first
implementation is qualified on Linux and currently requires Unix permissions;
Windows needs its own private-file/ACL qualification. Never share the connection
file or expose the bridge over a network. The internal bridge is not HTTP MCP.

The adapter provides standard newline JSON-RPC **MCP stdio, revision 2025-06-18**.
Initialization negotiates that supported revision. It advertises tools, a reference
resource and a capture prompt. Tested with the installed Python MCP SDK 1.27.1;
this does not qualify every model/provider/client. No headless fallback occurs if
the selected window closes. GUI workers own the session; the transport thread
only queues bounded requests and sends replies.

## Authoring workflow

1. Inspect the project, active occurrence, GUI selection, part catalog and nets.
2. Prepare a batch with the current project/session/revision stamp. Supported
   operations place, edit, move, delete, connect, disconnect, name a net, add a
   note and arrange an explicit set of symbols in ordered rows/columns.
3. Review its summary, operations, affected sheet occurrences, Before/Proposed
   overview and limitations in **AI / MCP**. Preparation does not dirty the project.
4. **Apply** commits once and **Discard** changes nothing. One ordinary **Ctrl+Z**
   restores the entire batch. The camera stays where the user left it; use **F**
   to fit newly added content. Save uses the ordinary project save controls.

External apply is disabled by default. The panel has an explicit option allowing
the connected client to call `apply_edit`. It resets on new/open/reopen or bridge
disable. Turning it off still permits reading an existing receipt. Clients do not
get filesystem, shell, provider credentials or automatic save tools.

All manual and AI edits use the same `Session::edit_candidate`/commit authority.
Proposals belong to the session, not the transport. Whole-project candidate
validation precedes a single history commit. Failed steps leave no live mutation;
IDs allocated inside a proposal remain stable while it is pending. A shared sheet
definition explicitly reports every affected occurrence. Concurrent edits, Undo,
Redo or reopen invalidate the base. Stale previews cannot apply.

`apply_edit` takes a unique request ID and the server-owned handle/base. An
identical retry returns the original receipt even after GUI Undo; it does not redo
the edit. Reusing the ID with different arguments fails. After a timeout, call
`edit_status` before retrying. A timeout can mean an already committed result lost
its reply. Requests that expire while queued are rejected before execution.
GUI Apply uses a `gui-…` receipt ID; an external client sees the new revision and
the removed pending proposal and should inspect the resulting project.

## Tools and bounds

| Tool | Scope |
| --- | --- |
| `pcb_reference` | Versioned capture/verification guidance |
| `inspect_project` | Snapshot, active occurrence, selection, sheets and capabilities |
| `list_parts` | Twelve original generic definitions with exact pin IDs/positions |
| `inspect_sheet` | Paginated symbols, components, wires, notes, ports and junctions |
| `inspect_nets` | Paginated occurrence-qualified terminal connectivity; not ERC |
| `prepare_edit` | Build one validated candidate, without changing the live document |
| `inspect_proposal` | Base/stale state, operation list, created IDs and candidate preview |
| `apply_edit` | One atomic, idempotent commit when external apply is enabled |
| `discard_edit` | Remove a pending proposal |
| `edit_status` | Request receipt, current snapshot and pending proposals |

Copy IDs exactly from inspection. They are opaque 16-byte arrays in the current
domain encoding; terminal IDs are strings. Tools publish strict JSON schemas.
Coordinates use integer nanometres, positive Y down. Guides are available through
`pcb_reference`, `pcbcraft://guides/schematic-capture` and the `schematic_capture`
prompt. See the [bundled guide](agent-guide.md) for operation examples and rules.

- A proposal has 1–64 operations; eight current proposals may be pending.
- This synchronous profile accepts projects with at most 4096 aggregate objects
  and 32768 expanded terminals/link endpoints, including repeated sheets.
  Background preparation for larger projects is future work.
- All 256 successful apply receipts are retained for the runtime session. The next
  AI commit refuses rather than evicting retry protection; save and reopen to reset.
- Request frames are capped at 512 KiB, replies at 4 MiB. Inspection pages contain
  at most 200 items. Proposal preview includes only its first 200 items and reports
  truncation. A complete candidate still validates before commit.
- The queue is bounded to eight requests; the desktop processes one per frame.
  A request has a ten-second dispatch deadline. Active gestures, property dialogs
  and file transitions return Busy rather than competing with the user.

## Placement, routes and remaining work

The initial `arrange` operation lays out explicitly ordered symbols with bounded
column/spacing settings, rejects overlapping reserved symbol areas, and verifies
unchanged connectivity. It never moves unlisted symbols. New `connect` operations
can calculate deterministic orthogonal routes around original-catalog symbol bodies
and unrelated pins. Failure asks for different placement or explicit reviewed bends.
Explicit bends receive structural path validation, not automatic obstacle checking.

This is **not** the full SA1 circuit arranger. Functional-group inference, exact
text/label clearance, congestion/crossing optimization, persisted locks and
rerouting existing networks as a whole remain ahead. The overview reuses the real
symbol painter; legacy automatic links are simplified and labeled as such. It is
not a pixel-identical full-sheet renderer, and text/ports/subsheets are available
through inspection rather than fully drawn in this first review thumbnail.

ERC, manufacturer pinouts/electrical types, external libraries, sheet creation
through MCP, PCB/3D/manufacturing, an in-app model conversation and streaming jobs
remain unavailable. Structural validation is never presented as electrical safety.
The [AI spec](specs/ai-authoring.md) retains those broader acceptance gates.

## Verification

Core tests cover invalid late steps, stable prepared IDs, stale edit/undo/reopen,
one-step Undo/Redo, persistence, repeated sheets, request reuse, arrangement and
pin connectivity. Protocol tests cover initialization, notifications, schemas,
bounded framing, paging and default write gating. Local socket tests cover request
delivery, invalid tokens and cleanup; GUI tests cover actual session edits/history
and compact review presentation. Socket tests need permission to bind localhost.

The Python SDK smoke test exercised real stdio initialization/tool discovery,
validated advertised schemas, read the live desktop/catalog, prepared a connected
sensor/controller pair, inspected its unchanged-live/candidate state, verified the
write gate, and fetched the guide/prompt. Native review captures are local evidence
under ignored `work/proof/mcp-review.png`; no provider/model was launched.

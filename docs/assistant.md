# In-app Assistant

The Assistant now has a bounded conversation and an ACP v1 stdio adapter for an
installed local agent. The adapter is implemented and tested against an original
scripted protocol peer; no real agent, provider account or model has been qualified
in this slice. Compatibility, sign-in and useful model behavior still need a
named agent/version qualification. The external [MCP workflow](mcp.md) remains
available independently.

## Configure and send

Build both binaries with `cargo build --offline --locked --bins`, then open
**Assistant → … → Connect an AI app…**. Under **In-app Assistant**, enter the absolute
path to an installed ACP executable and its arguments, one per line. Use that
agent's documented ACP command; PCBcraft does not install, discover or sign in to
an agent automatically. Choose **Use this agent**, write in **Your request**, then
choose **Send**. Saving the configuration does not launch the process.

Sending runs the selected command with its inherited environment and existing
account/permissions. A local agent may contact a remote model and incur that
provider's charges. PCBcraft does not create an operating-system sandbox or
store credentials. Only configure a command you intend to run. Configuration,
drafts and conversation history are session-local and are not written to `.pc`.
Manual editing needs no agent or account.

Replies stream into the panel while the composer remains available. **Stop**
ends the active turn, retains its partial text and rejects late updates.
**New conversation** in the menu clears the transcript while retaining pending
design proposals. Opening/collapsing Assistant does not cancel a response or
move Properties/filters. New/open/reopen clears the transcript and draft and
cancels the active turn; the chosen command remains configured.

## Review changes

The agent receives the request, bounded visible history and the active sheet,
selection and snapshot. Its tools reach this window's existing Electronics
session. **Conversation / Changes** switches between replies and proposal review;
**Review changes** opens the candidate preview. Apply and Discard remain user
actions. Apply is disabled while the response is active; after completion or Stop,
an unchanged-base proposal can be reviewed and applied once. One Undo restores
the whole committed change. Save/reopen uses the normal project controls.

The in-app bridge always denies agent `apply_edit` and `discard_edit`, even when
the separate external-MCP permission is enabled. Manual edits, Undo/Redo or a new
session incarnation invalidate active work; outdated tool requests cannot prepare
changes against the new revision. Stopped or failed requests may leave already
prepared proposals available for explicit review, but do not apply them.

## Adapter boundary and limits

Each request starts a fresh ACP process/session in an empty temporary directory,
with the bounded visible transcript supplied as JSON context. This first adapter
does not resume provider sessions or retain provider-internal context. It negotiates
protocol 1, passes a private `pcbcraft-app` MCP stdio connection and streams only
matching-session text updates. It ignores reasoning and unsupported content.
The command is executed directly, without shell parsing of the arguments.

Client filesystem, terminal, authentication and elicitation methods are unsupported.
Permission requests are granted once only for exact known `mcp__pcbcraft-app__…`
inspection/preparation tool titles; other requests are rejected or cancelled.
The Electronics session independently enforces the actual tool permissions and
snapshot. ACP permission titles are agent-reported, so this protocol restriction
is not a sandbox for an agent's own native capabilities. The selected executable
remains a trusted program running with the user's OS permissions.

The adapter sends cancellation on Stop, closes the direct agent process, and
revokes its private bridge. Pipe reading/writing and process cleanup run outside
editor frames. Application shutdown waits briefly for cancelled workers. Process
tree containment, crash recovery, native agent discovery and in-app authentication
are not implemented; agent-managed descendants require further qualification.

- Requests are limited to 16 KiB, replies to 64 KiB, and visible history to 24
  messages / 128 KiB. Old user/reply pairs are removed together when needed.
- Event/IO queues and protocol frames are bounded. Startup stages time out after
  30 seconds each, and a response after five minutes.
- Each turn permits at most 64 MCP calls. Existing project, operation, proposal
  and dispatch limits still apply; large synchronous preparations remain bounded.
- Original generic symbols, structural checks and arrangement retain their
  [existing limitations](mcp.md#placement-routes-and-remaining-work). No reply is
  evidence of electrical validation.

## Qualification evidence

Headless tests cover input/history bounds, streaming, Stop, late events, stale
revisions, save/reopen, proposal permissions and shared Undo. Linux tests use
`tests/fixtures/acp_peer.py` with `/usr/bin/python3` (standard library only) to
exercise the real stdio handshake, matching-session updates, permission replies,
cancellation, protocol errors and oversized output. The fixture is original
project test code, not a model or provider integration.

Desktop tests exercise the actual Send/Stop/review controls and a scripted
backend calling the real live-session bridge. Native full/compact captures in
ignored `work/proof/conversation-*.png` use that scripted ACP peer, including a
peer-launched real `pcbcraft-mcp` adapter that prepares two connected resistors
without changing the live revision. They demonstrate application behavior, not
AI design quality or provider qualification.

The explicit QA flag `--capture-conversation config.json` requires `--capture`;
it configures and sends only in that capture run. Its bounded JSON contains
`agent: {executable, arguments}`, `request` and optional `settings`, `review` or
`streaming` booleans. Never use a real agent in automated captures without explicit
authority. Normal launches do not read a capture configuration or start an agent.

Protocol sources: official ACP v1
[initialization](https://agentclientprotocol.com/protocol/v1/initialization),
[session setup](https://agentclientprotocol.com/protocol/v1/session-setup),
[prompt turns](https://agentclientprotocol.com/protocol/v1/prompt-turn) and
[cancellation](https://agentclientprotocol.com/protocol/v1/cancellation).
The [source policy](source-policy.md) applies to implementation and agent context.


## Browser edition

The WebAssembly target shares the Assistant panel but does not include the native
ACP/process or local MCP/socket transports. Copy request supplies the current
sheet document and selection as data for an external AI app; there is no live
connection or automatic proposal return to the tab. Native agent setup is omitted.
See the [browser guide](../web/README.md) for supported workflows and persistence
limits. A future browser connection can reuse the same Electronics operations.

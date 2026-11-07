<h1 align="center">PCBcraft Next (archived)</h1>

<p align="center">
  <strong>Active development has moved to <a href="https://github.com/tangency-labs/pcbcraft">the official PCBcraft project</a>.</strong>
</p>

This repository is archived and is no longer maintained. Visit
[official PCBcraft at tangency-labs/pcbcraft](https://github.com/tangency-labs/pcbcraft)
for current development, documentation and contributions.

The documentation below is retained as a historical record of this independent
prototype.

## Archived prototype documentation

<p align="center">
  <a href="https://travle1.github.io/pcbcraft-next/?demo=1">Try in browser</a> ·
  <a href="#run-it">Run it</a> ·
  <a href="#ai-and-mcp">AI &amp; MCP</a> ·
  <a href="ROADMAP.md">Roadmap</a> ·
  <a href="docs/desktop-guide.md">Editor guide</a> ·
  <a href="docs/framework.md">Architecture</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

PCBcraft is a standalone Rust desktop application in the SolidCraft family,
built around an editable electronics project and a shared command/undo core.
An initial WebAssembly target shares the editor and can be hosted on GitHub Pages;
see the [browser build guide](web/README.md) for local setup and its limits.
The goal is to move naturally between schematic capture, board layout, 3D review
and manufacturing, whether designing by hand or working with an AI agent.

**Early development:** the schematic editor runs today. Live MCP can inspect and propose schematic edits. PCB layout, 3D,
ERC/DRC, simulation and manufacturing are planned, not working features.

![PCBcraft schematic workspace showing the fan-controller example, project browser, grouped ribbon and component properties](docs/screenshots/schematic.png)

*Actual desktop build. The editable fan-controller example uses original generic
symbols; it is an illustrative draft, not an electrically validated reference design.*

## AI and MCP

**AI-assisted design uses the same live Electronics session as manual tools.**
External MCP clients can inspect sheets, components, pins and nets, then prepare
connected edits for review in the app. Apply commits the entire proposal with
one-step undo, revision checks and protection against duplicated retries.

The workflow is **inspect → propose → preview → apply → undo**. The **Assistant**
panel provides Before/Proposed views, operation details, Apply/Discard and tool
activity in a separate diagnostics view. It shows actual part/connection changes
and keeps Apply/Discard visible while reviewing. A request draft includes the
current sheet and selection. A configured ACP agent enables **Send**, streamed
replies, Stop and conversation history. The adapter has scripted-peer qualification;
real agents/providers remain to be qualified. Without an in-app agent,
**Copy request** copies a brief to paste into your AI app.
External apply requires an explicit setting; review in the app is the
default. The initial tools also offer ordered grid arrangement and bounded
orthogonal routes. They do not provide full automatic layout or electrical checking.
Connected selections also have a bounded grid profile with fixed symbols, region
limits and internal pin-to-pin rerouting; see [its limits](docs/mcp.md#placement-routes-and-remaining-work).

Build both executables with `cargo build --locked --bins`, expand **Assistant**
at the bottom right, choose **… → Connect an AI app… → External MCP app**, then
**Enable local connection** and **Copy configuration** into your MCP client.
The stdio adapter connects to that exact window; it does not edit a second copy
of the design. See [MCP setup, tools and limits](docs/mcp.md).

**Available now:** a Linux-qualified MCP stdio bridge, ten tools, bundled guidance,
atomic schematic proposals, live review and an initial ACP conversation adapter.
See [Assistant setup and qualification limits](docs/assistant.md).
**Still ahead:** real agent/provider qualification,
full circuit arrangement, external part libraries and ERC. The original generic
symbols are not verified manufacturer pinouts. No AI account is needed for manual editing.

## What works today

- **Schematic editing:** place, move, rotate and inspect twelve built-in symbol
  types with repeat placement; select groups; size text boxes and edit styled
  notes, references, values and web links.
- **Editable wiring:** start from pins, wires or empty space; add bends and
  branches; drag segments/corners; delete connections and heal compatible overlaps.
- **Sheets and hierarchy:** flat sheets, reusable subsheets, interface ports,
  nested navigation, page guides and occurrence-qualified connectivity.
- **Net inspection:** highlight connected terminals, rename nets, review connection
  removal and copy a JSON connectivity report.
- **Native projects:** one project-wide history and experimental `.pc` save/open,
  including all sheets. Built-in symbols work offline.
- **Desktop workspace:** grouped ribbon, command search, floating/dockable Parts
  library and Properties panels, and a SolidCraft-derived dark visual foundation.

<details>
<summary><strong>Watch the desktop tour</strong> — schematic, Parts library and command search</summary>

![Animated tour of three real PCBcraft desktop states: schematic overview, floating Parts library and command-search palette](docs/screenshots/desktop-tour.gif)

A 12-second loop assembled from three still captures of the running app, four
seconds per state. It shows implemented UI, not a continuous interaction recording
or an AI-generated circuit. Static views: [schematic](docs/screenshots/schematic.png),
[Parts library](docs/screenshots/library.png), [command search](docs/screenshots/commands.png).
[Capture details and reproduction](docs/screenshots/README.md).

</details>

**Still ahead:** real agent/provider qualification, full automatic arrangement, ERC/DRC, PCB layout and cross-probing UI,
3D, simulation, external libraries, KiCad interchange and manufacturing outputs.
Unavailable workspace/tools are marked accordingly. `.pc` is provisional;
autosave and crash recovery are not implemented. See the detailed
[status and limits](docs/implementation-status.md) before relying on saved drafts.

## Roadmap

The next priority is AI-assisted schematic capture alongside editing reliability:
broader AI tools, in-app conversation and more capable readable arrangement.
Atomic proposals, live MCP and a first review panel now work. Libraries,
ERC, the first board/DRC/cross-probing workflow, SolidCore-backed 3D, KiCad
interchange and manufacturing build on that foundation. UI quality and recovery
are part of each milestone.

The [product roadmap](ROADMAP.md) gives the working baseline, priorities and
acceptance gates without promising dates. Detailed tasks remain in the
[implementation plan](docs/implementation-plan.md).

## Run it

From a checkout, with **Rust 1.95+** and a desktop graphics environment:

```sh
cargo run --locked -- --demo
```

Omit `--demo` for an empty project, or open a saved project:

```sh
cargo run --locked -- path/to/project.pc
```

After dependencies are fetched, add `--offline`. The built executable is
`target/debug/pcbcraft`. The current build has been checked on Linux; packaged
releases and multi-platform qualification are still ahead. SolidCraft does not
need to be installed or checked out.

| Action | Control |
| --- | --- |
| Place a symbol / draw a wire / add text | **A** / **W** / **T** |
| Select / rotate / cancel | **V** / **R** / **Esc** |
| Box-select | Left-drag empty space or right-drag; Shift adds, Ctrl toggles |
| Context menu | Stationary right-click |
| Fit / zoom / pan | **F** / scroll / middle-drag |
| Command search | **S** |
| Undo / redo | **Ctrl+Z** / **Ctrl+Shift+Z** |
| Save / open | **Ctrl+S** / **Ctrl+O** |
| New flat sheet / hierarchical sheet | **Manage → Sheets → New sheet / Subsheet** |

Shortcuts defer to focused text fields. The [editor guide](docs/desktop-guide.md)
covers placement, panels and properties, with detailed guides for
[wire editing](docs/wire-editing.md), [selection/text](docs/selection-and-text.md),
[nets](docs/net-workflows.md) and [sheet workflows](docs/sheet-workflows.md).

## Development

The standalone egui app owns its Electronics model, session and native files.
It reuses qualified SolidCraft-family artwork/control patterns and targets
independent SolidCore geometry/rendering libraries as those features are added.
It has no dependency on the SolidCraft application, mechanical document engine
or Truck. See [architecture](docs/framework.md), the
[reuse register](docs/upstream-reuse-requests.md) and [standalone decision](docs/adr/0001-standalone-application.md).

Read [AGENTS.md](AGENTS.md), [requirements](docs/requirements.md),
[approved sources](docs/sources.md) and [development flow](docs/development-flow.md)
before contributing. The [implementation plan](docs/implementation-plan.md) and
[AI authoring tasks](docs/specs/ai-authoring.md#next-bounded-implementation-sequence-and-acceptance)
describe bounded next steps. An explicit [Pages workflow](.github/workflows/pages.yml)
builds/publishes the browser target when explicitly run or a preview tag is pushed; there is no automatic
validation CI or agent/PR queue.

```sh
cargo fmt --all -- --check
cargo check --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
cargo test --offline --locked
cargo test --offline --locked --no-default-features
```

Tests cover editing, identity/connectivity, history, persistence and egui
interaction. The no-default-features run checks the core without desktop dependencies.

## Contributing and community

Read [CONTRIBUTING.md](CONTRIBUTING.md) for setup, focused PRs, validation and source
rules. [Issues](https://github.com/travle1/pcbcraft-next/issues) are the place for
reproducible bugs, questions and workflow proposals; [Support](SUPPORT.md) explains
what to include. The maintainer contact is [@travle1](https://github.com/travle1).

Participation follows the [Code of Conduct](CODE_OF_CONDUCT.md). Sensitive reports
use its private-follow-up process or the [security policy](SECURITY.md).
[Attribution](ATTRIBUTION.md) covers source/asset notices and truthful human/AI
commit credit, including tool/model co-authors and review-only assistance.

## License

PCBcraft is starting over with an independent implementation in this repository.
The preserved GPL fork remains separate and is not being relicensed or ported.
Contributors and agents must follow the
[independent implementation and source policy](docs/source-policy.md), which
excludes KiCad implementation sources and the old fork's code and derived material.

Original contributions are available under [MIT](LICENSE-MIT) OR
[Apache-2.0](LICENSE-APACHE). Reused family artwork, controls and font retain their
[licenses and provenance](assets/family/README.md). KiCad support is planned as an
independent implementation from published specifications and black-box tests;
see the [source policy](AGENTS.md).

# Contributing to PCBcraft

Help with reproducible bugs, usability, tests, documentation and focused features
is welcome. Start with the [roadmap](ROADMAP.md) and
[implemented behavior](docs/implementation-status.md), then read [AGENTS.md](AGENTS.md)
and the relevant [framework spec](docs/framework.md). Participation follows the
[Code of Conduct](CODE_OF_CONDUCT.md).

## Get running

Use Rust 1.95+ and a desktop graphics environment:

```sh
git clone https://github.com/travle1/pcbcraft-next.git
cd pcbcraft-next
cargo run --locked -- --demo
```

Omit `--demo` for an empty project. See the [editor guide](docs/desktop-guide.md).
The core also builds/tests without the default desktop feature. Linux is the
currently checked desktop platform; do not infer other platform qualification.
No sibling SolidCraft checkout or installed SolidCraft application is required.

## Pick a bounded change

Search [issues](https://github.com/travle1/pcbcraft-next/issues) and
[open PRs](https://github.com/travle1/pcbcraft-next/pulls) for overlapping work.
For larger features or shared format/API changes, describe the intended behavior,
allowed files, dependencies and acceptance fixture before substantial implementation.
Small corrections can go straight to a focused PR. Coordinate scope with
[@travle1](https://github.com/travle1); there is no `/claim` bot or automatic queue.

Base independent work on `main`, or explicitly name a prerequisite branch/commit.
Target this repository, not a Tangency Labs repository. Use an isolated branch or
worktree per implementing contributor. Settle ownership of shared interfaces before
parallel tasks; keep each commit reviewable. Maintainer review decides what merges.
A new dependency, migration or broad UI change should explain its necessity and
compatibility impact, rather than arrive as an unrelated refactor.

## Code map and design boundaries

| Area | Current files |
| --- | --- |
| Domain and used definitions | `src/document.rs`, `src/project.rs`, `src/definitions.rs` |
| Editing and history | `src/session.rs`, `src/wiring.rs`, `src/selection.rs`, `src/text.rs` |
| Connectivity and future layout identities | `src/netlist.rs`, `src/probe.rs` |
| Native project IO | `src/persistence.rs` |
| Desktop and shared visual controls | `src/main.rs`, `src/desktop/` |
| Original example | `src/demo.rs` |

One Electronics session owns project edits, revisions, undo and saves. GUI and
future MCP/CLI use that authority; views and transports do not own separate copies.
Pure electrical edits must not invoke a mechanical document engine. Reuse qualified
family libraries and SolidCore capabilities; no Truck or whole SolidCraft app dependency.

For visible changes, use [theme roles](docs/theme-foundation.md), shared controls
and the [UI audit](docs/ui-audit.md). Provide actual before/after captures, including
compact, selected and unavailable states affected by the change. Concept art and
simulated success are not evidence that a feature runs.

## Sources, licensing and AI assistance

Original contributions use MIT OR Apache-2.0. Keep source/asset license notices and
record exact upstream revision, scope and provenance for reuse. Do not import the
preserved GPL fork, its source-derived notes/fixtures, or another GPL EDA
implementation as a reference. KiCad support uses published format specifications,
manuals, appropriately licensed data and original black-box tests only. Do not
inspect/copy/adapt/translate KiCad implementation, headers, patches or annotated
source/Doxygen implementation pages. See [approved sources](docs/sources.md).

AI-assisted contributions are welcome and need the same review and evidence as
other changes. The submitting contributor owns the result: inspect the diff,
validate claims and preserve provenance. Follow [commit attribution](ATTRIBUTION.md#commit-credit)
for the actual tools/models and roles involved; unknown model versions stay unknown.
Agent work uses the [task brief](docs/development-flow.md#task-brief), with an explicit
branch, allowed files, acceptance conditions and authority for any push/PR. A task
list does not authorize agents to publish, message others or launch more agents.

## Validate and submit

After fetching dependencies, the standard checks are:

```sh
cargo fmt --all -- --check
cargo check --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
cargo test --offline --locked
cargo test --offline --locked --no-default-features
```

Scale validation to the change: documentation-only work needs link, example and
media checks; behavior changes need the relevant core/interaction tests and normal
build checks. Add regression cases for meaningful behavior, including failure/undo
and stale state where relevant. Do not claim a skipped check passed. There is no
hosted CI workflow in this checkout, so record actual local commands/results.

A PR should explain the problem and resulting behavior, its scope/prerequisites,
validation and limitations. Include schema/migration changes, visual evidence and
new source/asset provenance when relevant. Update status/roadmap for capabilities
that land. Use [the PR template](.github/PULL_REQUEST_TEMPLATE.md); the longer
[development flow](docs/development-flow.md) covers integration and conflict handling.

For help use [Support](SUPPORT.md). Sensitive findings belong in
[Security](SECURITY.md) or the [conduct reporting process](CODE_OF_CONDUCT.md#reporting-and-response).

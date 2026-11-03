# Reference experiment: local SolidCraft baseline

SolidCraft desktop qualified November 3, 2026; separate upstream CLI error recorded.
**Scope corrected by the user:** PCBcraft is a standalone app. This experiment
built and launched SolidCraft, not PCBcraft. It does not complete the revised W0
standalone gate or make this checkout a build/runtime dependency. The earlier
compiled-in workspace/Session plan is superseded by the
[standalone decision](adr/0001-standalone-application.md). [Implementation plan](implementation-plan.md),
[historical PR assessment](migrated-host-assessment.md).

## Checkout and selected build

The separate upstream checkout was cloned for this task at
`/home/travisl/projects/PCBcraft/integrations/solidcraft`, then detached at
`c553365a2197a689e749389a6fbf707a38df9b61`. Origin is
`https://github.com/tangency-labs/solidcraft.git`. It is a shallow, filtered clone;
do not mistake its abbreviated local history for the complete upstream history.
The active original project remains `pcbcraft-next/`, with no remote configured.
The preserved GPL fork and FigCraft designs are unchanged.

GitHub reports #11351 merged at `2026-11-02T23:20:07Z`, merge commit
`ea772a0438b145d7d876eb724b744060fc8c59b6`; #11360 merged at
`2026-11-03T04:27:17Z`, merge commit
`2b921df189fc41e67edd485253c604a4672a8543`. The selected main snapshot is later
than both. The earlier open-PR assessment is historical, not the current baseline.

| Selected item | Verified value |
| --- | --- |
| Host | SolidCraft `0.40.0-rc.1`, pinned commit above |
| Platform | `x86_64-unknown-linux-gnu` |
| Desktop features | Defaults: `raytrace-cpu`, `bundled-render-library`, `profile` |
| UI | egui/eframe 0.36, wgpu 30 |
| SolidCore geometry | `solidcore` 1.0.1, Apache-2.0, revision `8ddce901671f1cebd9214a31d21895d725e886c8` |
| Shared record | `solidcore-record` 0.1.0, Apache-2.0, revision `f5e8e9a29e0c9b7cf6c4040a5bc8878382b3c62f` |
| Kernel wrapper | `solidcraft-smlib`, MIT OR Apache-2.0; pure Rust dependencies, no native feature or build script at this commit |
| Toolchain | rustc/cargo 1.98.1; build profile `ci`, eight jobs |

The two SolidCore packages have different explicit pins upstream; they must not
be silently unified. The record crate now exists and is consumed by the host.
This does not establish availability of every planned shared-layer API or its
suitability as the electrical domain model.

## Qualification evidence

Evidence is saved under `../../setup/2026-11-03/` relative to this document.
The initial offline metadata attempt found the new pinned record dependency
uncached. An authorized `cargo fetch --locked --target
x86_64-unknown-linux-gnu` completed; subsequent resolution/build use `--offline`.
Build scripts were inspected before compilation. No manifest, lockfile, dependency
pin, toolchain or security setting was changed.

| Gate | Result |
| --- | --- |
| Pinned separate checkout | Complete |
| Locked dependency fetch | Complete; `cargo-fetch.log` |
| Offline locked Cargo metadata | Passed; `cargo-metadata.json` |
| Selected default desktop tree | Passed; `desktop-dependencies.txt`, `dependency-summary.json`; no Truck package or SMLib native feature |
| Combined desktop and CLI build | Failed in CLI, E0432; `build.log`. CLI imports a removed `geom::brep::Brep` export |
| Default desktop-only build | Passed; `desktop-build.log` |
| Desktop startup/render inspection | Passed on NVIDIA GeForce RTX 4070 Ti/Vulkan, GPU renderer; `startup-result.json`, `app-startup.log`, `desktop.png`; clean exit 0 |
| Electronics integration | Not implemented; source map below |

Commands run from the separate checkout:

```sh
cargo metadata --offline --locked --format-version 1 --filter-platform x86_64-unknown-linux-gnu
cargo tree --offline --locked -p solidcraft --target x86_64-unknown-linux-gnu -e normal,build,features
cargo build --offline --locked --profile ci -p solidcraft -p solidcraft-cli -j 8
# After the CLI failure, qualify the required desktop target independently:
cargo build --offline --locked --profile ci -p solidcraft -j 8
```

The tree establishes Cargo dependency selection, not successful rendering or
every geometry capability. No requirement to wait for every upstream kernel
test is introduced. No broad upstream test suite was run for this unchanged
baseline; compile/startup and task-relevant behavior are separate gates.

The startup harness, `../../setup/2026-11-03/startup-smoke.py`, launches only this
pinned binary with `--sample` and a random localhost control port/token. It uses
separate settings/data/cache/recovery folders and disables background update
checks for this run; no normal user settings are changed. The sandbox initially
blocked local sockets before launch; the authorized unsandboxed probe succeeded.
`ui.inspect`, `ui.screenshot` and `app.quit` all returned `ok: true`.
The [1600 × 1000 screenshot](../../setup/2026-11-03/desktop.png) was visually
inspected: sample plate, document tab, ribbon, browser, themed icons, navigation
and history are rendered. This is the upstream mechanical sample, not a PCB UI or
evidence of PCB geometry/DRC coverage. Startup output reports GPU tessellation.

The CLI failure is specifically
`apps/solidcraft-cli/src/kernel_cases_export.rs:22`: unresolved import
`solidcraft_engine::geom::brep::Brep`. At this pin, `crates/geom/src/brep.rs` uses
record types from `solidcore_record` and does not export `Brep`. No local patch
was made. This blocks the upstream CLI/MCP qualification at this revision. PCBcraft will
own its CLI/MCP adapters; this upstream error is not a PCBcraft implementation
gate. Do not silently advance a pinned reference to hide the result.

## Reference map for standalone reuse

All source paths below are relative to this pinned reference checkout. Existing
app internals are evidence to assess, not APIs PCBcraft can automatically depend on.

| Concern | Actual source/API | Standalone implication |
| --- | --- | --- |
| Window/app | `apps/solidcraft/src/main.rs`, `crates/ui-egui/src/lib.rs`: `SolveApp`, `run`, `start` | PCBcraft owns a separate executable and app composition; do not extend or launch this app |
| Tabs/workspace | `documents.rs::{Documents,DocSlot}`, `workspace.rs::{TABS,SECTIONS,SPECIALTY_TABS}` | App-specific structures hold mechanical sessions/commands. Reuse independently extracted presentation utilities where useful, not these ownership assumptions |
| Theme/icons/widgets | `theme.rs`, `icons.rs::paint_icon`, `ui_kit/mod.rs`, `assets/icons` | Some exports are public, many crate-private. Whole ui-egui pulls in engine; obtain focused independent libraries/resource adapters with provenance |
| Domain/history | `crates/doc/src/document.rs::Document`, `crates/engine/src/lib.rs::{Session,Snapshot}` | PCBcraft owns its Electronics aggregate/session/history; generic utilities only if independently reusable |
| Commands/refresh | `Session::execute`, `crates/engine/src/cmd/mod.rs`, `SolveApp::run/start` | Mechanical registry/refresh is not PCBcraft's command authority. Original EDA operations are shared by our GUI/CLI/MCP |
| Revision/lifecycle | `Session::{uid,document_id,revision}`, `invalidate_view` | Reference behavior; define and test PCBcraft incarnation, revision, idempotency and undo contracts directly |
| Persistence | `crates/io/src/design.rs`, `design/drawing.rs::{FileKind,file_kind}`, `safe_write.rs` | Container/atomic-write code may be extracted independently. PCBcraft owns .pc type/schema/migrations and need not change SolidCraft FileKind |
| Viewport | `viewport.rs`, `render_workspace.rs`, `viewport_layout.rs` | Qualify independent renderer/camera utilities; no app-specific mechanical picking or Sel type as electrical identity |

## Corrected next slice

W0 establishes PCBcraft's own desktop/library boundary; F1 defines its Electronics
session contracts. W1 composes the user's Schematic/Layout/Render views in our
window, using independently reusable family components. H0 proves the editable
component, undo/reopen and operation parity there. No SolidCraft app build,
engine extension or upstream CLI fix is a prerequisite. The
[reuse register](upstream-reuse-requests.md) records bounded extraction needs;
no upstream requests have been sent.

This task changes notes and local qualification artifacts only. It does not add
dependencies to PCBcraft, edit host production code, launch implementing agents,
create commits, push or configure a PR queue.

At completion of the original experiment, documentation checks passed and the
starter source/manifests/lockfile/licenses matched their pre-task hashes. Later
standalone corrections changed guidance, Cargo description and module comments,
not implementation, dependency declarations or licenses. The upstream checkout
was left unchanged. Validation details:
`../../setup/2026-11-03/documentation-checks.json`.

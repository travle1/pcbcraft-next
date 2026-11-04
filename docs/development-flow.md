# Agent development flow

The standalone repository is [travle1/pcbcraft-next](https://github.com/travle1/pcbcraft-next),
created private on November 3, 2026; a read-only GitHub check later that day reports
it public, with `main` as the default branch and Issues enabled. `origin` targets
that repository; local feature-branch work is not automatically published. An explicit Pages publication workflow is present for the browser target, but no
automatic validation CI, agent launcher or PR queue is configured. Local builds
do not authorize running that workflow or changing hosted settings. Use a local ledger or existing Codex tasks; do not assume
upstream bots operate here. Do not reuse the preserved fork's remote. Start fresh
task contexts from this project's approved sources.

The user authorized the initial commits and push. Work was grouped as architecture
and licensing, headless Electronics core, qualified UI assets/controls, desktop
workspace, and usage/verification/repository guidance. The core snapshot passed
22 tests; the exact staged desktop snapshot passed 59 tests and Clippy. Subsequent
work should be committed in bounded functional sections as it is completed, with
validation recorded, rather than accumulating another untracked application.
Build output and local proof captures remain ignored.

The [product roadmap](../ROADMAP.md) summarizes priorities, while
[CONTRIBUTING.md](../CONTRIBUTING.md) is the entry point for contributors.
[Code of Conduct](../CODE_OF_CONDUCT.md), [Support](../SUPPORT.md) and
[Security](../SECURITY.md) explain participation and reporting through `@travle1`.

## Commit attribution

Follow [ATTRIBUTION.md](../ATTRIBUTION.md#commit-credit). Keep the existing human Git
identity and add Co-authored-by trailers for actual AI authors/editors; record
review-only help with AI-reviewed-by. Use verified tool/model information, with
generic tool names when the model is unknown. Preserve actual participants when
squashing and never infer a past contributor from a planned task. These are manual
conventions; no hook or bot has been installed. License/source provenance is separate.

## Starter backlog

Use the canonical [implementation plan](implementation-plan.md) for task IDs,
dependency DAG, allowed areas and gates. Standalone launch, schematic editing,
whole-project save and flat/hierarchical sheets now work. The full W1/H0 three-view
workflow, broader library contracts and jobs remain open. AI0 atomic proposals and the first live MCP/review slice now work; the next AI
work expands arrangement, provider conversation and qualification. See [MCP](mcp.md). The earlier SolidCraft
startup remains reference evidence only; no SolidCraft app/CLI prerequisite or
agent launch is implied.

Current [implementation status](implementation-status.md) records the first runnable
schematic subset and remaining gates. Validate it with the commands in README
before building the next slice.

## Dispatch and review

Every implementation brief and handoff must carry the
[source policy](source-policy.md). This is an independent restart, not a relicensing
or port of the preserved GPL fork. Do not pass excluded KiCad/fork implementation,
derived notes/fixtures or rewritten excerpts to an agent as context. Review source
provenance and notices as well as behavior; pause affected implementation and tell
the user if excluded material is encountered. Existing access, shared branding or
an AI attribution trailer does not authorize reuse.

Each task needs one outcome, owner, allowed files, contract and acceptance fixture.
Settle shared interfaces before dependent workers modify callers. Use one isolated
branch/worktree or clone per implementing agent, recording prerequisite SHAs.
Avoid overlapping edits to manifests, schemas and registries. Coordinate build
resources instead of running several full checks on the same checkout.

Check formatting, build/Clippy and relevant behavior tests. Add hosted CI only
when a new destination and setup are authorized. Review actual diffs, provenance,
acceptance evidence and rendered output when behavior is visible. Merge prerequisites
before dependents by default and test the combined result. Resolve conflicts
without discarding another worker's changes.

Each brief must explicitly authorize commits, a named branch push and draft PR
destination. Use this standalone repository for authorized pushes and PRs. Upstream work
requires current target-repository rules and separate authority. A local ledger
is not an installed queue, and a draft PR is not a completed merge.

## Task brief

```text
Task ID and outcome:
Repository: travle1/pcbcraft-next (origin)
Base SHA, branch and isolated checkout:
Prerequisite IDs/merged SHAs:
Read: AGENTS.md, source-policy.md, framework.md, task-linked specs, sources.md and target rules.
Allowed files / shared-interface owner:
Contract, compatibility and acceptance fixture:
Non-goals and unavailable APIs:

Build the standalone PCBcraft app, not a plugin/workspace inside SolidCraft.
It owns its executable/settings, native projects and Electronics session.
All views/GUI/CLI/MCP share one edit/history/save authority. Reuse actual independent
family UI/icon/theme/IO/job libraries and SolidCore through explicit adapters.
Audit selected dependency closure; no full SolidCraft app/mechanical engine,
sibling reference-checkout path dependency, Truck or invented SDK. Private
components need narrow licensed extraction/adapters with provenance and a
replacement plan. Preserve the user's FigCraft design and maximize real reuse.
Only selected library/capability gates apply, not whole-host migration or tests.
No initial cross-document ECAD/MCAD transactions. This is an independent restart,
not relicensing or porting the GPL fork. Do not inspect or reuse the preserved GPL
fork or source-derived notes/fixtures, including translations/AI rewrites or
delegated source inspection. KiCad support uses published specs/manuals
and black-box tests only; no implementation source, headers, patches or annotated
source/Doxygen pages. Record licenses and notices for reusable code/data.

Authority: local edits only unless exact commit/push/draft-PR destinations and
permissions are provided. No merge, upstream publication, dependency/config change
or unrelated message by implication.
Checks and evidence:
Attribution: actual authoring/review tool and model; mark unknown versions honestly.
Reviewer/integration owner:
Handoff: changes, checks/limits, exact head, PR state if authorized, blockers.
```

# Agent development flow

The standalone repository is [travle1/pcbcraft-next](https://github.com/travle1/pcbcraft-next),
created private on November 3, 2026. `origin` targets that repository and `main`
contains the initial sectional history. No hosted CI, agent launcher or automatic
PR queue is configured. Use a local ledger or existing Codex tasks; do not assume
upstream bots operate here. Do not reuse the preserved fork's remote. Start fresh
task contexts from this project's approved sources.

The user authorized the initial commits and push. Work was grouped as architecture
and licensing, headless Electronics core, qualified UI assets/controls, desktop
workspace, and usage/verification/repository guidance. The core snapshot passed
22 tests; the exact staged desktop snapshot passed 59 tests and Clippy. Subsequent
work should be committed in bounded functional sections as it is completed, with
validation recorded, rather than accumulating another untracked application.
Build output and local proof captures remain ignored.

## Starter backlog

Use the canonical [implementation plan](implementation-plan.md) for task IDs,
dependency DAG, allowed areas and gates. W0 now establishes PCBcraft's standalone
build/library boundary; the earlier full SolidCraft startup is reference evidence
only and does not complete W0. F1 defines Electronics contracts. W1 builds our
window/three views; D0/F0/J0 provide session edits, native persistence/offline
definitions and jobs. H0 proves the complete edit/undo/save/reopen path inside
PCBcraft. No SolidCraft app/CLI prerequisite or agent launch is implied.

Current [implementation status](implementation-status.md) records the first runnable
schematic subset and remaining gates. Validate it with the commands in README
before building the next slice.

## Dispatch and review

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
Repository: travle1/pcbcraft-next (origin; private at initial creation)
Base SHA, branch and isolated checkout:
Prerequisite IDs/merged SHAs:
Read: AGENTS.md, framework.md, task-linked specs, sources.md and target rules.
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
No initial cross-document ECAD/MCAD transactions. Do not use the preserved GPL
fork or source-derived notes/fixtures. KiCad support uses published specs/manuals
and black-box tests only; no implementation source, headers, patches or annotated
source/Doxygen pages. Record licenses and notices for reusable code/data.

Authority: local edits only unless exact commit/push/draft-PR destinations and
permissions are provided. No merge, upstream publication, dependency/config change
or unrelated message by implication.
Checks and evidence:
Reviewer/integration owner:
Handoff: changes, checks/limits, exact head, PR state if authorized, blockers.
```

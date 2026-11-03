# Historical migrated host assessment: PR #11360

**Historical evidence only.** PRs #11351/#11360 subsequently merged and the
[reference experiment](host-baseline.md) built main c553365. The user has since
clarified standalone PCBcraft deployment. Earlier advice to extend the host or
its Session is superseded by the [accepted decision](adr/0001-standalone-application.md).
Do not use this document as current implementation instructions. Open/unmerged
statuses, pins, optional C++ features and CI observations below describe their
stated historical snapshot.

Evidence snapshot: November 2, 2026, approximately 22:25 UTC. Route decision is
amended with Astra; local qualification pending. No checkout/build or app implementation occurred.
[Framework](framework.md), [implementation plan](implementation-plan.md).

## Status and dependency order

| Work | Inspected state | Revision / relationship |
| --- | --- | --- |
| [SolidCraft #11360](https://github.com/tangency-labs/solidcraft/pull/11360) | Open, non-draft, unmerged | Head `7ad1d3e12540fee1ded8391283567ab7fa24f588`; base branch `jyviko/merge-main-develop`, SHA `e6a4b0a9ae46386e7b0de7a2da50dbd8ebc35582` |
| [SolidCraft #11351](https://github.com/tangency-labs/solidcraft/pull/11351) | Open, non-draft, unmerged | Head is #11360's base above; base `main` at `59ffd380190c07b31b6b446eb08d179718a6575a`; merges develop/SMLib work into main |
| [SolidCraft #8526](https://github.com/tangency-labs/solidcraft/pull/8526) | Open draft; explicitly not for merging | Head `e30cfd2acf1de890677037cef2b41347e10a3ecf`; operation-by-operation SolidCore/default feature work incorporated into #11360 |

#11360's stated order is merge after #11351, then retarget its base to main. Do
not treat an active PR or a synthetic merge SHA as proof it is merged/released.
Pin exact revisions for qualification and recheck status before implementation.

## Source evidence beyond the title

At #11360 head the actual chain is:

```text
ui-egui → engine → kernel → solidcraft-smlib → solidcore
                                              rev 2dbd88b6e9b858144609690eb2860e0beba27e54
```

The wrapper crate name `solidcraft-smlib` is not proof of C++ linkage. Its `native`
feature is off by default; `kernel/smlib → solidcraft-smlib/native` enables C++.
Desktop defaults are raytrace-cpu, bundled-render-library and profile; CLI defaults
are raytrace-cpu/profile; web defaults to profile. None includes smlib. GPU does
not force it. The wrapper build script returns before C++ build/link work when
`CARGO_FEATURE_NATIVE` is absent. Runtime routing defaults to SolidCore, has no
automatic other-kernel fallback, and refuses C++ selection when not built.

Read-only inspection covered all 22 non-vendored workspace Cargo manifests and
the complete Cargo.lock at this head: no Truck declaration/name was found in
those manifests or lockfile. The root pins SolidCore at the revision above.
This is strong source/lockfile evidence that the old main-branch Truck chain
does not apply to this PR head. It is not a locally resolved Cargo graph, linker
inspection, successful desktop launch or proof every required operation works.

Pinned primary files: [workspace](https://github.com/tangency-labs/solidcraft/blob/7ad1d3e12540fee1ded8391283567ab7fa24f588/Cargo.toml),
[lockfile](https://github.com/tangency-labs/solidcraft/blob/7ad1d3e12540fee1ded8391283567ab7fa24f588/Cargo.lock),
[kernel](https://github.com/tangency-labs/solidcraft/blob/7ad1d3e12540fee1ded8391283567ab7fa24f588/crates/kernel/Cargo.toml),
[wrapper](https://github.com/tangency-labs/solidcraft/blob/7ad1d3e12540fee1ded8391283567ab7fa24f588/crates/smlib/Cargo.toml),
[desktop](https://github.com/tangency-labs/solidcraft/blob/7ad1d3e12540fee1ded8391283567ab7fa24f588/apps/solidcraft/Cargo.toml),
[build guard](https://github.com/tangency-labs/solidcraft/blob/7ad1d3e12540fee1ded8391283567ab7fa24f588/crates/smlib/build.rs),
[routing](https://github.com/tangency-labs/solidcraft/blob/7ad1d3e12540fee1ded8391283567ab7fa24f588/crates/smlib/src/route.rs).
Kernel source contains a stale introductory Truck comment despite its changed
manifest; dependency declarations and selected graph take precedence over that prose.

## Checks and remaining qualification

At the inspected head, PR checks and Native Linux Clippy had successful checks,
and Fillet rig reported success. Native Linux tests were still running, and some
repository/web checks were cancelled/skipped: this is not a clean complete CI run.
The [kernel-rig build/gate job](https://github.com/tangency-labs/solidcraft/actions/runs/41534275357/job/125582324814)
reported failure. Astra's separate exact-head log review establishes the important
nuance: release CLI compilation succeeded; the subsequent "Every command has a
case" coverage inventory failed with 84 uncovered entries, so actual rig shards
were skipped. This red check is not evidence that kernel geometry failed. The
Fillet rig built SolidCore alone and passed its known-failing-list gate. Linux
Clippy succeeded; native tests were running; repository checks were cancelled at
web-offline tests. These are time-specific observations, not a promised final CI
outcome. This local investigator verified check statuses; the failure-cause detail
comes from Astra's log review, since annotation endpoints were unavailable here.

The [PR discussion](https://github.com/tangency-labs/solidcraft/pull/11360#issuecomment-6589338806)
also identifies unlanded later pin work, conversion/typed-error gaps and known
cases. Those notes describe future work, not changes included at the pinned head.
Do not substitute #8526's reported tests for #11360 head qualification.

Direct pinned SolidCore geometry crate use and SolidCraft UI/service reuse are
independent concerns. Do not gate domain/library/base workspace work on complete
migration or all upstream kernel tests; only selected dependencies and required
capabilities gate a slice.

The recommendation at that snapshot was to qualify the migrated application
and extend its Session, replacing an earlier extraction-first plan. That was
based on an incorrect product assumption. The user explicitly requires a
standalone app; neither old route is an active instruction. Preserve the source
and CI findings above when assessing independently reusable libraries.

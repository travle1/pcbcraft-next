# Independent implementation and source policy

PCBcraft is starting over in `pcbcraft-next/` with independently written
implementation code. Original contributions use MIT OR Apache-2.0. Continuing the
PCBcraft name or product goals does not make the preserved GPL fork part of this
implementation. This is a restart, not a relicensing, port or cleanup of that fork.
The old checkout, license, local edits and Git history remain separate.

This document defines the project's clean-room working rules for contributors and
agents. It does not certify complete historical isolation: the
[source register](sources.md#original-starter-sources-and-policy) records earlier
review of the preserved fork before the restart. Preserve that record without
treating it as permission to consult or reuse the fork now.

## Excluded implementation sources

Do not inspect, copy, adapt, translate, port or use as an implementation reference:

- KiCad implementation source, headers, patches, tests or source-derived fixtures.
  This includes mirrors, forks, code-search excerpts and annotated-source/Doxygen
  implementation pages, even when reached from otherwise approved documentation.
- The preserved `../pcbcraft/` GPL fork, its source-derived notes, fixtures or
  historical implementation reviews. Parent-workspace archives are not approved
  implementation specifications for the restart.
- Another GPL/LGPL/AGPL EDA implementation or summaries, pseudocode and generated
  code derived from an excluded implementation.

Do not fetch an excluded source to compare algorithms, naming, structures or
similarity. Rewriting it in Rust, renaming identifiers, asking an AI to rewrite it,
or removing its copyright/license headers does not make it an approved source.
Do not merge, cherry-pick, transplant files or replay patches from the preserved
fork. Do not replace or remove its GPL notices or describe its code as relicensed.

These restrictions apply to searches, tools, prompts, attachments, task handoffs
and any delegated work. An agent must not have another agent inspect an excluded
source on its behalf. Access to a local checkout, public repository or past
conversation does not authorize its use.

## Approved implementation inputs

Write electronics behavior from original requirements, general engineering
knowledge, independently authored designs and fixtures, published format
specifications and user manuals. For compatibility, use black-box tests through
public interfaces with original inputs; record the tested program version and
the origin of the inputs and resulting observations. Do not inspect implementation
source to explain a black-box result.

The user's FigCraft work remains a visual reference and must not be changed unless
the task explicitly requests it. Reuse qualified SolidCraft-family and SolidCore
libraries within the standalone boundary. Their source, license, dependency graph
and notices need qualification; a familiar project name alone is insufficient.

Symbol/footprint library data, fonts, icons, examples and documentation have their
own licenses. Qualify those materials separately from application code and retain
their notices. The project's MIT OR Apache-2.0 license does not relicense imported
materials. Do not infer that KiCad program code and library data have the same
license, or import data merely because it is available from KiCad.

Before importing code, adding a dependency or using external data, record its
source URL/path, exact revision or version, license and selected license option
where applicable, scope of use, modifications and required notices in the
[source register](sources.md), [attribution](../ATTRIBUTION.md) or the relevant
asset register. Check transitive dependencies and enabled features for the actual
consumer. If provenance or license suitability is unresolved, leave that material
out of the change until the question is resolved.

## Agent task and review requirements

Start implementation tasks from this project's instructions and approved source
list. Include this policy in task briefs and handoffs. Review both new files and
edits for source provenance; a passing test, an AI attribution trailer or a new
license header is not evidence of independent authorship. Preserve third-party
notices and record imported material explicitly rather than presenting it as
original PCBcraft code.

If an agent encounters excluded implementation material, stop reading it and pause
the implementation work affected by that exposure. Tell the user what was
accessed, why it was excluded and which work may be affected. Record the location
and scope without copying the excluded code into notes, prompts or issues. Resolve
the affected task's source boundary before continuing; do not silently paraphrase
the material or claim that a fresh context alone establishes independence.

Do not claim a formal legal or historical clean-room certification from these
working rules or from a dependency/string scan. Report the evidence and its scope.
No part of this policy authorizes publication, PRs, agent launches or license
changes outside the user's explicit task scope.

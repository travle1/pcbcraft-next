# Library management research

November 4, 2026. Input to the [draft product spec](specs/library-management.md),
not a competitor implementation review or a legal/data-reuse qualification.
Existing PCBcraft requirements and domain contracts already establish durable
identity, project snapshots, generic components, separate physical mappings and
reviewed upgrades. The proposal elaborates those original contracts.

## Evidence and interpretation

| Observation | Source and scope | Implication for the proposal |
| --- | --- | --- |
| Product library workflows need several source contexts. | Autodesk's current [Electronics Library Manager manual](https://help.autodesk.com/cloudhelp/ENU/Fusion-ECAD/files/ECD-LIB-MGR-CPT.htm) describes hub, public and private/local libraries, activation and updates. This is documented behavior, not proof of a product defect. | Keep one discovery UI with explicit source identity; avoid requiring one storage/provider workflow for everyone. |
| Similar library names can hide different identities/versions. | A [2018 Autodesk forum report](https://forums.autodesk.com/t5/eagle-forum-read-only/multiple-versions-of-managed-libraries-problem/m-p/8003756) describes multiple same-name instances. Only search-result excerpts were available; the full page could not be retrieved. | Names are not identity, and update scope must identify the selected release. This is an historical report, not a claim about current Fusion behavior or prevalence. |
| Users can face several version counters with unclear synchronization direction. | A [2021 Autodesk forum thread](https://forums.autodesk.com/t5/fusion-electronics-forum/library-and-library-io-version-mismatch/m-p/10605356) includes a user report and a vendor explanation of separate team/managed versions. Reviewed search excerpts only; direct retrieval failed. | Distinguish source release, local draft and project-used revision, with actions explaining their actual scope. Do not present this as a verified current defect. |
| Revision, lifecycle and placed-component status are separate useful facts. | Altium's [placed-components manual](https://www.altium.com/documentation/altium-designer/schematic/placed-components), updated October 6, 2026, documents source/revision/lifecycle status for placed components. | Preserve exact project bindings and offer deliberate upgrade/where-used views. This does not establish automatic compatibility or qualification of any library data. |
| Reusable electrical and physical definitions can be distinct while supporting concrete placement. | LibrePCB's published [library-elements user tutorial](https://docs.librepcb.org/_branches/master/librepcb/quickstart/create-library-elements/) explains symbol/component/package/device relationships and reuse. Only the user-documentation overview was read, not implementation or file-format source. | Keep semantic separation in the data model, while making the common placement/authoring task simpler than its underlying record graph. PCBcraft's own domain contract already requires that separation. |

These examples support hypotheses about user friction. They are not a systematic
usability study, a measurement of “library hell,” a survey of every current EDA
tool or evidence that another product lacks project caching. No current KiCad
behavior claims from the excluded search results are used in this proposal.

The product recommendation is original PCBcraft design work: a useful maintained
starter, portable exact used definitions, explicit choices and optional workflow
policies over one model. It still needs user review and measured acceptance.
No manufacturer parts, footprints, external catalog data or new code dependencies
have been imported for this documentation task.

## Source-boundary event

The first broad web search returned excluded implementation excerpts alongside
manuals/user reports. Search-output exposure included results identifying
`KiCad / eeschema/project_rescue.cpp` on GitLab and
`KiCad / eeschema/dialogs/dialog_symbol_remap.cpp` on Fossies. No implementation
page was opened or fetched directly, and no source files, patches, tests or
fixtures were imported. The exposed code is deliberately not reproduced here.

The assistant told the user, stopped that search path and set its results aside.
Subsequent research used separately searched publisher documentation and Autodesk
user-report excerpts. No code or algorithm in those excluded results is a basis
for the spec. The existing PCBcraft domain/requirements documents used for the
proposal predate this exposure.

The [source policy](source-policy.md) requires affected implementation to remain
paused until its boundary is resolved. KiCad-specific library resolution/rescue,
remapping/import implementation and any source-derived implementation work from
this research context remain paused pending a source-boundary review. Writing
general product requirements from preexisting contracts does not resolve that
implementation boundary, and starting another agent/context is not a clean-room
certification or permission to resume it. Carry this record into relevant briefs.

## Further validation

Before freezing the first content/connector scope, walk through a hobbyist's
generic schematic, an exact-part prototype, a small team's approved catalog and
an offline/shared-project handoff. Check how users distinguish instance edits,
project-local definitions, publishing a source and accepting an upgrade. Use
original scenarios and user-provided materials only after source/license review.
Test the same tasks in native and browser builds; browser-local folder/Git and
persistent storage capabilities cannot be assumed from the native experience.

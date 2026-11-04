# Attribution and contribution credit

## Source and asset provenance

Original PCBcraft contributions use [MIT](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE).
Reused materials retain their own licenses and notices. This project does not
relicense inherited code or certify historical clean-room provenance.

PCBcraft is restarting with an independent implementation, not relicensing or
porting the preserved GPL fork. Its history, source and license stay separate.
The required [source policy](docs/source-policy.md) applies before source inspection,
implementation or imports, including AI-assisted work. Rewriting excluded code or
adding an MIT/Apache header is not an approved way to reuse it. Record the actual
source and license of reused libraries, data and assets; do not present them as
original PCBcraft contributions.

| Material | Provenance and notices |
| --- | --- |
| SolidCraft-family SVGs, palette data and IBM Plex Sans | [Resource register](assets/family/provenance.json), [licenses and notices](assets/family/README.md); family resources MIT OR Apache-2.0, font SIL OFL 1.1 |
| Narrow workspace-picker and panel/control adaptations | [Workspace provenance](docs/workspace-picker-provenance.md), [controls provenance](docs/panel-controls-provenance.md), retained [SolidCraft NOTICE](assets/family/SOLIDCRAFT-NOTICE) |
| Original application mark and EDA glyphs | [Original asset provenance](assets/original/README.md), root MIT OR Apache-2.0 licenses |
| Original EDA symbols, example circuit and desktop composition | Project source and root license; the example is not an electrical reference design |
| README screenshots/GIF | [Capture provenance](docs/screenshots/README.md); real PCBcraft UI containing the existing licensed resources |
| Architectural and documentation references | [Source register](docs/sources.md); links distinguish reference reading from imported material |

Record source URL, revision, license, scope and any modifications before importing
new code/assets/data. Commit credits do not replace license notices or data provenance.
The preserved GPL fork and KiCad implementation are excluded as implementation
references under [AGENTS.md](AGENTS.md). Do not infer attribution to upstream authors
merely because their documentation informed an independently written design.

## Commit credit

Keep the contributor's normal Git author/committer identity. Add an actual human
co-author with the name and email they want used, including their GitHub-provided
no-reply address if desired. Do not invent human collaborators or change local Git
identity to impersonate a tool, provider or another contributor.

For AI-authored or substantially AI-edited work, add a `Co-authored-by` trailer for
each **actual contributing tool/model**. PCBcraft uses the following project-local
labels. These addresses deliberately use reserved `.invalid`; they are attribution
identifiers, not mailboxes, provider-owned accounts or GitHub-linked bot identities.

| Verified participant | Trailer |
| --- | --- |
| Codex; model not reliably identified | `Co-authored-by: Codex <codex@ai.invalid>` |
| Codex using Astra | `Co-authored-by: Codex (Astra) <codex-astra@ai.invalid>` |
| Codex using Sol | `Co-authored-by: Codex (Sol) <codex-sol@ai.invalid>` |
| Claude using Opus | `Co-authored-by: Claude (Opus) <claude-opus@ai.invalid>` |
| Claude; model not reliably identified | `Co-authored-by: Claude <claude@ai.invalid>` |

Use model identity from the actual run's trusted metadata or explicit configuration.
If a version is known, include it in the display name or PR attribution note; never
infer it from task difficulty, a plan to use that model, or another agent's history.
Do not add both generic and model-specific entries for the same contribution.
For another tool/model, document its truthful label using the same local convention.

Credit review-only assistance with a separate `AI-reviewed-by` trailer, naming the
same tool/model identity. This custom trailer records assistance, not a human PR
approval or proof that tests passed. A reviewer whose edits were incorporated may
also merit co-author credit; do not automatically credit every agent mentioned in a
plan. The submitting contributor remains responsible for verifying the result.

Example syntax, **only when both participants actually performed these roles**:

```text
feat(schematic): add a bounded connection proposal

Describe the behavior and validation here.

Co-authored-by: Codex (Sol) <codex-sol@ai.invalid>
AI-reviewed-by: Codex (Astra) <codex-astra@ai.invalid>
```

Place the trailers together at the end, separated from prose by a blank line.
Preserve them when squashing; combine actual contributors and remove duplicates.
Do not add `Signed-off-by` for someone else or treat AI review as maintainer approval.
The PR should briefly identify each tool/model's contribution and any unknown version.
Do not rewrite published history or backfill guessed models. Apply this convention
to new commits; correcting older messages needs explicit scope and reliable evidence.

GitHub [documents the co-author trailer syntax and account-email requirements](https://docs.github.com/en/pull-requests/how-tos/commit-changes/creating-a-commit-with-multiple-authors).
The synthetic AI addresses here will remain visible commit text but do **not**
provide linked GitHub account/contribution credit. The `.invalid` reservation is
defined in [RFC 2606](https://www.rfc-editor.org/rfc/rfc2606.html#section-2).
No commit hook, bot or automatic model detection is installed by this policy.

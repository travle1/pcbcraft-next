# Flat sheets and hierarchical subsheets

Status, November 3, 2026: **not implemented**. The current document has one `sheet_id`
and one aggregate of symbols, wires, junctions and notes. The Main tree row navigates
that real sheet. Diagram headings such as Power and Controller are text notes,
not sheet boundaries or hierarchical blocks. The corrected browser does not change
this model or pretend that more sheets exist.

## Intended interaction

- **Sheets → New sheet** creates a flat sibling with its own sheet identity and content.
- **A sheet → New subsheet** creates a child definition/occurrence and places its
  hierarchical sheet symbol in the parent. Ports explicitly map the parent symbol
  terminals to the child interface. Drawing a box alone does not establish those connections.
- Click a sheet name to activate it; click its chevron to expand/collapse without
  navigation. Symbols/Nets beneath it open that occurrence's scoped Search results.
- Keep project document tabs distinct from sheet navigation/preview tabs. Each
  sheet view retains its camera and selection context; one project session owns undo/save.

These destinations are present as disabled context-menu actions with explanations.
Project settings are also currently unavailable. There is no hidden shortcut that
creates flat or hierarchical sheets in this build.

## Next bounded implementation sequence

1. Settle sheet definition versus occurrence IDs against the [domain contract](specs/domain.md).
   A sheet name/path is not its identity; repeated hierarchy must later be possible
   without sharing physical component identities. The first flat-sheet slice may
   require a distinct definition for each occurrence, but should not make that an
   architectural invariant.
2. Add flat-sheet create/rename/delete and active-sheet navigation through the single
   Electronics session. Assign all symbols, junctions, notes and connections a
   well-defined sheet scope. Validate endpoint scope; move/delete entire contents
   atomically, with review of destructive removal. Rendering, hit testing, search,
   text edits, net reports and selection must use the active sheet/occurrence.
3. Migrate existing v1–5 drafts into one Main sheet without changing existing object
   identity or connectivity. Persist all sheets in one native project; undo/redo and
   save/reopen cover the whole aggregate, with stale inspector/drag rejection on
   scope changes. Different local net labels on different sheets must remain isolated.
   Define explicit project-global connection semantics before advertising cross-sheet
   electrical connectivity; ordinary display strings still do not join circuits.
4. Add hierarchical definitions/occurrences, parent sheet symbols, interface ports,
   explicit terminal mappings and occurrence-qualified connectivity. Reject cycles
   and dangling mappings, and handle rename/reparent/delete and repeated occurrences
   without name/path-based identity. Implement this after flat sheets and persistence
   pass their acceptance checks; do not flatten a repeated definition into shared parts.

Acceptance: two independent flat sheets can contain similarly named local nets
without unintended connections; switching scopes changes rendering/search/picking;
creating/deleting/undoing and saving/reopening preserves all contents. Then a parent
plus two occurrences of a child can connect through explicit ports while preserving
distinct component occurrences, net membership and cross-probe targets. Invalid,
stale or cyclic changes leave zero mutation. Empty sheets and missing mappings
have truthful UI states. This is the next feature plan, not completed behavior.

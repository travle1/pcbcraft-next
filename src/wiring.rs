//! Original authored schematic wiring. Geometry never implicitly merges nets.
use crate::document::*;
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub enum Anchor {
    Existing(Endpoint),
    Free(Point),
    /// Split this explicit edge at a point on its visible path. Legacy automatic
    /// routes supply their current projection; authored routes must match storage.
    Split {
        wire: WireId,
        at: Point,
        path: Vec<Point>,
    },
}
impl Anchor {
    pub fn point(&self, doc: &Document) -> Option<Point> {
        match self {
            Self::Existing(e) => doc.endpoint(e),
            Self::Free(p) | Self::Split { at: p, .. } => Some(*p),
        }
    }
}
pub fn path(doc: &Document, w: &Wire) -> Result<Vec<Point>> {
    let mut points = vec![doc.endpoint(&w.from).ok_or("Missing wire start.")?];
    if let Some(bends) = &w.bends {
        points.extend(bends);
    }
    points.push(doc.endpoint(&w.to).ok_or("Missing wire end.")?);
    Ok(points)
}
pub fn validate_path(points: &[Point]) -> Result<()> {
    if !(2..=258).contains(&points.len()) || points.iter().any(|p| !p.valid()) {
        return Err("Wire requires 2–258 bounded points.".into());
    }
    if points
        .windows(2)
        .any(|p| p[0] == p[1] || (p[0].x != p[1].x && p[0].y != p[1].y))
    {
        return Err("Wire segments must be nonzero and horizontal or vertical.".into());
    }
    Ok(())
}
pub fn on_segment(p: Point, a: Point, b: Point) -> bool {
    (a.x == b.x && p.x == a.x && (a.y.min(b.y)..=a.y.max(b.y)).contains(&p.y))
        || (a.y == b.y && p.y == a.y && (a.x.min(b.x)..=a.x.max(b.x)).contains(&p.x))
}
pub fn simplify(points: &mut Vec<Point>) {
    points.dedup();
    let mut i = 1;
    while i + 1 < points.len() {
        if (points[i - 1].x == points[i].x && points[i].x == points[i + 1].x)
            || (points[i - 1].y == points[i].y && points[i].y == points[i + 1].y)
        {
            points.remove(i);
            points.dedup();
            i = i.saturating_sub(1).max(1);
        } else {
            i += 1;
        }
    }
}
fn free(doc: &mut Document, at: Point) -> Result<Endpoint> {
    if !at.valid() {
        return Err("Wire point is outside the sheet limits.".into());
    }
    let id = JunctionId::fresh()?;
    doc.junctions.push(Junction { id, at });
    Ok(Endpoint::Junction(JunctionRef { junction: id }))
}
fn resolve(doc: &mut Document, anchor: Anchor) -> Result<Endpoint> {
    match anchor {
        Anchor::Existing(e) => {
            doc.endpoint(&e).ok_or("Wire target no longer exists.")?;
            Ok(e)
        }
        Anchor::Free(at) => free(doc, at),
        Anchor::Split {
            wire,
            at,
            path: mut points,
        } => {
            let index = doc
                .wires
                .iter()
                .position(|w| w.id == wire)
                .ok_or("Wire target changed. Start again.")?;
            let old = doc.wires[index].clone();
            if old.label.is_some() && !old.draw_path {
                return Err("Start from the alias pin, not its label.".into());
            }
            simplify(&mut points);
            validate_path(&points)?;
            if points.first().copied() != doc.endpoint(&old.from)
                || points.last().copied() != doc.endpoint(&old.to)
            {
                return Err("Wire target endpoints changed.".into());
            }
            if old.bends.is_some() {
                let mut stored = path(doc, &old)?;
                simplify(&mut stored);
                if stored != points {
                    return Err("Wire route changed. Start again.".into());
                }
            }
            if points.first() == Some(&at) {
                return Ok(old.from);
            }
            if points.last() == Some(&at) {
                return Ok(old.to);
            }
            let segment = points
                .windows(2)
                .position(|p| on_segment(at, p[0], p[1]))
                .ok_or("Branch must meet the selected segment.")?;
            let node = free(doc, at)?;
            let mut left = points[..=segment].to_vec();
            left.push(at);
            simplify(&mut left);
            let mut right = vec![at];
            right.extend_from_slice(&points[segment + 1..]);
            simplify(&mut right);
            doc.wires[index].to = node.clone();
            doc.wires[index].bends = Some(left[1..left.len() - 1].to_vec());
            doc.wires[index].draw_path = true;
            doc.wires.push(Wire {
                id: WireId::fresh()?,
                from: node.clone(),
                to: old.to,
                label: old.label,
                draw_path: true,
                bends: Some(right[1..right.len() - 1].to_vec()),
            });
            Ok(node)
        }
    }
}
pub(crate) fn draw(doc: &mut Document, from: Anchor, to: Anchor, bends: Vec<Point>) -> Result<()> {
    if let (Anchor::Split { wire: a, .. }, Anchor::Split { wire: b, .. }) = (&from, &to)
        && a == b
    {
        return Err("Finish this branch on another wire, pin or free point.".into());
    }
    if bends.len() > 256 {
        return Err("Wire has too many corners.".into());
    }
    let from = resolve(doc, from)?;
    let to = resolve(doc, to)?;
    if from == to {
        return Err("A wire must connect different endpoints.".into());
    }
    let mut points = vec![doc.endpoint(&from).ok_or("Missing start.")?];
    points.extend(bends);
    points.push(doc.endpoint(&to).ok_or("Missing end.")?);
    simplify(&mut points);
    validate_path(&points)?;
    doc.wires.push(Wire {
        id: WireId::fresh()?,
        from,
        to,
        label: None,
        draw_path: true,
        bends: Some(points[1..points.len() - 1].to_vec()),
    });
    // Document validation rejects joining incompatible names atomically.
    Ok(())
}
pub(crate) fn prune_junctions(doc: &mut Document) {
    let used: BTreeSet<_> = doc
        .wires
        .iter()
        .flat_map(|w| [&w.from, &w.to])
        .cloned()
        .collect();
    doc.junctions
        .retain(|j| used.contains(&Endpoint::Junction(JunctionRef { junction: j.id })));
}
/// Keep a disconnected name on one largest surviving graph, clearing others.
/// Never joins anything by a label.
pub(crate) fn normalize_names(doc: &mut Document) {
    let mut groups = crate::netlist::groups(doc).nets;
    groups.sort_by(|a, b| {
        b.nodes
            .len()
            .cmp(&a.nodes.len())
            .then(a.anchor.cmp(&b.anchor))
    });
    let mut used = BTreeSet::new();
    for n in groups {
        if let Some(name) = n.name
            && !used.insert(name)
        {
            for w in &mut doc.wires {
                if n.links.contains(&w.id) {
                    w.label = None;
                    w.draw_path = true;
                }
            }
        }
    }
}
/// Stretch incident end legs on component movement, preserving interior authored
/// corners. This is part of the same candidate/undo entry as the component edit.
pub(crate) fn repair_moved_endpoints(before: &Document, after: &mut Document) -> Result<()> {
    for index in 0..after.wires.len() {
        let wire = &after.wires[index];
        let Some(old) = before.wires.iter().find(|w| w.id == wire.id) else {
            continue;
        };
        if wire.bends.is_none() || wire.from != old.from || wire.to != old.to {
            continue;
        }
        let a = after.endpoint(&wire.from).ok_or("Missing start.")?;
        let b = after.endpoint(&wire.to).ok_or("Missing end.")?;
        if Some(a) == before.endpoint(&old.from) && Some(b) == before.endpoint(&old.to) {
            continue;
        }
        let mut points = path(before, old)?;
        let n = points.len();
        if n < 2 {
            return Err("Invalid old path.".into());
        }
        let start_horizontal = points[0].y == points[1].y;
        let end_horizontal = points[n - 2].y == points[n - 1].y;
        points[0] = a;
        points[n - 1] = b;
        // New corner preserves the old end-leg direction without moving other junctions.
        if a.x != points[1].x && a.y != points[1].y {
            let corner = if start_horizontal {
                Point {
                    x: points[1].x,
                    y: a.y,
                }
            } else {
                Point {
                    x: a.x,
                    y: points[1].y,
                }
            };
            points.insert(1, corner);
        }
        let n = points.len();
        if points[n - 2].x != b.x && points[n - 2].y != b.y {
            let corner = if end_horizontal {
                Point {
                    x: points[n - 2].x,
                    y: b.y,
                }
            } else {
                Point {
                    x: b.x,
                    y: points[n - 2].y,
                }
            };
            points.insert(n - 1, corner);
        }
        simplify(&mut points);
        validate_path(&points)?;
        after.wires[index].bends = Some(points[1..points.len() - 1].to_vec());
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub enum WireEdit {
    /// Current legacy display paths, revision-bound by the caller. Only paths
    /// actually edited/healed are retained as authored geometry.
    WithRoutes {
        edit: Box<WireEdit>,
        routes: Vec<Projection>,
    },
    Draw {
        from: Anchor,
        to: Anchor,
        bends: Vec<Point>,
    },
    Attach {
        junction: JunctionId,
        target: Anchor,
    },
    Junction {
        id: JunctionId,
        at: Point,
    },
    Corner {
        wire: WireId,
        path: Vec<Point>,
        index: usize,
        at: Point,
    },
    Segment {
        wire: WireId,
        path: Vec<Point>,
        index: usize,
        at: Point,
    },
}
#[derive(Clone, Debug)]
pub struct Projection {
    pub wire: WireId,
    pub path: Vec<Point>,
}
/// The GUI can render this validated disposable candidate; only Session commits it.
pub fn preview_edit(doc: &Document, edit: &WireEdit) -> Result<Document> {
    let mut next = doc.clone();
    match edit {
        WireEdit::WithRoutes { edit, routes } => {
            if routes.len() > MAX_ITEMS || matches!(**edit, WireEdit::WithRoutes { .. }) {
                return Err("Invalid wire display context.".into());
            }
            let mut materialized = doc.clone();
            let mut seen = BTreeSet::new();
            for projection in routes {
                if !seen.insert(projection.wire) {
                    return Err("Duplicate projected wire.".into());
                }
                let Some(w) = doc.wires.iter().find(|w| w.id == projection.wire) else {
                    return Err("Projected wire disappeared.".into());
                };
                if w.bends.is_some() || (w.label.is_some() && !w.draw_path) {
                    continue;
                }
                validate_path(&projection.path)?;
                if projection.path.first().copied() != doc.endpoint(&w.from)
                    || projection.path.last().copied() != doc.endpoint(&w.to)
                {
                    return Err("Projected wire endpoints changed.".into());
                }
                if let Some(target) = materialized
                    .wires
                    .iter_mut()
                    .find(|w| w.id == projection.wire)
                {
                    target.bends = Some(projection.path[1..projection.path.len() - 1].to_vec());
                    target.draw_path = true;
                }
            }
            let mut next = preview_edit(&materialized, edit)?;
            for w in &mut next.wires {
                if let Some(old) = doc.wires.iter().find(|old| old.id == w.id)
                    && materialized
                        .wires
                        .iter()
                        .find(|m| m.id == w.id)
                        .is_some_and(|m| m == w)
                {
                    *w = old.clone();
                }
            }
            next.validate()?;
            return Ok(next);
        }
        WireEdit::Draw { from, to, bends } => {
            draw(&mut next, from.clone(), to.clone(), bends.clone())?
        }
        WireEdit::Attach { junction, target } => {
            let old = Endpoint::Junction(JunctionRef {
                junction: *junction,
            });
            let endpoint = resolve(&mut next, target.clone())?;
            let at = next
                .endpoint(&endpoint)
                .ok_or("Attachment target disappeared.")?;
            next.junctions
                .iter_mut()
                .find(|j| j.id == *junction)
                .ok_or("Wire point disappeared.")?
                .at = at;
            repair_moved_endpoints(doc, &mut next)?;
            merge_endpoints(&mut next, old, endpoint)?;
            deduplicate_paths(&mut next)?;
            prune_junctions(&mut next);
        }
        WireEdit::Junction { id, at } => {
            next.junctions
                .iter_mut()
                .find(|j| j.id == *id)
                .ok_or("Wire point no longer exists.")?
                .at = *at;
            repair_moved_endpoints(doc, &mut next)?;
        }
        WireEdit::Corner {
            wire,
            path: supplied,
            index,
            at,
        }
        | WireEdit::Segment {
            wire,
            path: supplied,
            index,
            at,
        } => {
            let w = next
                .wires
                .iter()
                .find(|w| w.id == *wire)
                .ok_or("Wire no longer exists.")?;
            validate_path(supplied)?;
            if supplied.first().copied() != doc.endpoint(&w.from)
                || supplied.last().copied() != doc.endpoint(&w.to)
            {
                return Err("Wire endpoints changed. Start the edit again.".into());
            }
            if w.bends.is_some() && path(doc, w)? != *supplied {
                return Err("Wire route changed. Start the edit again.".into());
            }
            if !at.valid() {
                return Err("Wire edit is outside the sheet limits.".into());
            }
            let mut points = supplied.clone();
            // A drag back to its original position must not freeze a legacy auto route.
            if matches!(edit, WireEdit::Corner { .. }) && points.get(*index) == Some(at) {
                return Ok(next);
            }
            if matches!(edit, WireEdit::Segment { .. }) && *index + 1 < points.len() {
                let a = points[*index];
                let b = points[*index + 1];
                if (a.y == b.y && a.y == at.y) || (a.x == b.x && a.x == at.x) {
                    return Ok(next);
                }
            }
            match edit {
                WireEdit::Corner { .. } => {
                    if *index == 0 || *index + 1 >= points.len() {
                        return Err("Select an interior wire corner.".into());
                    }
                    let a = points[*index - 1];
                    let b = points[*index + 1];
                    let old = points[*index];
                    let mut replacement = vec![a];
                    elbow(&mut replacement, *at, old.y == a.y);
                    elbow(&mut replacement, b, old.y == b.y);
                    points.splice(index - 1..=index + 1, replacement);
                }
                WireEdit::Segment { .. } => {
                    if *index + 1 >= points.len() {
                        return Err("Wire segment no longer exists.".into());
                    }
                    let old_a = points[*index];
                    let old_b = points[*index + 1];
                    if old_a.y == old_b.y {
                        points[*index].y = at.y;
                        points[*index + 1].y = at.y;
                    } else {
                        points[*index].x = at.x;
                        points[*index + 1].x = at.x;
                    }
                    if *index + 2 == supplied.len() {
                        points.push(old_b);
                    }
                    if *index == 0 {
                        points.insert(0, old_a);
                    }
                }
                WireEdit::Junction { .. }
                | WireEdit::Attach { .. }
                | WireEdit::Draw { .. }
                | WireEdit::WithRoutes { .. } => {
                    return Err("Invalid wire route operation.".into());
                }
            }
            simplify(&mut points);
            validate_path(&points)?;
            let wire = next
                .wires
                .iter_mut()
                .find(|w| w.id == *wire)
                .ok_or("Wire no longer exists.")?;
            wire.bends = Some(points[1..points.len() - 1].to_vec());
            wire.draw_path = true;
        }
    }
    heal_overlaps(doc, &mut next)?;
    next.validate()?;
    Ok(next)
}
fn elbow(points: &mut Vec<Point>, end: Point, horizontal: bool) {
    if let Some(start) = points.last().copied() {
        if start.x != end.x && start.y != end.y {
            points.push(if horizontal {
                Point {
                    x: end.x,
                    y: start.y,
                }
            } else {
                Point {
                    x: start.x,
                    y: end.y,
                }
            });
        }
        points.push(end);
    }
}

/// Normalize positive-length overlaps touched by this edit. Plain crossings are
/// excluded. Legacy auto routes have no stored geometry and are only materialized
/// when explicitly targeted by the UI. All work stays in the session candidate.
pub(crate) fn heal_overlaps(before: &Document, doc: &mut Document) -> Result<()> {
    let mut touched: BTreeSet<WireId> = doc
        .wires
        .iter()
        .filter(|w| {
            before
                .wires
                .iter()
                .find(|old| old.id == w.id)
                .is_none_or(|old| {
                    old != *w
                        || before.endpoint(&old.from) != doc.endpoint(&w.from)
                        || before.endpoint(&old.to) != doc.endpoint(&w.to)
                })
        })
        .map(|w| w.id)
        .collect();
    let mut budget = 200_000usize;
    for _ in 0..64 {
        let Some((a, b, p, q)) = overlap(doc, &touched, &mut budget)? else {
            collapse_runs(doc, &mut touched)?;
            return Ok(());
        };
        let groups = crate::netlist::groups(doc);
        let names = |id| {
            doc.wires
                .iter()
                .find(|w| w.id == id)
                .and_then(|w| groups.containing(&w.from))
                .and_then(|n| n.name.as_ref())
        };
        if let (Some(x), Some(y)) = (names(a), names(b))
            && x != y
        {
            return Err(
                "Overlapping wires belong to differently named nets. Reconcile their names first."
                    .into(),
            );
        }
        let old_ids: BTreeSet<_> = doc.wires.iter().map(|w| w.id).collect();
        let (ap, aq) = cut_pair(doc, a, p, q)?;
        let (bp, bq) = cut_pair(doc, b, p, q)?;
        merge_endpoints(doc, ap, bp)?;
        merge_endpoints(doc, aq, bq)?;
        deduplicate_paths(doc)?;
        prune_junctions(doc);
        touched.insert(a);
        touched.insert(b);
        touched.extend(
            doc.wires
                .iter()
                .filter(|w| !old_ids.contains(&w.id))
                .map(|w| w.id),
        );
    }
    Err("Too many overlapping runs in one edit. Split the edit into smaller steps.".into())
}
fn overlap(
    doc: &Document,
    touched: &BTreeSet<WireId>,
    budget: &mut usize,
) -> Result<Option<(WireId, WireId, Point, Point)>> {
    let authored: Vec<_> = doc
        .wires
        .iter()
        .filter(|w| w.bends.is_some())
        .map(|w| Ok((w.id, path(doc, w)?)))
        .collect::<Result<_>>()?;
    for (i, (a, pa)) in authored.iter().enumerate() {
        for (b, pb) in authored.iter().skip(i + 1) {
            if !touched.contains(a) && !touched.contains(b) {
                continue;
            }
            for x in pa.windows(2) {
                for y in pb.windows(2) {
                    *budget = budget
                        .checked_sub(1)
                        .ok_or("Wire overlap cleanup exceeds this draft's edit budget.")?;
                    if x[0].y == x[1].y && y[0].y == y[1].y && x[0].y == y[0].y {
                        let lo = x[0].x.min(x[1].x).max(y[0].x.min(y[1].x));
                        let hi = x[0].x.max(x[1].x).min(y[0].x.max(y[1].x));
                        if lo < hi {
                            return Ok(Some((
                                *a,
                                *b,
                                Point { x: lo, y: x[0].y },
                                Point { x: hi, y: x[0].y },
                            )));
                        }
                    } else if x[0].x == x[1].x && y[0].x == y[1].x && x[0].x == y[0].x {
                        let lo = x[0].y.min(x[1].y).max(y[0].y.min(y[1].y));
                        let hi = x[0].y.max(x[1].y).min(y[0].y.max(y[1].y));
                        if lo < hi {
                            return Ok(Some((
                                *a,
                                *b,
                                Point { x: x[0].x, y: lo },
                                Point { x: x[0].x, y: hi },
                            )));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}
fn cut_pair(doc: &mut Document, wire: WireId, p: Point, q: Point) -> Result<(Endpoint, Endpoint)> {
    let ids: BTreeSet<_> = doc.wires.iter().map(|w| w.id).collect();
    let w = doc
        .wires
        .iter()
        .find(|w| w.id == wire)
        .ok_or("Overlap wire disappeared.")?
        .clone();
    let route = path(doc, &w)?;
    let first = resolve(
        doc,
        Anchor::Split {
            wire,
            at: p,
            path: route,
        },
    )?;
    let second = doc
        .wires
        .iter()
        .filter(|w| w.id == wire || !ids.contains(&w.id))
        .find_map(|w| {
            path(doc, w)
                .ok()
                .filter(|p| p.windows(2).any(|s| on_segment(q, s[0], s[1])))
                .map(|p| (w.id, p))
        })
        .ok_or("Overlap boundary could not be resolved.")?;
    let second = resolve(
        doc,
        Anchor::Split {
            wire: second.0,
            at: q,
            path: second.1,
        },
    )?;
    Ok((first, second))
}
fn merge_endpoints(doc: &mut Document, a: Endpoint, b: Endpoint) -> Result<()> {
    if a == b {
        return Ok(());
    }
    if doc.endpoint(&a) != doc.endpoint(&b) {
        return Err("Wire junction positions disagree.".into());
    }
    let (keep, remove) = match (&a, &b) {
        (Endpoint::Pin(_), Endpoint::Pin(_)) => {
            return Err("Separate coincident component pins before merging wires.".into());
        }
        (Endpoint::Pin(_), _) => (a, b),
        (_, Endpoint::Pin(_)) => (b, a),
        _ => {
            if a < b {
                (a, b)
            } else {
                (b, a)
            }
        }
    };
    for w in &mut doc.wires {
        if w.from == remove {
            w.from = keep.clone();
        }
        if w.to == remove {
            w.to = keep.clone();
        }
    }
    Ok(())
}
fn deduplicate_paths(doc: &mut Document) -> Result<()> {
    let mut remove = BTreeSet::new();
    for i in 0..doc.wires.len() {
        for j in i + 1..doc.wires.len() {
            let a = &doc.wires[i];
            let b = &doc.wires[j];
            if remove.contains(&a.id) || remove.contains(&b.id) {
                continue;
            }
            let same = a.from == b.from && a.to == b.to;
            let reverse = a.from == b.to && a.to == b.from;
            if (same || reverse) && a.bends.is_some() && b.bends.is_some() {
                let mut pa = path(doc, a)?;
                let mut pb = path(doc, b)?;
                simplify(&mut pa);
                simplify(&mut pb);
                if reverse {
                    pb.reverse();
                }
                if pa == pb {
                    remove.insert(b.id);
                    if doc.wires[i].label.is_none() {
                        doc.wires[i].label = doc.wires[j].label.clone();
                    }
                }
            }
        }
    }
    doc.wires.retain(|w| !remove.contains(&w.id));
    Ok(())
}

fn collapse_runs(doc: &mut Document, touched: &mut BTreeSet<WireId>) -> Result<()> {
    for _ in 0..128 {
        let mut join = None;
        for j in &doc.junctions {
            let node = Endpoint::Junction(JunctionRef { junction: j.id });
            let edges: Vec<_> = doc
                .wires
                .iter()
                .filter(|w| w.from == node || w.to == node)
                .collect();
            if edges.len() != 2
                || edges.iter().any(|w| w.bends.is_none())
                || !edges.iter().any(|w| touched.contains(&w.id))
            {
                continue;
            }
            let a = edges[0];
            let b = edges[1];
            let from = if a.to == node {
                a.from.clone()
            } else {
                a.to.clone()
            };
            let to = if b.from == node {
                b.to.clone()
            } else {
                b.from.clone()
            };
            if from == to {
                continue;
            }
            let mut pa = path(doc, a)?;
            if a.from == node {
                pa.reverse();
            }
            let mut pb = path(doc, b)?;
            if b.to == node {
                pb.reverse();
            }
            pa.extend(pb.into_iter().skip(1));
            simplify(&mut pa);
            validate_path(&pa)?;
            join = Some((
                a.id,
                b.id,
                from,
                to,
                pa,
                a.label.clone().or(b.label.clone()),
            ));
            break;
        }
        let Some((a, b, from, to, points, label)) = join else {
            return Ok(());
        };
        let w = doc
            .wires
            .iter_mut()
            .find(|w| w.id == a)
            .ok_or("Wire run disappeared.")?;
        w.from = from;
        w.to = to;
        w.label = label;
        w.bends = Some(points[1..points.len() - 1].to_vec());
        w.draw_path = true;
        doc.wires.retain(|w| w.id != b);
        prune_junctions(doc);
        touched.insert(a);
    }
    Err("Too many redundant junctions to heal in one edit.".into())
}

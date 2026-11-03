//! Typed selection payloads for atomic group edits; selection itself is editor state.
use crate::{
    document::*,
    wiring::{self, Projection},
};
use std::collections::BTreeSet;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    pub symbols: BTreeSet<SymbolId>,
    pub wires: BTreeSet<WireId>,
    pub notes: BTreeSet<AnnotationId>,
}
impl Selection {
    pub fn len(&self) -> usize {
        self.symbols.len() + self.wires.len() + self.notes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn primary_symbol(&self) -> Option<SymbolId> {
        (self.len() == 1)
            .then(|| self.symbols.first().copied())
            .flatten()
    }
    pub fn retain(&mut self, d: &Document) {
        self.symbols.retain(|id| d.component(*id).is_some());
        self.wires.retain(|id| d.wires.iter().any(|w| w.id == *id));
        self.notes
            .retain(|id| d.annotations.iter().any(|a| a.id == *id));
    }
    fn validate(&self, d: &Document) -> Result<()> {
        let mut copy = self.clone();
        copy.retain(d);
        if self != &copy || self.is_empty() {
            return Err("Selection is empty or contains missing items.".into());
        }
        Ok(())
    }
}
pub(crate) fn delete(d: &mut Document, items: &Selection) -> Result<()> {
    items.validate(d)?;
    let components: BTreeSet<_> = d
        .symbols
        .iter()
        .filter(|s| items.symbols.contains(&s.id))
        .map(|s| s.component)
        .collect();
    d.symbols.retain(|s| !items.symbols.contains(&s.id));
    d.components.retain(|c| !components.contains(&c.id));
    d.wires.retain(|w| {
        !items.wires.contains(&w.id)
            && !items
                .symbols
                .iter()
                .any(|id| w.from.on_symbol(*id) || w.to.on_symbol(*id))
    });
    d.annotations.retain(|a| !items.notes.contains(&a.id));
    wiring::prune_junctions(d);
    wiring::normalize_names(d);
    Ok(())
}
fn translated(p: Point, delta: Point) -> Result<Point> {
    let p = Point {
        x: p.x.checked_add(delta.x).ok_or("Coordinate overflow.")?,
        y: p.y.checked_add(delta.y).ok_or("Coordinate overflow.")?,
    };
    if !p.valid() {
        return Err("Move exceeds sheet coordinate limits.".into());
    }
    Ok(p)
}
pub fn moved(
    doc: &Document,
    items: &Selection,
    delta: Point,
    routes: &[Projection],
) -> Result<Document> {
    items.validate(doc)?;
    if !delta.valid() || routes.len() > MAX_ITEMS {
        return Err("Invalid group move.".into());
    }
    let mut before = doc.clone();
    let mut seen = BTreeSet::new();
    for route in routes {
        if !seen.insert(route.wire) {
            return Err("Duplicate route projection.".into());
        }
        let w = before
            .wires
            .iter()
            .find(|w| w.id == route.wire)
            .ok_or("Projected wire no longer exists.")?;
        if route.path.len() > 258
            || route.path.first().copied() != before.endpoint(&w.from)
            || route.path.last().copied() != before.endpoint(&w.to)
        {
            return Err("Route projection does not match its wire.".into());
        }
        wiring::validate_path(&route.path)?;
        if items.wires.contains(&route.wire) && w.bends.is_none() {
            let w = before
                .wires
                .iter_mut()
                .find(|w| w.id == route.wire)
                .ok_or("Missing wire.")?;
            w.bends = Some(route.path[1..route.path.len() - 1].to_vec());
            w.draw_path = true;
        }
    }
    if delta == Point::default() {
        return Ok(doc.clone());
    }
    let mut next = before.clone();
    for s in &mut next.symbols {
        if items.symbols.contains(&s.id) {
            s.at = translated(s.at, delta)?;
        }
    }
    for a in &mut next.annotations {
        if items.notes.contains(&a.id) {
            a.at = translated(a.at, delta)?;
        }
    }
    let junctions: BTreeSet<_> = before
        .wires
        .iter()
        .filter(|w| items.wires.contains(&w.id))
        .flat_map(|w| [&w.from, &w.to])
        .filter_map(|e| {
            if let Endpoint::Junction(j) = e {
                Some(j.junction)
            } else {
                None
            }
        })
        .collect();
    for j in &mut next.junctions {
        if junctions.contains(&j.id) {
            j.at = translated(j.at, delta)?;
        }
    }
    wiring::repair_moved_endpoints(&before, &mut next)?;
    for id in &items.wires {
        let old = before
            .wires
            .iter()
            .find(|w| w.id == *id)
            .ok_or("Missing wire.")?;
        let mut points = wiring::path(&before, old)?
            .into_iter()
            .map(|p| translated(p, delta))
            .collect::<Result<Vec<_>>>()?;
        let a = next.endpoint(&old.from).ok_or("Missing endpoint.")?;
        let b = next.endpoint(&old.to).ok_or("Missing endpoint.")?;
        let first = *points.first().ok_or("Empty wire.")?;
        let last = *points.last().ok_or("Empty wire.")?;
        points.insert(0, Point { x: first.x, y: a.y });
        points.insert(0, a);
        points.push(Point { x: last.x, y: b.y });
        points.push(b);
        wiring::simplify(&mut points);
        wiring::validate_path(&points)?;
        let w = next
            .wires
            .iter_mut()
            .find(|w| w.id == *id)
            .ok_or("Missing wire.")?;
        w.bends = Some(points[1..points.len() - 1].to_vec());
        w.draw_path = true;
    }
    wiring::heal_overlaps(&before, &mut next)?;
    crate::text::prune(&mut next);
    next.validate()?;
    Ok(next)
}

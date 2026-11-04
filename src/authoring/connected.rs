//! Original, conservative connected-grid profile. No inference of circuit intent.
use super::{ArrangementRegion, layout};
use crate::{document::*, project::Project, wiring};
use std::collections::BTreeSet;

impl ArrangementRegion {
    fn contains(self, p: Point) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }
}

/// Positive-length collinear sharing is not an explicit junction. Perpendicular
/// crossings remain disconnected, as in manual authoring.
fn shares_segment(a: &[Point], b: &[Point]) -> bool {
    a.windows(2).any(|a| {
        b.windows(2).any(|b| {
            (a[0].y == a[1].y
                && b[0].y == b[1].y
                && a[0].y == b[0].y
                && a[0].x.min(a[1].x).max(b[0].x.min(b[1].x))
                    < a[0].x.max(a[1].x).min(b[0].x.max(b[1].x)))
                || (a[0].x == a[1].x
                    && b[0].x == b[1].x
                    && a[0].x == b[0].x
                    && a[0].y.min(a[1].y).max(b[0].y.min(b[1].y))
                        < a[0].y.max(a[1].y).min(b[0].y.max(b[1].y)))
        })
    })
}

// The operation is a single candidate transformation; only Session may commit it.
#[allow(clippy::too_many_arguments)]
pub(super) fn arrange(
    project: &mut Project,
    sheet: SheetOccurrenceId,
    ids: &[SymbolId],
    locked: &[SymbolId],
    region: ArrangementRegion,
    origin: Point,
    columns: usize,
    spacing: Point,
) -> Result<()> {
    let selected: BTreeSet<_> = ids.iter().copied().collect();
    let fixed: BTreeSet<_> = locked.iter().copied().collect();
    if ids.is_empty()
        || ids.len() > 20
        || selected.len() != ids.len()
        || fixed.len() != locked.len()
        || !fixed.is_subset(&selected)
        || !(1..=8).contains(&columns)
        || !origin.valid()
        || !spacing.valid()
        || origin != origin.snapped()
        || spacing != spacing.snapped()
        || spacing.x < 16 * GRID_NM
        || spacing.y < 16 * GRID_NM
        || !region.min.valid()
        || !region.max.valid()
        || region.min.x >= region.max.x
        || region.min.y >= region.max.y
    {
        return Err("Connected arrangement needs 1–20 distinct symbols, distinct locks within that selection, 1–8 columns, grid-aligned origin/spacing of at least 16 grid units and a bounded nonempty region.".into());
    }
    let before = project.document(sheet).ok_or("Unknown sheet.")?;
    if before.symbols.len() > 128 || before.wires.len() > 256 {
        return Err(
            "Connected arrangement supports sheets with at most 128 symbols and 256 wires.".into(),
        );
    }
    if !before.annotations.is_empty()
        || !before.text_overrides.is_empty()
        || !before.ports.is_empty()
        || !before.sheet_symbols.is_empty()
    {
        return Err("Connected arrangement currently supports catalog-only sheets without notes, custom text styling, ports or subsheet blocks; their clearance is not qualified.".into());
    }
    let connectivity = project.connectivity()?.nets;
    let mut next = before.clone();
    let mut moved = BTreeSet::new();
    for (i, id) in ids.iter().enumerate() {
        let s = next
            .symbols
            .iter_mut()
            .find(|s| s.id == *id)
            .ok_or_else(|| format!("Unknown arrangement symbol {id:?}."))?;
        if !fixed.contains(id) {
            let at = Point {
                x: origin.x + (i % columns) as i64 * spacing.x,
                y: origin.y + (i / columns) as i64 * spacing.y,
            };
            if !at.valid() {
                return Err(
                    format!("Arrangement target for {id:?} exceeds coordinate limits.").into(),
                );
            }
            if s.at != at {
                moved.insert(*id);
            }
            s.at = at;
        }
    }
    for id in ids {
        let area = layout::bounds(&next, *id, 6 * GRID_NM)?;
        let component = next.component(*id).ok_or("Missing component.")?;
        let pins_inside = component.kind.pins().iter().all(|p| {
            next.pin(&PinRef {
                symbol: *id,
                terminal: p.id.into(),
            })
            .is_some_and(|p| region.contains(p))
        });
        if !region.contains(area.min) || !region.contains(area.max) || !pins_inside {
            return Err(format!(
                "Symbol {id:?} and its reserved area/pins must fit inside the arrangement region."
            )
            .into());
        }
        for other in &next.symbols {
            if other.id != *id && area.overlaps(layout::bounds(&next, other.id, 6 * GRID_NM)?) {
                return Err(format!("Arrangement reserved areas overlap: {id:?} and {:?}. Increase spacing or move the region.", other.id).into());
            }
        }
    }
    let mut reroute = Vec::new();
    let mut paths = Vec::new();
    for (i, wire) in next.wires.iter().enumerate() {
        // Named aliases have no drawn route to modify.
        if wire.label.is_some() && !wire.draw_path {
            continue;
        }
        let internal = wire
            .from
            .pin()
            .zip(wire.to.pin())
            .is_some_and(|(a, b)| selected.contains(&a.symbol) && selected.contains(&b.symbol));
        if internal {
            reroute.push(i);
            continue;
        }
        if [&wire.from, &wire.to]
            .iter()
            .any(|e| e.pin().is_some_and(|p| moved.contains(&p.symbol)))
        {
            return Err(format!("Wire {:?} reaches a branch or outside the selection. Keep its symbol fixed or include both pin endpoints.", wire.id).into());
        }
        if wire.bends.is_none() {
            return Err(format!("Outside wire {:?} has an automatic display route. Author its path before connected arrangement.", wire.id).into());
        }
        let path = wiring::path(&next, wire)?;
        for id in &moved {
            let body = layout::bounds(&next, *id, 0)?;
            let component = next.component(*id).ok_or("Missing component.")?;
            let hits_pin = component
                .kind
                .pins()
                .iter()
                .filter_map(|p| {
                    next.pin(&PinRef {
                        symbol: *id,
                        terminal: p.id.into(),
                    })
                })
                .any(|p| path.windows(2).any(|s| wiring::on_segment(p, s[0], s[1])));
            if hits_pin || path.windows(2).any(|s| body.crosses(s[0], s[1])) {
                return Err(
                    format!("Symbol {id:?} would obstruct preserved wire {:?}.", wire.id).into(),
                );
            }
        }
        paths.push(path);
    }
    if reroute.len() > 64 {
        return Err("Connected arrangement supports at most 64 internal drawn connections.".into());
    }
    // Stored wire order is stable across save/reopen; no fresh IDs or random ties.
    for index in reroute {
        let wire = &next.wires[index];
        let (from, to) = wire
            .from
            .pin()
            .zip(wire.to.pin())
            .ok_or("Expected pin endpoints.")?;
        let bends = layout::route_filtered(&next, from, to, |path| {
            path.iter().all(|p| region.contains(*p))
                && !paths.iter().any(|other| shares_segment(path, other))
                && !next.junctions.iter().any(|j| path.windows(2).any(|s| wiring::on_segment(j.at, s[0],s[1])))
        }).map_err(|e| format!("Cannot arrange wire {:?} inside the region without obstructing bodies/pins or sharing wire segments: {e}", wire.id))?;
        let wire = &mut next.wires[index];
        wire.bends = Some(bends);
        wire.draw_path = true;
        paths.push(wiring::path(&next, &next.wires[index])?);
    }
    next.validate()?;
    let mut candidate = project.clone();
    let definition = next.sheet_id;
    *candidate
        .definition_mut(definition)
        .ok_or("Missing sheet definition.")? = next;
    candidate.validate()?;
    if candidate.connectivity()?.nets != connectivity {
        return Err("Connected arrangement changed terminal membership; proposal rejected.".into());
    }
    *project = candidate;
    Ok(())
}

#[cfg(test)]
mod tests;

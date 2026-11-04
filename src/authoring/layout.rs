//! Bounded deterministic helpers for the original catalog, not an autorouter.
use crate::{
    document::*,
    project::Project,
    session::{Command, Session},
    wiring,
};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
struct Bounds {
    min: Point,
    max: Point,
}
impl Bounds {
    fn overlaps(self, other: Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
    fn crosses(self, a: Point, b: Point) -> bool {
        if a.y == b.y {
            a.y > self.min.y
                && a.y < self.max.y
                && a.x.min(b.x) < self.max.x
                && a.x.max(b.x) > self.min.x
        } else {
            a.x > self.min.x
                && a.x < self.max.x
                && a.y.min(b.y) < self.max.y
                && a.y.max(b.y) > self.min.y
        }
    }
}
fn bounds(doc: &Document, id: SymbolId, margin: i64) -> Result<Bounds> {
    let s = doc
        .symbols
        .iter()
        .find(|s| s.id == id)
        .ok_or("Missing symbol.")?;
    let c = doc.component(id).ok_or("Missing component.")?;
    // Match the original catalog's body extents; layout reserves extra label space.
    let (mut x, mut y) = match c.kind {
        PartKind::Controller24 => (16, 40),
        PartKind::FanController => (15, 16),
        PartKind::DigitalTemperatureSensor => (8, 16),
        PartKind::Regulator | PartKind::TemperatureSensor | PartKind::Oscillator => (8, 8),
        PartKind::FanConnector => (7, 9),
        PartKind::Header6 => (6, 8),
        _ => (3, 3),
    };
    if s.quarter_turns % 2 == 1 {
        std::mem::swap(&mut x, &mut y);
    }
    Ok(Bounds {
        min: Point {
            x: s.at.x - x * GRID_NM - margin,
            y: s.at.y - y * GRID_NM - margin,
        },
        max: Point {
            x: s.at.x + x * GRID_NM + margin,
            y: s.at.y + y * GRID_NM + margin,
        },
    })
}
pub(super) fn arrange(
    project: &mut Project,
    sheet: SheetOccurrenceId,
    ids: &[SymbolId],
    origin: Point,
    columns: usize,
    spacing: Point,
) -> Result<()> {
    let connectivity = project.connectivity()?.nets;
    if ids.is_empty()
        || ids.len() > 64
        || columns == 0
        || columns > 8
        || !origin.valid()
        || !spacing.valid()
        || spacing.x < 16 * GRID_NM
        || spacing.y < 16 * GRID_NM
        || ids.iter().copied().collect::<BTreeSet<_>>().len() != ids.len()
    {
        return Err("Arrange needs 1–64 distinct symbols, 1–8 columns and bounded spacing of at least 16 grid units.".into());
    }
    for (i, id) in ids.iter().enumerate() {
        let at = Point {
            x: origin.x + (i % columns) as i64 * spacing.x,
            y: origin.y + (i / columns) as i64 * spacing.y,
        };
        if !at.valid() {
            return Err("Arrangement exceeds coordinate limits.".into());
        }
        Session::edit_candidate(project, sheet, Command::Move { symbol: *id, at })?;
    }
    let doc = project.document(sheet).ok_or("Unknown sheet.")?;
    for id in ids {
        let a = bounds(doc, *id, 6 * GRID_NM)?;
        for other in &doc.symbols {
            if other.id != *id && a.overlaps(bounds(doc, other.id, 6 * GRID_NM)?) {
                return Err("Arrangement overlaps another symbol's reserved body/label area. Increase spacing or choose a clear region.".into());
            }
        }
    }
    if project.connectivity()?.nets != connectivity {
        return Err("Arrangement would change connectivity; proposal rejected.".into());
    }
    Ok(())
}
pub(super) fn route(doc: &Document, from: &PinRef, to: &PinRef) -> Result<Vec<Point>> {
    let a = doc
        .pin(from)
        .ok_or("Unknown source terminal; inspect the catalog's pin IDs.")?;
    let b = doc
        .pin(to)
        .ok_or("Unknown destination terminal; inspect the catalog's pin IDs.")?;
    if a == b {
        return Err("Pins coincide; move the symbols before connecting.".into());
    }
    let mut obstacles = doc
        .symbols
        .iter()
        .map(|s| bounds(doc, s.id, 0))
        .collect::<Result<Vec<_>>>()?;
    for s in &doc.symbols {
        let c = doc.component(s.id).ok_or("Missing component.")?;
        for pin in c.kind.pins() {
            let r = PinRef {
                symbol: s.id,
                terminal: pin.id.into(),
            };
            if &r != from
                && &r != to
                && let Some(p) = doc.pin(&r)
            {
                obstacles.push(Bounds {
                    min: Point {
                        x: p.x - GRID_NM / 2,
                        y: p.y - GRID_NM / 2,
                    },
                    max: Point {
                        x: p.x + GRID_NM / 2,
                        y: p.y + GRID_NM / 2,
                    },
                });
            }
        }
    }
    let mut paths = vec![
        vec![a, b],
        vec![a, Point { x: b.x, y: a.y }, b],
        vec![a, Point { x: a.x, y: b.y }, b],
    ];
    let mut xs = vec![(a.x + b.x) / 2];
    let mut ys = vec![(a.y + b.y) / 2];
    for r in obstacles.iter().take(128) {
        xs.extend([r.min.x - 2 * GRID_NM, r.max.x + 2 * GRID_NM]);
        ys.extend([r.min.y - 2 * GRID_NM, r.max.y + 2 * GRID_NM]);
    }
    for x in xs {
        paths.push(vec![a, Point { x, y: a.y }, Point { x, y: b.y }, b]);
    }
    for y in ys {
        paths.push(vec![a, Point { x: a.x, y }, Point { x: b.x, y }, b]);
    }
    let path = paths.into_iter().map(|mut p| { wiring::simplify(&mut p); p })
        .filter(|p| wiring::validate_path(p).is_ok() && p.windows(2).all(|s| !obstacles.iter().any(|r| r.crosses(s[0],s[1]))))
        .min_by_key(|p| p.windows(2).map(|s| (s[0].x-s[1].x).abs()+(s[0].y-s[1].y).abs()).sum::<i64>() + p.len() as i64 * 2 * GRID_NM)
        .ok_or("No bounded orthogonal path avoids symbol bodies and unrelated pins. Move parts or supply reviewed bends.")?;
    Ok(path[1..path.len() - 1].to_vec())
}

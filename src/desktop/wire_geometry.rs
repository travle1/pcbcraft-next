//! Bounded orthogonal display routing. This does not author or infer connectivity.
use eframe::egui::{Pos2, Rect, Vec2, pos2};

pub fn route(
    a: Pos2,
    da: Vec2,
    b: Pos2,
    db: Vec2,
    obstacles: &[Rect],
    gap: f32,
) -> Option<Vec<Pos2>> {
    let mut candidates = vec![
        vec![a, b],
        vec![a, pos2(b.x, a.y), b],
        vec![a, pos2(a.x, b.y), b],
    ];
    let s = a + da * gap;
    let e = b + db * gap;
    let mut xs = vec![(a.x + b.x) * 0.5, s.x, e.x];
    let mut ys = vec![(a.y + b.y) * 0.5, s.y, e.y];
    let region = Rect::from_two_pos(a, b).expand(gap * 4.0);
    for r in obstacles.iter().filter(|r| r.intersects(region)).take(32) {
        xs.extend([r.left() - gap, r.right() + gap]);
        ys.extend([r.top() - gap, r.bottom() + gap]);
    }
    for x in xs {
        candidates.push(vec![a, pos2(x, a.y), pos2(x, b.y), b]);
        candidates.push(vec![a, s, pos2(x, s.y), pos2(x, e.y), e, b]);
    }
    for y in ys {
        candidates.push(vec![a, pos2(a.x, y), pos2(b.x, y), b]);
        candidates.push(vec![a, s, pos2(s.x, y), pos2(e.x, y), e, b]);
    }
    candidates
        .into_iter()
        .map(simplify)
        .filter(|p| {
            p.len() >= 2
                && (p[1] - a).dot(da) >= -0.01
                && (p[p.len() - 2] - b).dot(db) >= -0.01
                && p.windows(2)
                    .all(|s| axis(s[0], s[1]) && !obstacles.iter().any(|r| crosses(s[0], s[1], *r)))
                && p.windows(3)
                    .all(|s| (s[1] - s[0]).dot(s[2] - s[1]) >= -0.01)
        })
        .min_by(|a, b| score(a, gap).total_cmp(&score(b, gap)))
}
fn score(path: &[Pos2], gap: f32) -> f32 {
    path.windows(2).map(|s| (s[1] - s[0]).length()).sum::<f32>()
        + path.len().saturating_sub(2) as f32 * gap * 2.0
}
fn simplify(points: Vec<Pos2>) -> Vec<Pos2> {
    let mut out: Vec<Pos2> = Vec::new();
    for p in points {
        if out.last().is_some_and(|q| q.distance(p) < 0.01) {
            continue;
        }
        while out.len() >= 2 {
            let a = out[out.len() - 2];
            let b = out[out.len() - 1];
            if axis(a, p) && axis(a, b) && axis(b, p) && (b - a).dot(p - b) >= 0.0 {
                out.pop();
            } else {
                break;
            }
        }
        out.push(p);
    }
    out
}
fn axis(a: Pos2, b: Pos2) -> bool {
    (a.x - b.x).abs() < 0.01 || (a.y - b.y).abs() < 0.01
}
fn crosses(a: Pos2, b: Pos2, r: Rect) -> bool {
    if (a.y - b.y).abs() < 0.01 {
        a.y > r.top() && a.y < r.bottom() && a.x.min(b.x) < r.right() && a.x.max(b.x) > r.left()
    } else {
        a.x > r.left() && a.x < r.right() && a.y.min(b.y) < r.bottom() && a.y.max(b.y) > r.top()
    }
}
fn on_segment(p: Pos2, a: Pos2, b: Pos2) -> bool {
    axis(a, p) && axis(p, b) && Rect::from_two_pos(a, b).expand(0.01).contains(p)
}
/// Junction dots require three distinct directions within the SAME logical net.
/// Ordinary bends, duplicate strokes and crossings between nets get no dot.
pub fn junctions(paths: &[Vec<Pos2>]) -> Vec<Pos2> {
    let mut points = Vec::new();
    for p in paths.iter().flatten() {
        if points.iter().any(|q: &Pos2| q.distance(*p) < 0.01) {
            continue;
        }
        let mut directions = [false; 4];
        for segment in paths.iter().flat_map(|p| p.windows(2)) {
            if on_segment(*p, segment[0], segment[1]) {
                for q in segment {
                    if q.x < p.x - 0.01 {
                        directions[0] = true;
                    }
                    if q.x > p.x + 0.01 {
                        directions[1] = true;
                    }
                    if q.y < p.y - 0.01 {
                        directions[2] = true;
                    }
                    if q.y > p.y + 0.01 {
                        directions[3] = true;
                    }
                }
            }
        }
        if directions.iter().filter(|d| **d).count() >= 3 {
            points.push(*p);
        }
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aligned_and_perpendicular_pins_do_not_get_midpoint_jogs() {
        let a = pos2(0.0, 0.0);
        let b = pos2(100.0, 0.0);
        assert_eq!(route(a, Vec2::X, b, -Vec2::Y, &[], 10.0), Some(vec![a, b]));
        let b = pos2(100.0, 60.0);
        assert_eq!(
            route(a, Vec2::X, b, -Vec2::Y, &[], 10.0),
            Some(vec![a, pos2(100.0, 0.0), b])
        );
    }
    #[test]
    fn routing_avoids_body_and_never_departures_back_into_pin() {
        let a = pos2(0.0, 0.0);
        let b = pos2(100.0, 0.0);
        let obstacle = Rect::from_min_max(pos2(35.0, -20.0), pos2(65.0, 20.0));
        let p = route(a, Vec2::X, b, -Vec2::X, &[obstacle], 10.0);
        assert!(
            p.as_ref()
                .is_some_and(|p| p.windows(2).all(|s| !crosses(s[0], s[1], obstacle)))
        );
        assert!(
            route(
                a,
                Vec2::X,
                b,
                -Vec2::X,
                &[Rect::from_center_size(a, Vec2::splat(20.0))],
                10.0
            )
            .is_none()
        );
    }
    #[test]
    fn dots_are_branches_not_bends_or_duplicate_endpoints() {
        let a = pos2(0.0, 0.0);
        let b = pos2(50.0, 0.0);
        let c = pos2(100.0, 0.0);
        let d = pos2(50.0, 50.0);
        assert!(junctions(&[vec![a, b, d]]).is_empty());
        assert!(junctions(&[vec![a, c], vec![a, c]]).is_empty());
        assert_eq!(junctions(&[vec![a, c], vec![b, d]]), vec![b]);
    }
}

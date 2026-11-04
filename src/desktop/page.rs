//! Page guide and immediate 2D navigation, independent of electrical edits.
use super::{canvas::Canvas, theme};
use eframe::egui::{self, Pos2, Rect, Shape, Stroke, Vec2};
use pcbcraft_foundation::{document::Point, page::Page};
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum Guide {
    Auto,
    #[default]
    Always,
    Off,
}
impl Canvas {
    pub(super) fn fit_page_bounds(&mut self, page: Page, rect: Rect) {
        let (w, h) = page.dimensions_nm();
        let available = (rect.size() - Vec2::splat(36.0)).max(Vec2::splat(1.0));
        self.zoom = (available.x / (w as f32 / 1e6 * 8.0))
            .min(available.y / (h as f32 / 1e6 * 8.0))
            .clamp(0.025, 0.75);
        self.offset = Vec2::ZERO;
        self.fit = false;
    }
    pub(super) fn navigate(&mut self, events: &[egui::Event], at: Pos2, viewport: Rect) {
        let mut exponent = 0.0f32;
        let mut pan = 0.0;
        for event in events {
            match event {
                egui::Event::MouseWheel { unit, delta, .. } if delta.is_finite() => {
                    let scale = match unit {
                        egui::MouseWheelUnit::Point => 1.0,
                        egui::MouseWheelUnit::Line => 40.0,
                        egui::MouseWheelUnit::Page => viewport.height(),
                    };
                    exponent += (delta.y * scale * 0.004).clamp(-1.5, 1.5);
                    pan += (delta.x * scale).clamp(-2000.0, 2000.0);
                }
                egui::Event::Zoom(factor) if factor.is_finite() && *factor > 0.0 => {
                    exponent += factor.ln().clamp(-1.5, 1.5)
                }
                _ => {}
            }
        }
        if exponent != 0.0 {
            let before = (at - viewport.center() - self.offset) / self.zoom;
            self.zoom = (self.zoom * exponent.clamp(-2.0, 2.0).exp()).clamp(0.025, 8.0);
            self.offset = at - viewport.center() - before * self.zoom;
            self.fit = false;
        }
        self.offset.x += pan;
    }
    pub(super) fn paint_page(
        &self,
        p: &egui::Painter,
        page: Page,
        viewport: Rect,
        pointer: Option<Pos2>,
        placing: bool,
    ) {
        if self.page_guide == Guide::Off {
            return;
        }
        let (w, h) = page.dimensions_nm();
        let r = Rect::from_min_max(
            self.screen(
                Point {
                    x: -w / 2,
                    y: -h / 2,
                },
                viewport,
            ),
            self.screen(Point { x: w / 2, y: h / 2 }, viewport),
        );
        let ink = theme::current(p.ctx()).canvas.page_edge;
        for (mut a, mut b) in [
            (r.left_top(), r.right_top()),
            (r.right_top(), r.right_bottom()),
            (r.left_bottom(), r.right_bottom()),
            (r.left_top(), r.left_bottom()),
        ] {
            let origin = a;
            let horizontal = (a.y - b.y).abs() < 0.01;
            let mut alpha = 1.0;
            if self.page_guide == Guide::Auto {
                let Some(at) = pointer.filter(|_| placing) else {
                    continue;
                };
                let distance = if horizontal {
                    (at.y - a.y).abs()
                } else {
                    (at.x - a.x).abs()
                };
                alpha = (1.0 - distance / 72.0).clamp(0.0, 1.0);
                if alpha == 0.0 {
                    continue;
                }
                if horizontal {
                    let lo = a.x.min(b.x);
                    let hi = a.x.max(b.x);
                    a.x = (at.x - 120.0).clamp(lo, hi);
                    b.x = (at.x + 120.0).clamp(lo, hi);
                } else {
                    let lo = a.y.min(b.y);
                    let hi = a.y.max(b.y);
                    a.y = (at.y - 120.0).clamp(lo, hi);
                    b.y = (at.y + 120.0).clamp(lo, hi);
                }
            }
            // Clip before tessellation: an A2 edge at high zoom must not create
            // tens of thousands of offscreen dashes. Retain the original phase.
            let clip = p.clip_rect();
            if horizontal {
                if a.y < clip.top() || a.y > clip.bottom() {
                    continue;
                }
                a.x = a.x.max(clip.left());
                b.x = b.x.min(clip.right());
                if a.x >= b.x {
                    continue;
                }
            } else {
                if a.x < clip.left() || a.x > clip.right() {
                    continue;
                }
                a.y = a.y.max(clip.top());
                b.y = b.y.min(clip.bottom());
                if a.y >= b.y {
                    continue;
                }
            }
            p.extend(Shape::dashed_line_with_offset(
                &[a, b],
                Stroke::new(0.8, ink.gamma_multiply(alpha)),
                &[5.0],
                &[5.0],
                a.distance(origin) % 10.0,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wheel_is_immediate_cursor_anchored_invertible_and_has_no_idle_tail() {
        let mut c = Canvas::default();
        let viewport = Rect::from_min_size(Pos2::ZERO, egui::vec2(900.0, 600.0));
        c.fit_page_bounds(Page::default(), viewport);
        let at = egui::pos2(600.0, 200.0);
        let before = (at - viewport.center() - c.offset) / c.zoom;
        let zoom = c.zoom;
        let wheel = |y| egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            phase: egui::TouchPhase::Move,
            delta: egui::vec2(0.0, y),
            modifiers: egui::Modifiers::NONE,
        };
        c.navigate(&[wheel(1.0)], at, viewport);
        assert!(c.zoom > zoom * 1.15);
        assert!(((at - viewport.center() - c.offset) / c.zoom - before).length() < 0.001);
        let changed = (c.zoom, c.offset);
        c.navigate(&[], at, viewport);
        assert_eq!(changed, (c.zoom, c.offset));
        c.navigate(&[wheel(-1.0)], at, viewport);
        assert!((c.zoom - zoom).abs() < 1e-6);
        c.navigate(&[egui::Event::Zoom(2.0)], at, viewport);
        assert!((c.zoom - zoom * 2.0).abs() < 1e-6);
        let safe = (c.zoom, c.offset);
        c.navigate(
            &[wheel(f32::NAN), egui::Event::Zoom(f32::INFINITY)],
            at,
            viewport,
        );
        assert_eq!(safe, (c.zoom, c.offset));
    }
    #[test]
    fn page_fit_contains_full_frame_and_guides_are_clipped_and_optional() {
        let mut c = Canvas::default();
        let viewport = Rect::from_min_size(Pos2::ZERO, egui::vec2(900.0, 600.0));
        let page = Page::default();
        let (w, h) = page.dimensions_nm();
        c.fit_page_bounds(page, viewport);
        for point in [
            Point {
                x: -w / 2,
                y: -h / 2,
            },
            Point { x: w / 2, y: h / 2 },
        ] {
            assert!(viewport.shrink(17.0).contains(c.screen(point, viewport)));
        }
        let ctx = egui::Context::default();
        theme::apply(&ctx);
        let paint = |c: &Canvas, at, placing| {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(viewport),
                    ..Default::default()
                },
                |ui| {
                    let p = ui
                        .ctx()
                        .layer_painter(egui::LayerId::background())
                        .with_clip_rect(viewport);
                    c.paint_page(&p, page, viewport, at, placing);
                },
            )
        };
        let count = |out: egui::FullOutput| {
            let n = out
                .shapes
                .iter()
                .filter(|s| matches!(s.shape, Shape::LineSegment { .. }))
                .count();
            out.drop_without_applying_deltas();
            n
        };
        assert!(count(paint(&c, None, false)) > 100);
        c.page_guide = Guide::Off;
        assert_eq!(count(paint(&c, None, false)), 0);
        c.page_guide = Guide::Auto;
        let edge = c.screen(Point { x: 0, y: -h / 2 }, viewport);
        assert_eq!(count(paint(&c, Some(edge), false)), 0);
        assert!(count(paint(&c, Some(edge), true)) > 0);
        assert_eq!(count(paint(&c, Some(viewport.center()), true)), 0);
        c.page_guide = Guide::Always;
        c.zoom = 8.0;
        c.offset = egui::vec2(0.0, h as f32 / 2e6 * 8.0 * c.zoom);
        let n = count(paint(&c, None, false));
        assert!(
            n > 0 && n < 120,
            "Only visible edge dashes should be emitted: {n}"
        );
    }
}

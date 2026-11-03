use super::wire_geometry;
use super::{
    App, Tool,
    controls::{self, MenuItem},
    icons::Icon,
    theme,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, Vec2};
use pcbcraft_foundation::wiring::{self, Anchor, Projection, WireEdit};
use pcbcraft_foundation::{document::*, session::Command};

pub struct Canvas {
    pub zoom: f32,
    pub offset: Vec2,
    pub fit: bool,
    pub grid: bool,
    pub snap: bool,
    pub symbols_selectable: bool,
    pub pins_selectable: bool,
    pub nets_selectable: bool,
    pub pins_visible: bool,
    pub wires_visible: bool,
    pub drag: Option<(SymbolId, Point, Point)>,
    pub wire_start: Option<WireDraft>,
    pub wire_drag: Option<WireDrag>,
    pub capture_wire_hit: Option<Pos2>,
    routes: Vec<Projection>,
    context: Option<CanvasContext>,
    pub(super) context_open: bool,
    pub last_world: Point,
}
#[derive(Clone)]
pub struct WireDraft {
    pub from: Anchor,
    pub points: Vec<Point>,
    pub incarnation: ProjectId,
    pub revision: u64,
    pub vertical_first: bool,
}
#[derive(Clone)]
pub struct WireDrag {
    edit: WireEdit,
    routes: Vec<Projection>,
    origin: Point,
    initial: Point,
    incarnation: ProjectId,
    revision: u64,
}
#[derive(Clone)]
struct CanvasContext {
    wire: Option<WireId>,
    anchor: Option<Endpoint>,
    symbol: Option<SymbolId>,
    target: Anchor,
    incarnation: ProjectId,
    revision: u64,
}
impl Default for Canvas {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: Vec2::ZERO,
            fit: true,
            grid: true,
            snap: true,
            symbols_selectable: true,
            pins_selectable: true,
            nets_selectable: true,
            pins_visible: true,
            wires_visible: true,
            drag: None,
            wire_start: None,
            wire_drag: None,
            capture_wire_hit: None,
            routes: Vec::new(),
            context: None,
            context_open: false,
            last_world: Point::default(),
        }
    }
}
impl Canvas {
    pub(super) fn screen(&self, p: Point, rect: Rect) -> Pos2 {
        rect.center()
            + self.offset
            + egui::vec2(p.x as f32 / 1_000_000.0, p.y as f32 / 1_000_000.0) * 8.0 * self.zoom
    }
    pub(super) fn world(&self, p: Pos2, rect: Rect) -> Point {
        let v = (p - rect.center() - self.offset) / (8.0 * self.zoom);
        let at = Point {
            x: (v.x as f64 * 1_000_000.0)
                .round()
                .clamp(-LIMIT_NM as f64, LIMIT_NM as f64) as i64,
            y: (v.y as f64 * 1_000_000.0)
                .round()
                .clamp(-LIMIT_NM as f64, LIMIT_NM as f64) as i64,
        };
        if self.snap { at.snapped() } else { at }
    }
    fn shown_symbol(&self, s: &Symbol) -> Symbol {
        let mut s = s.clone();
        if let Some((id, _, at)) = self.drag
            && id == s.id
        {
            s.at = at;
        }
        s
    }
    fn pin(&self, doc: &Document, p: &PinRef) -> Option<Point> {
        self.shown_symbol(doc.symbols.iter().find(|s| s.id == p.symbol)?)
            .pin(doc.component(p.symbol)?.kind, &p.terminal)
    }
    fn endpoint(&self, doc: &Document, e: &Endpoint) -> Option<Point> {
        match e {
            Endpoint::Pin(p) => self.pin(doc, p),
            _ => doc.endpoint(e),
        }
    }
    fn direction(&self, doc: &Document, e: &Endpoint) -> Vec2 {
        e.pin().map_or(Vec2::ZERO, |p| self.pin_direction(doc, p))
    }
    fn pin_direction(&self, doc: &Document, p: &PinRef) -> Vec2 {
        let Some(c) = doc.component(p.symbol) else {
            return Vec2::ZERO;
        };
        let Some(pin) = c.kind.pins().iter().find(|pin| pin.id == p.terminal) else {
            return Vec2::ZERO;
        };
        let Some(symbol) = doc.symbols.iter().find(|s| s.id == p.symbol) else {
            return Vec2::ZERO;
        };
        let x = pin.x.signum() as f32;
        match self.shown_symbol(symbol).quarter_turns {
            0 => egui::vec2(x, 0.0),
            1 => egui::vec2(0.0, x),
            2 => egui::vec2(-x, 0.0),
            _ => egui::vec2(0.0, -x),
        }
    }
    fn fit_document(&mut self, doc: &Document, rect: Rect) {
        let mut bounds = Rect::NOTHING;
        for s in &doc.symbols {
            let center = Pos2::new(s.at.x as f32 / 1e6, s.at.y as f32 / 1e6);
            let size = doc.component(s.id).map_or(egui::vec2(100.0, 64.0), |c| {
                symbol_size(c.kind, s.quarter_turns)
            });
            bounds = bounds.union(Rect::from_center_size(center, size / 8.0));
        }
        for p in doc
            .junctions
            .iter()
            .map(|j| &j.at)
            .chain(doc.wires.iter().flat_map(|w| w.bends.iter().flatten()))
        {
            bounds.extend_with(Pos2::new(p.x as f32 / 1e6, p.y as f32 / 1e6));
        }
        for a in &doc.annotations {
            bounds.extend_with(Pos2::new(a.at.x as f32 / 1e6, a.at.y as f32 / 1e6));
        }
        if bounds.is_finite() {
            bounds = bounds.expand(18.0);
            self.zoom = (rect.width() / (bounds.width() * 8.0))
                .min(rect.height() / (bounds.height() * 8.0))
                .clamp(0.1, 0.75);
            self.offset = -bounds.center().to_vec2() * 8.0 * self.zoom;
        } else {
            self.offset = Vec2::ZERO;
            self.zoom = 1.0;
        }
        self.fit = false;
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui, input_enabled: bool) {
    app.refresh_nets();
    let t = theme::current(ui.ctx());
    let full = ui.max_rect();
    let response = ui.allocate_rect(full, Sense::click_and_drag());
    let side = 260.0;
    let left_side = side;
    let input_enabled = input_enabled
        && !app.library_ui.moving()
        && !app.inspector.moving()
        && !ui.input(|i| {
            i.pointer.interact_pos().is_some_and(|p| {
                app.layout.blocks_canvas(p) || (app.library && app.library_ui.rect.contains(p))
            })
        });
    let canvas = Rect::from_min_max(
        full.min + egui::vec2(left_side + 24.0, 52.0),
        full.max - egui::vec2(side + 28.0, 85.0),
    );
    let mut doc = app.session.document().clone();
    if let Some(drag) = &app.canvas.wire_drag {
        if drag.incarnation == app.session.incarnation && drag.revision == app.session.revision() {
            if let Ok(preview) = wiring::preview_edit(
                &doc,
                &WireEdit::WithRoutes {
                    edit: Box::new(drag.edit.clone()),
                    routes: drag.routes.clone(),
                },
            ) {
                doc = preview;
            }
        } else {
            app.canvas.wire_drag = None;
        }
    }
    if app.canvas.fit {
        let mut visible = canvas;
        if app.layout.drawer.is_finite() {
            visible.max.y = visible
                .bottom()
                .min(app.layout.navigation.top() - 60.0)
                .max(visible.top() + 80.0);
        }
        app.canvas.fit_document(&doc, visible);
        app.canvas.offset += visible.center() - canvas.center();
    }
    let painter = ui.painter().with_clip_rect(full);
    painter.rect_filled(full, 0.0, t.canvas.background);
    if app.canvas.grid {
        let mut step = GRID_NM as f32 / 1e6 * 8.0 * app.canvas.zoom;
        while step < t.schematic.grid_min_spacing {
            step *= 5.0;
        }
        let origin = app.canvas.screen(Point::default(), canvas);
        let first_x = ((full.left() - origin.x) / step).ceil() as i32;
        let last_x = ((full.right() - origin.x) / step).floor() as i32;
        for index in first_x..=last_x {
            let x = origin.x + index as f32 * step;
            let colour = if index.rem_euclid(5) == 0 {
                t.canvas.grid_major
            } else {
                t.canvas.grid
            };
            painter.line_segment(
                [Pos2::new(x, full.top()), Pos2::new(x, full.bottom())],
                Stroke::new(0.5, colour),
            );
        }
        let first_y = ((full.top() - origin.y) / step).ceil() as i32;
        let last_y = ((full.bottom() - origin.y) / step).floor() as i32;
        for index in first_y..=last_y {
            let y = origin.y + index as f32 * step;
            let colour = if index.rem_euclid(5) == 0 {
                t.canvas.grid_major
            } else {
                t.canvas.grid
            };
            painter.line_segment(
                [Pos2::new(full.left(), y), Pos2::new(full.right(), y)],
                Stroke::new(0.5, colour),
            );
        }
    }
    painter.text(
        Pos2::new(
            if app.layout.browser.is_finite() {
                app.layout.browser.right() + 12.0
            } else {
                canvas.left()
            },
            full.top() + 24.0,
        ),
        Align2::LEFT_CENTER,
        format!("{}  /  Main", doc.name),
        FontId::proportional(14.0),
        t.ui.text,
    );
    painter.text(
        Pos2::new(canvas.right(), full.top() + 24.0),
        Align2::RIGHT_CENTER,
        "1 / 1",
        FontId::proportional(12.0),
        t.ui.muted,
    );

    let pointer = response.hover_pos().filter(|_| input_enabled);
    if let Some(pos) = pointer {
        app.canvas.last_world = app.canvas.world(pos, canvas);
        if !app.busy() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.1 {
                let before = (pos - canvas.center() - app.canvas.offset) / app.canvas.zoom;
                app.canvas.zoom = (app.canvas.zoom * (scroll * 0.002).exp()).clamp(0.1, 4.0);
                app.canvas.offset = pos - canvas.center() - before * app.canvas.zoom;
            }
        }
    }
    if input_enabled
        && (response.dragged_by(egui::PointerButton::Middle)
            || (app.tool == Tool::Pan && response.dragged()))
    {
        app.canvas.offset += ui.input(|i| i.pointer.delta());
    }
    let mut hit = None;
    let mut pin_hit = None;
    let mut nearest_pin = 10.0;
    for s in &doc.symbols {
        let Some(component) = doc.component(s.id) else {
            continue;
        };
        let shown = app.canvas.shown_symbol(s);
        let center = app.canvas.screen(shown.at, canvas);
        if let Some(pos) = pointer {
            let hit_size = symbol_size(component.kind, shown.quarter_turns);
            if app.canvas.symbols_selectable
                && Rect::from_center_size(center, hit_size * app.canvas.zoom).contains(pos)
            {
                hit = Some(s.id);
            }
            for definition in component.kind.pins() {
                let terminal = definition.id;
                if app.canvas.pins_selectable
                    && let Some(p) = shown.pin(component.kind, terminal)
                    && app.canvas.screen(p, canvas).distance(pos) < nearest_pin
                {
                    nearest_pin = app.canvas.screen(p, canvas).distance(pos);
                    pin_hit = Some(PinRef {
                        symbol: s.id,
                        terminal: terminal.into(),
                    });
                }
            }
        }
    }
    for annotation in &doc.annotations {
        painter.text(
            app.canvas.screen(annotation.at, canvas),
            Align2::LEFT_BOTTOM,
            &annotation.text,
            FontId::proportional(
                (22.0 * app.canvas.zoom)
                    .clamp(if app.canvas.zoom >= 0.25 { 10.0 } else { 6.0 }, 22.0),
            ),
            t.canvas.symbol,
        );
    }
    let selected_net = app.nets.selected_net().cloned();
    let mut hit_net = None;
    let mut hit_wire: Option<(WireId, Vec<Pos2>)> = None;
    let mut projected = Vec::<(WireId, Vec<Pos2>)>::new();
    let mut nearest_link = 5.0;
    if app.canvas.wires_visible {
        let gap = 20.32 * app.canvas.zoom;
        let bodies: Vec<_> = doc
            .symbols
            .iter()
            .filter_map(|s| {
                let c = doc.component(s.id)?;
                let shown = app.canvas.shown_symbol(s);
                let center = app.canvas.screen(shown.at, canvas);
                let (w, h) = if c.kind.is_block() {
                    block_half_size(c.kind)
                } else {
                    (26.0, 32.0)
                };
                let half = if shown.quarter_turns.is_multiple_of(2) {
                    egui::vec2(w, h)
                } else {
                    egui::vec2(h, w)
                };
                Some(
                    Rect::from_center_size(center, half * 2.0 * app.canvas.zoom).expand(gap * 0.12),
                )
            })
            .collect();
        let mut net_paths = std::collections::BTreeMap::<Endpoint, Vec<Vec<Pos2>>>::new();
        let mut occupied = Vec::<(Endpoint, Rect)>::new();
        let mut routed_pins = std::collections::BTreeSet::new();
        let drawn_groups = drawn_groups(&doc);
        let mut path_labels = std::collections::BTreeSet::new();
        let mut ordered_wires: Vec<_> = doc.wires.iter().collect();
        ordered_wires.sort_by_cached_key(|w| {
            let a = app.canvas.endpoint(&doc, &w.from).unwrap_or_default();
            let b = app.canvas.endpoint(&doc, &w.to).unwrap_or_default();
            (a.y.min(b.y), a.x.min(b.x), a.y.max(b.y), a.x.max(b.x))
        });
        let mut aliases = std::collections::BTreeSet::new();
        for wire in ordered_wires {
            let highlighted = selected_net
                .as_ref()
                .is_some_and(|n| n.links.binary_search(&wire.id).is_ok());
            let wire_color = if highlighted {
                t.canvas.selection
            } else {
                t.canvas.wire
            };
            let width = t.schematic.wire_width + if highlighted { 0.8 } else { 0.0 };
            if let Some(label) = &wire.label
                && !wire.draw_path
            {
                for pin in [&wire.from, &wire.to] {
                    if drawn_groups.contains_key(pin) || !aliases.insert(pin.clone()) {
                        continue;
                    }
                    if let Some(at) = app.canvas.endpoint(&doc, pin) {
                        let a = app.canvas.screen(at, canvas);
                        let dir = app.canvas.direction(&doc, pin);
                        let b = a + dir * (32.0 * app.canvas.zoom).max(10.0);
                        painter.line_segment([a, b], Stroke::new(width, wire_color));
                        if app.canvas.nets_selectable
                            && let Some(p) = pointer
                        {
                            let distance = path_distance(p, &[a, b]);
                            if distance < nearest_link {
                                nearest_link = distance;
                                hit_net = Some(wire.from.clone());
                                hit_wire = Some((wire.id, Vec::new()));
                            }
                        }
                        let (anchor, align) = if dir.x < 0.0 {
                            (b - egui::vec2(2.0, 3.0), Align2::RIGHT_BOTTOM)
                        } else if dir.x > 0.0 {
                            (b + egui::vec2(2.0, -3.0), Align2::LEFT_BOTTOM)
                        } else if dir.y < 0.0 {
                            (b - egui::vec2(0.0, 3.0), Align2::CENTER_BOTTOM)
                        } else {
                            (b + egui::vec2(0.0, 3.0), Align2::CENTER_TOP)
                        };
                        painter.text(
                            anchor,
                            align,
                            label,
                            FontId::proportional(
                                (15.0 * app.canvas.zoom)
                                    .clamp(if app.canvas.zoom >= 0.25 { 8.5 } else { 6.0 }, 18.0),
                            ),
                            if highlighted {
                                t.canvas.selection
                            } else {
                                t.canvas.value
                            },
                        );
                    }
                }
                continue;
            }
            if let (Some(a), Some(b)) = (
                app.canvas.endpoint(&doc, &wire.from),
                app.canvas.endpoint(&doc, &wire.to),
            ) {
                let a = app.canvas.screen(a, canvas);
                let b = app.canvas.screen(b, canvas);
                let da = app.canvas.direction(&doc, &wire.from);
                let db = app.canvas.direction(&doc, &wire.to);
                let net = app.nets.snapshot.containing(&wire.from);
                let anchor = net.map_or_else(|| wire.from.clone(), |n| n.anchor.clone());
                let mut obstacles = bodies.clone();
                for symbol in &doc.symbols {
                    if let Some(c) = doc.component(symbol.id) {
                        for p in c.kind.pins() {
                            let pin = PinRef {
                                symbol: symbol.id,
                                terminal: p.id.into(),
                            };
                            if net.is_some_and(|n| n.terminals.contains(&pin)) {
                                continue;
                            }
                            if let Some(at) = app.canvas.pin(&doc, &pin) {
                                obstacles.push(Rect::from_center_size(
                                    app.canvas.screen(at, canvas),
                                    Vec2::splat(gap * 0.3),
                                ));
                            }
                        }
                    }
                }
                let mut clear = obstacles.clone();
                clear.extend(
                    occupied
                        .iter()
                        .filter(|(n, _)| *n != anchor)
                        .map(|(_, r)| *r),
                );
                let points = if let Some(bends) = &wire.bends {
                    let mut points = vec![a];
                    points.extend(bends.iter().map(|p| app.canvas.screen(*p, canvas)));
                    points.push(b);
                    Some(points)
                } else {
                    wire_geometry::route(a, da, b, db, &clear, gap)
                        .or_else(|| wire_geometry::route(a, da, b, db, &obstacles, gap))
                };
                let Some(points) = points else {
                    // Connectivity remains explicit, but an obstructed display route is not drawn through a body.
                    for (p, dir) in [(a, da), (b, db)] {
                        painter.line_segment([p, p + dir * gap], Stroke::new(width, t.ui.warning));
                    }
                    painter.text(
                        a + da * gap,
                        Align2::LEFT_BOTTOM,
                        "Unrouted",
                        FontId::proportional(10.0),
                        t.ui.warning,
                    );
                    continue;
                };
                projected.push((wire.id, points.clone()));
                for segment in points.windows(2) {
                    occupied.push((
                        anchor.clone(),
                        Rect::from_two_pos(segment[0], segment[1]).expand(gap * 0.16),
                    ));
                }
                net_paths
                    .entry(anchor.clone())
                    .or_default()
                    .push(points.clone());
                for (pin, p, dir) in [(&wire.from, a, da), (&wire.to, b, db)] {
                    if pin.pin().is_some() && routed_pins.insert(pin.clone()) {
                        net_paths
                            .entry(anchor.clone())
                            .or_default()
                            .push(vec![p, p - dir * gap * 0.5]);
                    }
                }
                if app.canvas.nets_selectable
                    && let Some(p) = pointer
                {
                    let distance = path_distance(p, &points);
                    if distance < nearest_link {
                        nearest_link = distance;
                        hit_net = Some(wire.from.clone());
                        hit_wire = Some((wire.id, points.clone()));
                    }
                }
                if let Some(label) = net.and_then(|n| n.name.as_ref())
                    && let Some(drawn_anchor) = drawn_groups.get(&wire.from)
                    && path_labels.insert(drawn_anchor.clone())
                {
                    painter.text(
                        points
                            .windows(2)
                            .filter(|p| (p[0].y - p[1].y).abs() < 0.01)
                            .max_by(|x, y| x[0].distance(x[1]).total_cmp(&y[0].distance(y[1])))
                            .map_or(a, |p| p[0].lerp(p[1], 0.5))
                            - egui::vec2(0.0, 5.0),
                        Align2::CENTER_BOTTOM,
                        label,
                        FontId::proportional((15.0 * app.canvas.zoom).clamp(6.0, 18.0)),
                        if highlighted {
                            t.canvas.selection
                        } else {
                            t.canvas.value
                        },
                    );
                }
                painter.add(Shape::line(points, Stroke::new(width, wire_color)));
            }
        }
        for (anchor, paths) in net_paths {
            let ink = if selected_net.as_ref().is_some_and(|n| n.anchor == anchor) {
                t.canvas.selection
            } else {
                t.canvas.wire
            };
            for junction in wire_geometry::junctions(&paths) {
                painter.circle_filled(junction, (4.5 * app.canvas.zoom).clamp(1.5, 3.2), ink);
            }
        }
    }
    if app.canvas.wire_drag.is_none() {
        app.canvas.routes = projected
            .iter()
            .filter_map(|(id, p)| {
                doc.wires.iter().find(|w| w.id == *id).map(|w| Projection {
                    wire: *id,
                    path: projected_world_path(app, &doc, canvas, w, p),
                })
            })
            .collect();
    }
    app.canvas.capture_wire_hit = projected
        .iter()
        .flat_map(|(_, p)| p.windows(2))
        .max_by(|a, b| a[0].distance(a[1]).total_cmp(&b[0].distance(b[1])))
        .map(|p| p[0].lerp(p[1], 0.5));
    if app.tool == Tool::Select
        && hit.is_none()
        && let Some(pos) = pointer
        && let Some((_, path)) = &hit_wire
    {
        if path.iter().any(|p| p.distance(pos) < 8.0) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
        } else if let Some(segment) = path
            .windows(2)
            .min_by(|a, b| path_distance(pos, a).total_cmp(&path_distance(pos, b)))
        {
            ui.ctx()
                .set_cursor_icon(if (segment[0].y - segment[1].y).abs() < 0.01 {
                    egui::CursorIcon::ResizeVertical
                } else {
                    egui::CursorIcon::ResizeHorizontal
                });
        }
    }
    if app.canvas.wires_visible {
        for j in &doc.junctions {
            let e = Endpoint::Junction(JunctionRef { junction: j.id });
            let degree = doc
                .wires
                .iter()
                .filter(|w| w.from == e || w.to == e)
                .count();
            let p = app.canvas.screen(j.at, canvas);
            if degree == 1 {
                painter.circle_stroke(p, 3.0, Stroke::new(1.2, t.canvas.wire));
            }
        }
        if let Some(id) = app.nets.wire
            && let Some(w) = doc.wires.iter().find(|w| w.id == id)
            && let Some((_, screen)) = projected.iter().find(|(wire, _)| *wire == w.id)
        {
            for p in screen {
                painter.rect_stroke(
                    Rect::from_center_size(*p, Vec2::splat(6.0)),
                    0.0,
                    Stroke::new(1.0, t.canvas.selection),
                    egui::StrokeKind::Middle,
                );
            }
        }
    }
    if let Some(net) = &selected_net {
        for pin in &net.terminals {
            if let Some(at) = app.canvas.pin(&doc, pin) {
                painter.circle_stroke(
                    app.canvas.screen(at, canvas),
                    3.5,
                    Stroke::new(1.4, t.canvas.selection),
                );
            }
        }
    }
    for s in &doc.symbols {
        if let Some(c) = doc.component(s.id) {
            let shown = app.canvas.shown_symbol(s);
            let center = app.canvas.screen(shown.at, canvas);
            let selected = app.selected == Some(s.id);
            let vertical = !shown.quarter_turns.is_multiple_of(2);
            paint_symbol(
                &painter,
                center,
                app.canvas.zoom,
                shown.quarter_turns,
                c.kind,
                if selected {
                    t.canvas.selection
                } else {
                    t.canvas.symbol
                },
            );
            painter.text(
                center
                    + (if c.kind.is_block() {
                        let size = symbol_size(c.kind, shown.quarter_turns);
                        egui::vec2(-size.x * 0.35, -size.y * 0.5 - 12.0)
                    } else if vertical {
                        egui::vec2(34.0, -8.0)
                    } else {
                        egui::vec2(-28.0, -34.0)
                    }) * app.canvas.zoom,
                Align2::LEFT_BOTTOM,
                &c.reference,
                if selected {
                    theme::semibold(
                        (t.schematic.reference_size * app.canvas.zoom)
                            .clamp(if app.canvas.zoom >= 0.25 { 9.0 } else { 6.0 }, 22.0),
                    )
                } else {
                    FontId::proportional(
                        (t.schematic.reference_size * app.canvas.zoom)
                            .clamp(if app.canvas.zoom >= 0.25 { 9.0 } else { 6.0 }, 22.0),
                    )
                },
                if selected {
                    t.canvas.selection
                } else {
                    t.canvas.reference
                },
            );
            if app.canvas.zoom >= 0.25 {
                painter.text(
                    center
                        + (if c.kind.is_block() {
                            let size = symbol_size(c.kind, shown.quarter_turns);
                            egui::vec2(-size.x * 0.35, size.y * 0.5 + 12.0)
                        } else if vertical {
                            egui::vec2(34.0, 8.0)
                        } else {
                            egui::vec2(-28.0, 30.0)
                        }) * app.canvas.zoom,
                    Align2::LEFT_TOP,
                    &c.value,
                    FontId::proportional(
                        (t.schematic.value_size * app.canvas.zoom).clamp(8.5, 20.0),
                    ),
                    t.canvas.value,
                );
            }
            if app.canvas.pins_visible && app.canvas.zoom >= 0.25 {
                for definition in c.kind.pins() {
                    let terminal = definition.id;
                    if let Some(p) = shown.pin(c.kind, terminal) {
                        let p = app.canvas.screen(p, canvas);
                        let connected = doc.wires.iter().any(|wire| {
                            [&wire.from, &wire.to].iter().any(|endpoint| {
                                endpoint.pin().is_some_and(|pin| {
                                    pin.symbol == s.id && pin.terminal == terminal
                                })
                            })
                        });
                        if !connected || app.tool == Tool::Wire {
                            painter.circle_stroke(p, 1.8, Stroke::new(0.75, t.canvas.symbol));
                        }
                        painter.text(
                            p + if vertical {
                                egui::vec2(-9.0, 0.0)
                            } else {
                                egui::vec2(0.0, -9.0)
                            },
                            if vertical {
                                Align2::RIGHT_CENTER
                            } else {
                                Align2::CENTER_BOTTOM
                            },
                            terminal,
                            FontId::proportional(
                                (t.schematic.pin_size * app.canvas.zoom).clamp(7.0, 16.0),
                            ),
                            t.ui.muted,
                        );
                    }
                }
            }
        }
    }
    if doc.symbols.is_empty() && doc.wires.is_empty() {
        let center = canvas.center();
        painter.text(
            center - egui::vec2(0.0, 38.0),
            Align2::CENTER_CENTER,
            "Start your schematic",
            FontId::proportional(26.0),
            t.ui.text,
        );
        painter.text(
            center + egui::vec2(0.0, 2.0),
            Align2::CENTER_CENTER,
            "Place a symbol to begin. Everything stays local.",
            FontId::proportional(14.0),
            t.ui.muted,
        );
        painter.text(
            center + egui::vec2(0.0, 32.0),
            Align2::CENTER_CENTER,
            "A  Place symbol     ·     W  Wire     ·     F  Fit",
            FontId::proportional(12.0),
            t.ui.muted,
        );
    }
    if let Some(pos) = pointer {
        if let Tool::Place(kind) = app.tool {
            paint_symbol(
                &painter,
                app.canvas.screen(app.canvas.last_world, canvas),
                app.canvas.zoom,
                0,
                kind,
                t.canvas.selection,
            );
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
        }
        if app.tool == Tool::Wire {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            let target = wire_target(app, &doc, canvas, pos, pin_hit.as_ref(), hit_wire.as_ref());
            if let Some(draft) = &app.canvas.wire_start {
                let mut points = draft.points.clone();
                extend_orthogonal(
                    &mut points,
                    target.point(&doc).unwrap_or(app.canvas.last_world),
                    draft.vertical_first,
                );
                painter.add(Shape::line(
                    points
                        .iter()
                        .map(|p| app.canvas.screen(*p, canvas))
                        .collect(),
                    Stroke::new(t.schematic.wire_width + 0.6, t.canvas.selection),
                ));
                for p in &draft.points {
                    painter.circle_filled(app.canvas.screen(*p, canvas), 2.5, t.canvas.selection);
                }
            }
            if let Some(p) = target.point(&doc) {
                painter.circle_stroke(
                    app.canvas.screen(p, canvas),
                    5.0,
                    Stroke::new(1.2, t.canvas.selection),
                );
            }
        }
    }

    if input_enabled
        && !app.busy()
        && app.pending.is_none()
        && let Some(part) = response.dnd_hover_payload::<super::library::PartDrag>()
        && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
    {
        let current =
            part.incarnation == app.session.incarnation && part.revision == app.session.revision();
        if current && canvas.contains(pos) {
            let at = app.canvas.world(pos, canvas);
            paint_symbol(
                &painter,
                app.canvas.screen(at, canvas),
                app.canvas.zoom,
                0,
                part.kind,
                t.canvas.selection,
            );
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            if response
                .dnd_release_payload::<super::library::PartDrag>()
                .is_some()
            {
                app.command(Command::Place {
                    kind: part.kind,
                    at,
                });
                app.tool = Tool::Select;
            }
        } else {
            ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed);
            if response
                .dnd_release_payload::<super::library::PartDrag>()
                .is_some()
                && !current
            {
                app.notice("The project changed during the drag. Choose the part again.");
            }
        }
        return;
    }
    if app.tool == Tool::Select
        && !app.busy()
        && app.pending.is_none()
        && (input_enabled || app.canvas.context_open)
    {
        if response.secondary_clicked()
            && input_enabled
            && let Some(pos) = pointer
        {
            app.canvas.context = Some(CanvasContext {
                wire: hit_wire.as_ref().map(|(id, _)| *id),
                anchor: hit_net.clone(),
                symbol: hit,
                target: wire_target(app, &doc, canvas, pos, pin_hit.as_ref(), hit_wire.as_ref()),
                incarnation: app.session.incarnation,
                revision: app.session.revision(),
            });
        }
        let context = app.canvas.context.clone();
        app.canvas.context_open = egui::Popup::context_menu(&response)
            .frame(controls::menu_frame(ui.ctx()))
            .show(|ui| {
                controls::menu_content(ui);
                let Some(context) = &context else {
                    return;
                };
                if context.incarnation != app.session.incarnation
                    || context.revision != app.session.revision()
                {
                    ui.label("Project changed. Open the menu again.");
                    return;
                }
                if let Some(symbol) = context.symbol {
                    if MenuItem::new("Properties", Icon::Properties)
                        .show(ui)
                        .clicked()
                    {
                        app.select(Some(symbol));
                        app.properties = true;
                        ui.close();
                    }
                    if MenuItem::new("Rotate", Icon::Rotate)
                        .shortcut("R")
                        .show(ui)
                        .clicked()
                    {
                        app.command(Command::Rotate { symbol });
                        ui.close();
                    }
                    controls::menu_separator(ui);
                    if MenuItem::new("Delete symbol", Icon::Delete)
                        .shortcut("Del")
                        .danger()
                        .show(ui)
                        .clicked()
                    {
                        app.command(Command::Delete { symbol });
                        ui.close();
                    }
                } else if let Some(wire) = context.wire {
                    if MenuItem::new("Start wire here", Icon::Wire)
                        .shortcut("W")
                        .show(ui)
                        .clicked()
                    {
                        app.tool = Tool::Wire;
                        app.canvas.wire_start = None;
                        wire_click(app, context.target.clone(), false);
                        ui.close();
                    }
                    if MenuItem::new("Select net", Icon::Select).show(ui).clicked() {
                        if let Some(anchor) = &context.anchor {
                            app.select_net(anchor.clone());
                        }
                        ui.close();
                    }
                    controls::menu_separator(ui);
                    if MenuItem::new("Delete wire", Icon::Delete)
                        .shortcut("Del")
                        .danger()
                        .show(ui)
                        .clicked()
                    {
                        app.command(Command::Disconnect { wire });
                        ui.close();
                    }
                    if MenuItem::new("Delete whole net", Icon::Delete)
                        .danger()
                        .show(ui)
                        .clicked()
                    {
                        if let Some(anchor) = &context.anchor {
                            app.command(Command::DeleteNet {
                                anchor: anchor.clone(),
                            });
                        }
                        ui.close();
                    }
                } else {
                    if MenuItem::new("Draw wire", Icon::Wire)
                        .shortcut("W")
                        .show(ui)
                        .clicked()
                    {
                        app.tool = Tool::Wire;
                        app.canvas.wire_start = None;
                        wire_click(app, context.target.clone(), false);
                        ui.close();
                    }
                    if MenuItem::new("Place symbol", Icon::Symbol)
                        .shortcut("A")
                        .show(ui)
                        .clicked()
                    {
                        app.library = true;
                        ui.close();
                    }
                    if MenuItem::new("Fit sheet", Icon::Fit)
                        .shortcut("F")
                        .show(ui)
                        .clicked()
                    {
                        app.canvas.fit = true;
                        ui.close();
                    }
                }
                controls::menu_separator(ui);
                if MenuItem::new("Undo", Icon::Undo)
                    .shortcut("Ctrl+Z")
                    .enabled(app.session.can_undo())
                    .show(ui)
                    .clicked()
                {
                    app.history(false);
                    ui.close();
                }
                if MenuItem::new("Redo", Icon::Redo)
                    .shortcut("Ctrl+Shift+Z")
                    .enabled(app.session.can_redo())
                    .show(ui)
                    .clicked()
                {
                    app.history(true);
                    ui.close();
                }
            })
            .is_some();
    }
    if app.busy() || !input_enabled {
        if ui.input(|i| i.pointer.any_released()) {
            app.canvas.wire_drag = None;
        }
        return;
    }

    if app.tool == Tool::Wire && !ui.ctx().egui_wants_keyboard_input() {
        if ui.input(|i| i.key_pressed(egui::Key::Space))
            && let Some(draft) = &mut app.canvas.wire_start
        {
            draft.vertical_first = !draft.vertical_first;
        }
        if ui.input(|i| i.key_pressed(egui::Key::Backspace))
            && let Some(draft) = &mut app.canvas.wire_start
            && draft.points.len() > 1
        {
            draft.points.pop();
        }
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            finish_wire(app, Anchor::Free(app.canvas.last_world));
        }
        if response.secondary_clicked() {
            app.canvas.wire_start = None;
        }
    }
    if response.clicked() {
        match app.tool {
            Tool::Place(kind) => {
                app.command(Command::Place {
                    kind,
                    at: app.canvas.last_world,
                });
                app.tool = Tool::Select;
            }
            Tool::Wire => {
                if let Some(pos) = pointer {
                    let target =
                        wire_target(app, &doc, canvas, pos, pin_hit.as_ref(), hit_wire.as_ref());
                    let finish = !matches!(target, Anchor::Free(_)) || response.double_clicked();
                    wire_click(app, target, finish);
                }
            }
            Tool::Select => {
                if hit.is_some() {
                    app.select(hit);
                } else if let Some(anchor) = hit_net {
                    app.select_net(anchor);
                    app.nets.wire = hit_wire.as_ref().map(|(id, _)| *id);
                } else {
                    app.select(None);
                }
            }
            Tool::Pan => {}
        }
    }
    if app.tool == Tool::Select
        && response.drag_started_by(egui::PointerButton::Primary)
        && let Some(id) = hit
    {
        app.select(Some(id));
        if let Some(s) = doc.symbols.iter().find(|s| s.id == id) {
            app.canvas.drag = Some((id, s.at, s.at));
        }
    }
    if app.tool == Tool::Select
        && app.canvas.drag.is_none()
        && app.canvas.nets_selectable
        && response.drag_started_by(egui::PointerButton::Primary)
        && let Some(origin) = ui.input(|i| i.pointer.press_origin())
        && let Some((wire, path)) = projected
            .iter()
            .filter(|(_, p)| path_distance(origin, p) <= 8.0)
            .min_by(|a, b| path_distance(origin, &a.1).total_cmp(&path_distance(origin, &b.1)))
    {
        start_wire_drag(app, &doc, canvas, origin, *wire, path);
    }
    if (response.dragged_by(egui::PointerButton::Primary) || response.drag_stopped())
        && let Some(pos) = pointer
    {
        let current = app.canvas.world(pos, canvas);
        if let Some(drag) = &mut app.canvas.wire_drag {
            let at = Point {
                x: (drag.initial.x + current.x - drag.origin.x).clamp(-LIMIT_NM, LIMIT_NM),
                y: (drag.initial.y + current.y - drag.origin.y).clamp(-LIMIT_NM, LIMIT_NM),
            };
            match &mut drag.edit {
                WireEdit::Junction { at: target, .. }
                | WireEdit::Corner { at: target, .. }
                | WireEdit::Segment { at: target, .. } => *target = at,
                WireEdit::Attach { .. } | WireEdit::WithRoutes { .. } | WireEdit::Draw { .. } => {}
            }
            ui.ctx().request_repaint();
        }
    }
    if response.drag_stopped()
        && let Some(mut drag) = app.canvas.wire_drag.take()
    {
        if let WireEdit::Junction { id, .. } = drag.edit
            && let Some(pos) = pointer
            && let Some(target) =
                attachment_target(app, canvas, pos, id, &projected, pin_hit.as_ref())
        {
            drag.edit = WireEdit::Attach {
                junction: id,
                target,
            };
        }
        if app.session.incarnation == drag.incarnation {
            match app.session.apply(
                drag.revision,
                Command::EditWire {
                    edit: WireEdit::WithRoutes {
                        edit: Box::new(drag.edit),
                        routes: drag.routes,
                    },
                },
            ) {
                Ok(_) => {
                    app.refresh_selection();
                    app.refresh_nets();
                }
                Err(e) => {
                    app.notice(e);
                    app.drawer = Some("Messages");
                }
            }
        } else {
            app.notice("Project changed. Wire move was cancelled.");
        }
    }
    if response.dragged_by(egui::PointerButton::Primary)
        && let Some((id, start, at)) = app.canvas.drag
    {
        let delta = ui.input(|i| i.pointer.delta()) / (8.0 * app.canvas.zoom);
        let next = Point {
            x: (at.x as f64 + delta.x as f64 * 1e6).clamp(-LIMIT_NM as f64, LIMIT_NM as f64) as i64,
            y: (at.y as f64 + delta.y as f64 * 1e6).clamp(-LIMIT_NM as f64, LIMIT_NM as f64) as i64,
        };
        app.canvas.drag = Some((id, start, next));
    }
    if response.drag_stopped()
        && let Some((symbol, _, at)) = app.canvas.drag.take()
    {
        app.command(Command::Move {
            symbol,
            at: if app.canvas.snap { at.snapped() } else { at },
        });
    }
}

// Label every visibly connected island once. Remote aliases still refer to explicit links.
fn drawn_groups(doc: &Document) -> std::collections::BTreeMap<Endpoint, Endpoint> {
    let mut edges = std::collections::BTreeMap::<Endpoint, Vec<Endpoint>>::new();
    for w in doc
        .wires
        .iter()
        .filter(|w| w.label.is_none() || w.draw_path)
    {
        edges.entry(w.from.clone()).or_default().push(w.to.clone());
        edges.entry(w.to.clone()).or_default().push(w.from.clone());
    }
    let mut groups = std::collections::BTreeMap::new();
    for start in edges.keys() {
        if groups.contains_key(start) {
            continue;
        }
        let mut pending = vec![start.clone()];
        while let Some(pin) = pending.pop() {
            if groups.contains_key(&pin) {
                continue;
            }
            groups.insert(pin.clone(), start.clone());
            if let Some(neighbors) = edges.get(&pin) {
                pending.extend(neighbors.iter().cloned());
            }
        }
    }
    groups
}

fn path_distance(p: Pos2, path: &[Pos2]) -> f32 {
    path.windows(2)
        .map(|pair| {
            let delta = pair[1] - pair[0];
            let t = if delta.length_sq() > 0.0 {
                ((p - pair[0]).dot(delta) / delta.length_sq()).clamp(0.0, 1.0)
            } else {
                0.0
            };
            p.distance(pair[0] + delta * t)
        })
        .fold(f32::INFINITY, f32::min)
}

fn block_half_size(kind: PartKind) -> (f32, f32) {
    match kind {
        PartKind::FanController => (15.0 * 10.16, 16.0 * 10.16),
        PartKind::FanConnector => (7.0 * 10.16, 9.0 * 10.16),
        _ => (7.0 * 10.16, 8.0 * 10.16),
    }
}
pub(super) fn symbol_size(kind: PartKind, turns: u8) -> Vec2 {
    let size = if kind.is_block() {
        let (_, h) = block_half_size(kind);
        egui::vec2(
            if kind == PartKind::FanController {
                406.4
            } else {
                243.84
            },
            h * 2.0,
        )
    } else {
        egui::vec2(101.6, 64.0)
    };
    if turns.is_multiple_of(2) {
        size
    } else {
        egui::vec2(size.y, size.x)
    }
}

pub fn paint_symbol(
    p: &egui::Painter,
    center: Pos2,
    scale: f32,
    turns: u8,
    kind: PartKind,
    color: Color32,
) {
    let xy = |x: f32, y: f32| {
        let (x, y) = match turns % 4 {
            0 => (x, y),
            1 => (-y, x),
            2 => (-x, -y),
            _ => (y, -x),
        };
        center + egui::vec2(x, y) * scale
    };
    let stroke = Stroke::new(
        (theme::current(p.ctx()).schematic.symbol_width * scale).clamp(0.85, 2.0),
        color,
    );
    let line = |points: &[(f32, f32)]| {
        p.add(Shape::line(
            points.iter().map(|(x, y)| xy(*x, *y)).collect(),
            stroke,
        ));
    };
    if kind.is_block() {
        let (w, h) = block_half_size(kind);
        line(&[(-w, -h), (w, -h), (w, h), (-w, h), (-w, -h)]);
        for pin in kind.pins() {
            let (x, y) = (pin.x as f32 * 10.16, pin.y as f32 * 10.16);
            let edge = x.signum() * w;
            line(&[(x, y), (edge, y)]);
            if scale < 0.25 {
                continue;
            }
            p.text(
                xy(edge - x.signum() * 8.0, y),
                if turns == 0 {
                    if x < 0.0 {
                        Align2::LEFT_CENTER
                    } else {
                        Align2::RIGHT_CENTER
                    }
                } else {
                    Align2::CENTER_CENTER
                },
                pin.name,
                FontId::proportional((13.0 * scale).clamp(8.0, 20.0)),
                color,
            );
        }
        return;
    }
    line(&[(-50.8, 0.0), (-25.4, 0.0)]);
    line(&[(25.4, 0.0), (50.8, 0.0)]);
    match kind {
        PartKind::Resistor => line(&[
            (-25.4, 0.0),
            (-20.0, -9.0),
            (-12.0, 9.0),
            (-4.0, -9.0),
            (4.0, 9.0),
            (12.0, -9.0),
            (20.0, 9.0),
            (25.4, 0.0),
        ]),
        PartKind::Capacitor => {
            line(&[(-25.4, 0.0), (-5.0, 0.0)]);
            line(&[(-5.0, -18.0), (-5.0, 18.0)]);
            line(&[(5.0, -18.0), (5.0, 18.0)]);
            line(&[(5.0, 0.0), (25.4, 0.0)]);
        }
        PartKind::Led => {
            line(&[(-25.4, 0.0), (-13.0, 0.0)]);
            line(&[(-13.0, -15.0), (13.0, 0.0), (-13.0, 15.0), (-13.0, -15.0)]);
            line(&[(13.0, -15.0), (13.0, 15.0)]);
            line(&[(13.0, 0.0), (25.4, 0.0)]);
            line(&[(0.0, -20.0), (13.0, -31.0), (8.0, -30.0)]);
            line(&[(11.0, -17.0), (24.0, -28.0), (19.0, -27.0)]);
        }
        PartKind::Connector => {
            line(&[(-25.4, 0.0), (-12.0, 0.0)]);
            line(&[(12.0, 0.0), (25.4, 0.0)]);
            line(&[
                (-25.0, -18.0),
                (25.0, -18.0),
                (25.0, 18.0),
                (-25.0, 18.0),
                (-25.0, -18.0),
            ]);
            p.circle_stroke(xy(-9.0, 0.0), 3.0 * scale, stroke);
            p.circle_stroke(xy(9.0, 0.0), 3.0 * scale, stroke);
        }
        PartKind::FanController
        | PartKind::FanConnector
        | PartKind::Regulator
        | PartKind::TemperatureSensor => {}
    }
}

fn extend_orthogonal(points: &mut Vec<Point>, at: Point, vertical_first: bool) {
    if let Some(last) = points.last().copied() {
        if last.x != at.x && last.y != at.y {
            points.push(if vertical_first {
                Point { x: last.x, y: at.y }
            } else {
                Point { x: at.x, y: last.y }
            });
        }
        if points.last() != Some(&at) {
            points.push(at);
        }
    }
}
fn projected_world_path(
    app: &App,
    doc: &Document,
    canvas: Rect,
    wire: &Wire,
    screen: &[Pos2],
) -> Vec<Point> {
    if wire.bends.is_some() {
        wiring::path(doc, wire).unwrap_or_default()
    } else {
        // Recover the current legacy projection, preserving exact endpoint coordinates.
        let a = doc.endpoint(&wire.from).unwrap_or_default();
        let b = doc.endpoint(&wire.to).unwrap_or_default();
        let sa = app.canvas.screen(a, canvas);
        let sb = app.canvas.screen(b, canvas);
        screen
            .iter()
            .map(|p| {
                let v = (*p - canvas.center() - app.canvas.offset) / (8.0 * app.canvas.zoom);
                Point {
                    x: if (p.x - sa.x).abs() < 0.01 {
                        a.x
                    } else if (p.x - sb.x).abs() < 0.01 {
                        b.x
                    } else {
                        (v.x as f64 * 1000.0).round() as i64 * 1000
                    },
                    y: if (p.y - sa.y).abs() < 0.01 {
                        a.y
                    } else if (p.y - sb.y).abs() < 0.01 {
                        b.y
                    } else {
                        (v.y as f64 * 1000.0).round() as i64 * 1000
                    },
                }
            })
            .collect()
    }
}
fn wire_target(
    app: &App,
    doc: &Document,
    canvas: Rect,
    pos: Pos2,
    pin: Option<&PinRef>,
    hit: Option<&(WireId, Vec<Pos2>)>,
) -> Anchor {
    if let Some(pin) = pin {
        return Anchor::Existing(pin.clone().into());
    }
    if app.canvas.wires_visible
        && app.canvas.nets_selectable
        && let Some(j) = doc
            .junctions
            .iter()
            .filter(|j| app.canvas.screen(j.at, canvas).distance(pos) <= 8.0)
            .min_by(|a, b| {
                app.canvas
                    .screen(a.at, canvas)
                    .distance(pos)
                    .total_cmp(&app.canvas.screen(b.at, canvas).distance(pos))
            })
    {
        return Anchor::Existing(Endpoint::Junction(JunctionRef { junction: j.id }));
    }
    if let Some((id, screen)) = hit
        && let Some(wire) = doc.wires.iter().find(|w| w.id == *id)
    {
        if screen.is_empty() {
            let endpoint = [&wire.from, &wire.to]
                .into_iter()
                .filter_map(|e| {
                    doc.endpoint(e)
                        .map(|p| (e, app.canvas.screen(p, canvas).distance(pos)))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((e, _)) = endpoint {
                return Anchor::Existing(e.clone());
            }
        }
        let points = projected_world_path(app, doc, canvas, wire, screen);
        let snapped = app.canvas.world(pos, canvas);
        if let Some(segment) = points.windows(2).min_by(|a, b| {
            path_distance(
                pos,
                &[
                    app.canvas.screen(a[0], canvas),
                    app.canvas.screen(a[1], canvas),
                ],
            )
            .total_cmp(&path_distance(
                pos,
                &[
                    app.canvas.screen(b[0], canvas),
                    app.canvas.screen(b[1], canvas),
                ],
            ))
        }) {
            let at = if segment[0].x == segment[1].x {
                Point {
                    x: segment[0].x,
                    y: snapped.y.clamp(
                        segment[0].y.min(segment[1].y),
                        segment[0].y.max(segment[1].y),
                    ),
                }
            } else {
                Point {
                    y: segment[0].y,
                    x: snapped.x.clamp(
                        segment[0].x.min(segment[1].x),
                        segment[0].x.max(segment[1].x),
                    ),
                }
            };
            return Anchor::Split {
                wire: *id,
                at,
                path: points,
            };
        }
    }
    Anchor::Free(app.canvas.world(pos, canvas))
}
fn wire_click(app: &mut App, target: Anchor, finish: bool) {
    if let Some(draft) = &mut app.canvas.wire_start {
        if finish {
            finish_wire(app, target);
        } else if let Some(at) = target.point(app.session.document()) {
            if draft.points.len() < 254 {
                extend_orthogonal(&mut draft.points, at, draft.vertical_first);
            } else {
                app.notice("Finish this wire before adding more corners.");
            }
        }
    } else if let Some(at) = target.point(app.session.document()) {
        app.canvas.wire_start = Some(WireDraft {
            from: target,
            points: vec![at],
            incarnation: app.session.incarnation,
            revision: app.session.revision(),
            vertical_first: false,
        });
    }
}
fn finish_wire(app: &mut App, target: Anchor) {
    let Some(draft) = app.canvas.wire_start.clone() else {
        return;
    };
    if app.session.incarnation != draft.incarnation || app.session.revision() != draft.revision {
        app.canvas.wire_start = None;
        app.notice("The project changed during wiring. Start the wire again.");
        return;
    }
    let Some(at) = target.point(app.session.document()) else {
        return;
    };
    let mut points = draft.points.clone();
    extend_orthogonal(&mut points, at, draft.vertical_first);
    wiring::simplify(&mut points);
    if points.len() < 2 {
        return;
    }
    match app.session.apply(
        draft.revision,
        Command::EditWire {
            edit: WireEdit::WithRoutes {
                routes: app.canvas.routes.clone(),
                edit: Box::new(WireEdit::Draw {
                    from: draft.from,
                    to: target,
                    bends: points[1..points.len() - 1].to_vec(),
                }),
            },
        },
    ) {
        Ok(_) => {
            app.canvas.wire_start = None;
            app.refresh_selection();
            app.refresh_nets();
        }
        Err(e) => {
            app.notice(e);
            app.drawer = Some("Messages");
        }
    }
}

fn attachment_target(
    app: &App,
    canvas: Rect,
    pos: Pos2,
    id: JunctionId,
    projected: &[(WireId, Vec<Pos2>)],
    pin: Option<&PinRef>,
) -> Option<Anchor> {
    let doc = app.session.document();
    if let Some(pin) = pin {
        return Some(Anchor::Existing(pin.clone().into()));
    }
    if let Some(j) = doc
        .junctions
        .iter()
        .filter(|j| j.id != id && app.canvas.screen(j.at, canvas).distance(pos) < 8.0)
        .min_by(|a, b| {
            app.canvas
                .screen(a.at, canvas)
                .distance(pos)
                .total_cmp(&app.canvas.screen(b.at, canvas).distance(pos))
        })
    {
        return Some(Anchor::Existing(Endpoint::Junction(JunctionRef {
            junction: j.id,
        })));
    }
    let own = Endpoint::Junction(JunctionRef { junction: id });
    let hit = projected
        .iter()
        .filter(|(wire, points)| {
            path_distance(pos, points) < 8.0
                && doc
                    .wires
                    .iter()
                    .find(|w| w.id == *wire)
                    .is_some_and(|w| w.from != own && w.to != own)
        })
        .min_by(|a, b| path_distance(pos, &a.1).total_cmp(&path_distance(pos, &b.1)))?;
    let target = wire_target(app, doc, canvas, pos, None, Some(hit));
    if matches!(&target, Anchor::Free(_)) || matches!(&target,Anchor::Existing(e) if *e==own) {
        None
    } else {
        Some(target)
    }
}
fn start_wire_drag(
    app: &mut App,
    doc: &Document,
    canvas: Rect,
    origin: Pos2,
    id: WireId,
    screen: &[Pos2],
) {
    let Some(wire) = doc.wires.iter().find(|w| w.id == id) else {
        return;
    };
    let points = projected_world_path(app, doc, canvas, wire, screen);
    let near = screen
        .iter()
        .enumerate()
        .filter(|(_, p)| p.distance(origin) < 8.0)
        .min_by(|a, b| a.1.distance(origin).total_cmp(&b.1.distance(origin)))
        .map(|(i, _)| i);
    let (edit, initial) = if let Some(index) = near {
        let endpoint = if index == 0 {
            Some(&wire.from)
        } else if index + 1 == points.len() {
            Some(&wire.to)
        } else {
            None
        };
        let Some(at) = points.get(index).copied() else {
            return;
        };
        if let Some(e) = endpoint {
            match e {
                Endpoint::Junction(j) => (WireEdit::Junction { id: j.junction, at }, at),
                Endpoint::Pin(_) => return,
            }
        } else {
            (
                WireEdit::Corner {
                    wire: id,
                    path: points.clone(),
                    index,
                    at,
                },
                at,
            )
        }
    } else {
        let Some((index, _)) = screen
            .windows(2)
            .enumerate()
            .min_by(|a, b| path_distance(origin, a.1).total_cmp(&path_distance(origin, b.1)))
        else {
            return;
        };
        let at = app.canvas.world(origin, canvas);
        (
            WireEdit::Segment {
                wire: id,
                path: points,
                index,
                at,
            },
            at,
        )
    };
    app.select_net(wire.from.clone());
    app.nets.wire = Some(id);
    app.canvas.wire_drag = Some(WireDrag {
        routes: app.canvas.routes.clone(),
        edit,
        initial,
        origin: app.canvas.world(origin, canvas),
        incarnation: app.session.incarnation,
        revision: app.session.revision(),
    });
}

#[cfg(test)]
mod tests {
    use super::super::Options;
    use super::*;

    #[test]
    fn fit_large_demo_keeps_symbol_bounds_in_compact_viewport() -> Result<()> {
        let doc = pcbcraft_foundation::session::Session::demo()?
            .document()
            .clone();
        let mut canvas = Canvas::default();
        let viewport = Rect::from_min_size(Pos2::ZERO, egui::vec2(420.0, 330.0));
        canvas.fit_document(&doc, viewport);
        for symbol in &doc.symbols {
            let kind = doc.component(symbol.id).ok_or("Missing component")?.kind;
            let bounds = Rect::from_center_size(
                canvas.screen(symbol.at, viewport),
                symbol_size(kind, symbol.quarter_turns) * canvas.zoom,
            );
            assert!(
                viewport.contains_rect(bounds),
                "{bounds:?} outside compact viewport"
            );
        }
        Ok(())
    }

    fn app() -> Result<(App, egui::Context)> {
        let app = App::create(Options {
            demo: false,
            capture: None,
            exit_after_capture: false,
            capture_workspace: false,
            capture_library: false,
            capture_panels: false,
            capture_search: false,
            capture_properties: false,
            capture_nets: false,
            capture_wire_menu: false,
            open: None,
        })?;
        let ctx = egui::Context::default();
        theme::apply(&ctx);
        Ok((app, ctx))
    }
    fn frame(app: &mut App, ctx: &egui::Context, events: Vec<egui::Event>) {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(1600.0, 1000.0))),
            events,
            ..Default::default()
        };
        ctx.run_ui(input, |ui| {
            app.shortcuts(ctx);
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ui, |ui| show(app, ui, true));
        })
        .drop_without_applying_deltas();
    }
    fn pointer(pos: Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }
    fn click(app: &mut App, ctx: &egui::Context, pos: Pos2) {
        frame(app, ctx, vec![egui::Event::PointerMoved(pos)]);
        frame(app, ctx, vec![pointer(pos, true)]);
        frame(app, ctx, vec![pointer(pos, false)]);
    }

    #[test]
    fn clicking_wire_selects_its_net_without_editing() -> Result<()> {
        let (mut app, ctx) = app()?;
        let mut ids = Vec::new();
        for x in [-25_400_000, 25_400_000] {
            ids.push(
                app.session
                    .apply(
                        app.session.revision(),
                        Command::Place {
                            kind: PartKind::Resistor,
                            at: Point { x, y: 0 },
                        },
                    )?
                    .ok_or("No component")?,
            );
        }
        app.session.apply(
            app.session.revision(),
            Command::Connect {
                from: PinRef {
                    symbol: ids[0],
                    terminal: "2".into(),
                },
                to: PinRef {
                    symbol: ids[1],
                    terminal: "1".into(),
                },
            },
        )?;
        app.canvas.fit = false;
        app.select(None);
        frame(&mut app, &ctx, vec![]);
        let before = app.session.document().clone();
        click(&mut app, &ctx, Pos2::new(798.0, 483.5));
        assert_eq!(
            app.nets
                .selected_net()
                .ok_or("Wire did not select a net")?
                .terminals
                .len(),
            2
        );
        assert_eq!(app.session.document(), &before);
        Ok(())
    }

    #[test]
    fn pointer_placement_drag_and_undo_use_one_session() -> Result<()> {
        let (mut app, ctx) = app()?;
        frame(&mut app, &ctx, vec![]);
        let center = Pos2::new(798.0, 483.5);
        app.tool = Tool::Place(PartKind::Resistor);
        click(&mut app, &ctx, center);
        let symbol = app
            .session
            .document()
            .symbols
            .first()
            .ok_or("Pointer did not place symbol")?
            .clone();
        assert_eq!(symbol.at, Point::default());
        let placed = app.session.document().clone();
        let revision = app.session.revision();
        frame(&mut app, &ctx, vec![pointer(center, true)]);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerMoved(center + egui::vec2(10.0, 0.0))],
        );
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerMoved(center + egui::vec2(60.0, 30.0))],
        );
        assert_eq!(
            app.session.document(),
            &placed,
            "Drag preview must not mutate the document"
        );
        frame(
            &mut app,
            &ctx,
            vec![pointer(center + egui::vec2(60.0, 30.0), false)],
        );
        assert_eq!(app.session.revision(), revision + 1);
        assert_ne!(app.session.document(), &placed);
        app.history(false);
        assert_eq!(app.session.document(), &placed);
        app.history(true);
        assert_ne!(app.session.document(), &placed);
        Ok(())
    }

    #[test]
    fn clicking_pin_anchors_creates_a_connection_and_escape_cancels() -> Result<()> {
        let (mut app, ctx) = app()?;
        frame(&mut app, &ctx, vec![]);
        app.tool = Tool::Place(PartKind::Resistor);
        click(&mut app, &ctx, Pos2::new(650.0, 483.5));
        app.tool = Tool::Place(PartKind::Led);
        click(&mut app, &ctx, Pos2::new(950.0, 483.5));
        let doc = app.session.document().clone();
        let a = doc
            .symbols
            .first()
            .ok_or("No resistor")?
            .pin(PartKind::Resistor, "2")
            .ok_or("No pin")?;
        let b = doc
            .symbols
            .get(1)
            .ok_or("No LED")?
            .pin(PartKind::Resistor, "1")
            .ok_or("No pin")?;
        let rect = Rect::from_min_max(Pos2::new(284.0, 52.0), Pos2::new(1312.0, 915.0));
        let a = app.canvas.screen(a, rect);
        let b = app.canvas.screen(b, rect);
        app.tool = Tool::Wire;
        app.canvas.pins_selectable = false;
        click(&mut app, &ctx, a);
        assert!(
            app.canvas
                .wire_start
                .as_ref()
                .is_some_and(|d| matches!(d.from, Anchor::Free(_))),
            "The pin filter prevents attachment, but free wiring remains available"
        );
        app.canvas.wire_start = None;
        app.canvas.pins_selectable = true;
        click(&mut app, &ctx, a);
        assert!(app.canvas.wire_start.is_some());
        click(&mut app, &ctx, b);
        assert_eq!(app.session.document().wires.len(), 1);
        click(&mut app, &ctx, a);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!(app.canvas.wire_start.is_none());
        assert!(app.tool == Tool::Select);
        assert_eq!(app.session.document().wires.len(), 1);
        Ok(())
    }
    fn key(app: &mut App, ctx: &egui::Context, key: egui::Key) {
        frame(
            app,
            ctx,
            vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        frame(
            app,
            ctx,
            vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }
    #[test]
    fn pointer_free_wires_branch_selection_delete_and_undo() -> Result<()> {
        let (mut app, ctx) = app()?;
        frame(&mut app, &ctx, vec![]);
        app.canvas.snap = false;
        key(&mut app, &ctx, egui::Key::W);
        click(&mut app, &ctx, Pos2::new(600.0, 400.0));
        assert!(
            app.session.document().wires.is_empty(),
            "Wire preview is not committed"
        );
        click(&mut app, &ctx, Pos2::new(950.0, 400.0));
        key(&mut app, &ctx, egui::Key::Enter);
        assert_eq!(app.session.document().wires.len(), 1);
        assert!(app.canvas.wire_start.is_none());
        let before = app.session.document().clone();
        // Start in the middle of an existing segment, not on a part or endpoint.
        click(&mut app, &ctx, Pos2::new(750.0, 400.0));
        click(&mut app, &ctx, Pos2::new(750.0, 650.0));
        key(&mut app, &ctx, egui::Key::Enter);
        assert_eq!(
            app.session.document().wires.len(),
            3,
            "Branch splits and attaches atomically: {:?}",
            app.notices
        );
        let branched = app.session.document().clone();
        let n = pcbcraft_foundation::netlist::Netlist::compile(&branched)?;
        assert_eq!(n.nets.len(), 1);
        assert!(n.nets[0].terminals.is_empty());
        key(&mut app, &ctx, egui::Key::Escape);
        click(&mut app, &ctx, Pos2::new(750.0, 550.0));
        assert!(app.nets.wire.is_some(), "Canvas selects the exact wire");
        key(&mut app, &ctx, egui::Key::Delete);
        assert_eq!(app.session.document().wires.len(), 2);
        app.history(false);
        assert_eq!(*app.session.document(), branched);
        app.history(false);
        assert_eq!(*app.session.document(), before);
        app.history(true);
        assert_eq!(*app.session.document(), branched);
        app.select_net(n.nets[0].anchor.clone());
        key(&mut app, &ctx, egui::Key::Delete);
        assert!(app.session.document().wires.is_empty());
        app.history(false);
        assert_eq!(*app.session.document(), branched);
        Ok(())
    }
    #[test]
    fn pointer_bends_cancel_stale_preview_and_free_end_extension() -> Result<()> {
        let (mut app, ctx) = app()?;
        frame(&mut app, &ctx, vec![]);
        app.canvas.snap = false;
        app.tool = Tool::Wire;
        click(&mut app, &ctx, Pos2::new(600.0, 350.0));
        click(&mut app, &ctx, Pos2::new(760.0, 450.0));
        assert_eq!(app.session.revision(), 0);
        key(&mut app, &ctx, egui::Key::Space);
        assert!(
            app.canvas
                .wire_start
                .as_ref()
                .is_some_and(|d| d.vertical_first)
        );
        key(&mut app, &ctx, egui::Key::Backspace);
        key(&mut app, &ctx, egui::Key::Escape);
        assert!(
            app.session.document().wires.is_empty() && app.session.document().junctions.is_empty()
        );
        app.tool = Tool::Wire;
        click(&mut app, &ctx, Pos2::new(600.0, 350.0));
        click(&mut app, &ctx, Pos2::new(760.0, 450.0));
        key(&mut app, &ctx, egui::Key::Enter);
        assert!(
            app.session.document().wires[0]
                .bends
                .as_ref()
                .is_some_and(|b| !b.is_empty())
        );
        let first = app.session.document().clone();
        click(&mut app, &ctx, Pos2::new(760.0, 450.0));
        click(&mut app, &ctx, Pos2::new(950.0, 450.0));
        key(&mut app, &ctx, egui::Key::Enter);
        assert_eq!(
            app.session.document().wires.len(),
            1,
            "Extending a free end heals into one continuous run"
        );
        assert_eq!(
            pcbcraft_foundation::netlist::Netlist::compile(app.session.document())?
                .nets
                .len(),
            1
        );
        app.history(false);
        assert_eq!(*app.session.document(), first);
        app.tool = Tool::Wire;
        click(&mut app, &ctx, Pos2::new(600.0, 600.0));
        let draft = app.canvas.wire_start.clone();
        app.session.apply(
            app.session.revision(),
            Command::Place {
                kind: PartKind::Resistor,
                at: Point::default(),
            },
        )?;
        let changed = app.session.document().clone();
        assert!(draft.is_some());
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerMoved(Pos2::new(900.0, 600.0))],
        );
        key(&mut app, &ctx, egui::Key::Enter);
        assert_eq!(*app.session.document(), changed);
        assert!(app.canvas.wire_start.is_none());
        Ok(())
    }
    #[test]
    fn dragging_segments_and_endpoints_is_previewed_cancellable_and_undoable() -> Result<()> {
        let (mut app, ctx) = app()?;
        frame(&mut app, &ctx, vec![]);
        app.canvas.snap = false;
        app.tool = Tool::Wire;
        click(&mut app, &ctx, Pos2::new(600.0, 400.0));
        click(&mut app, &ctx, Pos2::new(950.0, 400.0));
        key(&mut app, &ctx, egui::Key::Enter);
        key(&mut app, &ctx, egui::Key::Escape);
        let before = app.session.document().clone();
        let revision = app.session.revision();
        let start = Pos2::new(750.0, 400.0);
        let end = Pos2::new(750.0, 500.0);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerMoved(start), pointer(start, true)],
        );
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(end)]);
        assert!(app.canvas.wire_drag.is_some());
        assert_eq!(*app.session.document(), before);
        frame(&mut app, &ctx, vec![pointer(end, false)]);
        assert_eq!(app.session.revision(), revision + 1);
        let shifted = app.session.document().clone();
        assert_ne!(shifted, before);
        assert_eq!(
            pcbcraft_foundation::netlist::Netlist::compile(&before)?,
            pcbcraft_foundation::netlist::Netlist::compile(&shifted)?
        );
        // Select the newly visible corner and cancel its preview with Escape.
        let corner = Pos2::new(600.0, 500.0);
        let moved = Pos2::new(650.0, 530.0);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerMoved(corner), pointer(corner, true)],
        );
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(moved)]);
        assert!(app.canvas.wire_drag.is_some());
        key(&mut app, &ctx, egui::Key::Escape);
        frame(&mut app, &ctx, vec![pointer(moved, false)]);
        assert_eq!(*app.session.document(), shifted);
        // A free endpoint can be repositioned; its other endpoint stays fixed.
        let endpoint = Pos2::new(600.0, 400.0);
        let moved = Pos2::new(560.0, 350.0);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerMoved(endpoint), pointer(endpoint, true)],
        );
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(moved)]);
        assert!(app.canvas.wire_drag.is_some());
        frame(&mut app, &ctx, vec![pointer(moved, false)]);
        assert_ne!(*app.session.document(), shifted);
        app.history(false);
        assert_eq!(*app.session.document(), shifted);
        app.history(false);
        assert_eq!(*app.session.document(), before);
        Ok(())
    }
    #[test]
    fn endpoint_drop_onto_wire_creates_a_junction_in_one_undo_step() -> Result<()> {
        let (mut app, ctx) = app()?;
        frame(&mut app, &ctx, vec![]);
        app.canvas.snap = false;
        app.tool = Tool::Wire;
        click(&mut app, &ctx, Pos2::new(600.0, 400.0));
        click(&mut app, &ctx, Pos2::new(950.0, 400.0));
        key(&mut app, &ctx, egui::Key::Enter);
        click(&mut app, &ctx, Pos2::new(750.0, 600.0));
        click(&mut app, &ctx, Pos2::new(750.0, 700.0));
        key(&mut app, &ctx, egui::Key::Enter);
        key(&mut app, &ctx, egui::Key::Escape);
        let before = app.session.document().clone();
        let revision = app.session.revision();
        assert_eq!(
            pcbcraft_foundation::netlist::Netlist::compile(&before)?
                .nets
                .len(),
            2
        );
        let start = Pos2::new(750.0, 600.0);
        let end = Pos2::new(750.0, 400.0);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerMoved(start), pointer(start, true)],
        );
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(end)]);
        assert_eq!(*app.session.document(), before);
        frame(&mut app, &ctx, vec![pointer(end, false)]);
        assert_eq!(app.session.revision(), revision + 1);
        assert_eq!(
            pcbcraft_foundation::netlist::Netlist::compile(app.session.document())?
                .nets
                .len(),
            1
        );
        assert_eq!(app.session.document().wires.len(), 3);
        app.history(false);
        assert_eq!(*app.session.document(), before);
        Ok(())
    }
}

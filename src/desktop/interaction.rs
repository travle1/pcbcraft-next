//! Box selection and stamped, disposable group/text drag previews.
use super::{App, Tool, canvas, theme};
use eframe::egui::{self, Pos2, Rect, Stroke, vec2};
use pcbcraft_foundation::{
    document::*,
    selection::{self, Selection},
    session::Command,
    text::{self, Appearance, TextTarget},
    wiring::Projection,
};
#[derive(Clone)]
pub(super) enum Gesture {
    Box {
        start: Pos2,
        at: Pos2,
        base: Selection,
        shift: bool,
        toggle: bool,
        button: egui::PointerButton,
        stamp: (ProjectId, u64),
    },
    Move {
        items: Selection,
        origin: Point,
        delta: Point,
        routes: Vec<Projection>,
        stamp: (ProjectId, u64),
    },
    Text {
        target: TextTarget,
        origin: Point,
        start: Appearance,
        current: Appearance,
        stamp: (ProjectId, u64),
    },
}
impl Gesture {
    fn stamp(&self) -> (ProjectId, u64) {
        match self {
            Self::Box { stamp, .. } | Self::Move { stamp, .. } | Self::Text { stamp, .. } => *stamp,
        }
    }
}
fn stamp(app: &App) -> (ProjectId, u64) {
    (app.session.incarnation, app.session.revision())
}
pub(super) fn preview(app: &mut App, doc: &mut Document) {
    if app
        .canvas
        .gesture
        .as_ref()
        .is_some_and(|g| g.stamp() != stamp(app))
    {
        app.canvas.gesture = None;
        return;
    }
    match &app.canvas.gesture {
        Some(Gesture::Move {
            items,
            delta,
            routes,
            ..
        }) => {
            if let Ok(next) = selection::moved(doc, items, *delta, routes) {
                *doc = next;
            }
        }
        Some(Gesture::Text {
            target, current, ..
        }) => {
            doc.text_overrides.retain(|o| o.target != *target);
            doc.text_overrides.push(text::Override {
                target: *target,
                appearance: current.clone(),
            });
        }
        _ => {}
    }
}
fn symbol_at(app: &App, doc: &Document, canvas: Rect, at: Pos2) -> Option<SymbolId> {
    if !app.canvas.symbols_selectable {
        return None;
    }
    doc.symbols
        .iter()
        .rev()
        .find(|s| {
            doc.component(s.id).is_some_and(|c| {
                Rect::from_center_size(
                    app.canvas.screen(s.at, canvas),
                    canvas::symbol_size(c.kind, s.quarter_turns) * app.canvas.zoom,
                )
                .contains(at)
            })
        })
        .map(|s| s.id)
}
fn wire_at(app: &App, paths: &[(WireId, Vec<Pos2>)], at: Pos2) -> Option<WireId> {
    if !app.canvas.nets_selectable {
        return None;
    }
    paths
        .iter()
        .filter(|(_, p)| canvas::path_distance(at, p) <= 7.0)
        .min_by(|a, b| canvas::path_distance(at, &a.1).total_cmp(&canvas::path_distance(at, &b.1)))
        .map(|(id, _)| *id)
}
fn text_at(app: &App, at: Pos2) -> Option<TextTarget> {
    app.canvas
        .text_hits
        .iter()
        .rev()
        .find(|(target, r)| {
            (if matches!(target, TextTarget::Note(_)) {
                app.canvas.texts_selectable
            } else {
                app.canvas.labels_selectable
            }) && r.expand(3.0).contains(at)
        })
        .map(|(target, _)| *target)
}
fn hit_selection(
    app: &App,
    doc: &Document,
    canvas: Rect,
    paths: &[(WireId, Vec<Pos2>)],
    at: Pos2,
) -> Selection {
    let mut s = Selection::default();
    if let Some(text) = text_at(app, at) {
        match text {
            TextTarget::Note(id) => {
                s.notes.insert(id);
            }
            TextTarget::Reference(id) | TextTarget::Value(id) => {
                s.symbols.insert(id);
            }
            TextTarget::Net(id) => {
                s.wires.insert(id);
            }
        }
    } else if let Some(id) = symbol_at(app, doc, canvas, at) {
        s.symbols.insert(id);
    } else if let Some(id) = wire_at(app, paths, at) {
        s.wires.insert(id);
    }
    s
}
fn combine(base: &Selection, hit: &Selection, shift: bool, toggle: bool) -> Selection {
    if !shift && !toggle {
        return hit.clone();
    }
    let mut out = base.clone();
    macro_rules! merge {
        ($field:ident) => {
            for id in &hit.$field {
                if toggle && !out.$field.insert(*id) {
                    out.$field.remove(id);
                } else {
                    out.$field.insert(*id);
                }
            }
        };
    }
    merge!(symbols);
    merge!(wires);
    merge!(notes);
    out
}
fn inside(rect: Rect, body: Rect, crossing: bool) -> bool {
    if crossing {
        rect.intersects(body)
    } else {
        rect.contains_rect(body)
    }
}
fn boxed(
    app: &App,
    doc: &Document,
    canvas: Rect,
    paths: &[(WireId, Vec<Pos2>)],
    start: Pos2,
    at: Pos2,
) -> Selection {
    let rect = Rect::from_two_pos(start, at);
    let crossing = at.x < start.x;
    let mut out = Selection::default();
    if app.canvas.symbols_selectable {
        for s in &doc.symbols {
            if let Some(c) = doc.component(s.id) {
                let body = Rect::from_center_size(
                    app.canvas.screen(s.at, canvas),
                    canvas::symbol_size(c.kind, s.quarter_turns) * app.canvas.zoom,
                );
                if inside(rect, body, crossing) {
                    out.symbols.insert(s.id);
                }
            }
        }
    }
    if app.canvas.nets_selectable {
        for (id, path) in paths {
            let hit = if crossing {
                path.windows(2)
                    .any(|p| rect.intersects(Rect::from_two_pos(p[0], p[1]).expand(0.1)))
            } else {
                !path.is_empty() && path.iter().all(|p| rect.contains(*p))
            };
            if hit {
                out.wires.insert(*id);
            }
        }
    }
    if app.canvas.texts_selectable {
        for (target, bounds) in &app.canvas.text_hits {
            if let TextTarget::Note(id) = target
                && inside(rect, *bounds, crossing)
            {
                out.notes.insert(*id);
            }
        }
    }
    out
}
/// Returns true when this gesture consumes canvas input (including its release).
pub(super) fn update(
    app: &mut App,
    ui: &mut egui::Ui,
    response: &egui::Response,
    doc: &Document,
    canvas: Rect,
    paths: &[(WireId, Vec<Pos2>)],
    enabled: bool,
) -> bool {
    if app.tool != Tool::Select || app.busy() {
        app.canvas.gesture = None;
        return false;
    }
    if !enabled {
        if ui.input(|i| i.pointer.any_released()) {
            app.canvas.gesture = None;
        }
        return app.canvas.gesture.is_some();
    }
    let pointer = ui.input(|i| i.pointer.interact_pos());
    let modifiers = ui.input(|i| i.modifiers);
    if app.canvas.gesture.is_none()
        && let Some(origin) = ui.input(|i| i.pointer.press_origin())
    {
        let right = response.drag_started_by(egui::PointerButton::Secondary);
        let left = response.drag_started_by(egui::PointerButton::Primary);
        if right || left {
            let hit = hit_selection(app, doc, canvas, paths, origin);
            if right || hit.is_empty() || modifiers.shift || modifiers.ctrl {
                app.canvas.gesture = Some(Gesture::Box {
                    start: origin,
                    at: origin,
                    base: app.selection.clone(),
                    shift: modifiers.shift,
                    toggle: modifiers.ctrl,
                    button: if right {
                        egui::PointerButton::Secondary
                    } else {
                        egui::PointerButton::Primary
                    },
                    stamp: stamp(app),
                });
            } else {
                let contained = hit.symbols.is_subset(&app.selection.symbols)
                    && hit.wires.is_subset(&app.selection.wires)
                    && hit.notes.is_subset(&app.selection.notes);
                if (contained && app.selection.len() > 1) || !hit.notes.is_empty() {
                    let items = if contained {
                        app.selection.clone()
                    } else {
                        hit
                    };
                    app.set_selection(items.clone());
                    let routes = app
                        .canvas
                        .routes
                        .iter()
                        .filter(|r| items.wires.contains(&r.wire))
                        .cloned()
                        .collect();
                    app.canvas.gesture = Some(Gesture::Move {
                        items,
                        origin: app.canvas.world(origin, canvas),
                        delta: Point::default(),
                        routes,
                        stamp: stamp(app),
                    });
                } else if let Some(target) = text_at(app, origin) {
                    app.open_text(target);
                    let start = text::appearance(doc, target);
                    app.canvas.gesture = Some(Gesture::Text {
                        target,
                        origin: app.canvas.world(origin, canvas),
                        current: start.clone(),
                        start,
                        stamp: stamp(app),
                    });
                }
            }
        }
    }
    if let Some(mut gesture) = app.canvas.gesture.take() {
        if gesture.stamp() != stamp(app) {
            app.notice("Project changed. Drag cancelled.");
            return true;
        }
        if let Some(pos) = pointer {
            let world = app.canvas.world(pos, canvas);
            match &mut gesture {
                Gesture::Box { at, .. } => *at = pos,
                Gesture::Move { origin, delta, .. } => {
                    *delta = Point {
                        x: world.x - origin.x,
                        y: world.y - origin.y,
                    };
                    if app.canvas.snap {
                        *delta = delta.snapped();
                    }
                }
                Gesture::Text {
                    origin,
                    start,
                    current,
                    ..
                } => {
                    current.offset = Point {
                        x: (start.offset.x + world.x - origin.x).clamp(-LIMIT_NM, LIMIT_NM),
                        y: (start.offset.y + world.y - origin.y).clamp(-LIMIT_NM, LIMIT_NM),
                    }
                }
            }
        }
        let button = if let Gesture::Box { button, .. } = &gesture {
            *button
        } else {
            egui::PointerButton::Primary
        };
        let released = ui.input(|i| i.pointer.button_released(button));
        if let Gesture::Box { start, at, .. } = &gesture {
            let rect = Rect::from_two_pos(*start, *at);
            let t = theme::current(ui.ctx());
            let p = ui.painter();
            p.rect_filled(rect, 0, t.ui.selection.gamma_multiply(0.25));
            if at.x < start.x {
                for edge in [
                    [rect.left_top(), rect.right_top()],
                    [rect.right_top(), rect.right_bottom()],
                    [rect.right_bottom(), rect.left_bottom()],
                    [rect.left_bottom(), rect.left_top()],
                ] {
                    let len = edge[0].distance(edge[1]);
                    let mut n = 0.0;
                    while n < len {
                        p.line_segment(
                            [
                                edge[0].lerp(edge[1], n / len),
                                edge[0].lerp(edge[1], ((n + 5.0) / len).min(1.0)),
                            ],
                            Stroke::new(1.0, t.ui.accent),
                        );
                        n += 9.0;
                    }
                }
            } else {
                p.rect_stroke(
                    rect,
                    0,
                    Stroke::new(1.0, t.ui.accent),
                    egui::StrokeKind::Inside,
                );
            }
            let hits = boxed(app, doc, canvas, paths, *start, *at);
            p.text(
                rect.left_top() - vec2(0.0, 5.0),
                egui::Align2::LEFT_BOTTOM,
                format!(
                    "{} · {} items",
                    if at.x < start.x { "Crossing" } else { "Window" },
                    hits.len()
                ),
                egui::FontId::proportional(t.metrics.small),
                t.ui.text,
            );
        }
        if released {
            match gesture {
                Gesture::Box {
                    start,
                    at,
                    base,
                    shift,
                    toggle,
                    ..
                } => {
                    let hit = boxed(app, doc, canvas, paths, start, at);
                    app.set_selection(combine(&base, &hit, shift, toggle));
                }
                Gesture::Move {
                    items,
                    delta,
                    routes,
                    stamp: (_, revision),
                    ..
                } => {
                    match app.apply(
                        revision,
                        Command::MoveItems {
                            items,
                            delta,
                            routes,
                        },
                    ) {
                        Ok(_) => {
                            app.refresh_selection();
                            app.refresh_nets();
                        }
                        Err(e) => app.notice(e),
                    }
                }
                Gesture::Text {
                    target,
                    current,
                    stamp: (_, revision),
                    ..
                } => {
                    if let Some(content) = target.content(app.document()) {
                        match app.apply(
                            revision,
                            Command::SetText {
                                target,
                                content,
                                appearance: current,
                            },
                        ) {
                            Ok(_) => app.open_text(target),
                            Err(e) => app.notice(e),
                        }
                    }
                }
            }
        } else {
            app.canvas.gesture = Some(gesture);
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            ui.ctx().request_repaint();
        }
        return true;
    }
    if response.clicked()
        && let Some(pos) = pointer
    {
        let hit = hit_selection(app, doc, canvas, paths, pos);
        if let Some(target) = text_at(app, pos) {
            let appearance = text::appearance(doc, target);
            if modifiers.ctrl && !appearance.link.is_empty() {
                ui.ctx().open_url(egui::OpenUrl::new_tab(appearance.link));
                return true;
            }
            if !modifiers.shift && !modifiers.ctrl {
                app.open_text(target);
                return true;
            }
        }
        if modifiers.shift || modifiers.ctrl {
            app.set_selection(combine(
                &app.selection,
                &hit,
                modifiers.shift,
                modifiers.ctrl,
            ));
            return true;
        }
    }
    false
}

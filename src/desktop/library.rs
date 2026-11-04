//! Parts-browser presentation for the original built-in schematic catalog.
//! Views and previews are disposable UI state; placement uses the Electronics session.
use super::{
    App, Tool, canvas, controls,
    icons::{self, Icon},
    theme,
};
use eframe::egui::{self, Align2, FontId, Rect, RichText, Sense, Stroke, StrokeKind, pos2, vec2};
use pcbcraft_foundation::document::{PartKind, ProjectId};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum View {
    Small,
    #[default]
    Medium,
    Large,
    List,
}
pub(super) struct State {
    pub search: String,
    pub view: View,
    selected: Option<PartKind>,
    was_open: bool,
    pub dock: Dock,
    position: Option<egui::Pos2>,
    dragging: Option<(egui::Pos2, egui::Pos2, Dock)>,
    pub rect: Rect,
}
impl Default for State {
    fn default() -> Self {
        Self {
            search: String::new(),
            view: View::default(),
            selected: None,
            was_open: false,
            dock: Dock::Floating,
            position: None,
            dragging: None,
            rect: Rect::NOTHING,
        }
    }
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(super) enum Dock {
    Left,
    Right,
    #[default]
    Floating,
}
impl State {
    pub fn undock(&mut self, position: egui::Pos2) {
        self.dock = Dock::Floating;
        self.position = Some(position);
    }
    pub fn cancel_drag(&mut self) {
        if let Some((_, original, dock)) = self.dragging.take() {
            self.position = Some(original);
            self.dock = dock;
        }
    }
    pub fn moving(&self) -> bool {
        self.dragging.is_some()
    }
}
#[derive(Clone, Copy)]
pub(super) struct PartDrag {
    pub kind: PartKind,
    pub incarnation: ProjectId,
    pub revision: u64,
}

impl App {
    pub(super) fn parts_library(&mut self, ctx: &egui::Context) {
        if !self.library {
            self.library_ui.was_open = false;
            return;
        }
        let opening = !self.library_ui.was_open;
        self.library_ui.was_open = true;
        let t = theme::current(ctx);
        let layout = self.layout;
        let floating_size = vec2(260.0, layout.floating.height().min(490.0));
        let at = self
            .library_ui
            .position
            .unwrap_or(layout.bounds.min + vec2(280.0, 36.0));
        let at = pos2(
            at.x.clamp(
                layout.floating.left(),
                (layout.floating.right() - floating_size.x).max(layout.floating.left()),
            ),
            at.y.clamp(
                layout.floating.top(),
                (layout.floating.bottom() - floating_size.y).max(layout.floating.top()),
            ),
        );
        let floating = Rect::from_min_size(at, floating_size);
        let rect = match self.library_ui.dock {
            Dock::Left => layout.library_left,
            Dock::Right => layout.library_right,
            Dock::Floating => floating,
        };
        self.library_ui.rect = rect;
        let width = rect.width() - 22.0;
        let compact = rect.height() < 340.0;
        let view = if compact {
            View::List
        } else {
            self.library_ui.view
        };
        let busy = self.busy() || self.pending.is_some();
        controls::card(ctx, "parts_library", rect, egui::Order::Foreground, |ui| {
            ui.set_width(width);
            ui.horizontal(|ui| {
                let (header, response) =
                    ui.allocate_exact_size(vec2(width - 56.0, 26.0), Sense::drag());
                controls::grip(ui, header);
                ui.painter().text(
                    header.left_center() + vec2(18.0, 0.0),
                    Align2::LEFT_CENTER,
                    "Parts library",
                    theme::semibold(t.metrics.body),
                    t.ui.text,
                );
                self.library_header_drag(ctx, &response, rect);
                response
                    .on_hover_cursor(egui::CursorIcon::Grab)
                    .on_hover_text("Drag to float. Release at the left or right edge to dock.");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icons::button(ui, Icon::Close, "Close parts library", true).clicked() {
                        self.library = false;
                        if matches!(self.tool, Tool::Place(_)) {
                            self.tool = Tool::Select;
                        }
                        egui::DragAndDrop::clear_payload(ctx);
                    }
                    controls::menu_button(ui, "…", |ui| {
                        controls::section(ui, "PANEL POSITION");
                        for (dock, label) in [
                            (Dock::Floating, "Float"),
                            (Dock::Left, "Dock left"),
                            (Dock::Right, "Dock right"),
                        ] {
                            if controls::MenuItem::new(label, Icon::Pan)
                                .checked(self.library_ui.dock == dock)
                                .show(ui)
                                .clicked()
                            {
                                self.library_ui.dock = dock;
                                self.library_ui.position =
                                    Some(layout.bounds.min + vec2(280.0, 36.0));
                                ui.close();
                            }
                        }
                    })
                    .response
                    .on_hover_text("Float or dock the library");
                });
            });
            ui.separator();
            let query = self.library_ui.search.trim().to_lowercase();
            let parts: Vec<_> = PartKind::ALL
                .into_iter()
                .filter(|kind| {
                    let text = format!("{} {}", kind.name(), kind.value()).to_lowercase();
                    query.split_whitespace().all(|word| text.contains(word))
                })
                .collect();
            if rect.height() >= 240.0 {
                ui.horizontal(|ui| {
                    ui.add(Icon::Folder.image(14.0, ui.ctx()));
                    ui.label(RichText::new("Built-in symbols").small().color(t.ui.muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} {}",
                                parts.len(),
                                if parts.len() == 1 { "part" } else { "parts" }
                            ))
                            .small()
                            .color(t.ui.muted),
                        );
                    });
                });
            }
            ui.horizontal(|ui| {
                let search_width = width
                    - if self.library_ui.search.is_empty() {
                        0.0
                    } else {
                        26.0
                    };
                let response = ui.add_sized(
                    [search_width, 28.0],
                    controls::text_input(
                        egui::TextEdit::singleline(&mut self.library_ui.search)
                            .hint_text("Search parts…"),
                    ),
                );
                if opening {
                    response.request_focus();
                }
                if !self.library_ui.search.is_empty()
                    && icons::button(ui, Icon::Close, "Clear search", true).clicked()
                {
                    self.library_ui.search.clear();
                }
            });
            if !compact {
                ui.horizontal(|ui| {
                    theme::quiet_buttons(ui);
                    ui.spacing_mut().item_spacing.x = 2.0;
                    for (view, name) in [
                        (View::Small, "Small"),
                        (View::Medium, "Medium"),
                        (View::Large, "Large"),
                        (View::List, "List"),
                    ] {
                        if ui
                            .add_sized(
                                [(width - 6.0) / 4.0, 22.0],
                                egui::Button::new(RichText::new(name).small()).selected(
                                    if compact {
                                        view == View::List
                                    } else {
                                        self.library_ui.view == view
                                    },
                                ),
                            )
                            .clicked()
                        {
                            self.library_ui.view = view;
                        }
                    }
                });
            }
            ui.add_space(3.0);
            egui::ScrollArea::vertical()
                .id_salt("library_parts")
                .max_height((ui.available_height() - 36.0).max(0.0))
                .min_scrolled_height(0.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if parts.is_empty() {
                        ui.add_space(18.0);
                        ui.label(
                            RichText::new("No matching parts")
                                .font(theme::semibold(t.metrics.body)),
                        );
                        ui.label(
                            RichText::new("Try a name or value.")
                                .small()
                                .color(t.ui.muted),
                        );
                        if controls::action(
                            ui,
                            "Clear search",
                            controls::ActionTone::Secondary,
                            true,
                        )
                        .clicked()
                        {
                            self.library_ui.search.clear();
                        }
                        ui.add_space(18.0);
                    } else {
                        let columns = match view {
                            View::Small => 3,
                            View::Medium => 2,
                            View::Large | View::List => 1,
                        };
                        let tile_width = (width - 8.0 * (columns - 1) as f32) / columns as f32;
                        for row in parts.chunks(columns) {
                            ui.horizontal_top(|ui| {
                                ui.spacing_mut().item_spacing.x = 8.0;
                                for &kind in row {
                                    let response = ui
                                        .add_enabled_ui(!busy, |ui| {
                                            tile(
                                                ui,
                                                kind,
                                                view,
                                                tile_width,
                                                self.library_ui.selected == Some(kind),
                                            )
                                        })
                                        .inner;
                                    let activate = response.clicked()
                                        || (response.has_focus()
                                            && ui.input_mut(|i| {
                                                i.consume_key(
                                                    egui::Modifiers::NONE,
                                                    egui::Key::Enter,
                                                ) || i.consume_key(
                                                    egui::Modifiers::NONE,
                                                    egui::Key::Space,
                                                )
                                            }));
                                    if activate && !busy {
                                        self.library_ui.selected = Some(kind);
                                        self.tool = Tool::Place(kind);
                                        self.canvas.drag = None;
                                        self.canvas.wire_start = None;
                                    }
                                    if response.drag_started() {
                                        self.library_ui.selected = Some(kind);
                                        self.tool = Tool::Select;
                                        self.canvas.drag = None;
                                        self.canvas.wire_start = None;
                                    }
                                    response.dnd_set_drag_payload(PartDrag {
                                        kind,
                                        incarnation: self.session.incarnation,
                                        revision: self.session.revision(),
                                    });
                                }
                            });
                        }
                    }
                });
            let hint = if busy {
                "Finish the file operation before placing parts.".to_owned()
            } else if let Tool::Place(kind) = self.tool {
                format!(
                    "Click to place {} repeatedly. Right-click or Esc finishes.",
                    kind.name()
                )
            } else if compact {
                "Drag a part onto the sheet. Float for larger previews.".to_owned()
            } else {
                "Click a part, then the sheet. Or drag to place.".to_owned()
            };
            controls::catalog_footer(ui, &hint);
        });
        if self.library_ui.moving() {
            let pointer = ctx.input(|i| i.pointer.interact_pos());
            if let Some(pos) = pointer {
                let dock = self.library_snap(pos);
                let preview =
                    super::panels::Layout::for_library_dock(self, layout.bounds.expand(12.0), dock);
                let target = match dock {
                    Dock::Left => Some(preview.library_left),
                    Dock::Right => Some(preview.library_right),
                    Dock::Floating => None,
                };
                if let Some(target) = target {
                    let painter = ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Tooltip,
                        "library_snap_preview".into(),
                    ));
                    painter.rect_filled(target, 12, t.ui.selection.gamma_multiply(0.7));
                    painter.rect_stroke(
                        target,
                        12,
                        Stroke::new(2.0, t.ui.accent),
                        StrokeKind::Inside,
                    );
                    painter.text(
                        target.center(),
                        Align2::CENTER_CENTER,
                        "Release to dock",
                        theme::semibold(t.metrics.body),
                        t.ui.text,
                    );
                }
            }
        }
    }

    fn library_snap(&self, pos: egui::Pos2) -> Dock {
        let bounds = self.layout.bounds;
        if pos.y < bounds.top() || pos.y > bounds.bottom() {
            return Dock::Floating;
        }
        if pos.x < bounds.left() + 44.0 {
            Dock::Left
        } else if pos.x > bounds.right() - 44.0 {
            Dock::Right
        } else {
            Dock::Floating
        }
    }

    fn library_header_drag(&mut self, ctx: &egui::Context, response: &egui::Response, rect: Rect) {
        if response.drag_started()
            && let Some(origin) = ctx.input(|i| i.pointer.press_origin())
        {
            self.library_ui.dragging = Some((origin, rect.min, self.library_ui.dock));
        }
        if response.dragged()
            && let Some((origin, position, _)) = self.library_ui.dragging
            && let Some(pointer) = response.interact_pointer_pos()
        {
            self.library_ui.undock(position + (pointer - origin));
            ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        }
        if response.drag_stopped()
            && self.library_ui.dragging.take().is_some()
            && let Some(pointer) = ctx.input(|i| i.pointer.interact_pos())
        {
            self.library_ui.dock = self.library_snap(pointer);
            ctx.request_repaint();
        }
    }
}

fn tile(
    ui: &mut egui::Ui,
    kind: PartKind,
    view: View,
    width: f32,
    selected: bool,
) -> egui::Response {
    let t = theme::current(ui.ctx());
    let height = match view {
        View::Small => 94.0,
        View::Medium => 126.0,
        View::Large => 180.0,
        View::List => 58.0,
    } * (t.metrics.body / 12.0).max(1.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click_and_drag());
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let preview = if view == View::List {
        Rect::from_min_size(rect.min, vec2(66.0, height - 4.0))
    } else {
        Rect::from_min_max(rect.min, pos2(rect.right(), rect.bottom() - 35.0))
    };
    painter.rect_filled(
        preview,
        t.metrics.radius,
        if selected {
            t.ui.selection
        } else {
            t.ui.raised
        },
    );
    if selected || response.hovered() || response.has_focus() {
        painter.rect_stroke(
            preview.shrink(0.5),
            t.metrics.radius,
            Stroke::new(
                1.0,
                if selected || response.has_focus() {
                    t.ui.accent
                } else {
                    t.ui.border
                },
            ),
            StrokeKind::Inside,
        );
    }
    let size = canvas::symbol_size(kind, 0);
    let scale = ((preview.width() - 20.0) / size.x)
        .min((preview.height() - 25.0) / size.y)
        .clamp(0.04, 1.4);
    canvas::paint_symbol(
        &painter,
        preview.center() + vec2(0.0, 3.0),
        scale,
        0,
        kind,
        t.canvas.symbol,
    );
    if view != View::List {
        let badge = Rect::from_min_size(preview.min + vec2(5.0, 5.0), vec2(28.0, 13.0));
        painter.rect_filled(badge, 3, t.ui.surface);
        painter.text(
            badge.center(),
            Align2::CENTER_CENTER,
            "SYM",
            FontId::proportional(8.0),
            t.ui.muted,
        );
    }
    let (x, y, max_width) = if view == View::List {
        (rect.left() + 76.0, rect.top() + 9.0, width - 80.0)
    } else {
        (rect.left() + 2.0, preview.bottom() + 4.0, width - 4.0)
    };
    for (line, offset, font, color) in [
        (
            kind.name(),
            0.0,
            theme::semibold(t.metrics.small),
            t.ui.text,
        ),
        (
            kind.value(),
            16.0,
            FontId::proportional(t.metrics.small),
            t.ui.muted,
        ),
    ] {
        let mut job = egui::text::LayoutJob::simple_singleline(line.into(), font, color);
        job.wrap.max_width = max_width;
        job.wrap.max_rows = 1;
        painter.galley(pos2(x, y + offset), painter.layout_job(job), color);
    }
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), kind.name())
    });
    response
        .on_hover_text(format!(
            "{} · {}\n{} pins · built-in symbol\nClick to place or drag onto the sheet.",
            kind.name(),
            kind.value(),
            kind.pins().len()
        ))
        .on_disabled_hover_text("Finish the current file operation first.")
}

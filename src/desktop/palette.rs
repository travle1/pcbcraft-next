//! PCBcraft command search, following SolidCraft ui_kit/search composition.
//! Original catalog and dispatch. Reference/provenance: docs/panel-controls-provenance.md.
use super::{Action, App, Tool, controls, icons::Icon, theme};
use eframe::egui::{self, Align2, Rect, RichText, Sense, Stroke, pos2, vec2};

pub(super) struct State {
    pub anchor: Rect,
    pub selected: usize,
    was_open: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            anchor: Rect::NOTHING,
            selected: 0,
            was_open: false,
        }
    }
}
#[derive(Clone, Copy)]
struct Entry {
    id: &'static str,
    name: &'static str,
    path: &'static str,
    icon: Icon,
    key: &'static str,
}
const ENTRIES: [Entry; 14] = [
    Entry {
        id: "symbol",
        name: "Place symbol",
        path: "CAPTURE › PLACE",
        icon: Icon::Symbol,
        key: "A",
    },
    Entry {
        id: "wire",
        name: "Draw wire",
        path: "CAPTURE › CONNECT",
        icon: Icon::Wire,
        key: "W",
    },
    Entry {
        id: "sheet.new",
        name: "New flat sheet",
        path: "MANAGE › SHEETS",
        icon: Icon::Sheet,
        key: "",
    },
    Entry {
        id: "sheet.child",
        name: "New hierarchical subsheet",
        path: "MANAGE › SHEETS",
        icon: Icon::Sheet,
        key: "",
    },
    Entry {
        id: "sheet.port",
        name: "Add interface port",
        path: "CAPTURE › PLACE",
        icon: Icon::Port,
        key: "",
    },
    Entry {
        id: "text",
        name: "Add text",
        path: "CAPTURE › PLACE",
        icon: Icon::Annotate,
        key: "T",
    },
    Entry {
        id: "fit",
        name: "Fit schematic",
        path: "VIEW › NAVIGATION",
        icon: Icon::Zoom,
        key: "F",
    },
    Entry {
        id: "undo",
        name: "Undo",
        path: "EDIT › HISTORY",
        icon: Icon::Undo,
        key: "Ctrl Z",
    },
    Entry {
        id: "redo",
        name: "Redo",
        path: "EDIT › HISTORY",
        icon: Icon::Redo,
        key: "Ctrl Shift Z",
    },
    Entry {
        id: "save",
        name: "Save draft",
        path: "FILE",
        icon: Icon::Save,
        key: "Ctrl S",
    },
    Entry {
        id: "open",
        name: "Open draft",
        path: "FILE",
        icon: Icon::Folder,
        key: "Ctrl O",
    },
    Entry {
        id: "new",
        name: "New schematic",
        path: "FILE",
        icon: Icon::Plus,
        key: "Ctrl N",
    },
    Entry {
        id: "example",
        name: "Open example",
        path: "FILE",
        icon: Icon::Home,
        key: "",
    },
    Entry {
        id: "erc",
        name: "Show ERC availability",
        path: "ANALYZE · CHECKS NOT IMPLEMENTED",
        icon: Icon::Erc,
        key: "",
    },
];

impl App {
    fn palette_enabled(&self, entry: Entry) -> bool {
        !self.busy()
            && self.pending.is_none()
            && match entry.id {
                "undo" => self.session.can_undo(),
                "redo" => self.session.can_redo(),
                _ => true,
            }
    }
    pub(super) fn command_palette(&mut self, ctx: &egui::Context) {
        if !self.palette {
            self.palette_ui.was_open = false;
            return;
        }
        let opening = !self.palette_ui.was_open;
        self.palette_ui.was_open = true;
        let t = theme::current(ctx);
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.palette = false;
            return;
        }
        let anchor = self.palette_ui.anchor;
        let width = 520.0_f32.min(ctx.content_rect().width() - 32.0);
        let position = pos2((anchor.right() - width).max(12.0), anchor.top());
        let mut run = None;
        let response = egui::Area::new("command_palette".into())
            .fixed_pos(position)
            .order(egui::Order::Foreground)
            .movable(false)
            .show(ctx, |ui| {
                ui.set_width(width);
                ui.spacing_mut().item_spacing.y = 6.0;
                egui::Frame::new()
                    .fill(t.ui.field)
                    .stroke(Stroke::new(1.0, t.ui.accent))
                    .corner_radius(12)
                    .inner_margin(8)
                    .show(ui, |ui| {
                        ui.set_width(width - 18.0);
                        ui.horizontal(|ui| {
                            ui.add(Icon::Zoom.image(18.0, ui.ctx()));
                            let field = ui.add_sized(
                                [width - 94.0, 26.0],
                                controls::text_input(
                                    egui::TextEdit::singleline(&mut self.query)
                                        .frame(egui::Frame::NONE)
                                        .id("command_search_input".into())
                                        .hint_text("Search commands")
                                        .char_limit(256),
                                ),
                            );
                            if opening {
                                field.request_focus();
                            }
                            if field.changed() {
                                self.palette_ui.selected = 0;
                            }
                            if ui
                                .small_button("Esc")
                                .on_hover_text("Close command search")
                                .clicked()
                            {
                                self.palette = false;
                            }
                        });
                    });
                t.panel_frame()
                    .corner_radius(12)
                    .inner_margin(6)
                    .show(ui, |ui| {
                        ui.set_width(width - 14.0);
                        let query = self.query.trim().to_lowercase();
                        let entries: Vec<_> = ENTRIES
                            .into_iter()
                            .filter(|e| {
                                format!("{} {}", e.name, e.path)
                                    .to_lowercase()
                                    .contains(&query)
                            })
                            .collect();
                        self.palette_ui.selected = self
                            .palette_ui
                            .selected
                            .min(entries.len().saturating_sub(1));
                        let changed = ctx.input_mut(|i| {
                            if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
                                self.palette_ui.selected = (self.palette_ui.selected + 1)
                                    .min(entries.len().saturating_sub(1));
                                true
                            } else if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                                self.palette_ui.selected =
                                    self.palette_ui.selected.saturating_sub(1);
                                true
                            } else {
                                false
                            }
                        });
                        if let Some(entry) = entries.get(self.palette_ui.selected).copied() {
                            if ctx.input_mut(|i| {
                                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                            }) && self.palette_enabled(entry)
                            {
                                run = Some(entry.id);
                            }
                            if ctx
                                .input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab))
                            {
                                self.query = entry.name.into();
                                self.palette_ui.selected = 0;
                            }
                        }
                        controls::section(ui, &format!("COMMANDS  {}", entries.len()));
                        egui::ScrollArea::vertical()
                            .max_height(
                                (ctx.content_rect().bottom() - position.y - 138.0).min(318.0),
                            )
                            .min_scrolled_height(0.0)
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                if entries.is_empty() {
                                    ui.add_space(12.0);
                                    ui.label("No matching commands");
                                    ui.label(
                                        RichText::new("Try “symbol”, “wire”, or “save”.")
                                            .color(t.ui.muted),
                                    );
                                    ui.add_space(12.0);
                                }
                                for (index, entry) in entries.iter().copied().enumerate() {
                                    let enabled = self.palette_enabled(entry);
                                    let (rect, response) = ui.allocate_exact_size(
                                        vec2(ui.available_width(), 38.0),
                                        Sense::click(),
                                    );
                                    let selected = index == self.palette_ui.selected;
                                    if enabled && (selected || response.hovered()) {
                                        ui.painter().rect_filled(rect, 6, t.ui.selection);
                                    }
                                    if selected && changed {
                                        response.scroll_to_me(Some(egui::Align::Center));
                                    }
                                    let tile = Rect::from_center_size(
                                        rect.left_center() + vec2(19.0, 0.0),
                                        vec2(26.0, 26.0),
                                    );
                                    ui.painter().rect_filled(tile, 6, t.ui.raised);
                                    entry
                                        .icon
                                        .image(16.0, ui.ctx())
                                        .paint_at(ui, tile.shrink(5.0));
                                    let ink = if enabled { t.ui.text } else { t.ui.disabled };
                                    ui.painter().text(
                                        rect.min + vec2(40.0, 3.0),
                                        Align2::LEFT_TOP,
                                        entry.name,
                                        egui::FontId::proportional(t.metrics.body),
                                        ink,
                                    );
                                    ui.painter().text(
                                        rect.min + vec2(40.0, 20.0),
                                        Align2::LEFT_TOP,
                                        entry.path,
                                        egui::FontId::proportional(t.metrics.small),
                                        t.ui.muted,
                                    );
                                    ui.painter().text(
                                        rect.right_center() - vec2(8.0, 0.0),
                                        Align2::RIGHT_CENTER,
                                        entry.key,
                                        egui::FontId::proportional(t.metrics.small),
                                        t.ui.muted,
                                    );
                                    response.widget_info(|| {
                                        egui::WidgetInfo::labeled(
                                            egui::WidgetType::Button,
                                            enabled,
                                            entry.name,
                                        )
                                    });
                                    if !enabled {
                                        response.clone().on_hover_text(
                                            "Unavailable in the current editor state",
                                        );
                                    }
                                    if enabled && response.clicked() {
                                        run = Some(entry.id);
                                    }
                                }
                            });
                        ui.separator();
                        ui.label(
                            RichText::new("↑ ↓ choose    Enter run    Tab fill name    Esc close")
                                .small()
                                .color(t.ui.muted),
                        );
                    });
            });
        // The dismissal click is consumed by the palette for this entire frame.
        if !opening
            && ctx.input(|i| {
                i.pointer.any_click()
                    && i.pointer
                        .interact_pos()
                        .is_some_and(|p| !response.response.rect.contains(p))
            })
        {
            self.palette = false;
        }
        if let Some(id) = run {
            self.palette = false;
            self.query.clear();
            match id {
                "symbol" => self.library = true,
                "sheet.new" => self.start_sheet_dialog(
                    super::sheets::Kind::Create { parent: None },
                    self.active_sheet(),
                ),
                "sheet.child" => self.start_sheet_dialog(
                    super::sheets::Kind::Create {
                        parent: Some(self.active_sheet()),
                    },
                    self.active_sheet(),
                ),
                "sheet.port" => {
                    self.start_sheet_dialog(super::sheets::Kind::Port(None), self.active_sheet())
                }
                "text" => self.add_note(self.canvas.last_world),
                "wire" => {
                    self.tool = Tool::Wire;
                    self.canvas.wire_start = None;
                }
                "fit" => self.canvas.fit = true,
                "undo" => self.history(false),
                "redo" => self.history(true),
                "save" => self.save(false),
                "open" => self.request(Action::Open),
                "new" => self.request(Action::New),
                "example" => self.request(Action::Example),
                _ => self.drawer = Some("ERC"),
            }
        }
    }
}

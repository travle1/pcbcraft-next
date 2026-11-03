//! Original workspace composition from the user's FigCraft panel/drawer states.
//! All bounds share one workspace. Opening a drawer never changes the camera transform.
use super::{App, Tool, controls, icons::Icon, library::Dock, theme};
use eframe::egui::{self, Align2, Rect, RichText, Sense, Stroke, pos2, vec2};

#[derive(Default)]
pub(super) struct State {
    pub browser_collapsed: bool,
    pub properties_collapsed: bool,
    pub filters_collapsed: bool,
    pub search: String,
    pub browser_height: Option<f32>,
    pub browser_nets: bool,
    pub browser_body: Option<Rect>,
    pub browser_grip: Option<Rect>,
}

#[derive(Clone, Copy)]
pub(super) struct Layout {
    pub bounds: Rect,
    pub browser: Rect,
    pub properties: Rect,
    pub filters: Rect,
    pub drawer: Rect,
    pub navigation: Rect,
    pub library_left: Rect,
    pub library_right: Rect,
    pub floating: Rect,
}
impl Default for Layout {
    fn default() -> Self {
        Self {
            bounds: Rect::NOTHING,
            browser: Rect::NOTHING,
            properties: Rect::NOTHING,
            filters: Rect::NOTHING,
            drawer: Rect::NOTHING,
            navigation: Rect::NOTHING,
            library_left: Rect::NOTHING,
            library_right: Rect::NOTHING,
            floating: Rect::NOTHING,
        }
    }
}
impl Layout {
    pub fn new(app: &App, workspace: Rect) -> Self {
        Self::for_library_dock(app, workspace, app.library_ui.dock)
    }
    pub fn for_library_dock(app: &App, workspace: Rect, dock: Dock) -> Self {
        let bounds = workspace.shrink(12.0);
        let width = 260.0;
        let gap = 12.0;
        let left_library = app.library && dock == Dock::Left;
        let left_properties = app.properties && app.inspector.dock == Dock::Left;
        let right_library = app.library && dock == Dock::Right;
        let drawer = if app.drawer.is_some() {
            Rect::from_min_max(
                pos2(
                    bounds.left(),
                    bounds.bottom() - (bounds.height() * 0.34).clamp(174.0, 236.0),
                ),
                pos2(bounds.right() - width - gap, bounds.bottom()),
            )
        } else {
            Rect::NOTHING
        };
        let free_bottom = if drawer.is_finite() {
            drawer.top() - gap
        } else {
            bounds.bottom() - 50.0
        };
        let browser_budget = free_bottom - bounds.top() - if left_library { 252.0 } else { 0.0 };
        let browser_height = if left_properties
            || app.panel_ui.browser_collapsed
            || browser_budget < 180.0
            || (left_library && drawer.is_finite())
        {
            50.0
        } else {
            browser_budget.min(app.panel_ui.browser_height.unwrap_or(520.0))
        };
        let browser = if app.browser {
            Rect::from_min_size(bounds.min, vec2(width, browser_height))
        } else {
            Rect::NOTHING
        };
        let property_height: f32 = if app.panel_ui.properties_collapsed || right_library {
            50.0
        } else if app.text_ui.draft.is_some() {
            600.0
        } else if app.selection.len() > 1 {
            260.0
        } else if app.nets.selected.is_some() {
            420.0
        } else if app.selected_symbol().is_some() {
            370.0
        } else {
            152.0
        };
        let property_height = property_height
            .min((bounds.height() - if app.filters { 112.0 } else { 50.0 }).max(50.0));
        let properties = if app.properties {
            Rect::from_min_size(
                pos2(bounds.right() - width, bounds.top()),
                vec2(width, property_height),
            )
        } else {
            Rect::NOTHING
        };
        let properties = if app.properties {
            app.inspector
                .bounds(properties, bounds, browser, free_bottom)
        } else {
            properties
        };
        let filters_bottom = bounds.bottom() - 50.0;
        let property_bottom = if properties.is_finite() && app.inspector.dock == Dock::Right {
            properties.bottom()
        } else {
            bounds.top()
        };
        let filter_height = if app.panel_ui.filters_collapsed
            || right_library
            || filters_bottom - property_bottom < 260.0
        {
            50.0
        } else {
            260.0
        };
        let filters = if app.filters {
            Rect::from_min_size(
                pos2(bounds.right() - width, filters_bottom - filter_height),
                vec2(width, filter_height),
            )
        } else {
            Rect::NOTHING
        };
        let library_left = Rect::from_min_max(
            pos2(
                bounds.left(),
                if browser.is_finite() {
                    browser.bottom() + gap
                } else {
                    bounds.top()
                },
            ),
            pos2(bounds.left() + width, free_bottom),
        );
        let library_right = Rect::from_min_max(
            pos2(bounds.right() - width, property_bottom + gap),
            pos2(
                bounds.right(),
                if filters.is_finite() {
                    filters.top() - gap
                } else {
                    filters_bottom
                },
            ),
        );
        let navigation = Rect::from_center_size(
            pos2(
                bounds.center().x,
                if drawer.is_finite() {
                    drawer.top() - 32.0
                } else {
                    bounds.bottom() - 20.0
                },
            ),
            vec2(316.0, 40.0),
        );
        let floating = Rect::from_min_max(bounds.min, pos2(bounds.right(), free_bottom));
        Self {
            bounds,
            browser,
            properties,
            filters,
            drawer,
            navigation,
            library_left,
            library_right,
            floating,
        }
    }
    pub fn blocks_canvas(self, p: egui::Pos2) -> bool {
        [
            self.browser,
            self.properties,
            self.filters,
            self.drawer,
            self.navigation,
        ]
        .iter()
        .any(|r| r.contains(p))
    }
}

/// A left-aligned panel title with an explicit collapse control, as in FigCraft.
/// This does not use egui Window's centered/collapsible title bar.
pub(super) fn header(ui: &mut egui::Ui, title: &str, collapsed: bool) -> egui::Response {
    let t = theme::current(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::click());
    if response.hovered() || response.has_focus() {
        ui.painter().rect_filled(rect, 4, t.ui.hover);
    }
    ui.painter().text(
        rect.left_center() + vec2(2.0, 0.0),
        Align2::LEFT_CENTER,
        title,
        theme::semibold(t.metrics.body),
        t.ui.text,
    );
    let c = rect.right_center() - vec2(10.0, 0.0);
    let sign = if collapsed { -1.0 } else { 1.0 };
    ui.painter().line_segment(
        [c + vec2(-4.0, -2.0 * sign), c + vec2(0.0, 2.0 * sign)],
        Stroke::new(1.3, t.ui.muted),
    );
    ui.painter().line_segment(
        [c + vec2(0.0, 2.0 * sign), c + vec2(4.0, -2.0 * sign)],
        Stroke::new(1.3, t.ui.muted),
    );
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            true,
            format!("{} {title}", if collapsed { "Expand" } else { "Collapse" }),
        )
    });
    response.on_hover_text(if collapsed {
        "Expand panel"
    } else {
        "Collapse panel"
    })
}

fn collapse_button(ui: &mut egui::Ui, tooltip: &str) -> egui::Response {
    let t = theme::current(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(vec2(26.0, 24.0), Sense::click());
    if response.hovered() || response.has_focus() {
        ui.painter().rect_filled(rect, 4, t.ui.hover);
    }
    let c = rect.center();
    ui.painter().line_segment(
        [c + vec2(-4.0, -2.0), c + vec2(0.0, 2.0)],
        Stroke::new(1.3, t.ui.muted),
    );
    ui.painter().line_segment(
        [c + vec2(0.0, 2.0), c + vec2(4.0, -2.0)],
        Stroke::new(1.3, t.ui.muted),
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tooltip));
    response.on_hover_text(tooltip)
}

fn card<R>(
    ctx: &egui::Context,
    id: &'static str,
    rect: Rect,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    super::controls::card(ctx, id, rect, egui::Order::Middle, body)
}

impl App {
    pub(super) fn panels(&mut self, ctx: &egui::Context) {
        let layout = self.layout;
        if self.browser {
            card(ctx, "browser", layout.browser, |ui| {
                let collapsed = layout.browser.height() < 60.0;
                if header(ui, "Browser", collapsed).clicked() {
                    self.panel_ui.browser_collapsed = !collapsed;
                    if collapsed && self.library_ui.dock == Dock::Left {
                        self.library_ui
                            .undock(layout.floating.min + vec2(280.0, 40.0));
                    }
                }
                if !collapsed {
                    ui.separator();
                    ui.horizontal(|ui| {
                        theme::quiet_buttons(ui);
                        ui.selectable_value(&mut self.panel_ui.browser_nets, false, "Symbols");
                        ui.selectable_value(&mut self.panel_ui.browser_nets, true, "Nets");
                    });
                    let available = (ui.available_height() - 52.0).max(0.0);
                    ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                    let output = egui::ScrollArea::vertical()
                        .id_salt(("project_tree", self.panel_ui.browser_nets))
                        .min_scrolled_height(0.0)
                        .max_height(available)
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if self.panel_ui.browser_nets {
                                self.net_browser(ui);
                            } else {
                                self.browser_contents(ui);
                            }
                        });
                    self.panel_ui.browser_body = Some(output.inner_rect);
                    controls::catalog_footer(ui, "Scroll list · drag below to resize");
                    let (grip, resize) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 8.0), Sense::drag());
                    self.panel_ui.browser_grip = Some(grip);
                    let t = theme::current(ctx);
                    ui.painter().hline(
                        grip.center().x - 16.0..=grip.center().x + 16.0,
                        grip.center().y,
                        Stroke::new(2.0, t.ui.border),
                    );
                    if resize.dragged()
                        && let Some(p) = resize.interact_pointer_pos()
                    {
                        self.panel_ui.browser_height =
                            Some((p.y - layout.browser.top() + 5.0).max(210.0));
                    }
                    resize
                        .on_hover_cursor(egui::CursorIcon::ResizeVertical)
                        .on_hover_text("Resize browser");
                }
            });
        }
        if self.properties {
            self.property_panel(ctx);
        }
        if self.filters {
            card(ctx, "display_controls", layout.filters, |ui| {
                let collapsed = layout.filters.height() < 60.0;
                if header(ui, "Selection filters", collapsed).clicked() {
                    self.panel_ui.filters_collapsed = !collapsed;
                    if collapsed && self.library_ui.dock == Dock::Right {
                        self.library_ui
                            .undock(layout.floating.min + vec2(280.0, 40.0));
                    }
                    if collapsed
                        && self.layout.properties.height() > 200.0
                        && layout.bounds.height() < 600.0
                    {
                        self.panel_ui.properties_collapsed = true;
                    }
                }
                if !collapsed {
                    ui.separator();
                    self.filter_contents(ui);
                }
            });
        }
        self.bottom_drawer(ctx);
        self.navigation(ctx);
        self.parts_library(ctx);
    }

    fn browser_contents(&mut self, ui: &mut egui::Ui) {
        let t = theme::current(ui.ctx());
        let doc = self.session.document().clone();
        ui.horizontal(|ui| {
            ui.add(Icon::Folder.image(18.0, ui.ctx()));
            ui.label(RichText::new(&doc.name).font(theme::semibold(t.metrics.body)));
        });
        ui.add_space(6.0);
        if ui
            .add_sized(
                [ui.available_width(), 28.0],
                egui::Button::new("Main    ·    1 sheet")
                    .selected(self.selected_symbol().is_none()),
            )
            .clicked()
        {
            self.select(None);
        }

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Symbols").color(t.ui.muted));
            ui.label(
                RichText::new(doc.symbols.len().to_string())
                    .small()
                    .color(t.ui.muted),
            );
        });
        for s in &doc.symbols {
            if let Some(c) = doc.component(s.id) {
                let selected = self.selection.symbols.contains(&s.id);
                if ui
                    .add_enabled_ui(!self.busy(), |ui| {
                        controls::tree_row(ui, &format!("{}   {}", c.reference, c.value), selected)
                    })
                    .inner
                    .clicked()
                {
                    self.select(Some(s.id));
                    self.tool = Tool::Select;
                }
            }
        }
        if doc.symbols.is_empty() {
            ui.label(
                RichText::new("Place your first symbol from the Parts library.").color(t.ui.muted),
            );
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add(Icon::Wire.image(15.0, ui.ctx()));
            ui.label(
                RichText::new(format!("{} connections", doc.wires.len()))
                    .small()
                    .color(t.ui.muted),
            );
        });
    }

    fn bottom_drawer(&mut self, ctx: &egui::Context) {
        let t = theme::current(ctx);
        let layout = self.layout;
        if let Some(active) = self.drawer {
            card(ctx, "bottom_drawer", layout.drawer, |ui| {
                ui.horizontal(|ui| {
                    theme::quiet_buttons(ui);
                    for tab in ["Search", "ERC", "Messages"] {
                        if ui
                            .add_sized([80.0, 26.0], egui::Button::new(tab).selected(active == tab))
                            .clicked()
                        {
                            self.drawer = Some(tab);
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if collapse_button(ui, "Collapse bottom drawer").clicked() {
                            self.drawer = None;
                        }
                    });
                });
                ui.separator();
                egui::ScrollArea::vertical().id_salt("drawer_body").max_height(layout.drawer.height() - 72.0)
                    .auto_shrink([false, false]).show(ui, |ui| {
                    match active {
                        "Search" => self.sheet_search(ui),
                        "ERC" => {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| { ui.add(Icon::Erc.image(20.0, ui.ctx())); ui.label(RichText::new("Electrical rules check").font(theme::semibold(t.metrics.body))); });
                            ui.add_space(8.0);
                            ui.label(RichText::new("Not available in this build").color(t.ui.warning));
                            ui.label(RichText::new("No electrical validation has run. Wires join explicit pins and junctions; crossings do not connect.").color(t.ui.muted));
                        }
                        _ => {
                            controls::section(ui, &format!("PROJECT MESSAGES  {}", self.notices.len()));
                            ui.add_space(6.0);
                            for message in self.notices.iter().rev() { ui.label(message); ui.add_space(4.0); }
                        }
                    }
                });
            });
        } else {
            let rect = Rect::from_min_size(
                pos2(layout.bounds.left(), layout.bounds.bottom() - 40.0),
                vec2(292.0, 40.0),
            );
            egui::Area::new("bottom_tools".into())
                .fixed_pos(rect.min)
                .movable(false)
                .fade_in(false)
                .show(ctx, |ui| {
                    t.panel_frame()
                        .inner_margin(4)
                        .corner_radius(12)
                        .show(ui, |ui| {
                            theme::quiet_buttons(ui);
                            ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;
                            ui.horizontal(|ui| {
                                for tab in ["Search", "ERC", "Messages"] {
                                    if ui.add_sized([90.0, 30.0], egui::Button::new(tab)).clicked()
                                    {
                                        self.drawer = Some(tab);
                                    }
                                }
                            });
                        });
                });
        }
    }

    fn sheet_search(&mut self, ui: &mut egui::Ui) {
        let t = theme::current(ui.ctx());
        ui.horizontal(|ui| {
            ui.add_sized(
                [ui.available_width() - 110.0, 26.0],
                egui::TextEdit::singleline(&mut self.panel_ui.search)
                    .hint_text("Search schematic…"),
            );
            ui.label(RichText::new("Current sheet").small().color(t.ui.muted));
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Symbols").color(t.ui.text));
            ui.separator();
            ui.label(RichText::new("Reference / value").small().color(t.ui.muted));
        });
        let query = self.panel_ui.search.trim().to_lowercase();
        let doc = self.session.document().clone();
        let mut count = 0;
        for symbol in &doc.symbols {
            if let Some(c) = doc.component(symbol.id) {
                let label = format!("{}    {}    ·    Main", c.reference, c.value);
                if !label.to_lowercase().contains(&query) {
                    continue;
                }
                count += 1;
                if ui
                    .add_sized(
                        [ui.available_width(), 24.0],
                        egui::Button::new(label)
                            .frame(false)
                            .selected(self.selected_symbol() == Some(symbol.id)),
                    )
                    .clicked()
                {
                    self.select(Some(symbol.id));
                    self.tool = Tool::Select;
                }
            }
        }
        if count == 0 {
            ui.label(RichText::new("No matching symbols").color(t.ui.muted));
        }
    }
}

impl App {
    fn filter_contents(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let mut all = self.canvas.symbols_selectable
                && self.canvas.pins_selectable
                && self.canvas.nets_selectable
                && self.canvas.texts_selectable
                && self.canvas.labels_selectable;
            if controls::checkbox(ui, &mut all, "All items")
                .on_hover_text("All supported selectable item types")
                .changed()
            {
                self.canvas.symbols_selectable = all;
                self.canvas.pins_selectable = all;
                self.canvas.nets_selectable = all;
                self.canvas.texts_selectable = all;
                self.canvas.labels_selectable = all;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Reset").clicked() {
                    self.canvas.symbols_selectable = true;
                    self.canvas.pins_selectable = true;
                    self.canvas.nets_selectable = true;
                    self.canvas.texts_selectable = true;
                    self.canvas.labels_selectable = true;
                }
            });
        });
        ui.separator();
        egui::Grid::new("selection_filters")
            .num_columns(2)
            .spacing([20.0, 7.0])
            .show(ui, |ui| {
                controls::checkbox(ui, &mut self.canvas.symbols_selectable, "Symbols");
                controls::checkbox(ui, &mut self.canvas.pins_selectable, "Pins");
                ui.end_row();
                for (index, label) in [
                    "Wires",
                    "Labels",
                    "Graphics",
                    "Images",
                    "Text",
                    "Other items",
                    "Rule areas",
                ]
                .iter()
                .enumerate()
                {
                    let mut unavailable = false;
                    if *label == "Wires" {
                        controls::checkbox(ui, &mut self.canvas.nets_selectable, "Wires")
                            .on_hover_text("Select the connected net from a wire or alias stub");
                    } else if *label == "Text" {
                        controls::checkbox(ui, &mut self.canvas.texts_selectable, "Text");
                    } else if *label == "Labels" {
                        controls::checkbox(ui, &mut self.canvas.labels_selectable, "Labels");
                    } else {
                        ui.add_enabled_ui(false, |ui| {
                            controls::checkbox(ui, &mut unavailable, label)
                        })
                        .inner
                        .on_disabled_hover_text(format!(
                            "{label} selection is not implemented yet."
                        ));
                    }
                    if index % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
    }
    fn navigation(&mut self, ctx: &egui::Context) {
        let t = theme::current(ctx);
        let rect = self.layout.navigation;
        egui::Area::new("navigation".into())
            .fixed_pos(rect.min)
            .movable(false)
            .fade_in(false)
            .show(ctx, |ui| {
                t.panel_frame()
                    .inner_margin(4)
                    .corner_radius(12)
                    .show(ui, |ui| {
                        theme::quiet_buttons(ui);
                        ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
                        ui.spacing_mut().interact_size.y = 30.0;
                        ui.horizontal(|ui| {
                            if ui
                                .add(
                                    egui::Button::image(Icon::Select.image(23.0, ui.ctx()))
                                        .min_size(egui::vec2(32.0, 32.0))
                                        .selected(self.tool == Tool::Select),
                                )
                                .on_hover_text("Select / drag symbols · V")
                                .clicked()
                            {
                                self.tool = Tool::Select;
                            }
                            if ui
                                .add_sized([32.0, 30.0], egui::Button::new("−"))
                                .on_hover_text("Zoom out")
                                .clicked()
                            {
                                self.canvas.zoom = (self.canvas.zoom / 1.2).max(0.1);
                            }
                            if ui
                                .add_sized([88.0, 30.0], egui::Button::new("Fit sheet"))
                                .on_hover_text("Fit schematic · F")
                                .clicked()
                            {
                                self.canvas.fit = true;
                            }
                            if ui
                                .add_sized([32.0, 30.0], egui::Button::new("+"))
                                .on_hover_text("Zoom in")
                                .clicked()
                            {
                                self.canvas.zoom = (self.canvas.zoom * 1.2).min(4.0);
                            }
                            ui.separator();
                            if ui
                                .add(
                                    egui::Button::image(Icon::Pan.image(22.0, ui.ctx()))
                                        .min_size(egui::vec2(32.0, 32.0))
                                        .selected(self.tool == Tool::Pan),
                                )
                                .on_hover_text("Pan · middle-drag also works")
                                .clicked()
                            {
                                self.tool = Tool::Pan;
                            }
                            let response = ui
                                .add(
                                    egui::Button::image(Icon::Zoom.image(20.0, ui.ctx()))
                                        .min_size(egui::vec2(30.0, 32.0)),
                                )
                                .on_hover_text("Zoom level");
                            egui::Popup::menu(&response).show(|ui| {
                                ui.label("Zoom");
                                ui.add(
                                    egui::Slider::new(&mut self.canvas.zoom, 0.1..=4.0)
                                        .logarithmic(true)
                                        .custom_formatter(|zoom, _| {
                                            format!("{:.0}%", zoom * 100.0)
                                        }),
                                );
                            });
                        });
                    });
            });
        if self.tool != Tool::Select {
            egui::Area::new("tool_hint".into())
                .fixed_pos(rect.center_top() - vec2(0.0, 8.0))
                .pivot(Align2::CENTER_BOTTOM)
                .movable(false)
                .fade_in(false)
                .show(ctx, |ui| {
                    t.panel_frame()
                        .inner_margin(6)
                        .corner_radius(8)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let text = match self.tool {
                                    Tool::Place(k) => {
                                        format!("Place {} · click the sheet", k.name())
                                    }
                                    Tool::Wire if self.canvas.wire_start.is_some() => {
                                        "Wire · click for bend · connect to pin/wire · Enter finish".into()
                                    }
                                    Tool::Wire => "Wire · click anywhere to start · click a wire to branch".into(),
                                    Tool::Pan => "Pan · drag the sheet".into(),
                                    Tool::Select => String::new(),
                                };
                                ui.label(RichText::new(text).small().color(t.ui.muted))
                                    .on_hover_text("Space: flip next bend · Backspace: remove last corner · Enter/double-click: finish free end · Right-click/Esc: cancel");
                                if ui
                                    .small_button("Esc")
                                    .on_hover_text("Cancel current tool")
                                    .clicked()
                                {
                                    self.tool = Tool::Select;
                                    self.canvas.wire_start = None;
                                    self.canvas.drag = None;
                                }
                            });
                        });
                });
        }
    }
}

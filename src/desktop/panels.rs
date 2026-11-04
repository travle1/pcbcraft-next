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
    pub search_nets: bool,
    pub project_folded: bool,
    pub sheets_folded: bool,
    pub main_folded: bool,
    pub search_body: Option<Rect>,
    pub browser_body: Option<Rect>,
    pub browser_grip: Option<Rect>,
    pub drawer_height: Option<f32>,
    pub drawer_grip: Option<Rect>,
    pub symbols_open: std::collections::BTreeSet<pcbcraft_foundation::document::SheetOccurrenceId>,
    pub nets_open: std::collections::BTreeSet<pcbcraft_foundation::document::SheetOccurrenceId>,
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
                    bounds.bottom()
                        - app
                            .panel_ui
                            .drawer_height
                            .unwrap_or((bounds.height() * 0.34).clamp(236.0, 268.0))
                            .clamp(160.0, (bounds.height() - 90.0).max(160.0)),
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
            let rows = app.browser_visible_rows();
            browser_budget.min(
                app.panel_ui
                    .browser_height
                    .unwrap_or(70.0 + rows as f32 * 28.0),
            )
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
        } else if app.selected_symbol().is_some() || app.sheets.selected.is_some() {
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
                ui.spacing_mut().item_spacing.y = 2.0;
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
                    let available = (ui.available_height() - 12.0).max(0.0);
                    ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                    let output = egui::ScrollArea::vertical()
                        .id_salt("project_tree")
                        .min_scrolled_height(0.0)
                        .max_height(available)
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                        )
                        .auto_shrink([false, false])
                        .show(ui, |ui| self.browser_contents(ui));
                    self.panel_ui.browser_body = Some(output.inner_rect);
                    ui.add_space(4.0);
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
                            Some((p.y - layout.browser.top() + 5.0).max(130.0));
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

    pub(super) fn open_sheet_index(&mut self, nets: bool) {
        self.panel_ui.search_nets = nets;
        self.panel_ui.search.clear();
        self.drawer = Some("Search");
    }
    fn browser_visible_rows(&self) -> usize {
        fn sheet(app: &App, id: pcbcraft_foundation::document::SheetOccurrenceId) -> usize {
            if app.sheets.folded.contains(&id)
                || id == app.session.project().root && app.panel_ui.main_folded
            {
                return 1;
            }
            let items = app.session.project().document(id).map_or(0, |doc| {
                let symbols = if app.panel_ui.symbols_open.contains(&id) {
                    doc.symbols.len().max(1)
                } else {
                    0
                };
                let nets = if app.panel_ui.nets_open.contains(&id) {
                    pcbcraft_foundation::netlist::Netlist::compile(doc)
                        .map_or(1, |n| n.nets.len().max(1))
                } else {
                    0
                };
                symbols + nets
            });
            3 + items
                + app
                    .session
                    .project()
                    .occurrences
                    .iter()
                    .filter(|o| o.parent == Some(id))
                    .map(|o| sheet(app, o.id))
                    .sum::<usize>()
        }
        if self.panel_ui.project_folded {
            1
        } else if self.panel_ui.sheets_folded {
            3
        } else {
            3 + self
                .session
                .project()
                .occurrences
                .iter()
                .filter(|o| o.parent.is_none())
                .map(|o| sheet(self, o.id))
                .sum::<usize>()
        }
    }
    fn browser_contents(&mut self, ui: &mut egui::Ui) {
        use controls::NavigationRow as Row;
        let project = self.session.project();
        let root_id = ui.id().with(("project", format!("{:?}", project.id)));
        let name = project.name.clone();
        let (response, fold) = Row {
            label: &name,
            icon: Icon::Folder,
            depth: 0,
            selected: false,
            fold: Some(!self.panel_ui.project_folded),
        }
        .show(ui, root_id);
        if fold || response.double_clicked() {
            self.panel_ui.project_folded = !self.panel_ui.project_folded;
        }
        egui::Popup::context_menu(&response)
            .frame(controls::menu_frame(ui.ctx()))
            .show(|ui| self.sheet_menu(ui, None));
        if self.panel_ui.project_folded {
            return;
        }
        ui.add_enabled_ui(false, |ui| {
            Row {
                label: "Project settings",
                icon: Icon::Rules,
                depth: 1,
                selected: false,
                fold: None,
            }
            .show(ui, root_id.with("settings"));
        });
        let (response, fold) = Row {
            label: "Sheets",
            icon: Icon::Folder,
            depth: 1,
            selected: false,
            fold: Some(!self.panel_ui.sheets_folded),
        }
        .show(ui, root_id.with("sheets"));
        if fold || response.double_clicked() {
            self.panel_ui.sheets_folded = !self.panel_ui.sheets_folded;
        }
        egui::Popup::context_menu(&response)
            .frame(controls::menu_frame(ui.ctx()))
            .show(|ui| self.sheet_menu(ui, None));
        if self.panel_ui.sheets_folded {
            return;
        }
        let roots: Vec<_> = self
            .session
            .project()
            .occurrences
            .iter()
            .filter(|o| o.parent.is_none())
            .map(|o| o.id)
            .collect();
        for id in roots {
            self.sheet_browser_row(ui, id, 2);
        }
    }
    fn sheet_browser_row(
        &mut self,
        ui: &mut egui::Ui,
        id: pcbcraft_foundation::document::SheetOccurrenceId,
        depth: u8,
    ) {
        use controls::NavigationRow as Row;
        let Some(o) = self.session.project().occurrence(id) else {
            return;
        };
        let name = o.name.clone();
        let is_root = id == self.session.project().root;
        let folded = self.sheets.folded.contains(&id) || is_root && self.panel_ui.main_folded;
        let row_id = ui.id().with(("sheet", format!("{id:?}")));
        let active = id == self.active_sheet();
        let (response, fold) = Row {
            label: &name,
            icon: Icon::Sheet,
            depth: depth.min(6),
            selected: active,
            fold: Some(!folded),
        }
        .show(ui, row_id);
        if fold {
            if folded {
                self.sheets.folded.remove(&id);
            } else {
                self.sheets.folded.insert(id);
            }
            if is_root {
                self.panel_ui.main_folded = !folded;
            }
        }
        let response = response.on_hover_text(self.session.project().path(id));
        if response.clicked() {
            self.activate_sheet(id);
        }
        egui::Popup::context_menu(&response)
            .frame(controls::menu_frame(ui.ctx()))
            .show(|ui| self.sheet_menu(ui, Some(id)));
        if folded {
            return;
        }
        for (nets, label, icon) in [(false, "Symbols", Icon::Symbol), (true, "Nets", Icon::Wire)] {
            let open = if nets {
                self.panel_ui.nets_open.contains(&id)
            } else {
                self.panel_ui.symbols_open.contains(&id)
            };
            let (response, fold) = (Row {
                label,
                icon,
                depth: (depth + 1).min(7),
                selected: false,
                fold: Some(open),
            })
            .show(ui, row_id.with(label));
            if fold || response.clicked() {
                if nets {
                    if open {
                        self.panel_ui.nets_open.remove(&id);
                    } else {
                        self.panel_ui.nets_open.insert(id);
                    }
                } else if open {
                    self.panel_ui.symbols_open.remove(&id);
                } else {
                    self.panel_ui.symbols_open.insert(id);
                }
            }
            if open {
                self.browser_sheet_items(ui, id, row_id.with(label), depth + 2, nets);
            }
        }
        let children: Vec<_> = self
            .session
            .project()
            .occurrences
            .iter()
            .filter(|o| o.parent == Some(id))
            .map(|o| o.id)
            .collect();
        for child in children {
            self.sheet_browser_row(ui, child, depth + 1);
        }
    }

    fn browser_sheet_items(
        &mut self,
        ui: &mut egui::Ui,
        sheet: pcbcraft_foundation::document::SheetOccurrenceId,
        row_id: egui::Id,
        depth: u8,
        nets: bool,
    ) {
        use controls::NavigationRow as Row;
        let Some(doc) = self.session.project().document(sheet).cloned() else {
            return;
        };
        let active = sheet == self.active_sheet();
        if nets {
            let Ok(snapshot) = pcbcraft_foundation::netlist::Netlist::compile(&doc) else {
                return;
            };
            let mut items = snapshot.nets;
            items.sort_by_key(|n| n.title(&doc));
            if items.is_empty() {
                ui.label(
                    egui::RichText::new("No connected nets")
                        .small()
                        .color(theme::current(ui.ctx()).ui.muted),
                );
            }
            for net in items {
                let title = net.title(&doc);
                let (response, _) = Row {
                    label: &title,
                    icon: Icon::Wire,
                    depth: depth.min(7),
                    selected: active && self.nets.selected.as_ref() == Some(&net.anchor),
                    fold: None,
                }
                .show(ui, row_id.with(format!("{:?}", net.anchor)));
                if response
                    .on_hover_text(format!("{} pins · {}", net.terminals.len(), title))
                    .clicked()
                {
                    self.activate_sheet(sheet);
                    self.select_net(net.anchor);
                }
            }
        } else {
            if doc.symbols.is_empty() {
                ui.label(
                    egui::RichText::new("No symbols placed")
                        .small()
                        .color(theme::current(ui.ctx()).ui.muted),
                );
            }
            for symbol in &doc.symbols {
                let Some(c) = doc.component(symbol.id) else {
                    continue;
                };
                let label = format!("{}  {}", c.reference, c.value);
                let (response, _) = Row {
                    label: &label,
                    icon: Icon::Symbol,
                    depth: depth.min(7),
                    selected: active && self.selection.symbols.contains(&symbol.id),
                    fold: None,
                }
                .show(ui, row_id.with(format!("{:?}", symbol.id)));
                if response.on_hover_text(&label).clicked() {
                    self.activate_sheet(sheet);
                    self.select(Some(symbol.id));
                    self.tool = Tool::Select;
                }
            }
        }
    }

    fn bottom_drawer(&mut self, ctx: &egui::Context) {
        let t = theme::current(ctx);
        let layout = self.layout;
        if let Some(active) = self.drawer {
            if !layout.drawer.is_finite() {
                ctx.request_repaint();
                return;
            }
            card(ctx, "bottom_drawer", layout.drawer, |ui| {
                let (grip, resize) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 8.0), Sense::drag());
                self.panel_ui.drawer_grip = Some(grip);
                ui.painter().hline(
                    grip.center().x - 22.0..=grip.center().x + 22.0,
                    grip.center().y,
                    Stroke::new(
                        2.0,
                        if resize.hovered() || resize.dragged() {
                            t.ui.accent
                        } else {
                            t.ui.border
                        },
                    ),
                );
                if resize.dragged()
                    && let Some(p) = resize.interact_pointer_pos()
                {
                    self.panel_ui.drawer_height =
                        Some((layout.bounds.bottom() - p.y + 14.0).max(160.0));
                }
                resize
                    .on_hover_cursor(egui::CursorIcon::ResizeVertical)
                    .on_hover_text("Drag to resize drawer");
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
                if active == "Search" {
                    self.sheet_search(ui, (layout.drawer.height() - 82.0).max(0.0));
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt("drawer_body")
                        .max_height(layout.drawer.height() - 82.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if active == "ERC" {
                                controls::section(ui, "ELECTRICAL RULES CHECK");
                                ui.colored_label(t.ui.warning, "Not available in this build");
                                ui.label(
                                    "No electrical validation has run. Crossings do not connect.",
                                );
                            } else {
                                controls::section(
                                    ui,
                                    &format!("PROJECT MESSAGES  {}", self.notices.len()),
                                );
                                for message in self.notices.iter().rev() {
                                    ui.label(message);
                                    ui.add_space(4.0);
                                }
                            }
                        });
                }
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

    fn sheet_search(&mut self, ui: &mut egui::Ui, height: f32) {
        let t = theme::current(ui.ctx());
        ui.horizontal(|ui| {
            ui.add_sized(
                [(ui.available_width() - 128.0).max(80.0), 28.0],
                controls::text_input(
                    egui::TextEdit::singleline(&mut self.panel_ui.search)
                        .hint_text("Search schematic…"),
                ),
            );
            ui.add_sized(
                [120.0, 28.0],
                egui::Label::new(
                    RichText::new(format!("Sheet: {}", self.sheet_name()))
                        .small()
                        .color(t.ui.muted),
                )
                .truncate(),
            )
            .on_hover_text(self.session.project().path(self.active_sheet()));
        });
        ui.horizontal(|ui| {
            theme::quiet_buttons(ui);
            ui.selectable_value(&mut self.panel_ui.search_nets, false, "Symbols");
            ui.selectable_value(&mut self.panel_ui.search_nets, true, "Nets");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.panel_ui.search_nets
                    && controls::action(ui, "Netlist…", controls::ActionTone::Secondary, true)
                        .clicked()
                {
                    self.open_net_report();
                }
            });
        });
        let query = self.panel_ui.search.trim().to_lowercase();
        let doc = self.document().clone();
        let output = egui::ScrollArea::vertical()
            .id_salt(("sheet_results", self.panel_ui.search_nets))
            .min_scrolled_height(0.0)
            .max_height((height - 68.0).max(24.0))
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.panel_ui.search_nets {
                    self.net_results(ui, &query);
                    return;
                }
                controls::result_row(
                    ui,
                    ui.id().with("head"),
                    &[("Reference", 0.19), ("Value", 0.56), ("Sheet", 0.25)],
                    false,
                    true,
                );
                let mut count = 0;
                for symbol in &doc.symbols {
                    if let Some(c) = doc.component(symbol.id) {
                        if !format!("{} {}", c.reference, c.value)
                            .to_lowercase()
                            .contains(&query)
                        {
                            continue;
                        }
                        count += 1;
                        if controls::result_row(
                            ui,
                            ui.id().with(format!("{:?}", symbol.id)),
                            &[
                                (&c.reference, 0.19),
                                (&c.value, 0.56),
                                (self.sheet_name(), 0.25),
                            ],
                            self.selection.symbols.contains(&symbol.id),
                            false,
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
            });
        self.panel_ui.search_body = Some(output.inner_rect);
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
                                self.canvas.zoom = (self.canvas.zoom / 1.2).max(0.025);
                            }
                            if ui
                                .add_sized([88.0, 30.0], egui::Button::new("Fit page"))
                                .on_hover_text("Fit page · Shift+F. Fit circuit content with F.")
                                .clicked()
                            {
                                self.canvas.fit = true;
                                self.canvas.fit_page = true;
                            }
                            if ui
                                .add_sized([32.0, 30.0], egui::Button::new("+"))
                                .on_hover_text("Zoom in")
                                .clicked()
                            {
                                self.canvas.zoom = (self.canvas.zoom * 1.2).min(8.0);
                            }
                            ui.label(
                                RichText::new(format!("{:.0}%", self.canvas.zoom * 100.0))
                                    .small()
                                    .color(t.ui.muted),
                            );
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
                                    egui::Slider::new(&mut self.canvas.zoom, 0.025..=8.0)
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
                                        format!("Place {} · click to repeat · right-click/Esc to finish", k.name())
                                    }
                                    Tool::Wire if self.canvas.wire_start.is_some() => {
                                        "Wire · click for bend · connect to pin/wire · Enter finish".into()
                                    }
                                    Tool::Wire => "Wire · click anywhere to start · click a wire to branch".into(),
                                    Tool::SheetPlacement => "Place sheet object · click to repeat · right-click/Esc to finish".into(),
                                    Tool::Text => "Text · drag a box or click to place · right-click/Esc to finish".into(),
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
                                    self.sheets.cancel_drag();
                                }
                            });
                        });
                });
        }
    }
}

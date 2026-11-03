//! Standalone ribbon composition; layout stays separate from command ownership.
use super::{App, Tool, controls, icons::Icon, theme};
use eframe::egui::{self, Align2};
use pcbcraft_foundation::session::Command;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub(super) enum Tab {
    #[default]
    Capture,
    Analyze,
    Libraries,
    Manage,
    View,
}
impl Tab {
    fn groups(self) -> &'static [RibbonGroup] {
        match self {
            Self::Capture => CAPTURE_GROUPS,
            Self::Analyze => ANALYZE_GROUPS,
            Self::Libraries => LIBRARY_GROUPS,
            Self::Manage => MANAGE_GROUPS,
            Self::View => VIEW_GROUPS,
        }
    }
}

impl App {
    pub(super) fn ribbon(&mut self, ui: &mut egui::Ui) {
        let t = theme::current(ui.ctx());
        let m = t.metrics.ribbon();
        egui::Panel::top("ribbon")
            .exact_size(m.height())
            .frame(egui::Frame::new().fill(t.canvas.background))
            .show(ui, |ui| {
                let bounds = ui.max_rect().shrink2(egui::vec2(10.0, m.outer_margin));
                let workspace = egui::Rect::from_min_size(
                    bounds.min,
                    egui::vec2(m.workspace_width, bounds.height()),
                );
                let commands = egui::Rect::from_min_max(
                    egui::pos2(workspace.right() + 8.0, bounds.top()),
                    bounds.max,
                );
                for rect in [workspace, commands] {
                    ui.painter().rect(
                        rect,
                        t.metrics.radius,
                        t.ui.surface,
                        egui::Stroke::new(1.0, t.ui.border),
                        egui::StrokeKind::Inside,
                    );
                }
                // Both cards share the same outer bounds. Rows have explicit vertical budgets.
                super::workspace::picker(ui, workspace);
                let content = commands.shrink(m.card_inset);
                let tabs = egui::Rect::from_min_size(
                    content.min,
                    egui::vec2(content.width(), m.tabs_height),
                );
                let rule_y = tabs.bottom() + m.header_gap * 0.5;
                ui.painter().line_segment(
                    [
                        egui::pos2(content.left(), rule_y),
                        egui::pos2(content.right(), rule_y),
                    ],
                    egui::Stroke::new(1.0, t.ui.border),
                );
                let tools = egui::Rect::from_min_max(
                    egui::pos2(content.left(), tabs.bottom() + m.header_gap),
                    content.max,
                );
                let mut tabs_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("ribbon_tabs")
                        .max_rect(tabs)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );
                tabs_ui.set_clip_rect(tabs.intersect(ui.clip_rect()));
                self.ribbon_tabs(&mut tabs_ui);
                let mut tools_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt("ribbon_tools")
                        .max_rect(tools)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );
                tools_ui.set_clip_rect(tools.intersect(ui.clip_rect()));
                tools_ui.spacing_mut().scroll.bar_width = m.scroll_gutter;
                egui::ScrollArea::horizontal()
                    .id_salt("ribbon_scroll")
                    .auto_shrink([false, false])
                    .max_height(tools.height())
                    .show(&mut tools_ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.horizontal_top(|ui| self.ribbon_tools(ui));
                    });
            });
    }
    fn ribbon_tabs(&mut self, ui: &mut egui::Ui) {
        let t = theme::current(ui.ctx());
        ui.spacing_mut().interact_size.y = t.metrics.ribbon().tabs_height;
        ui.horizontal(|ui| {
            theme::quiet_buttons(ui);
            ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
            for (tab, label) in [
                (Tab::Capture, "Capture"),
                (Tab::Analyze, "Analyze"),
                (Tab::Libraries, "Libraries"),
                (Tab::Manage, "Manage"),
                (Tab::View, "View"),
            ] {
                if ui
                    .add(egui::Button::new(label).selected(self.ribbon_tab == tab))
                    .clicked()
                {
                    self.ribbon_tab = tab;
                }
            }
            ui.add_enabled(false, egui::Button::image(Icon::Plus.image(14.0, ui.ctx())))
                .on_disabled_hover_text("Ribbon customization is not implemented yet.");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let button = egui::Button::image_and_text(
                    Icon::Zoom.image(14.0, ui.ctx()),
                    "Search commands",
                )
                .stroke(egui::Stroke::new(1.0, t.ui.border));
                let response = ui.add_sized(
                    [
                        170.0 * t.metrics.body / 12.0,
                        t.metrics.ribbon().tabs_height,
                    ],
                    button,
                );
                self.palette_ui.anchor = response.rect;
                let key = egui::Rect::from_center_size(
                    egui::pos2(response.rect.right() - 9.0, response.rect.center().y),
                    egui::vec2(11.0, 15.0),
                );
                ui.painter().rect_stroke(
                    key,
                    2,
                    egui::Stroke::new(1.0, t.ui.border),
                    egui::StrokeKind::Inside,
                );
                ui.painter().text(
                    key.center(),
                    Align2::CENTER_CENTER,
                    "S",
                    egui::FontId::proportional(t.metrics.small),
                    t.ui.muted,
                );
                if response.clicked() {
                    self.palette = true;
                }
            });
        });
    }
    fn ribbon_tools(&mut self, ui: &mut egui::Ui) {
        // At the minimum window width, reduce only inter-group padding. Labels
        // keep their normal size; larger text uses the existing horizontal scroll.
        let compact = ui.available_width() < 920.0;
        for (index, group) in self.ribbon_tab.groups().iter().enumerate() {
            if index > 0 {
                group_separator(ui, compact);
            }
            if let Some(action) = tool_group(ui, self, group, compact) {
                self.run_ribbon_action(action);
            }
        }
    }
    fn run_ribbon_action(&mut self, action: RibbonAction) {
        if action.disabled_reason(self).is_some() {
            return;
        }
        match action {
            RibbonAction::Symbol => self.library = true,
            RibbonAction::Text => self.add_note(self.canvas.last_world),
            RibbonAction::Wire => {
                self.tool = Tool::Wire;
                self.canvas.wire_start = None;
            }
            RibbonAction::Move => self.tool = Tool::Select,
            RibbonAction::Rotate => {
                if let Some(symbol) = self.selected_symbol() {
                    self.command(Command::Rotate { symbol });
                }
            }
            RibbonAction::Delete => {
                self.delete_selection();
            }
            RibbonAction::Erc => self.drawer = Some("ERC"),
            RibbonAction::Browser => self.browser = !self.browser,
            RibbonAction::Properties => self.properties = !self.properties,
            RibbonAction::Filters => self.filters = !self.filters,
            RibbonAction::Grid => self.canvas.grid = !self.canvas.grid,
            RibbonAction::Snap => self.canvas.snap = !self.canvas.snap,
            RibbonAction::Pins => self.canvas.pins_visible = !self.canvas.pins_visible,
            RibbonAction::Wires => self.canvas.wires_visible = !self.canvas.wires_visible,
            RibbonAction::Fit => self.canvas.fit = true,
            RibbonAction::Save => self.save(false),
            RibbonAction::Undo => self.history(false),
            RibbonAction::Redo => self.history(true),
            _ => {} // Planned commands have no edit path.
        }
    }
}

#[derive(Clone, Copy)]
enum RibbonAction {
    Symbol,
    Text,
    Power,
    Port,
    Wire,
    Bus,
    Label,
    Move,
    Rotate,
    Mirror,
    Rules,
    Erc,
    Annotate,
    Bom,
    Sync,
    Delete,
    Browser,
    Properties,
    Filters,
    Grid,
    Snap,
    Pins,
    Wires,
    Fit,
    Save,
    Undo,
    Redo,
    Sources,
    Upgrade,
    Variants,
}
impl RibbonAction {
    fn shortcut(self) -> &'static str {
        match self {
            Self::Symbol => "A",
            Self::Text => "T",
            Self::Wire => "W",
            Self::Move => "V",
            Self::Rotate => "R",
            Self::Delete => "Del",
            Self::Fit => "F",
            Self::Save => "Ctrl+S",
            Self::Undo => "Ctrl+Z",
            Self::Redo => "Ctrl+Shift+Z",
            _ => "",
        }
    }

    fn selected(self, app: &App) -> bool {
        match self {
            Self::Browser => app.browser,
            Self::Properties => app.properties,
            Self::Filters => app.filters,
            Self::Grid => app.canvas.grid,
            Self::Snap => app.canvas.snap,
            Self::Pins => app.canvas.pins_visible,
            Self::Wires => app.canvas.wires_visible,
            Self::Wire => app.tool == Tool::Wire,
            _ => false,
        }
    }

    fn disabled_reason(self, app: &App) -> Option<&'static str> {
        match self {
            Self::Sources => Some("External library sources are not implemented yet."),
            Self::Upgrade => Some("Reviewable library upgrades are not implemented yet."),
            Self::Variants => Some("Assembly variants are not implemented yet."),
            Self::Undo if !app.session.can_undo() => Some("Nothing to undo."),
            Self::Redo if !app.session.can_redo() => Some("Nothing to redo."),
            Self::Browser
            | Self::Properties
            | Self::Filters
            | Self::Grid
            | Self::Snap
            | Self::Pins
            | Self::Wires
            | Self::Fit => None,
            Self::Power => Some("Power symbols are not implemented yet."),
            Self::Port => Some("Hierarchical ports are not implemented yet."),
            Self::Bus => Some("Buses are not implemented yet."),
            Self::Label => Some("Net labels are not implemented yet."),
            Self::Mirror => Some("Symbol mirroring is not implemented yet."),
            Self::Rules => Some("Rule editing is not implemented yet."),
            Self::Annotate => Some("Automatic annotation is not implemented yet."),
            Self::Bom => Some("BOM generation is not implemented yet."),
            Self::Sync => Some("Schematic-to-board synchronization is not implemented yet."),
            Self::Erc => None,
            _ if app.busy() => Some("Finish the current file operation first."),
            Self::Delete if app.selection.is_empty() && app.nets.selected.is_none() => {
                Some("Select a symbol, wire or net first.")
            }
            Self::Rotate if app.selected_symbol().is_none() => Some("Select a symbol first."),
            _ => None,
        }
    }
}
struct RibbonTool {
    action: RibbonAction,
    label: &'static str,
    icon: Icon,
    hint: &'static str,
}
struct RibbonGroup {
    label: &'static str,
    tools: &'static [RibbonTool],
    extras: &'static [RibbonTool],
}
const CAPTURE_GROUPS: &[RibbonGroup] = &[
    RibbonGroup {
        label: "Place",
        tools: &[
            RibbonTool {
                action: RibbonAction::Symbol,
                label: "Symbol",
                icon: Icon::Symbol,
                hint: "Place a symbol · A",
            },
            RibbonTool {
                action: RibbonAction::Power,
                label: "Power",
                icon: Icon::Power,
                hint: "Place a power symbol",
            },
            RibbonTool {
                action: RibbonAction::Port,
                label: "Port",
                icon: Icon::Port,
                hint: "Place a hierarchical port",
            },
        ],
        extras: &[RibbonTool {
            action: RibbonAction::Text,
            label: "Text",
            icon: Icon::Annotate,
            hint: "Add sheet text · T",
        }],
    },
    RibbonGroup {
        label: "Connect",
        tools: &[
            RibbonTool {
                action: RibbonAction::Wire,
                label: "Wire",
                icon: Icon::Wire,
                hint: "Draw wires, bends and branches · W",
            },
            RibbonTool {
                action: RibbonAction::Bus,
                label: "Bus",
                icon: Icon::Bus,
                hint: "Draw a bus",
            },
            RibbonTool {
                action: RibbonAction::Label,
                label: "Label",
                icon: Icon::Label,
                hint: "Place a net label",
            },
        ],
        extras: &[],
    },
    RibbonGroup {
        label: "Modify",
        tools: &[
            RibbonTool {
                action: RibbonAction::Move,
                label: "Move",
                icon: Icon::Move,
                hint: "Select and drag a symbol · V",
            },
            RibbonTool {
                action: RibbonAction::Rotate,
                label: "Rotate",
                icon: Icon::Rotate,
                hint: "Rotate selected symbol · R",
            },
            RibbonTool {
                action: RibbonAction::Mirror,
                label: "Mirror",
                icon: Icon::Mirror,
                hint: "Mirror selected symbol",
            },
        ],
        extras: &[RibbonTool {
            action: RibbonAction::Delete,
            label: "Delete",
            icon: Icon::Delete,
            hint: "Delete selected wire, net or symbol",
        }],
    },
    RibbonGroup {
        label: "Inspect",
        tools: &[
            RibbonTool {
                action: RibbonAction::Rules,
                label: "Rules",
                icon: Icon::Rules,
                hint: "Electrical design rules",
            },
            RibbonTool {
                action: RibbonAction::Erc,
                label: "ERC",
                icon: Icon::Erc,
                hint: "ERC status · electrical checks are not implemented yet",
            },
            RibbonTool {
                action: RibbonAction::Annotate,
                label: "Annotate",
                icon: Icon::Annotate,
                hint: "Assign reference designators",
            },
            RibbonTool {
                action: RibbonAction::Bom,
                label: "BOM",
                icon: Icon::Bom,
                hint: "Generate a bill of materials",
            },
        ],
        extras: &[],
    },
    RibbonGroup {
        label: "Design",
        tools: &[RibbonTool {
            action: RibbonAction::Sync,
            label: "Sync",
            icon: Icon::Sync,
            hint: "Synchronize schematic and PCB",
        }],
        extras: &[],
    },
];
const fn tool(
    action: RibbonAction,
    label: &'static str,
    icon: Icon,
    hint: &'static str,
) -> RibbonTool {
    RibbonTool {
        action,
        label,
        icon,
        hint,
    }
}
const ANALYZE_GROUPS: &[RibbonGroup] = &[
    RibbonGroup {
        label: "Electrical",
        tools: &[
            tool(
                RibbonAction::Erc,
                "ERC",
                Icon::Erc,
                "Open electrical check status",
            ),
            tool(
                RibbonAction::Rules,
                "Rules",
                Icon::Rules,
                "Electrical rules",
            ),
        ],
        extras: &[],
    },
    RibbonGroup {
        label: "Reports",
        tools: &[tool(
            RibbonAction::Bom,
            "BOM",
            Icon::Bom,
            "Bill of materials",
        )],
        extras: &[],
    },
];
const LIBRARY_GROUPS: &[RibbonGroup] = &[RibbonGroup {
    label: "Components",
    tools: &[
        tool(
            RibbonAction::Symbol,
            "Parts",
            Icon::Symbol,
            "Browse original built-in parts · A",
        ),
        tool(
            RibbonAction::Sources,
            "Sources",
            Icon::Folder,
            "Manage library sources",
        ),
        tool(
            RibbonAction::Upgrade,
            "Updates",
            Icon::Sync,
            "Review library updates",
        ),
    ],
    extras: &[],
}];
const MANAGE_GROUPS: &[RibbonGroup] = &[
    RibbonGroup {
        label: "Project",
        tools: &[
            tool(
                RibbonAction::Save,
                "Save",
                Icon::Save,
                "Save project · Ctrl+S",
            ),
            tool(
                RibbonAction::Variants,
                "Variants",
                Icon::Bom,
                "Assembly variants",
            ),
        ],
        extras: &[],
    },
    RibbonGroup {
        label: "History",
        tools: &[
            tool(RibbonAction::Undo, "Undo", Icon::Undo, "Undo · Ctrl+Z"),
            tool(
                RibbonAction::Redo,
                "Redo",
                Icon::Redo,
                "Redo · Ctrl+Shift+Z",
            ),
        ],
        extras: &[],
    },
];
const VIEW_GROUPS: &[RibbonGroup] = &[
    RibbonGroup {
        label: "Panels",
        tools: &[
            tool(
                RibbonAction::Browser,
                "Browser",
                Icon::Folder,
                "Show or hide project browser",
            ),
            tool(
                RibbonAction::Properties,
                "Properties",
                Icon::Rules,
                "Show or hide properties",
            ),
            tool(
                RibbonAction::Filters,
                "Filters",
                Icon::Select,
                "Show or hide selection filters",
            ),
        ],
        extras: &[],
    },
    RibbonGroup {
        label: "Display",
        tools: &[
            tool(
                RibbonAction::Grid,
                "Grid",
                Icon::Layout,
                "Show or hide grid",
            ),
            tool(
                RibbonAction::Snap,
                "Snap",
                Icon::Move,
                "Snap to 1.27 mm grid",
            ),
            tool(
                RibbonAction::Pins,
                "Pins",
                Icon::Port,
                "Show or hide pin numbers",
            ),
            tool(
                RibbonAction::Wires,
                "Wires",
                Icon::Wire,
                "Show or hide wires",
            ),
        ],
        extras: &[],
    },
    RibbonGroup {
        label: "Navigate",
        tools: &[tool(
            RibbonAction::Fit,
            "Fit sheet",
            Icon::Zoom,
            "Fit schematic · F",
        )],
        extras: &[],
    },
];
fn group_separator(ui: &mut egui::Ui, compact: bool) {
    let t = theme::current(ui.ctx());
    let m = t.metrics.ribbon();
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(if compact { 8.0 } else { 12.0 }, m.group_height()),
        egui::Sense::hover(),
    );
    ui.painter().line_segment(
        [
            egui::pos2(rect.center().x, rect.top() + 2.0),
            egui::pos2(rect.center().x, rect.bottom()),
        ],
        egui::Stroke::new(1.0, t.ui.border),
    );
}
fn tool_group(
    ui: &mut egui::Ui,
    app: &App,
    group: &RibbonGroup,
    compact: bool,
) -> Option<RibbonAction> {
    let t = theme::current(ui.ctx());
    let m = t.metrics.ribbon();
    let padding = if compact { 8.0 } else { 12.0 };
    let count = group.tools.len();
    let width = count as f32 * m.tool_width + count.saturating_sub(1) as f32 * 2.0 + padding;
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(width, m.group_height()), egui::Sense::hover());
    let tools = egui::Rect::from_min_size(
        rect.min + egui::vec2(padding * 0.5, 0.0),
        egui::vec2(width - padding, m.tool_height),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(group.label)
            .max_rect(tools)
            .layout(egui::Layout::left_to_right(egui::Align::Min)),
    );
    child.spacing_mut().item_spacing.x = 2.0;
    let mut picked = None;
    for tool in group.tools {
        let reason = tool.action.disabled_reason(app);
        if tool_button(
            &mut child,
            tool.icon,
            tool.label,
            reason.unwrap_or(tool.hint),
            reason.is_none(),
            tool.action.selected(app),
        )
        .clicked()
        {
            picked = Some(tool.action);
        }
    }
    let caption = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.top() + m.tool_height + m.caption_gap),
        egui::vec2(width, m.caption_height),
    );
    let response = ui.interact(
        caption,
        ui.id().with(("ribbon_group", group.label)),
        egui::Sense::click(),
    );
    if response.hovered() || response.has_focus() {
        ui.painter().rect_filled(caption, 3, t.ui.hover);
    }
    let galley = ui.painter().layout_no_wrap(
        group.label.into(),
        egui::FontId::proportional(t.metrics.small),
        t.ui.muted,
    );
    let center = caption.center() - egui::vec2(5.0, 0.0);
    let arrow = egui::pos2(center.x + galley.size().x * 0.5 + 7.0, center.y);
    ui.painter()
        .galley(center - galley.size() * 0.5, galley, t.ui.muted);
    ui.painter().add(egui::Shape::line(
        vec![
            arrow + egui::vec2(-2.5, -1.0),
            arrow + egui::vec2(0.0, 1.5),
            arrow + egui::vec2(2.5, -1.0),
        ],
        egui::Stroke::new(1.0, t.ui.muted),
    ));
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, group.label));
    egui::Popup::menu(&response)
        .frame(controls::menu_frame(ui.ctx()))
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            controls::menu_content(ui);
            for (index, tool) in group.tools.iter().chain(group.extras).enumerate() {
                if index == group.tools.len() {
                    controls::menu_separator(ui);
                }
                let reason = tool.action.disabled_reason(app);
                let mut item = controls::MenuItem::new(tool.label, tool.icon)
                    .shortcut(tool.action.shortcut())
                    .enabled(reason.is_none());
                if matches!(
                    tool.action,
                    RibbonAction::Browser
                        | RibbonAction::Properties
                        | RibbonAction::Filters
                        | RibbonAction::Grid
                        | RibbonAction::Snap
                        | RibbonAction::Pins
                        | RibbonAction::Wires
                ) {
                    item = item.checked(tool.action.selected(app));
                }
                if matches!(tool.action, RibbonAction::Delete) {
                    item = item.danger();
                }
                let response = item
                    .show(ui)
                    .on_hover_text(tool.hint)
                    .on_disabled_hover_text(reason.unwrap_or(tool.hint));
                if response.clicked() {
                    picked = Some(tool.action);
                    ui.close();
                }
            }
        });
    picked
}
fn tool_button(
    ui: &mut egui::Ui,
    icon: Icon,
    label: &str,
    hint: &str,
    enabled: bool,
    selected: bool,
) -> egui::Response {
    let t = theme::current(ui.ctx());
    let m = t.metrics.ribbon();
    let response = ui.add_enabled(
        enabled,
        egui::Button::new("")
            .selected(selected)
            .min_size(egui::vec2(m.tool_width, m.tool_height))
            .frame_when_inactive(false),
    );
    let rect = response.rect;
    let image = icon.image(24.0, ui.ctx());
    let image = if enabled {
        image
    } else {
        image.tint(egui::Color32::from_white_alpha(85))
    };
    image.paint_at(
        ui,
        egui::Rect::from_center_size(
            egui::pos2(rect.center().x, rect.top() + 17.0),
            egui::vec2(24.0, 24.0),
        ),
    );
    ui.painter().text(
        egui::pos2(rect.center().x, rect.bottom() - t.metrics.body * 0.5 - 4.0),
        Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(t.metrics.body),
        if enabled { t.ui.text } else { t.ui.disabled },
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    response.on_hover_text(hint).on_disabled_hover_text(hint)
}

//! FigCraft compact inspector and detailed properties, composed with shared controls.
//! Presentation state is separate from the single Electronics session.
use super::{
    App, canvas, controls,
    icons::{self, Icon},
    library::Dock,
    theme,
};
use eframe::egui::{self, Align2, Rect, RichText, Sense, pos2, vec2};
use pcbcraft_foundation::{document::*, session::Command};

pub(super) struct State {
    pub dock: Dock,
    position: Option<egui::Pos2>,
    drag: Option<(egui::Pos2, egui::Pos2, Dock)>,
    pub draft: Option<Draft>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            dock: Dock::Right,
            position: None,
            drag: None,
            draft: None,
        }
    }
}
pub(super) struct Draft {
    symbol: SymbolId,
    incarnation: ProjectId,
    revision: u64,
    reference: String,
    value: String,
    turns: u8,
    tab: &'static str,
    error: Option<String>,
}
impl State {
    pub fn moving(&self) -> bool {
        self.drag.is_some()
    }
    pub fn cancel_drag(&mut self) {
        if let Some((_, position, dock)) = self.drag.take() {
            self.position = Some(position);
            self.dock = dock;
        }
    }
    pub fn bounds(&self, right: Rect, bounds: Rect, browser: Rect, free_bottom: f32) -> Rect {
        if self.dock == Dock::Right {
            return right;
        }
        let top = if self.dock == Dock::Left && browser.is_finite() {
            browser.bottom() + 12.0
        } else {
            bounds.top()
        };
        let size = vec2(
            right.width(),
            right.height().min((free_bottom - top).max(80.0)),
        );
        let at = if self.dock == Dock::Left {
            pos2(bounds.left(), top)
        } else {
            self.position.unwrap_or(bounds.min + vec2(300.0, 40.0))
        };
        let at = pos2(
            at.x.clamp(bounds.left(), (bounds.right() - size.x).max(bounds.left())),
            at.y.clamp(top, (free_bottom - size.y).max(top)),
        );
        Rect::from_min_size(at, size)
    }
}

impl App {
    fn position_properties(&mut self, dock: Dock) {
        self.inspector.dock = dock;
        // A rail has one expanded utility panel. Preserve the other as a float.
        if dock != Dock::Floating && self.library && self.library_ui.dock == dock {
            self.library_ui
                .undock(self.layout.floating.min + vec2(280.0, 40.0));
        }
    }
    pub(super) fn property_panel(&mut self, ctx: &egui::Context) {
        let t = theme::current(ctx);
        let rect = self.layout.properties;
        let collapsed = rect.height() < 60.0;
        controls::card(ctx, "properties", rect, egui::Order::Middle, |ui| {
            ui.horizontal(|ui| {
                let (r, response) = ui.allocate_exact_size(
                    vec2((ui.available_width() - 64.0).max(30.0), 24.0),
                    Sense::drag(),
                );
                controls::grip(ui, r);
                ui.painter().text(
                    r.left_center() + vec2(18.0, 0.0),
                    Align2::LEFT_CENTER,
                    "Properties",
                    theme::semibold(t.metrics.body),
                    t.ui.text,
                );
                if response.drag_started()
                    && let Some(origin) = ctx.input(|i| i.pointer.press_origin())
                {
                    self.inspector.drag = Some((origin, rect.min, self.inspector.dock));
                }
                if response.dragged()
                    && let Some((origin, position, _)) = self.inspector.drag
                    && let Some(pointer) = response.interact_pointer_pos()
                {
                    self.inspector.position = Some(position + (pointer - origin));
                    self.inspector.dock = Dock::Floating;
                    ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
                }
                if response.drag_stopped()
                    && self.inspector.drag.take().is_some()
                    && let Some(pointer) = ctx.input(|i| i.pointer.interact_pos())
                {
                    let dock = self.property_snap(pointer);
                    self.position_properties(dock);
                }
                response
                    .on_hover_cursor(egui::CursorIcon::Grab)
                    .on_hover_text("Drag to float; release at an edge to dock.");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icons::button(ui, Icon::Close, "Close properties", true).clicked() {
                        self.properties = false;
                    }
                    controls::menu_button(ui, "…", |ui| {
                        for (dock, label) in [
                            (Dock::Floating, "Float"),
                            (Dock::Left, "Dock left"),
                            (Dock::Right, "Dock right"),
                        ] {
                            if controls::MenuItem::new(label, Icon::Pan)
                                .checked(self.inspector.dock == dock)
                                .show(ui)
                                .clicked()
                            {
                                self.inspector.position = Some(rect.min);
                                self.position_properties(dock);
                                ui.close();
                            }
                        }
                        controls::menu_separator(ui);
                        if controls::MenuItem::new(
                            if collapsed {
                                "Expand properties"
                            } else {
                                "Collapse properties"
                            },
                            Icon::Properties,
                        )
                        .show(ui)
                        .clicked()
                        {
                            self.panel_ui.properties_collapsed = !collapsed;
                            if collapsed && self.library_ui.dock == Dock::Right {
                                self.library_ui
                                    .undock(self.layout.floating.min + vec2(280.0, 40.0));
                            }
                            ui.close();
                        }
                    });
                });
            });
            if !collapsed {
                ui.separator();
                ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                if self.text_ui.draft.is_some() {
                    self.text_properties(ui, (rect.height() - 67.0).max(0.0));
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt("inspector_body")
                        .min_scrolled_height(0.0)
                        .max_height((rect.height() - 67.0).max(0.0))
                        .auto_shrink([false, false])
                        .show(ui, |ui| self.property_contents(ui));
                }
            }
        });
        if self.inspector.moving()
            && let Some(p) = ctx.input(|i| i.pointer.interact_pos())
        {
            let dock = self.property_snap(p);
            if dock != Dock::Floating {
                let x = if dock == Dock::Left {
                    self.layout.bounds.left()
                } else {
                    self.layout.bounds.right() - 260.0
                };
                let target =
                    Rect::from_min_size(pos2(x, self.layout.bounds.top()), vec2(260.0, 50.0));
                let painter = ctx.layer_painter(egui::LayerId::new(
                    egui::Order::Tooltip,
                    "property_snap".into(),
                ));
                painter.rect_filled(target, 8, t.ui.selection);
                painter.text(
                    target.center(),
                    Align2::CENTER_CENTER,
                    "Release to dock properties",
                    theme::semibold(t.metrics.body),
                    t.ui.text,
                );
            }
        }
    }
    fn property_snap(&self, p: egui::Pos2) -> Dock {
        let b = self.layout.bounds;
        if p.y < b.top() || p.y > b.bottom() {
            Dock::Floating
        } else if p.x < b.left() + 44.0 {
            Dock::Left
        } else if p.x > b.right() - 44.0 {
            Dock::Right
        } else {
            Dock::Floating
        }
    }
    fn property_contents(&mut self, ui: &mut egui::Ui) {
        if self.selection.len() > 1 {
            let t = theme::current(ui.ctx());
            controls::section(ui, "SELECTION");
            ui.label(
                RichText::new(format!("{} items selected", self.selection.len()))
                    .font(theme::semibold(t.metrics.heading)),
            );
            ui.label(format!(
                "{} symbols · {} wires · {} text notes",
                self.selection.symbols.len(),
                self.selection.wires.len(),
                self.selection.notes.len()
            ));
            ui.label("Drag a selected item to move the group. Shift adds; Ctrl toggles.");
            if controls::action(
                ui,
                "Delete selection",
                controls::ActionTone::Danger,
                !self.busy(),
            )
            .clicked()
            {
                self.delete_selection();
            }
            if controls::action(ui, "Clear selection", controls::ActionTone::Secondary, true)
                .clicked()
            {
                self.select(None);
            }
            return;
        }
        if self.nets.selected.is_some() {
            self.net_properties(ui);
            return;
        }
        let t = theme::current(ui.ctx());
        let doc = self.session.document().clone();
        ui.add_enabled_ui(!self.busy(), |ui| {
            let Some((symbol, c)) = self
                .selected_symbol()
                .and_then(|id| doc.component(id).map(|c| (id, c)))
            else {
                ui.add_space(12.0);
                ui.label(RichText::new("Nothing selected").strong());
                ui.label(
                    RichText::new("Select a symbol or wire to inspect its properties.")
                        .color(t.ui.muted),
                );
                return;
            };
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(35.0, 42.0), Sense::hover());
                let size = canvas::symbol_size(c.kind, 0);
                canvas::paint_symbol(
                    ui.painter(),
                    r.center(),
                    (31.0 / size.x).min(31.0 / size.y),
                    0,
                    c.kind,
                    t.canvas.symbol,
                );
                ui.vertical(|ui| {
                    ui.label(RichText::new(&c.reference).font(theme::semibold(17.0)));
                    ui.add(
                        egui::Label::new(RichText::new(&c.value).small().color(t.ui.muted))
                            .truncate(),
                    );
                });
            });
            ui.add_space(4.0);
            controls::field_row(ui, "Reference", |ui| {
                ui.add_sized(
                    [ui.available_width(), 28.0],
                    controls::text_input(egui::TextEdit::singleline(&mut self.reference)),
                );
            });
            controls::field_row(ui, "Value", |ui| {
                ui.add_sized(
                    [ui.available_width(), 28.0],
                    controls::text_input(egui::TextEdit::singleline(&mut self.value)),
                );
            });
            controls::field_row(ui, "Footprint", |ui| {
                readonly(ui, "Not assigned");
            });
            ui.horizontal(|ui| {
                if controls::action(ui, "Reference style", controls::ActionTone::Secondary, true)
                    .clicked()
                {
                    self.open_text(pcbcraft_foundation::text::TextTarget::Reference(symbol));
                }
                if controls::action(ui, "Value style", controls::ActionTone::Secondary, true)
                    .clicked()
                {
                    self.open_text(pcbcraft_foundation::text::TextTarget::Value(symbol));
                }
            });
            controls::field_row(ui, "Rotation", |ui| {
                let turns = doc
                    .symbols
                    .iter()
                    .find(|s| s.id == symbol)
                    .map_or(0, |s| s.quarter_turns);
                if ui
                    .add_sized(
                        [ui.available_width(), 28.0],
                        egui::Button::new(format!("{}°", u16::from(turns) * 90)),
                    )
                    .clicked()
                {
                    self.command(Command::Rotate { symbol });
                }
            });
            if (self.reference != c.reference || self.value != c.value)
                && controls::action(
                    ui,
                    "Apply changes",
                    controls::ActionTone::Primary,
                    !self.busy(),
                )
                .clicked()
            {
                self.command(Command::Edit {
                    symbol,
                    reference: self.reference.clone(),
                    value: self.value.clone(),
                });
            }
            ui.add_space(4.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{} pins", c.kind.pins().len()))
                        .small()
                        .color(t.ui.muted),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_enabled(false, egui::Button::new("Datasheet ↗").frame(false))
                        .on_disabled_hover_text("No datasheet is assigned to this generic symbol.");
                });
            });
            if ui
                .add_sized(
                    [ui.available_width(), 28.0],
                    egui::Button::new("More properties  ↗"),
                )
                .clicked()
            {
                self.open_properties();
            }
        });
    }
    pub(super) fn open_properties(&mut self) {
        if let Some(symbol) = self.selected_symbol()
            && let Some(s) = self
                .session
                .document()
                .symbols
                .iter()
                .find(|s| s.id == symbol)
        {
            self.inspector.draft = Some(Draft {
                symbol,
                incarnation: self.session.incarnation,
                revision: self.session.revision(),
                reference: self.reference.clone(),
                value: self.value.clone(),
                turns: s.quarter_turns,
                tab: "General",
                error: None,
            });
        }
    }
    fn apply_property_draft(&mut self, d: &Draft) -> Result<()> {
        if self.busy() || self.session.incarnation != d.incarnation {
            return Err("The project changed. Reopen properties before applying.".into());
        }
        self.session.apply(
            d.revision,
            Command::SetProperties {
                symbol: d.symbol,
                reference: d.reference.clone(),
                value: d.value.clone(),
                quarter_turns: d.turns,
            },
        )?;
        self.refresh_selection();
        Ok(())
    }
    pub(super) fn more_properties(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.inspector.draft.take() else {
            return;
        };
        let Some(c) = self.session.document().component(d.symbol).cloned() else {
            return;
        };
        let t = theme::current(ctx);
        let mut close = false;
        let mut apply = false;
        let result=egui::Modal::new("symbol_properties".into()).frame(t.panel_frame().inner_margin(20).corner_radius(12)).show(ctx,|ui| {
            ui.set_width((ctx.content_rect().width()-80.0).min(760.0));
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{} · Symbol properties",c.reference)).font(theme::semibold(20.0)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui| {if icons::button(ui,Icon::Close,"Close symbol properties",true).clicked(){close=true;}});
            });
            ui.label(RichText::new(&c.value).color(t.ui.muted)); ui.add_space(14.0);
            ui.horizontal(|ui| {
                theme::quiet_buttons(ui);
                for tab in ["General","Fields","Pin functions","Assembly","Library","Files","Simulation"] {
                    let available=matches!(tab,"General"|"Pin functions"|"Library");
                    if ui.add_enabled(available,egui::Button::new(tab).selected(d.tab==tab)).on_disabled_hover_text("Not implemented in this schematic draft.").clicked(){d.tab=tab;}
                }
            }); ui.separator();
            egui::ScrollArea::vertical().id_salt("detailed_properties").max_height((ctx.content_rect().height()-270.0).clamp(160.0,380.0)).min_scrolled_height(0.0).auto_shrink([false,false]).show(ui,|ui| {
                ui.set_min_height(300.0);
                match d.tab {
                    "General"=> {
                        ui.columns(2,|cols| {
                            controls::section(&mut cols[0],"COMPONENT");
                            controls::field_row(&mut cols[0],"Reference",|ui| {ui.add_sized([ui.available_width(),28.0],controls::text_input(egui::TextEdit::singleline(&mut d.reference)));});
                            controls::field_row(&mut cols[0],"Value",|ui| {ui.add_sized([ui.available_width(),28.0],controls::text_input(egui::TextEdit::singleline(&mut d.value)));});
                            controls::field_row(&mut cols[0],"Footprint",|ui|readonly(ui,"Not assigned"));
                            controls::section(&mut cols[1],"PLACEMENT");
                            controls::field_row(&mut cols[1],"Unit",|ui|readonly(ui,"1"));
                            controls::field_row(&mut cols[1],"Body style",|ui|readonly(ui,"Standard"));
                            controls::field_row(&mut cols[1],"Rotation",|ui| {
                                egui::ComboBox::from_id_salt("property_rotation").selected_text(format!("{}°",u16::from(d.turns)*90)).width(ui.available_width()).show_ui(ui,|ui| {
                                    controls::menu_content(ui);
                                    for turns in 0..4 {
                                        let label = format!("{}°",u16::from(turns)*90);
                                        if controls::MenuItem::new(&label, Icon::Rotate).checked(d.turns==turns).show(ui).clicked() {d.turns=turns; ui.close();}
                                    }
                                });
                            });
                            controls::field_row(&mut cols[1],"Mirror",|ui|readonly(ui,"Not mirrored"));
                        });
                        ui.add_space(12.0); controls::section(ui,"DESCRIPTION");
                        ui.label(format!("{} · Original generic symbol",c.kind.name()));
                        ui.label(RichText::new("No manufacturer, package or datasheet is assigned.").color(t.ui.muted));
                    }
                    "Pin functions"=> {
                        controls::section(ui,"TERMINALS");
                        ui.label(RichText::new("Names are descriptive. Electrical pin types / ERC are not implemented.").color(t.ui.muted));
                        egui::Grid::new("terminal_table").num_columns(2).spacing(vec2(48.0,10.0)).show(ui,|ui| {
                            ui.strong("Pin"); ui.strong("Name"); ui.end_row();
                            for pin in c.kind.pins(){ui.label(pin.id); ui.label(if pin.name.is_empty(){"—"}else{pin.name});ui.end_row();}
                        });
                    }
                    _=> {
                        controls::section(ui,"PROJECT DEFINITION");
                        ui.label("Original built-in catalog");
                        ui.label(format!("Definition set version {}",self.session.document().builtin_definitions_version));
                        ui.label(RichText::new("Available offline. External sources and reviewable library upgrades are planned.").color(t.ui.muted));
                    }
                }
            });
            if let Some(error)=&d.error {ui.colored_label(t.ui.warning,error);}
            ui.separator(); ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{} · Main sheet",c.reference)).color(t.ui.muted));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui| {
                    if controls::action(ui, "Done", controls::ActionTone::Primary, true).clicked(){apply=true;}
                    if controls::action(ui, "Cancel", controls::ActionTone::Secondary, true).clicked(){close=true;}
                });
            });
        });
        if apply {
            match self.apply_property_draft(&d) {
                Ok(()) => close = true,
                Err(e) => d.error = Some(e.to_string()),
            }
        }
        if !close && !result.should_close() {
            self.inspector.draft = Some(d);
        }
    }
}
fn readonly(ui: &mut egui::Ui, text: &str) {
    let t = theme::current(ui.ctx());
    let (r, response) = ui.allocate_exact_size(vec2(ui.available_width(), 28.0), Sense::hover());
    ui.painter().rect_filled(r, 5, t.ui.field);
    ui.painter().text(
        r.left_center() + vec2(9.0, 0.0),
        Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(t.metrics.body),
        t.ui.muted,
    );
    response.on_hover_text(text);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detailed_properties_commit_once_and_reject_stale_or_reopened_drafts() -> Result<()> {
        let (mut app, _) = super::super::tests::library_fixture()?;
        let before = app.session.document().clone();
        app.open_properties();
        let mut d = app.inspector.draft.take().ok_or("No draft")?;
        d.reference = "U9".into();
        d.value = "Control module".into();
        d.turns = 3;
        let rev = app.session.revision();
        app.apply_property_draft(&d)?;
        assert_eq!(app.session.revision(), rev + 1);
        app.session.undo()?;
        assert_eq!(app.session.document(), &before);
        assert!(app.apply_property_draft(&d).is_err());
        assert_eq!(app.session.document(), &before);
        app.refresh_selection();
        app.open_properties();
        let d = app.inspector.draft.take().ok_or("No draft")?;
        app.session = pcbcraft_foundation::session::Session::new(before.clone())?;
        assert!(app.apply_property_draft(&d).is_err());
        assert_eq!(app.session.document(), &before);
        Ok(())
    }
}

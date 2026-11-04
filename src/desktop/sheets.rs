//! Sheet navigation is view state; all content changes use the single project session.
use super::{App, Tool, controls, icons::Icon, theme};
use eframe::egui::{self, RichText};
use pcbcraft_foundation::{
    document::*,
    project::{self, Direction},
    selection::Selection,
    session::Command,
};
use std::collections::{BTreeMap, BTreeSet};
#[path = "sheet_placement.rs"]
mod placement;

#[derive(Default)]
pub(super) struct State {
    pub active: Option<SheetOccurrenceId>,
    pub folded: BTreeSet<SheetOccurrenceId>,
    cameras: BTreeMap<SheetOccurrenceId, (f32, egui::Vec2, bool, Selection)>,
    pub dialog: Option<Dialog>,
    placement: Option<placement::Placement>,
    pub selected: Option<Object>,
    drag: Option<Drag>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Object {
    Block(SheetSymbolId),
    Port(PortId),
}
impl State {
    pub fn cancel_drag(&mut self) {
        self.drag = None;
        self.placement = None;
        self.selected = None;
    }
}
struct Drag {
    object: Object,
    origin: Point,
    start: Point,
    at: Point,
    incarnation: ProjectId,
    revision: u64,
}
#[derive(Clone, Copy)]
pub(super) enum Kind {
    Page,
    Create { parent: Option<SheetOccurrenceId> },
    Rename,
    Delete,
    Port(Option<PortId>),
    DeletePort(PortId),
}
#[derive(Clone)]
pub(super) struct Dialog {
    page: pcbcraft_foundation::page::Page,
    kind: Kind,
    occurrence: SheetOccurrenceId,
    name: String,
    existing: Option<SheetId>,
    direction: Direction,
    at: Point,
    incarnation: ProjectId,
    revision: u64,
    error: Option<String>,
    impact: Option<String>,
}
impl App {
    pub(super) fn active_sheet(&self) -> SheetOccurrenceId {
        self.sheets
            .active
            .filter(|id| self.session.project().occurrence(*id).is_some())
            .unwrap_or(self.session.project().root)
    }
    pub(super) fn document(&self) -> &Document {
        self.session
            .project()
            .document(self.active_sheet())
            .unwrap_or(&self.session.project().main)
    }
    pub(super) fn apply(&mut self, revision: u64, command: Command) -> Result<Option<SymbolId>> {
        self.session
            .apply_on(revision, self.active_sheet(), command)
    }
    pub(super) fn ensure_sheet(&mut self) {
        if self
            .sheets
            .active
            .is_some_and(|id| self.session.project().occurrence(id).is_none())
        {
            self.sheets.active = None;
            self.reset_sheet_transients();
            self.canvas.fit = true;
            self.canvas.fit_page = true;
        }
    }
    fn reset_sheet_transients(&mut self) {
        self.canvas.drag = None;
        self.canvas.wire_drag = None;
        self.canvas.wire_start = None;
        self.canvas.gesture = None;
        self.canvas.clear_context();
        self.sheets.drag = None;
        self.sheets.placement = None;
        self.sheets.selected = None;
        self.selection.clear();
        self.text_ui = Default::default();
        self.nets = Default::default();
        self.inspector.draft = None;
        self.library_ui.cancel_drag();
        self.tool = Tool::Select;
        self.refresh_selection();
    }
    pub(super) fn activate_sheet(&mut self, id: SheetOccurrenceId) {
        if id == self.active_sheet() || self.session.project().occurrence(id).is_none() {
            return;
        }
        self.sheets.cameras.insert(
            self.active_sheet(),
            (
                self.canvas.zoom,
                self.canvas.offset,
                self.canvas.fit,
                self.selection.clone(),
            ),
        );
        self.sheets.active = Some(id);
        self.reset_sheet_transients();
        self.panel_ui.search.clear();
        if let Some((zoom, offset, fit, selection)) = self.sheets.cameras.get(&id).cloned() {
            self.canvas.zoom = zoom;
            self.canvas.offset = offset;
            self.canvas.fit = fit;
            self.selection = selection;
            self.refresh_selection();
        } else {
            self.canvas.fit = true;
            self.canvas.fit_page = true;
            self.canvas.offset = egui::Vec2::ZERO;
        }
    }
    pub(super) fn sheet_name(&self) -> &str {
        self.session
            .project()
            .occurrence(self.active_sheet())
            .map_or("Main", |o| o.name.as_str())
    }
    pub(super) fn start_sheet_dialog(&mut self, kind: Kind, occurrence: SheetOccurrenceId) {
        if self.busy() {
            return;
        }
        let Some(o) = self.session.project().occurrence(occurrence) else {
            return;
        };
        let mut name = o.name.clone();
        let mut direction = Direction::Input;
        let mut at = self.canvas.last_world;
        if let Kind::Create { .. } = kind {
            name = "New sheet".into();
        }
        if let Kind::Port(id) = kind {
            name = "SIGNAL".into();
            if let Some(p) = id.and_then(|id| {
                self.session
                    .project()
                    .document(occurrence)?
                    .ports
                    .iter()
                    .find(|p| p.id == id)
            }) {
                name = p.name.clone();
                direction = p.direction;
                at = p.at;
            }
        }
        let removal = match kind {
            Kind::Delete => Some(project::Command::Delete { occurrence }),
            Kind::DeletePort(id) => Some(project::Command::DeletePort { occurrence, id }),
            _ => None,
        };
        let impact = removal.and_then(|command| self.session.project().preview(command).ok()).map(|next| {
            let count = |p: &project::Project| p.occurrences.iter().filter_map(|o| p.document(o.id)).fold((0,0), |(parts,wires), d| (parts + d.components.len(), wires + d.wires.len()));
            let before = count(self.session.project()); let after = count(&next);
            format!("Removes {} sheet occurrences, {} physical component occurrences and {} wire occurrences.", self.session.project().occurrences.len() - next.occurrences.len(), before.0 - after.0, before.1 - after.1)
        });
        self.sheets.dialog = Some(Dialog {
            page: self
                .session
                .project()
                .document(occurrence)
                .map_or(pcbcraft_foundation::page::Page::default(), |d| d.page),
            kind,
            occurrence,
            name,
            existing: None,
            direction,
            at,
            incarnation: self.session.incarnation,
            revision: self.session.revision(),
            error: None,
            impact,
        });
    }
    pub(super) fn sheet_menu(&mut self, ui: &mut egui::Ui, occurrence: Option<SheetOccurrenceId>) {
        controls::menu_content(ui);
        let id = occurrence.unwrap_or(self.active_sheet());
        if controls::MenuItem::new("New sheet…", Icon::Sheet)
            .enabled(!self.busy())
            .show(ui)
            .clicked()
        {
            self.start_sheet_dialog(Kind::Create { parent: None }, id);
            ui.close();
        }
        if controls::MenuItem::new("New subsheet…", Icon::Sheet)
            .enabled(!self.busy())
            .show(ui)
            .clicked()
        {
            self.start_sheet_dialog(Kind::Create { parent: Some(id) }, id);
            ui.close();
        }
        if let Some(id) = occurrence {
            ui.separator();
            if controls::MenuItem::new("Rename sheet…", Icon::Label)
                .show(ui)
                .clicked()
            {
                self.start_sheet_dialog(Kind::Rename, id);
                ui.close();
            }
            if controls::MenuItem::new("Add interface port…", Icon::Port)
                .show(ui)
                .clicked()
            {
                self.start_sheet_dialog(Kind::Port(None), id);
                ui.close();
            }
            if controls::MenuItem::new("Delete sheet…", Icon::Delete)
                .danger()
                .enabled(id != self.session.project().root)
                .show(ui)
                .clicked()
            {
                self.start_sheet_dialog(Kind::Delete, id);
                ui.close();
            }
        }
    }
    pub(super) fn sheet_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.sheets.dialog.take() else {
            return;
        };
        let t = theme::current(ctx);
        let mut apply = false;
        let mut close = false;
        let title = match d.kind {
            Kind::Page => "Sheet page",
            Kind::Create { parent: None } => "New flat sheet",
            Kind::Create { .. } => "New hierarchical subsheet",
            Kind::Rename => "Rename sheet",
            Kind::Delete => "Delete sheet",
            Kind::Port(None) => "Add interface port",
            Kind::Port(Some(_)) => "Edit interface port",
            Kind::DeletePort(_) => "Delete interface port",
        };
        let destructive = matches!(d.kind, Kind::Delete | Kind::DeletePort(_));
        let response = egui::Modal::new("sheet_edit".into()).frame(t.panel_frame().inner_margin(20)).show(ctx, |ui| {
            ui.set_width(420.0); ui.label(RichText::new(title).font(theme::semibold(18.0)));
            ui.label(RichText::new(self.session.project().path(d.occurrence)).color(t.ui.muted)); ui.add_space(12.0);
            if matches!(d.kind,Kind::Page) {
                controls::section(ui,"PAGE SIZE");
                ui.horizontal(|ui| { for size in pcbcraft_foundation::page::Size::ALL { ui.selectable_value(&mut d.page.size,size,size.label()); } });
                ui.add_space(12.0);
                controls::section(ui,"ORIENTATION");
                ui.horizontal(|ui| { ui.selectable_value(&mut d.page.portrait,false,"Landscape"); ui.selectable_value(&mut d.page.portrait,true,"Portrait"); });
                let (w,h)=d.page.dimensions_nm();
                ui.label(format!("{} × {} mm",w as f64 / 1e6,h as f64 / 1e6));
                ui.add_space(12.0);
                ui.label(RichText::new("The dashed page edge guides composition. Changing it never moves or clips circuitry. Shared sheet occurrences use the same page.").small().color(t.ui.muted));
            } else if destructive {
                if let Some(impact) = &d.impact { ui.label(RichText::new(impact).strong()); ui.add_space(8.0); }
                ui.label("This removes the sheet or port and its attached connections. Child sheets are removed from this branch; definitions used elsewhere remain.");
                ui.colored_label(t.ui.warning, "Editing a shared definition affects every occurrence of that definition. Undo restores the entire change.");
            } else {
                controls::section(ui, "NAME");
                ui.add_sized([ui.available_width(), 30.0], controls::text_input(egui::TextEdit::singleline(&mut d.name).char_limit(80)));
                if matches!(d.kind, Kind::Create { .. }) {
                    ui.add_space(12.0); controls::section(ui, "CONTENTS");
                    egui::ComboBox::from_id_salt("sheet_definition").width(380.0).selected_text(d.existing.and_then(|id| self.session.project().definition(id)).map_or("New empty definition", |d| d.name.as_str())).show_ui(ui, |ui| {
                        ui.selectable_value(&mut d.existing, None, "New empty definition");
                        for sheet in self.session.project().definitions() { ui.selectable_value(&mut d.existing, Some(sheet.sheet_id), &sheet.name); }
                    });
                    ui.add_space(8.0);
                    ui.label(RichText::new(if d.existing.is_some() { "Reusing a definition shares its drawing. Each occurrence keeps distinct physical component identities." } else if matches!(d.kind, Kind::Create { parent: Some(_) }) { "Choose placement, then click in the parent to place each subsheet. Right-click or Esc finishes. Double-click a placed block to open its child." } else { "Creates an independent sheet at project level. Local net names do not connect sheets." }).small().color(t.ui.muted));
                }
                if matches!(d.kind, Kind::Port(_)) {
                    ui.add_space(12.0); controls::section(ui, "ELECTRICAL DIRECTION");
                    egui::ComboBox::from_id_salt("port_direction").selected_text(d.direction.label()).show_ui(ui, |ui| { for direction in Direction::ALL { ui.selectable_value(&mut d.direction, direction, direction.label()); } });
                    ui.label(RichText::new("Direction is recorded for future ERC. Parent pins update together; names alone never create a connection.").small().color(t.ui.muted));
                }
            }
            if let Some(e) = &d.error { ui.colored_label(t.ui.warning, e); }
            ui.add_space(12.0); ui.separator();
            ui.horizontal(|ui| {
                if controls::action(ui, "Cancel", controls::ActionTone::Secondary, true).clicked() { close = true; }
                let label = if destructive { "Delete" } else if matches!(d.kind, Kind::Create { parent: Some(_) } | Kind::Port(None)) { "Place on sheet" } else if matches!(d.kind, Kind::Create { .. }) { "Create sheet" } else { "Apply" };
                if controls::action(ui, label, if destructive { controls::ActionTone::Danger } else { controls::ActionTone::Primary }, !self.busy()).clicked() { apply = true; }
            });
        });
        if apply && matches!(d.kind, Kind::Create { parent: Some(_) } | Kind::Port(None)) {
            match self.begin_sheet_placement(&d) {
                Ok(()) => close = true,
                Err(error) => d.error = Some(error.to_string()),
            }
            apply = false;
        }
        if apply {
            let command = match d.kind {
                Kind::Page => project::Command::SetPage {
                    occurrence: d.occurrence,
                    page: d.page,
                },
                Kind::Create { parent } => project::Command::Create {
                    name: d.name.trim().into(),
                    parent,
                    existing: d.existing,
                    at: d.at,
                },
                Kind::Rename => project::Command::Rename {
                    occurrence: d.occurrence,
                    name: d.name.trim().into(),
                },
                Kind::Delete => project::Command::Delete {
                    occurrence: d.occurrence,
                },
                Kind::Port(id) => project::Command::SetPort {
                    occurrence: d.occurrence,
                    id,
                    name: d.name.trim().into(),
                    direction: d.direction,
                    at: d.at,
                },
                Kind::DeletePort(id) => project::Command::DeletePort {
                    occurrence: d.occurrence,
                    id,
                },
            };
            let result = if d.incarnation != self.session.incarnation {
                Err("Project was reopened. Review the sheet operation again.".into())
            } else {
                self.session.project_command(d.revision, command)
            };
            match result {
                Ok(effect) => {
                    close = true;
                    if matches!(d.kind, Kind::Page) {
                        self.canvas.fit = true;
                        self.canvas.fit_page = true;
                    }
                    if let Some(id) = effect.created {
                        self.panel_ui.sheets_folded = false;
                        self.panel_ui.project_folded = false;
                        self.panel_ui.main_folded = false;
                        let mut current = Some(id);
                        while let Some(id) = current {
                            self.sheets.folded.remove(&id);
                            current = self.session.project().occurrence(id).and_then(|o| o.parent);
                        }
                        self.activate_sheet(id);
                    }
                    self.ensure_sheet();
                    self.refresh_selection();
                    self.nets = Default::default();
                }
                Err(e) => d.error = Some(e.to_string()),
            }
        }
        if !close && !response.should_close() {
            self.sheets.dialog = Some(d);
        }
    }
    pub(super) fn sheet_properties(&mut self, ui: &mut egui::Ui) -> bool {
        let Some(object) = self.sheets.selected else {
            return false;
        };
        let t = theme::current(ui.ctx());
        match object {
            Object::Block(id) => {
                let Some(child) = self
                    .session
                    .project()
                    .child(self.active_sheet(), id)
                    .cloned()
                else {
                    self.sheets.selected = None;
                    return false;
                };
                ui.horizontal(|ui| {
                    ui.add(Icon::Sheet.image(28.0, ui.ctx()));
                    ui.label(RichText::new(&child.name).font(theme::semibold(17.0)));
                });
                ui.label(RichText::new("Hierarchical subsheet").color(t.ui.muted));
                ui.separator();
                if let Some(d) = self.session.project().document(child.id) {
                    controls::field_row(ui, "Ports", |ui| {
                        ui.label(d.ports.len().to_string());
                    });
                    let instances = self
                        .session
                        .project()
                        .occurrences
                        .iter()
                        .filter(|o| o.definition == d.sheet_id)
                        .count();
                    controls::field_row(ui, "Occurrences", |ui| {
                        ui.label(instances.to_string());
                    });
                    if instances > 1 {
                        ui.label(
                            RichText::new(
                                "Drawing edits are shared; physical components remain distinct.",
                            )
                            .small()
                            .color(t.ui.muted),
                        );
                    }
                }
                if controls::action(ui, "Open subsheet", controls::ActionTone::Primary, true)
                    .clicked()
                {
                    self.activate_sheet(child.id);
                }
                if controls::action(ui, "Rename…", controls::ActionTone::Secondary, true).clicked()
                {
                    self.start_sheet_dialog(Kind::Rename, child.id);
                }
                if controls::action(ui, "Delete sheet…", controls::ActionTone::Danger, true)
                    .clicked()
                {
                    self.start_sheet_dialog(Kind::Delete, child.id);
                }
            }
            Object::Port(id) => {
                let Some(port) = self.document().ports.iter().find(|p| p.id == id).cloned() else {
                    self.sheets.selected = None;
                    return false;
                };
                ui.horizontal(|ui| {
                    ui.add(Icon::Port.image(28.0, ui.ctx()));
                    ui.label(RichText::new(&port.name).font(theme::semibold(17.0)));
                });
                ui.label(RichText::new("Sheet interface port").color(t.ui.muted));
                ui.separator();
                controls::field_row(ui, "Direction", |ui| {
                    ui.label(port.direction.label());
                });
                ui.label(RichText::new("Wire this anchor to a component. Every parent symbol exposes the same port identity.").small().color(t.ui.muted));
                if controls::action(ui, "Edit port…", controls::ActionTone::Primary, true).clicked()
                {
                    self.start_sheet_dialog(Kind::Port(Some(id)), self.active_sheet());
                }
                if controls::action(ui, "Delete port…", controls::ActionTone::Danger, true)
                    .clicked()
                {
                    self.start_sheet_dialog(Kind::DeletePort(id), self.active_sheet());
                }
            }
        }
        true
    }
    pub(super) fn sheet_objects(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        doc: &Document,
        canvas: egui::Rect,
        enabled: bool,
    ) -> bool {
        let t = theme::current(ui.ctx());
        let painter = ui.painter();
        let pointer = if response.drag_started_by(egui::PointerButton::Primary) {
            ui.input(|i| i.pointer.press_origin()).filter(|_| enabled)
        } else {
            response.hover_pos().filter(|_| enabled)
        };
        let mut hit = None;
        for s in &doc.sheet_symbols {
            let rect = egui::Rect::from_two_pos(
                self.canvas.screen(s.at, canvas),
                self.canvas.screen(s.bottom_right(), canvas),
            );
            let selected = self.sheets.selected == Some(Object::Block(s.id));
            let color = if selected {
                t.canvas.selection
            } else {
                t.canvas.symbol
            };
            painter.rect(
                rect,
                3.0,
                t.ui.surface,
                egui::Stroke::new(1.2, color),
                egui::StrokeKind::Inside,
            );
            let child = self.session.project().child(self.active_sheet(), s.id);
            let mut job = egui::text::LayoutJob::simple_singleline(
                child.map_or("Sheet", |o| o.name.as_str()).into(),
                egui::FontId::proportional((13.0 * self.canvas.zoom).max(9.0)),
                color,
            );
            job.wrap.max_width = (rect.width() - 8.0).max(1.0);
            job.wrap.max_rows = 1;
            let title = painter.layout_job(job);
            let title_pos = rect.left_top() - egui::vec2(0.0, title.size().y + 4.0);
            let title_rect = egui::Rect::from_min_size(title_pos, title.size());
            painter.galley(title_pos, title, color);
            for p in &s.pins {
                if let Some(at) = s.pin(p.id) {
                    let pos = self.canvas.screen(at, canvas);
                    let right = p.direction == Direction::Output;
                    painter.circle_stroke(pos, 2.5, egui::Stroke::new(1.1, color));
                    if self.canvas.zoom >= 0.35 {
                        painter.with_clip_rect(rect.shrink(3.0)).text(
                            pos + egui::vec2(if right { -6.0 } else { 6.0 }, 0.0),
                            if right {
                                egui::Align2::RIGHT_CENTER
                            } else {
                                egui::Align2::LEFT_CENTER
                            },
                            &p.name,
                            egui::FontId::proportional((11.0 * self.canvas.zoom).max(9.0)),
                            color,
                        );
                    }
                }
            }
            if self.canvas.symbols_selectable
                && pointer.is_some_and(|p| rect.union(title_rect).contains(p))
            {
                hit = Some(Object::Block(s.id));
            }
        }
        for port in &doc.ports {
            let pos = self.canvas.screen(port.at, canvas);
            let color = if self.sheets.selected == Some(Object::Port(port.id)) {
                t.canvas.selection
            } else {
                t.canvas.symbol
            };
            let label = painter.text(
                pos + egui::vec2(9.0, 0.0),
                egui::Align2::LEFT_CENTER,
                &port.name,
                egui::FontId::proportional((12.0 * self.canvas.zoom).max(8.0)),
                color,
            );
            painter.circle_stroke(pos, 3.0, egui::Stroke::new(1.3, color));
            painter.line_segment(
                [pos, pos + egui::vec2(7.0, 0.0)],
                egui::Stroke::new(1.2, color),
            );
            if self.canvas.pins_selectable
                && pointer.is_some_and(|p| {
                    label
                        .union(egui::Rect::from_center_size(pos, egui::Vec2::splat(12.0)))
                        .contains(p)
                })
            {
                hit = Some(Object::Port(port.id));
            }
        }
        let menu_open = egui::Popup::is_id_open(ui.ctx(), response.id.with("sheet_object"));
        if self.busy() || self.pending.is_some() {
            self.sheets.drag = None;
            return false;
        }
        if !enabled && self.sheets.drag.is_some() && ui.input(|i| i.pointer.any_released()) {
            self.sheets.drag = None;
        }
        if (!enabled && !menu_open && self.sheets.drag.is_none()) || self.tool != Tool::Select {
            return false;
        }
        if let Some(object) = hit {
            if response.clicked() || response.drag_started() || response.secondary_clicked() {
                self.select(None);
                self.sheets.selected = Some(object);
            }
            if response.double_clicked() {
                match object {
                    Object::Block(id) => {
                        if let Some(child) = self.session.project().child(self.active_sheet(), id) {
                            let id = child.id;
                            self.activate_sheet(id);
                        }
                    }
                    Object::Port(id) => {
                        self.start_sheet_dialog(Kind::Port(Some(id)), self.active_sheet())
                    }
                }
                return true;
            }
            if response.drag_started_by(egui::PointerButton::Primary)
                && let Some(pos) = response.interact_pointer_pos()
            {
                let at = match object {
                    Object::Block(id) => {
                        doc.sheet_symbols.iter().find(|s| s.id == id).map(|s| s.at)
                    }
                    Object::Port(id) => doc.ports.iter().find(|p| p.id == id).map(|p| p.at),
                };
                if let Some(at) = at {
                    self.sheets.drag = Some(Drag {
                        object,
                        origin: at,
                        start: self.canvas.world(pos - response.drag_delta(), canvas),
                        at,
                        incarnation: self.session.incarnation,
                        revision: self.session.revision(),
                    });
                }
            }
        }
        if let Some(drag) = &mut self.sheets.drag {
            if let Some(pos) = response.interact_pointer_pos() {
                let world = self.canvas.world(pos, canvas);
                drag.at = Point {
                    x: drag.origin.x + world.x - drag.start.x,
                    y: drag.origin.y + world.y - drag.start.y,
                }
                .snapped();
            }
            if response.drag_stopped_by(egui::PointerButton::Primary)
                && let Some(d) = self.sheets.drag.take()
            {
                let command =
                    match d.object {
                        Object::Block(id) => Some(project::Command::MoveSheet {
                            occurrence: self.active_sheet(),
                            symbol: id,
                            at: d.at,
                        }),
                        Object::Port(id) => doc.ports.iter().find(|p| p.id == id).map(|p| {
                            project::Command::SetPort {
                                occurrence: self.active_sheet(),
                                id: Some(id),
                                name: p.name.clone(),
                                direction: p.direction,
                                at: d.at,
                            }
                        }),
                    };
                if d.incarnation == self.session.incarnation
                    && let Some(command) = command
                    && let Err(e) = self.session.project_command(d.revision, command)
                {
                    self.notice(e);
                }
            }
            return true;
        }
        let selected = self.sheets.selected;
        if hit.is_some()
            || selected.is_some()
                && egui::Popup::is_id_open(ui.ctx(), response.id.with("sheet_object"))
        {
            egui::Popup::context_menu(response)
                .id(response.id.with("sheet_object"))
                .frame(controls::menu_frame(ui.ctx()))
                .show(|ui| {
                    controls::menu_content(ui);
                    if let Some(object) = selected {
                        match object {
                            Object::Block(id) => {
                                if let Some(child) =
                                    self.session.project().child(self.active_sheet(), id)
                                {
                                    let id = child.id;
                                    if controls::MenuItem::new("Open subsheet", Icon::Sheet)
                                        .show(ui)
                                        .clicked()
                                    {
                                        self.activate_sheet(id);
                                        ui.close();
                                    }
                                    self.sheet_menu(ui, Some(id));
                                }
                            }
                            Object::Port(id) => {
                                if controls::MenuItem::new("Edit port…", Icon::Port)
                                    .show(ui)
                                    .clicked()
                                {
                                    self.start_sheet_dialog(
                                        Kind::Port(Some(id)),
                                        self.active_sheet(),
                                    );
                                    ui.close();
                                }
                                if controls::MenuItem::new("Delete port…", Icon::Delete)
                                    .danger()
                                    .show(ui)
                                    .clicked()
                                {
                                    self.start_sheet_dialog(
                                        Kind::DeletePort(id),
                                        self.active_sheet(),
                                    );
                                    ui.close();
                                }
                            }
                        }
                    }
                });
            return true;
        }
        false
    }
    pub(super) fn preview_sheet_objects(&self, doc: &mut Document) {
        if let Some(d) = &self.sheets.drag
            && d.incarnation == self.session.incarnation
            && d.revision == self.session.revision()
        {
            let command = match d.object {
                Object::Block(id) => Some(project::Command::MoveSheet {
                    occurrence: self.active_sheet(),
                    symbol: id,
                    at: d.at,
                }),
                Object::Port(id) => {
                    doc.ports
                        .iter()
                        .find(|p| p.id == id)
                        .map(|p| project::Command::SetPort {
                            occurrence: self.active_sheet(),
                            id: Some(id),
                            name: p.name.clone(),
                            direction: p.direction,
                            at: d.at,
                        })
                }
            };
            if let Some(command) = command
                && let Ok(next) = self.session.project().preview(command)
                && let Some(sheet) = next.document(self.active_sheet())
            {
                *doc = sheet.clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::tests::{library_fixture, ribbon_text, workspace_click, workspace_frame};
    fn settle(app: &mut App, ctx: &egui::Context) {
        for _ in 0..3 {
            workspace_frame(app, ctx, vec![]).drop_without_applying_deltas();
        }
    }
    fn click_text(app: &mut App, ctx: &egui::Context, text: &str) -> Result<()> {
        settle(app, ctx);
        let frame = workspace_frame(app, ctx, vec![]);
        let p = ribbon_text(&frame, text)?.0.center();
        frame.drop_without_applying_deltas();
        workspace_click(app, ctx, p);
        Ok(())
    }
    fn create(
        app: &mut App,
        ctx: &egui::Context,
        name: &str,
        parent: Option<SheetOccurrenceId>,
        existing: Option<SheetId>,
    ) -> Result<SheetOccurrenceId> {
        app.start_sheet_dialog(Kind::Create { parent }, app.active_sheet());
        let d = app.sheets.dialog.as_mut().ok_or("Missing dialog")?;
        d.name = name.into();
        d.existing = existing;
        if parent.is_some() {
            click_text(app, ctx, "Place on sheet")?;
            app.commit_sheet_placement(Point::default());
            let child = app
                .session
                .project()
                .occurrences
                .iter()
                .find(|o| o.parent == parent && o.name == name)
                .ok_or("New child")?
                .id;
            app.sheets.cancel_drag();
            app.activate_sheet(child);
        } else {
            click_text(app, ctx, "Create sheet")?;
        }
        assert!(app.sheets.dialog.is_none());
        Ok(app.active_sheet())
    }
    #[test]
    fn port_preview_repeats_cancels_and_rejects_changes_during_placement() -> Result<()> {
        let (mut app, ctx) = library_fixture()?;
        app.library = false;
        settle(&mut app, &ctx);
        let before = app.session.project().clone();
        let center = app.layout.bounds.center();
        click_text(&mut app, &ctx, "Port")?;
        click_text(&mut app, &ctx, "Place on sheet")?;
        assert_eq!(app.tool, Tool::SheetPlacement);
        workspace_frame(&mut app, &ctx, vec![egui::Event::PointerMoved(center)])
            .drop_without_applying_deltas();
        assert_eq!(app.session.project(), &before);
        workspace_click(&mut app, &ctx, center);
        workspace_click(&mut app, &ctx, center + egui::vec2(35.0, 35.0));
        assert_eq!(app.document().ports.len(), 2);
        assert_eq!(app.document().ports[0].name, "SIGNAL");
        assert_eq!(app.document().ports[1].name, "SIGNAL_2");
        assert_ne!(app.document().ports[0].at, app.document().ports[1].at);
        let placed = app.session.project().clone();
        for pressed in [true, false] {
            workspace_frame(
                &mut app,
                &ctx,
                vec![egui::Event::PointerButton {
                    pos: center,
                    button: egui::PointerButton::Secondary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                }],
            )
            .drop_without_applying_deltas();
        }
        assert_eq!(app.tool, Tool::Select);
        assert!(app.sheets.placement.is_none());
        assert_eq!(app.session.project(), &placed);
        assert!(!egui::Popup::is_any_open(&ctx));
        app.history(false);
        assert_eq!(app.document().ports.len(), 1);
        // A concurrent edit invalidates the pending placement, without extra edits.
        click_text(&mut app, &ctx, "Port")?;
        app.sheets.dialog.as_mut().ok_or("dialog")?.name = "OTHER".into();
        click_text(&mut app, &ctx, "Place on sheet")?;
        app.session.apply(
            app.session.revision(),
            Command::AddNote {
                at: Point::default(),
                text: "Concurrent edit".into(),
            },
        )?;
        let changed = app.session.project().clone();
        settle(&mut app, &ctx);
        assert_eq!(app.tool, Tool::Select);
        assert_eq!(app.session.project(), &changed);
        Ok(())
    }
    #[test]
    fn capture_sheet_tools_and_view_page_setup_use_real_project_commands() -> Result<()> {
        let (mut app, ctx) = library_fixture()?;
        app.library = false;
        let root = app.active_sheet();
        click_text(&mut app, &ctx, "New sheet")?;
        let d = app
            .sheets
            .dialog
            .as_mut()
            .ok_or("Missing flat-sheet dialog")?;
        assert!(matches!(d.kind, Kind::Create { parent: None }));
        d.name = "Power".into();
        click_text(&mut app, &ctx, "Create sheet")?;
        let flat = app.active_sheet();
        assert_ne!(flat, root);
        click_text(&mut app, &ctx, "Subsheet")?;
        let d = app
            .sheets
            .dialog
            .as_mut()
            .ok_or("Missing subsheet dialog")?;
        assert!(matches!(d.kind, Kind::Create { parent: Some(id) } if id == flat));
        d.name = "Sensor".into();
        let before_placement = app.session.project().clone();
        click_text(&mut app, &ctx, "Place on sheet")?;
        assert_eq!(app.session.project(), &before_placement);
        let center = app.layout.bounds.center();
        workspace_click(&mut app, &ctx, center);
        assert_eq!(app.active_sheet(), flat);
        let child = app
            .session
            .project()
            .occurrences
            .iter()
            .find(|o| o.name == "Sensor")
            .ok_or("placed child")?
            .id;
        app.activate_sheet(child);
        assert_eq!(
            app.session
                .project()
                .occurrence(child)
                .ok_or("child")?
                .parent,
            Some(flat)
        );
        click_text(&mut app, &ctx, "View")?;
        click_text(&mut app, &ctx, "Page setup")?;
        assert!(matches!(
            app.sheets.dialog.as_ref().ok_or("page dialog")?.kind,
            Kind::Page
        ));
        let before = app.session.project().clone();
        click_text(&mut app, &ctx, "A4")?;
        click_text(&mut app, &ctx, "Portrait")?;
        click_text(&mut app, &ctx, "Apply")?;
        assert_eq!(
            app.document().page,
            pcbcraft_foundation::page::Page {
                size: pcbcraft_foundation::page::Size::A4,
                portrait: true,
            }
        );
        assert!(app.sheets.dialog.is_none());
        assert!(app.canvas.fit_page);
        app.history(false);
        assert_eq!(app.session.project(), &before);
        click_text(&mut app, &ctx, "Fit design")?;
        assert!(!app.canvas.fit_page);
        click_text(&mut app, &ctx, "Fit page")?;
        assert!(app.canvas.fit_page);
        Ok(())
    }
    #[test]
    fn sheet_dialog_browser_scoped_edit_camera_history_and_save_reopen() -> Result<()> {
        let (mut app, ctx) = library_fixture()?;
        app.library = false;
        settle(&mut app, &ctx);
        let root = app.active_sheet();
        let old = app.document().clone();
        let camera = (app.canvas.zoom, app.canvas.offset);
        let flat = create(&mut app, &ctx, "Power", None, None)?;
        assert_ne!(flat, root);
        assert!(app.document().symbols.is_empty());
        app.command(Command::Place {
            kind: PartKind::Capacitor,
            at: Point::default(),
        });
        let capacitor = app.selected_symbol().ok_or("No capacitor")?;
        assert_eq!(app.session.document(), &old);
        settle(&mut app, &ctx);
        click_text(&mut app, &ctx, "Main")?;
        assert_eq!(app.active_sheet(), root);
        assert_eq!((app.canvas.zoom, app.canvas.offset), camera);
        let frame = workspace_frame(&mut app, &ctx, vec![]);
        let power = crate::desktop::tests::panel_text(&frame, app.layout.browser, "Power")?
            .0
            .center();
        frame.drop_without_applying_deltas();
        workspace_click(&mut app, &ctx, power);
        assert_eq!(app.active_sheet(), flat);
        assert_eq!(app.selected_symbol(), Some(capacitor));
        let child = create(&mut app, &ctx, "Sensor", Some(flat), None)?;
        assert_eq!(
            app.session
                .project()
                .occurrence(child)
                .ok_or("Missing test item")?
                .parent,
            Some(flat)
        );
        app.command(Command::Place {
            kind: PartKind::Resistor,
            at: Point::default(),
        });
        assert_eq!(app.document().components[0].reference, "R1");
        let full = app.session.project().clone();
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("sheets.pc");
        app.path = Some(path.clone());
        app.save(false);
        for _ in 0..100 {
            app.poll(&ctx);
            if !app.busy() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(!app.busy());
        assert_eq!(
            pcbcraft_foundation::persistence::read_project(&path)?.0,
            full
        );
        assert!(!app.session.dirty());
        app.start_sheet_dialog(Kind::Delete, child);
        click_text(&mut app, &ctx, "Delete")?;
        assert!(app.session.project().occurrence(child).is_none());
        assert_eq!(app.active_sheet(), root);
        app.history(false);
        assert_eq!(app.session.project(), &full);
        app.load(Some(path));
        for _ in 0..100 {
            app.poll(&ctx);
            if !app.busy() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(app.session.project(), &full);
        assert_eq!(app.active_sheet(), root);
        Ok(())
    }
    #[test]
    fn switching_sheets_cancels_drafts_and_stale_sheet_dialog_cannot_apply() -> Result<()> {
        let (mut app, ctx) = library_fixture()?;
        app.library = false;
        let root = app.active_sheet();
        let flat = create(&mut app, &ctx, "Other", None, None)?;
        app.canvas.wire_start = Some(super::super::canvas::WireDraft {
            from: pcbcraft_foundation::wiring::Anchor::Free(Point::default()),
            points: vec![Point::default()],
            incarnation: app.session.incarnation,
            revision: app.session.revision(),
            vertical_first: false,
        });
        app.start_sheet_dialog(Kind::Rename, flat);
        app.command(Command::Place {
            kind: PartKind::Resistor,
            at: Point::default(),
        });
        let before = app.session.project().clone();
        click_text(&mut app, &ctx, "Apply")?;
        assert!(
            app.sheets
                .dialog
                .as_ref()
                .ok_or("Missing test item")?
                .error
                .is_some()
        );
        assert_eq!(app.session.project(), &before);
        app.sheets.dialog = None;
        app.activate_sheet(root);
        assert!(app.canvas.wire_start.is_none());
        assert!(app.nets.selected.is_none());
        assert!(app.text_ui.draft.is_none());
        app.history(false);
        app.history(false); // Place, then creation.
        assert!(app.session.project().occurrence(flat).is_none());
        assert_eq!(app.active_sheet(), root);
        Ok(())
    }
    #[test]
    fn port_ribbon_and_sheet_block_pointer_navigation_and_drag_are_real() -> Result<()> {
        let (mut app, ctx) = library_fixture()?;
        app.library = false;
        app.session = pcbcraft_foundation::session::Session::new(Document::empty("Sheets")?)?;
        let root = app.active_sheet();
        let child = create(&mut app, &ctx, "Amplifier", Some(root), None)?;
        click_text(&mut app, &ctx, "Port")?;
        app.sheets
            .dialog
            .as_mut()
            .ok_or("Port action did not open")?
            .name = "IN".into();
        click_text(&mut app, &ctx, "Place on sheet")?;
        assert!(app.document().ports.is_empty());
        let center = app.layout.bounds.center();
        workspace_click(&mut app, &ctx, center);
        assert_eq!(app.document().ports.len(), 1);
        let port_id = app.document().ports[0].id;
        app.activate_sheet(root);
        settle(&mut app, &ctx);
        let s = app.document().sheet_symbols[0].clone();
        assert_eq!(s.pins[0].id, port_id);
        // Titles remain outside pin rows even when a compact viewport forces a low zoom.
        for zoom in [0.2, 0.45, 0.75] {
            app.canvas.zoom = zoom;
            app.canvas.offset = egui::Vec2::ZERO;
            app.canvas.fit = false;
            let frame = workspace_frame(&mut app, &ctx, vec![]);
            let area = egui::Rect::from_min_max(
                egui::pos2(
                    app.layout.browser.right() + 12.0,
                    app.layout.bounds.top() + 40.0,
                ),
                egui::pos2(
                    app.layout.properties.left() - 12.0,
                    app.layout.bounds.bottom() - 70.0,
                ),
            );
            if zoom >= 0.35 {
                let title = crate::desktop::tests::panel_text(&frame, area, "Amplifier")?.0;
                let pin = crate::desktop::tests::panel_text(&frame, area, "IN")?.0;
                assert!(!title.intersects(pin));
            } else {
                assert!(crate::desktop::tests::panel_text(&frame, area, "IN").is_err());
            }
            frame.drop_without_applying_deltas();
        }
        app.canvas.fit = true;
        settle(&mut app, &ctx);
        let full = app.layout.bounds.expand(12.0);
        let canvas = egui::Rect::from_min_max(
            full.min + egui::vec2(284.0, 52.0),
            full.max - egui::vec2(288.0, 85.0),
        );
        let start = app.canvas.screen(
            Point {
                x: s.at.x + 6 * GRID_NM,
                y: s.at.y + GRID_NM,
            },
            canvas,
        );
        let before = app.session.project().clone();
        let event = |pos, pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        workspace_frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerMoved(start), event(start, true)],
        )
        .drop_without_applying_deltas();
        let end = start + egui::vec2(35.0, 20.0);
        workspace_frame(&mut app, &ctx, vec![egui::Event::PointerMoved(end)])
            .drop_without_applying_deltas();
        assert!(
            app.sheets.drag.is_some(),
            "Expected sheet drag, selected={:?}, pointer={start:?}, canvas={canvas:?}",
            app.sheets.selected
        );
        assert_eq!(app.session.project(), &before, "Dragging is a preview");
        workspace_frame(&mut app, &ctx, vec![event(end, false)]).drop_without_applying_deltas();
        assert_ne!(
            app.document().sheet_symbols[0].at,
            s.at,
            "start={start:?} end={end:?} canvas={canvas:?} selected={:?} drag={} notices={:?}",
            app.sheets.selected,
            app.sheets.drag.is_some(),
            app.notices
        );
        app.history(false);
        assert_eq!(app.session.project(), &before);
        app.sheets.selected = Some(Object::Block(s.id));
        app.delete_selection();
        assert!(matches!(
            app.sheets.dialog.as_ref().map(|d| &d.kind),
            Some(Kind::Delete)
        ));
        assert_eq!(app.session.project(), &before);
        app.sheets.dialog = None;
        click_text(&mut app, &ctx, "Open subsheet")?;
        assert_eq!(app.active_sheet(), child);
        app.command(Command::Place {
            kind: PartKind::Resistor,
            at: Point {
                x: 20 * GRID_NM,
                y: 0,
            },
        });
        let r = app.selected_symbol().ok_or("Missing resistor")?;
        app.canvas.fit = true;
        settle(&mut app, &ctx);
        let full = app.layout.bounds.expand(12.0);
        let canvas = egui::Rect::from_min_max(
            full.min + egui::vec2(284.0, 52.0),
            full.max - egui::vec2(288.0, 85.0),
        );
        let port_at = app.document().ports[0].at;
        let terminal = PinRef {
            symbol: r,
            terminal: "1".into(),
        };
        let pin_at = app
            .document()
            .pin(&terminal)
            .ok_or("Missing resistor pin")?;
        let from = app.canvas.screen(port_at, canvas);
        let to = app.canvas.screen(pin_at, canvas);
        app.tool = Tool::Wire;
        workspace_click(&mut app, &ctx, from);
        assert!(app.canvas.wire_start.is_some());
        workspace_click(&mut app, &ctx, to);
        assert!(app.canvas.wire_start.is_none());
        assert_eq!(app.document().wires.len(), 1);
        assert!(matches!(app.document().wires[0].from, Endpoint::Port(_)));
        let probe = app.session.probe_selection(
            child,
            &Selection::default(),
            Some(&Endpoint::Port(project::PortRef { port: port_id })),
        )?;
        assert!(
            probe
                .terminals
                .iter()
                .any(|t| t.component.occurrence == child)
        );
        app.history(false);
        assert!(app.document().wires.is_empty());
        Ok(())
    }
}

//! Connectivity presentation. The Electronics session remains the sole edit authority.
use super::{App, Tool, controls, icons::Icon, theme};
use eframe::egui::{self, RichText};
use pcbcraft_foundation::{
    document::*,
    netlist::{self, Net, Netlist},
    session::Command,
};

#[derive(Default)]
pub(super) struct State {
    stamp: Option<(ProjectId, u64)>,
    pub snapshot: Netlist,
    pub selected: Option<Endpoint>,
    pub wire: Option<WireId>,
    pub name: String,
    error: Option<String>,
    report: Option<String>,
    disconnect: Option<Pending>,
}
struct Pending {
    wire: WireId,
    incarnation: ProjectId,
    revision: u64,
    title: String,
    fragments: usize,
    unnamed: usize,
    name_removed: bool,
    error: Option<String>,
}
impl State {
    pub fn modal(&self) -> bool {
        self.report.is_some() || self.disconnect.is_some()
    }
    pub fn selected_net(&self) -> Option<&Net> {
        self.snapshot.containing(self.selected.as_ref()?)
    }
}
impl App {
    pub(super) fn refresh_nets(&mut self) {
        let stamp = (self.session.incarnation, self.session.revision());
        if self.nets.stamp == Some(stamp) {
            return;
        }
        self.nets.selected = None;
        self.nets.wire = None;
        self.nets.error = None;
        match Netlist::compile(self.document()) {
            Ok(snapshot) => self.nets.snapshot = snapshot,
            Err(e) => {
                self.nets.snapshot = Netlist::default();
                self.nets.error = Some(e.to_string());
            }
        }
        self.nets.stamp = Some(stamp);
    }
    pub(super) fn select_net(&mut self, anchor: Endpoint) {
        self.refresh_nets();
        let Some(net) = self.nets.snapshot.containing(&anchor).cloned() else {
            return;
        };
        self.select(None);
        self.nets.name = net.name.clone().unwrap_or_default();
        self.nets.selected = Some(net.anchor);
        self.nets.error = None;
        self.tool = Tool::Select;
        self.properties = true;
        self.panel_ui.properties_collapsed = false;
    }
    pub(super) fn open_net_report(&mut self) {
        match pcbcraft_foundation::project::report(
            self.session.project(),
            self.session.incarnation,
            self.session.revision(),
        ) {
            Ok(report) => self.nets.report = Some(report),
            Err(e) => self.nets.error = Some(e.to_string()),
        }
    }
    pub(super) fn net_results(&mut self, ui: &mut egui::Ui, query: &str) {
        let t = theme::current(ui.ctx());
        controls::result_row(
            ui,
            ui.id().with("head"),
            &[("Net", 0.55), ("Pins", 0.15), ("Sheet", 0.30)],
            false,
            true,
        );
        let mut nets = self.nets.snapshot.nets.clone();
        nets.sort_by_key(|n| n.title(self.document()));
        let mut count = 0;
        for net in nets {
            let title = net.title(self.document());
            if !title.to_lowercase().contains(query) {
                continue;
            }
            count += 1;
            if controls::result_row(
                ui,
                ui.id().with(format!("{:?}", net.anchor)),
                &[
                    (&title, 0.55),
                    (&net.terminals.len().to_string(), 0.15),
                    (self.sheet_name(), 0.30),
                ],
                self.nets.selected.as_ref() == Some(&net.anchor),
                false,
            )
            .clicked()
            {
                self.select_net(net.anchor);
            }
        }
        if count == 0 {
            ui.label(RichText::new("No matching nets").color(t.ui.muted));
        }
        if let Some(error) = &self.nets.error {
            ui.colored_label(t.ui.warning, error);
        }
    }
    pub(super) fn net_properties(&mut self, ui: &mut egui::Ui) {
        let Some(net) = self.nets.selected_net().cloned() else {
            return;
        };
        let t = theme::current(ui.ctx());
        let doc = self.document().clone();
        ui.horizontal(|ui| {
            ui.add(Icon::Wire.image(24.0, ui.ctx()));
            ui.label(RichText::new(net.title(&doc)).font(theme::semibold(16.0)));
        });
        ui.label(
            RichText::new(format!(
                "{} pins · {} connections",
                net.terminals.len(),
                net.links.len()
            ))
            .small()
            .color(t.ui.muted),
        );
        ui.label(
            RichText::new(if self.nets.wire.is_some() {
                "Wire selected · Delete removes this wire"
            } else {
                "Net selected · Delete removes all its wires"
            })
            .small()
            .color(t.ui.muted),
        );
        ui.add_space(6.0);
        controls::field_row(ui, "Net name", |ui| {
            ui.add_sized(
                [ui.available_width(), 28.0],
                controls::text_input(
                    egui::TextEdit::singleline(&mut self.nets.name).hint_text("Unnamed"),
                ),
            );
        });
        if controls::action(
            ui,
            "Apply name",
            controls::ActionTone::Primary,
            !self.busy() && self.nets.name != net.name.clone().unwrap_or_default(),
        )
        .clicked()
            && let Some((incarnation, revision)) = self.nets.stamp
        {
            let result = self.net_edit(
                incarnation,
                revision,
                Command::RenameNet {
                    anchor: net.anchor.clone(),
                    name: self.nets.name.clone(),
                },
            );
            match result {
                Ok(()) => self.select_net(net.anchor.clone()),
                Err(e) => self.nets.error = Some(e.to_string()),
            }
        }
        if let Some(error) = &self.nets.error {
            ui.colored_label(t.ui.warning, error);
        }
        ui.horizontal(|ui| {
            if controls::action(
                ui,
                "Draw wire",
                controls::ActionTone::Secondary,
                !self.busy(),
            )
            .on_hover_text("Draw wire · W")
            .clicked()
            {
                self.tool = Tool::Wire;
                self.canvas.wire_start = None;
            }
            if controls::action(ui, "Delete net", controls::ActionTone::Danger, !self.busy())
                .on_hover_text(
                    "Remove every wire in this net. Components remain; Ctrl+Z restores the net.",
                )
                .clicked()
            {
                self.command(Command::DeleteNet {
                    anchor: net.anchor.clone(),
                });
            }
        });
        if controls::action(
            ui,
            "Text appearance…",
            controls::ActionTone::Secondary,
            !self.busy(),
        )
        .clicked()
            && let Some(id) = net.links.first()
        {
            self.open_text(pcbcraft_foundation::text::TextTarget::Net(*id));
        }
        controls::section(ui, "CONNECTED PINS");
        for pin in &net.terminals {
            let c = doc.component(pin.symbol);
            let label = format!(
                "{}  {}",
                netlist::terminal_name(&doc, pin),
                c.and_then(|c| c.kind.pins().iter().find(|p| p.id == pin.terminal))
                    .map_or("", |p| p.name)
            );
            if ui
                .add(egui::Button::new(label).frame(false))
                .on_hover_text("Select component")
                .clicked()
            {
                self.select(Some(pin.symbol));
            }
        }
        controls::section(ui, "CONNECTIONS");
        for wire in doc.wires.iter().filter(|w| net.links.contains(&w.id)) {
            let label = format!(
                "{} ↔ {}",
                netlist::endpoint_name(&doc, &wire.from),
                netlist::endpoint_name(&doc, &wire.to)
            );
            ui.horizontal(|ui| {
                ui.add_sized(
                    [ui.available_width() - 32.0, 26.0],
                    egui::Label::new(&label).truncate(),
                )
                .on_hover_text(&label);
                if ui
                    .add_enabled(
                        !self.busy(),
                        egui::Button::image(Icon::Delete.danger_image(14.0, ui.ctx())).frame(false),
                    )
                    .on_hover_text("Review removal of this connection")
                    .clicked()
                {
                    self.review_disconnect(wire.id, label.clone());
                }
            });
        }
        ui.label(
            RichText::new(
                "W draws wires. Click a wire and press Delete to remove it; Undo restores it.",
            )
            .small()
            .color(t.ui.muted),
        );
    }
    pub(super) fn review_disconnect(&mut self, wire: WireId, title: String) {
        match netlist::preview_disconnect(self.document(), wire) {
            Ok(preview) => {
                self.nets.disconnect = Some(Pending {
                    wire,
                    title,
                    incarnation: self.session.incarnation,
                    revision: self.session.revision(),
                    fragments: preview.fragments,
                    unnamed: preview.unnamed_fragments,
                    name_removed: preview.name_removed,
                    error: None,
                })
            }
            Err(e) => self.nets.error = Some(e.to_string()),
        }
    }
    fn net_edit(&mut self, incarnation: ProjectId, revision: u64, command: Command) -> Result<()> {
        if self.busy() || self.pending.is_some() || self.session.incarnation != incarnation {
            return Err("Project changed. Review the edit again.".into());
        }
        self.apply(revision, command)?;
        self.refresh_selection();
        self.refresh_nets();
        Ok(())
    }
    pub(super) fn net_dialogs(&mut self, ctx: &egui::Context) {
        let t = theme::current(ctx);
        if let Some(mut pending) = self.nets.disconnect.take() {
            let mut close = false;
            let mut apply = false;
            let response = egui::Modal::new("disconnect_review".into())
                .frame(t.panel_frame().inner_margin(20))
                .show(ctx, |ui| {
                    ui.set_width(440.0);
                    ui.label(RichText::new("Remove connection").font(theme::semibold(18.0)));
                    ui.label(&pending.title);
                    ui.add_space(10.0);
                    if pending.fragments == 0 {
                        ui.label("No wire remains in this net; unused wire points are removed.");
                    } else if pending.fragments > 1 {
                        ui.label(format!(
                            "This splits the net into {} fragments, including any isolated pins.",
                            pending.fragments
                        ));
                    } else {
                        ui.label("Other links keep these pins connected; the net will not split.");
                    }
                    if pending.unnamed > 0 {
                        ui.colored_label(t.ui.warning, format!(
                            "The largest linked fragment retains the name. {} other fragment(s) become unnamed, with their paths visible.",
                            pending.unnamed
                        ));
                    }
                    if pending.name_removed {
                        ui.label("No connected fragment remains; the net name is removed.");
                    }
                    ui.label("The entire change can be undone in one step.");
                    if let Some(e) = &pending.error {
                        ui.colored_label(t.ui.warning, e);
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        if controls::action(ui, "Cancel", controls::ActionTone::Secondary, true).clicked() {
                            close = true;
                        }
                        if controls::action(ui, "Remove connection", controls::ActionTone::Danger, !self.busy()).clicked() {
                            apply = true;
                        }
                    });
                });
            if apply {
                match self.net_edit(
                    pending.incarnation,
                    pending.revision,
                    Command::Disconnect { wire: pending.wire },
                ) {
                    Ok(()) => close = true,
                    Err(e) => pending.error = Some(e.to_string()),
                }
            }
            if !close && !response.should_close() {
                self.nets.disconnect = Some(pending);
            }
        }
        if let Some(report) = self.nets.report.take() {
            let mut close = false;
            let response = egui::Modal::new("netlist_report".into())
                .frame(t.panel_frame().inner_margin(20))
                .show(ctx, |ui| {
                    ui.set_width((ctx.content_rect().width() - 90.0).min(760.0));
                    ui.label(RichText::new("Netlist representation").font(theme::semibold(18.0)));
                    ui.label(RichText::new("Snapshot of explicit connectivity · not a KiCad or SPICE interchange file").color(t.ui.muted));
                    ui.separator();
                    egui::ScrollArea::both()
                        .max_height((ctx.content_rect().height() - 240.0).max(160.0))
                        .show(ui, |ui| {
                            ui.add(egui::Label::new(RichText::new(&report).monospace()).selectable(true).extend());
                        });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if controls::action(ui, "Copy JSON", controls::ActionTone::Primary, true).clicked() {
                            ctx.copy_text(report.clone());
                        }
                        if controls::action(ui, "Close", controls::ActionTone::Secondary, true).clicked() {
                            close = true;
                        }
                    });
                });
            if !close && !response.should_close() {
                self.nets.report = Some(report);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disconnect_review_rejects_stale_revision_and_reopened_project() -> Result<()> {
        let (mut app, _) = super::super::tests::library_fixture()?;
        let wire = app.document().wires[0].id;
        app.review_disconnect(wire, "Original link".into());
        let p = app.nets.disconnect.take().ok_or("No review")?;
        let symbol = app.document().symbols[0].id;
        app.command(Command::Rotate { symbol });
        let changed = app.document().clone();
        assert!(
            app.net_edit(p.incarnation, p.revision, Command::Disconnect { wire })
                .is_err()
        );
        assert_eq!(app.document(), &changed);
        app.session = pcbcraft_foundation::session::Session::new(changed.clone())?;
        assert!(
            app.net_edit(
                p.incarnation,
                app.session.revision(),
                Command::Disconnect { wire }
            )
            .is_err()
        );
        assert_eq!(app.document(), &changed);
        Ok(())
    }
}

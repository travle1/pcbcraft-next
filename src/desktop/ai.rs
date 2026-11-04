//! Live MCP connection and human proposal review; no pretend model/chat connection.
use super::{App, Tool, controls, theme};
use eframe::egui::{self, RichText};
use pcbcraft_foundation::{authoring::Apply, bridge::Bridge, document::*, mcp};
use serde_json::json;

#[derive(Default)]
pub(super) struct State {
    pub open: bool,
    bridge: Option<Bridge>,
    allow_apply: bool,
    before: bool,
    activity: Vec<String>,
}
impl App {
    pub(crate) fn open_ai(&mut self) {
        self.ai.open = true;
    }
    pub(crate) fn enable_mcp(&mut self, ctx: &egui::Context) {
        if self.ai.bridge.is_some() {
            return;
        }
        let wake = ctx.clone();
        match Bridge::start(move || wake.request_repaint()) {
            Ok(bridge) => {
                eprintln!("PCBcraft MCP connection: {}", bridge.path().display());
                self.notice("MCP enabled for this window. Edits require proposal review.");
                self.ai.bridge = Some(bridge);
            }
            Err(e) => self.notice(e),
        }
    }
    pub(super) fn poll_mcp(&mut self) {
        // One bounded request per frame. GUI gestures/file transitions win over remote work.
        let Some(pending) = self.ai.bridge.as_ref().and_then(Bridge::next) else {
            return;
        };
        if std::time::Instant::now() >= pending.deadline {
            pending.reply(Err(mcp::Failure {
                code: "expired".into(),
                message: "Request expired before execution; no mutation.".into(),
            }));
            return;
        }
        if self.busy()
            || self.pending.is_some()
            || self.canvas.drag.is_some()
            || self.canvas.wire_drag.is_some()
            || self.canvas.gesture.is_some()
            || self.inspector.draft.is_some()
            || self.text_ui.draft.is_some()
            || self.sheets.dialog.is_some()
            || self.nets.modal()
        {
            pending.reply(Err(mcp::Failure {
                code: "busy".into(),
                message:
                    "Finish the active editor gesture or file operation; request did not execute."
                        .into(),
            }));
            return;
        }
        let context = mcp::Context {
            active_sheet: self.active_sheet(),
            selection: json!({"sheet":self.active_sheet(),"symbols":self.selection.symbols,"wires":self.selection.wires,"notes":self.selection.notes}),
            allow_apply: self.ai.allow_apply,
        };
        let revision = self.session.revision();
        let name = pending.call.name.clone();
        let reply = mcp::dispatch(&mut self.session, &context, pending.call.clone());
        self.ai.activity.push(format!(
            "{} · {}",
            name,
            if reply.is_ok() {
                "completed"
            } else {
                "rejected"
            }
        ));
        if self.ai.activity.len() > 12 {
            self.ai.activity.remove(0);
        }
        if name == "prepare_edit" && reply.is_ok() {
            self.ai.open = true;
        }
        if revision != self.session.revision() {
            self.after_ai_edit();
        }
        pending.reply(reply);
    }
    fn after_ai_edit(&mut self) {
        self.reset_sheet_transients();
        self.ensure_sheet();
        self.refresh_selection();
        self.refresh_nets();
        self.tool = Tool::Select;
        self.notice("AI proposal applied · one Undo restores the whole change.");
    }
    pub(super) fn ai_panel(&mut self, ctx: &egui::Context) {
        if !self.ai.open {
            return;
        }
        let t = theme::current(ctx);
        let mut open = true;
        egui::Window::new("AI / MCP")
            .id("pcbcraft-ai".into()).title_bar(false)
            .default_width(480.0).default_height(700.0)
            .max_height((ctx.content_rect().height() - 100.0).max(380.0))
            .default_pos(egui::pos2(780.0,190.0))
            .resizable(true).frame(t.panel_frame().inner_margin(14))
            .show(ctx,|ui| {
                ui.horizontal(|ui| {
                    let (rect, response) = ui.allocate_exact_size(egui::vec2((ui.available_width() - 34.0).max(50.0), 28.0), egui::Sense::hover());
                    controls::grip(ui, rect);
                    ui.painter().text(rect.left_center() + egui::vec2(18.0, 0.0), egui::Align2::LEFT_CENTER, "AI / MCP", theme::semibold(t.metrics.heading), t.ui.text);
                    response.on_hover_cursor(egui::CursorIcon::Grab).on_hover_text("Drag to move this panel");
                    if super::icons::button(ui, super::icons::Icon::Close, "Close AI / MCP", true).clicked() { open = false; }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(RichText::new(if self.ai.bridge.is_some(){"LIVE SCHEMATIC BRIDGE"}else{"EXTERNAL AGENTS"}).small().color(t.ui.accent));
                    ui.label(RichText::new(if self.ai.bridge.is_some(){"Enabled"}else{"Off"}).color(t.ui.muted));
                });
                ui.add_space(6.0);
                ui.label("Connect your MCP client to this window. Agents can inspect and propose edits; you review them here.");
                ui.add_space(10.0);
                if let Some(bridge)=&self.ai.bridge {
                    let path=bridge.path();
                    ui.horizontal(|ui| {
                        if controls::action(ui,"Copy client config",controls::ActionTone::Secondary,true).clicked() {
                            let executable=std::env::current_exe().ok().map(|p|p.with_file_name("pcbcraft-mcp")).unwrap_or_else(||"pcbcraft-mcp".into());
                            ui.ctx().copy_text(serde_json::to_string_pretty(&json!({"mcpServers":{"pcbcraft":{"command":executable,"args":["--connect",path]}}})).unwrap_or_default());
                        }
                        if controls::action(ui,"Disable",controls::ActionTone::Secondary,true).clicked() {
                            self.ai.bridge=None; self.ai.allow_apply=false;
                        }
                    });
                    ui.label(RichText::new(path.display().to_string()).small().color(t.ui.muted));
                    controls::checkbox(ui,&mut self.ai.allow_apply,"Allow external clients to apply prepared edits");
                    ui.label(RichText::new("Applies only to this open project/session. No file or shell access.").small().color(t.ui.muted));
                } else if controls::action(ui,"Enable local MCP",controls::ActionTone::Primary,true).clicked() { self.enable_mcp(ctx); }
                ui.add_space(8.0); ui.separator();
                let proposals=self.session.proposals();
                if proposals.is_empty() {
                    ui.add_space(12.0);
                    ui.label(RichText::new("Ready for a proposal").strong());
                    ui.label("Ask your connected agent to inspect the sheet and propose a circuit edit. The preview will appear here before anything changes.");
                    ui.add_space(8.0);
                    ui.label(RichText::new("In-app model chat is not connected. This panel reviews external MCP work.").small().color(t.ui.muted));
                }
                let preview_height = (ui.available_height() - 185.0).clamp(90.0, 210.0);
                egui::ScrollArea::vertical().id_salt("ai-review").max_height(ui.available_height().max(120.0)).show(ui,|ui| {
                    for p in proposals {
                        let stale=p.base!=self.session.snapshot();
                        ui.add_space(10.0);
                        ui.label(RichText::new(&p.summary).strong());
                        ui.label(RichText::new(format!("{} operations · {} sheet occurrence(s) · revision {}",p.operations.len(),p.affected_occurrences.len(),p.base.revision)).small().color(t.ui.muted));
                        if stale { ui.colored_label(t.ui.warning,"Project changed. Discard and prepare again."); }
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.ai.before,true,"Before");
                            ui.selectable_value(&mut self.ai.before,false,"Proposed");
                        });
                        let project=if self.ai.before { Some(self.session.project()) } else { self.session.proposal_project(p.handle).ok() };
                        if let Some(doc)=project.and_then(|pjt|pjt.document(p.sheet)) { preview(ui,doc,preview_height); }
                        ui.collapsing("Operations and checks",|ui| {
                            for op in &p.operations { ui.label(format_operation(op, &p, self.session.project(), self.session.proposal_project(p.handle).ok())); }
                            for finding in &p.findings { ui.label(RichText::new(finding).small().color(t.ui.muted)); }
                        });
                        ui.horizontal(|ui| {
                            if controls::action(ui,"Apply",controls::ActionTone::Primary,!stale&&!self.busy()).clicked() {
                                let request=Apply{request_id:format!("gui-{:?}",p.handle),expected:p.base.clone(),handle:p.handle};
                                match self.session.apply_proposal(request) {
                                    Ok(_)=>self.after_ai_edit(), Err(e)=>self.notice(e),
                                }
                            }
                            if controls::action(ui,"Discard",controls::ActionTone::Secondary,true).clicked() { self.session.discard_proposal(p.handle); }
                        });
                        ui.separator();
                    }
                    if !self.ai.activity.is_empty() { ui.add_space(10.0); ui.label(RichText::new("TOOL ACTIVITY").small().color(t.ui.muted)); }
                    for line in self.ai.activity.iter().rev().take(5) { ui.label(RichText::new(line).small().color(t.ui.muted)); }
                });
            });
        self.ai.open = open;
    }
    pub(super) fn reset_ai_permissions(&mut self) {
        self.ai.allow_apply = false;
    }
}
fn format_operation(
    op: &pcbcraft_foundation::authoring::Operation,
    proposal: &pcbcraft_foundation::authoring::Proposal,
    before: &pcbcraft_foundation::project::Project,
    after: Option<&pcbcraft_foundation::project::Project>,
) -> String {
    let reference = |r: &pcbcraft_foundation::authoring::SymbolRef| {
        let id = match r {
            pcbcraft_foundation::authoring::SymbolRef::Id(id) => Some(*id),
            pcbcraft_foundation::authoring::SymbolRef::Local(key) => {
                proposal.created.get(key).copied()
            }
        };
        id.and_then(|id| {
            after
                .and_then(|p| p.document(proposal.sheet))
                .and_then(|d| d.component(id))
                .or_else(|| {
                    before
                        .document(proposal.sheet)
                        .and_then(|d| d.component(id))
                })
        })
        .map(|c| c.reference.clone())
        .unwrap_or_else(|| "Missing symbol".into())
    };
    use pcbcraft_foundation::authoring::Operation::*;
    match op {
        Place {
            kind,
            reference,
            value,
            ..
        } => format!(
            "Place {} · {} · {}",
            kind.name(),
            reference.as_deref().unwrap_or("automatic reference"),
            value.as_deref().unwrap_or(kind.value())
        ),
        Move { symbol, at } => format!(
            "Move {} to {:.2}, {:.2} mm",
            reference(symbol),
            at.x as f64 / 1e6,
            at.y as f64 / 1e6
        ),
        Edit {
            reference, value, ..
        } => format!("Edit {reference} · {value}"),
        Delete { symbol } => format!("Delete {} and its connections", reference(symbol)),
        Connect { from, to, .. } => format!(
            "Connect {}.{} → {}.{}",
            reference(&from.symbol),
            from.terminal,
            reference(&to.symbol),
            to.terminal
        ),
        Disconnect { .. } => "Remove connection".into(),
        RenameNet { name, .. } => format!("Name net {name}"),
        Note { text, .. } => format!("Add note: {text}"),
        Arrange {
            symbols, columns, ..
        } => format!("Arrange {} symbols in {columns} columns", symbols.len()),
    }
}
fn preview(ui: &mut egui::Ui, doc: &Document, height: f32) {
    let t = theme::current(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 6, t.canvas.background);
    if doc.symbols.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Empty sheet",
            egui::FontId::proportional(12.0),
            t.ui.muted,
        );
        return;
    }
    let world = |p: Point| egui::pos2(p.x as f32 / 1e6, p.y as f32 / 1e6);
    let mut bounds = egui::Rect::NOTHING;
    for s in &doc.symbols {
        if let Some(c) = doc.component(s.id) {
            bounds = bounds.union(egui::Rect::from_center_size(
                world(s.at),
                super::canvas::symbol_size(c.kind, s.quarter_turns) / 8.0,
            ));
        }
    }
    bounds = bounds.expand(14.0);
    let zoom = ((rect.width() - 24.0) / bounds.width())
        .min((rect.height() - 24.0) / bounds.height())
        .min(5.0);
    let screen = |p: Point| rect.center() + (world(p) - bounds.center()) * zoom;
    for w in &doc.wires {
        if let Ok(points) = pcbcraft_foundation::wiring::path(doc, w) {
            if w.bends.is_some() {
                painter.add(egui::Shape::line(
                    points.into_iter().map(screen).collect(),
                    egui::Stroke::new(1.0, t.canvas.wire),
                ));
            } else if let (Some(a), Some(b)) = (doc.endpoint(&w.from), doc.endpoint(&w.to)) {
                painter.line_segment([screen(a), screen(b)], egui::Stroke::new(1.0, t.ui.muted));
            }
        }
    }
    for s in &doc.symbols {
        if let Some(c) = doc.component(s.id) {
            super::canvas::paint_symbol(
                &painter,
                screen(s.at),
                zoom / 8.0,
                s.quarter_turns,
                c.kind,
                t.canvas.symbol,
            );
            let size = super::canvas::symbol_size(c.kind, s.quarter_turns) * (zoom / 8.0);
            painter.text(
                screen(s.at) - egui::vec2(0.0, size.y / 2.0 + 4.0),
                egui::Align2::CENTER_BOTTOM,
                &c.reference,
                egui::FontId::proportional(10.0),
                t.canvas.reference,
            );
        }
    }
    ui.label(RichText::new("Overview · legacy automatic routes are shown as endpoint links; inspect the live sheet after applying.").small().color(t.ui.muted));
}

#[cfg(test)]
mod tests {
    use super::*;
    use pcbcraft_foundation::{authoring::Prepare, bridge};

    fn remote(app: &mut App, call: mcp::Call) -> Result<serde_json::Value> {
        let (tx, rx) = std::sync::mpsc::channel();
        app.ai.bridge = Some(Bridge::start(move || {
            let _ = tx.send(());
        })?);
        let path = app.ai.bridge.as_ref().ok_or("No bridge")?.path();
        let connection = bridge::connect(&path)?;
        let worker = std::thread::spawn(move || connection.call(call));
        rx.recv_timeout(std::time::Duration::from_secs(3))
            .map_err(|e| e.to_string())?;
        app.poll_mcp();
        worker
            .join()
            .map_err(|_| "Client stopped")?
            .map_err(|e| e.message.into())
    }
    #[test]
    fn remote_apply_updates_real_gui_session_and_gui_undo_restores_it() -> Result<()> {
        let (mut app, _) = crate::desktop::tests::library_fixture()?;
        let before = app.session.project().clone();
        let expected = app.session.snapshot();
        let sheet = app.active_sheet();
        let plan = remote(
            &mut app,
            mcp::Call {
                name: "prepare_edit".into(),
                arguments: json!({"expected":expected,"sheet":sheet,"summary":"Add an agent note","operations":[{"op":"note","at":{"x":0,"y":0},"text":"Prepared by the test client"}]}),
            },
        )?;
        assert_eq!(app.session.project(), &before);
        assert!(app.ai.open);
        let args = json!({"expected":expected,"handle":plan["handle"],"request_id":"gui-bridge"});
        assert!(
            remote(
                &mut app,
                mcp::Call {
                    name: "apply_edit".into(),
                    arguments: args.clone()
                }
            )
            .is_err()
        );
        app.ai.allow_apply = true;
        remote(
            &mut app,
            mcp::Call {
                name: "apply_edit".into(),
                arguments: args.clone(),
            },
        )?;
        assert_eq!(app.session.revision(), 1);
        assert_eq!(
            app.document().annotations.len(),
            before.main.annotations.len() + 1
        );
        app.history(false);
        assert_eq!(app.session.project(), &before);
        remote(
            &mut app,
            mcp::Call {
                name: "apply_edit".into(),
                arguments: args,
            },
        )?;
        assert_eq!(app.session.project(), &before);
        assert_eq!(app.session.revision(), 2);
        app.act(crate::desktop::Action::New);
        assert!(!app.ai.allow_apply);
        Ok(())
    }
    #[test]
    fn review_panel_draws_actions_within_compact_window() -> Result<()> {
        let (mut app, ctx) = crate::desktop::tests::library_fixture()?;
        app.ai.open = true;
        let request:Prepare=serde_json::from_value(json!({"expected":app.session.snapshot(),"sheet":app.active_sheet(),"summary":"Reviewable circuit change","operations":[{"op":"note","at":{"x":0,"y":0},"text":"Note"}]})).map_err(|e|e.to_string())?;
        app.session.prepare(request)?;
        for size in [egui::vec2(1600.0, 1000.0), egui::vec2(1000.0, 700.0)] {
            let mut output = None;
            for _ in 0..4 {
                let mut frame = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                        ..Default::default()
                    },
                    |ui| app.show(ui),
                );
                frame.textures_delta.clear();
                output = Some(frame);
            }
            let output = output.ok_or("No frame")?;
            let texts = output
                .shapes
                .iter()
                .filter_map(|s| {
                    if let egui::Shape::Text(t) = &s.shape {
                        Some((t.galley.job.text.as_str(), t.pos))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert!(texts.iter().any(|(s, _)| *s == "Reviewable circuit change"));
            assert!(texts.iter().any(|(s, _)| *s == "Before"));
            let actions = texts
                .iter()
                .filter(|(s, _)| *s == "Apply" || *s == "Discard")
                .collect::<Vec<_>>();
            assert_eq!(actions.len(), 2);
            for (_, pos) in actions {
                assert!(
                    pos.x >= 0.0 && pos.y >= 0.0 && pos.x < size.x && pos.y < size.y,
                    "{pos:?} outside {size:?}"
                );
            }
        }
        assert_eq!(app.session.revision(), 0);
        Ok(())
    }
}

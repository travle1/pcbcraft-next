//! Live MCP, conversation and human proposal review through the same Electronics session.
#[cfg(not(target_arch = "wasm32"))]
mod conversation;
#[cfg(target_arch = "wasm32")]
mod conversation_web;
use super::{App, Tool, controls, theme};
use eframe::egui::{self, RichText};
use pcbcraft_foundation::{assistant, authoring::Apply, document::*};
#[cfg(not(target_arch = "wasm32"))]
use pcbcraft_foundation::{bridge::Bridge, mcp};
use serde_json::json;

#[derive(Default)]
pub(super) struct State {
    pub open: bool,
    width: Option<f32>,
    #[cfg(not(target_arch = "wasm32"))]
    bridge: Option<Bridge>,
    allow_apply: bool,
    before: bool,
    activity: Vec<String>,
    setup: bool,
    diagnostics: bool,
    draft: String,
    copied: Option<String>,
    active: Option<ProjectId>,
    applied: Option<pcbcraft_foundation::authoring::Snapshot>,
    chat: assistant::Conversation,
    #[cfg(not(target_arch = "wasm32"))]
    worker: Option<assistant::Worker>,
    #[cfg(not(target_arch = "wasm32"))]
    retiring: Vec<assistant::Worker>,
    #[cfg(not(target_arch = "wasm32"))]
    chat_bridge: Option<Bridge>,
    #[cfg(not(target_arch = "wasm32"))]
    chat_tools: usize,
    conversation_view: bool,
    #[cfg(not(target_arch = "wasm32"))]
    agent: Option<assistant::acp::Config>,
    #[cfg(not(target_arch = "wasm32"))]
    agent_path: String,
    #[cfg(not(target_arch = "wasm32"))]
    agent_arguments: String,
    #[cfg(not(target_arch = "wasm32"))]
    setup_error: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    capture_request: bool,
    capture_wait: bool,
    #[cfg(not(target_arch = "wasm32"))]
    capture_review: bool,
    #[cfg(test)]
    test_backend: Option<Box<dyn assistant::Backend>>,
}
#[cfg(not(target_arch = "wasm32"))]
impl Drop for State {
    fn drop(&mut self) {
        if let Some(bridge) = &self.chat_bridge {
            bridge.stop();
        }
        if let Some(worker) = self.worker.take() {
            worker.shutdown();
        }
        for worker in self.retiring.drain(..) {
            worker.shutdown();
        }
    }
}
impl State {
    pub(super) fn capture_ready(&self) -> bool {
        !self.capture_wait || (!self.chat.messages().is_empty() && self.chat.active().is_none())
    }
    pub(super) fn panel_width(&self, workspace_width: f32) -> f32 {
        self.width
            .unwrap_or(360.0)
            // Keep the bottom launcher and compact navigation clear of each other.
            .clamp(320.0, (workspace_width - 616.0).clamp(320.0, 600.0))
    }
}
impl App {
    pub(crate) fn open_ai(&mut self) {
        self.ai.open = true;
    }
    #[cfg(not(target_arch = "wasm32"))]
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
    #[cfg(not(target_arch = "wasm32"))]
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
        self.ai.applied = Some(self.session.snapshot());
    }
    pub(super) fn ai_panel(&mut self, ctx: &egui::Context) {
        if !self.ai.open {
            let rect = self.layout.assistant_entry;
            egui::Area::new("assistant_entry".into())
                .fixed_pos(rect.min)
                .movable(false)
                .fade_in(false)
                .show(ctx, |ui| {
                    if controls::assistant_entry(ui, rect.size()).clicked() {
                        self.open_ai();
                    }
                });
            return;
        }
        let t = theme::current(ctx);
        let rect = self.layout.assistant;
        let proposals = self.session.proposals();
        let active = proposals
            .iter()
            .find(|p| Some(p.handle) == self.ai.active)
            .or_else(|| proposals.first())
            .cloned();
        self.ai.active = active.as_ref().map(|p| p.handle);
        let mut open = true;
        controls::card(
            ctx,
            "assistant_panel",
            rect,
            egui::Order::Foreground,
            |ui| {
                let rail = egui::Rect::from_min_max(
                    ui.max_rect().min - egui::vec2(10.0, 0.0),
                    egui::pos2(ui.max_rect().left() - 4.0, ui.max_rect().bottom()),
                );
                let clip = ui.clip_rect();
                ui.set_clip_rect(rect.shrink(1.0));
                let resize = ui
                    .interact(rail, "assistant_resize".into(), egui::Sense::drag())
                    .on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
                ui.set_clip_rect(clip);
                if resize.dragged() {
                    self.ai.width = Some(rect.width() - ui.input(|i| i.pointer.delta().x));
                }
                ui.horizontal(|ui| {
                    ui.add(super::icons::Icon::Assistant.image(20.0, ctx));
                    ui.label(RichText::new("Assistant").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if super::panels::collapse_button(ui, "Collapse Assistant").clicked() {
                            open = false;
                        }
                        controls::menu_button(ui, "…", |ui| {
                            use super::icons::Icon;
                            if controls::MenuItem::new(
                                if cfg!(target_arch = "wasm32") {
                                    "Assistant connection…"
                                } else {
                                    "Connect an AI app…"
                                },
                                Icon::Assistant,
                            )
                            .show(ui)
                            .clicked()
                            {
                                self.ai.setup = true;
                                ui.close();
                            }
                            if controls::MenuItem::new("Diagnostics…", Icon::Properties)
                                .show(ui)
                                .clicked()
                            {
                                self.ai.diagnostics = true;
                                ui.close();
                            }
                            if controls::MenuItem::new("New conversation", Icon::Plus)
                                .enabled(!self.ai.chat.messages().is_empty())
                                .show(ui)
                                .clicked()
                            {
                                self.stop_assistant();
                                self.ai.chat.clear();
                                ui.close();
                            }
                        });
                    });
                });
                ui.separator();
                if active.is_some() && !self.ai.chat.messages().is_empty() {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.ai.conversation_view, true, "Conversation");
                        ui.selectable_value(&mut self.ai.conversation_view, false, "Changes");
                    });
                }
                ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                let footer_height = if active.is_some() { 152.0 } else { 198.0 };
                let action_height = if active.is_some() { 44.0 } else { 0.0 };
                let body_height =
                    (ui.available_height() - footer_height - action_height - 8.0).max(70.0);
                egui::ScrollArea::vertical().id_salt(if self.ai.conversation_view {"assistant_conversation"} else {"assistant_review"})
                .stick_to_bottom(self.ai.conversation_view)
                .min_scrolled_height(body_height).max_height(body_height).auto_shrink([false,false])
                .show(ui,|ui| {
                    if self.ai.conversation_view && !self.ai.chat.messages().is_empty() {
                        self.assistant_messages(ui);
                    } else if let Some(p)=&active {
                        if proposals.len()>1 {
                            controls::menu_button(ui,&format!("{} proposals",proposals.len()),|ui| {
                                for item in &proposals {
                                    if controls::MenuItem::new(&item.summary,super::icons::Icon::Sheet)
                                        .checked(item.handle==p.handle).show(ui).clicked() {
                                        self.ai.active=Some(item.handle); ui.close();
                                    }
                                }
                            });
                        }
                        if self.ai.chat.messages().is_empty() {ui.label(RichText::new("Proposed changes").strong());}
                        ui.label(&p.summary);
                        if p.affected_occurrences.len()>1 {
                            ui.label(RichText::new(format!("Updates {} uses of this sheet",p.affected_occurrences.len())).small().color(t.ui.muted));
                        }
                        let stale=p.base!=self.session.snapshot();
                        if stale {
                            ui.colored_label(t.ui.warning,"The project changed. Request an updated proposal before applying.");
                        }
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.ai.before,true,"Before");
                            ui.selectable_value(&mut self.ai.before,false,"Proposed");
                        });
                        let candidate=self.session.proposal_project(p.handle).ok();
                        let display=if self.ai.before {Some(self.session.project())} else {candidate};
                        if let Some(doc)=display.and_then(|project|project.document(p.sheet)) {
                            preview(ui,doc,(body_height-200.0).clamp(80.0,220.0));
                        }
                        if let Some((before,after))=self.session.project().document(p.sheet)
                            .zip(candidate.and_then(|project|project.document(p.sheet))) {
                            let summary=ReviewSummary::between(before,after);
                            if summary.rows.is_empty() && before==after { ui.label("No changes to apply"); }
                            ui.scope(|ui| {
                                ui.spacing_mut().interact_size.y=16.0;
                                for (label,count) in summary.rows {
                                    ui.horizontal(|ui| {
                                        ui.label(label);
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui| {ui.label(count.to_string());});
                                    });
                                }
                            });
                            if summary.connections_unchanged {
                                ui.label(RichText::new("Connections preserved · Electrical checks unavailable").small().color(t.ui.muted));
                            } else {
                                ui.label(RichText::new("Electrical checks unavailable").small().color(t.ui.muted));
                            }
                        }
                        ui.collapsing("Change details",|ui| {
                            for op in &p.operations {
                                ui.label(format_operation(op,p,self.session.project(),candidate));
                            }
                            for finding in &p.findings { ui.label(RichText::new(finding).small().color(t.ui.muted)); }
                        });
                    } else if self.ai.applied.as_ref()==Some(&self.session.snapshot()) {
                        ui.add_space(12.0);
                        ui.label(RichText::new("Changes applied").strong());
                        if controls::action(ui,"Undo changes",controls::ActionTone::Secondary,self.session.can_undo()&&!self.busy()).clicked() {
                            self.history(false); self.ai.applied=None;
                        }
                    } else if !self.ai.chat.messages().is_empty() {
                        self.assistant_messages(ui);
                    } else {
                        ui.add_space(16.0);
                        ui.label(RichText::new("Work on this sheet").strong());
                        ui.label(if self.has_chat_backend() {
                            "Ask about this sheet or describe a change. Proposed edits appear here for review."
                        } else if cfg!(target_arch = "wasm32") {"Copy a request with this sheet’s data to use in an external AI app. Local agent connections are available in the desktop app."} else {"Connect a local agent to send requests here, or copy a request to an external AI app."});
                        ui.add_space(12.0);
                        if !cfg!(target_arch = "wasm32") && !self.has_chat_backend() && controls::action(ui,"Connect an AI app",controls::ActionTone::Secondary,true).clicked() {
                            self.ai.setup=true;
                        }
                    }
                });
                if active.is_some() && self.ai.conversation_view {
                    ui.add_space(4.0);
                    if controls::action_width(
                        ui,
                        "Review changes",
                        controls::ActionTone::Primary,
                        true,
                        ui.available_width(),
                    )
                    .clicked()
                    {
                        self.ai.conversation_view = false;
                    }
                } else if let Some(p) = &active {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let changed = self
                            .session
                            .proposal_project(p.handle)
                            .is_ok_and(|candidate| candidate != self.session.project());
                        let commit_width =
                            ui.available_width() - 72.0 - ui.spacing().item_spacing.x;
                        if controls::action_width(
                            ui,
                            "Apply changes",
                            controls::ActionTone::Primary,
                            p.base == self.session.snapshot()
                                && !self.busy()
                                && changed
                                && self.ai.chat.active().is_none(),
                            commit_width,
                        )
                        .clicked()
                        {
                            let request = Apply {
                                request_id: format!("gui-{:?}", p.handle),
                                expected: p.base.clone(),
                                handle: p.handle,
                            };
                            match self.session.apply_proposal(request) {
                                Ok(_) => self.after_ai_edit(),
                                Err(e) => self.notice(e),
                            }
                        }
                        if controls::action(ui, "Discard", controls::ActionTone::Secondary, true)
                            .clicked()
                        {
                            self.session.discard_proposal(p.handle);
                        }
                    });
                }
                ui.separator();
                self.assistant_composer(ui, active.is_none());
            },
        );
        self.ai.open = open;
        self.assistant_settings(ctx);
    }

    fn assistant_composer(&mut self, ui: &mut egui::Ui, suggestions: bool) {
        ui.label(RichText::new("Your request").strong());
        let sheet = self.active_sheet();
        let name = self
            .session
            .project()
            .occurrences
            .iter()
            .find(|o| o.id == sheet)
            .map(|o| o.name.as_str())
            .unwrap_or("Sheet");
        let refs = self
            .selection
            .symbols
            .iter()
            .filter_map(|id| self.document().component(*id))
            .take(2)
            .map(|c| c.reference.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let context = if self.selection.len() > 2 || (!self.selection.is_empty() && refs.is_empty())
        {
            format!("{name} · {} selected", self.selection.len())
        } else if refs.is_empty() {
            name.into()
        } else {
            format!("{name} · {refs}")
        };
        ui.horizontal(|ui| {
            ui.add(super::icons::Icon::Sheet.image(14.0, ui.ctx()));
            ui.add(egui::Label::new(context).truncate());
        });
        if suggestions {
            ui.horizontal(|ui| {
                if controls::action(ui,"Explain sheet",controls::ActionTone::Secondary,true).clicked() {
                    self.ai.draft="Explain the circuit on this sheet.".into(); self.ai.copied=None;
                }
                if controls::action(ui,"Review connections",controls::ActionTone::Secondary,true).clicked() {
                    self.ai.draft="Review the connections on this sheet and identify assumptions or missing information.".into(); self.ai.copied=None;
                }
            });
        }
        let changed = egui::ScrollArea::vertical()
            .id_salt("assistant_draft")
            .min_scrolled_height(54.0)
            .max_height(54.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.ai.draft)
                        .hint_text("Ask about this sheet or describe a change…")
                        .desired_rows(3)
                        .desired_width(ui.available_width())
                        .char_limit(4000),
                )
                .changed()
            })
            .inner;
        if changed {
            self.ai.copied = None;
        }
        if self.ai.chat.active().is_some() {
            if controls::action(ui, "Stop", controls::ActionTone::Secondary, true).clicked() {
                self.stop_assistant();
            }
            return;
        }
        if self.has_chat_backend() {
            if controls::action(
                ui,
                "Send",
                controls::ActionTone::Primary,
                !self.ai.draft.trim().is_empty() && !self.assistant_editor_busy(),
            )
            .clicked()
                && let Err(error) = self.send_assistant(ui.ctx())
            {
                self.notice(error);
            }
            return;
        }
        let request = self.assistant_request();
        let label = if self.ai.copied.as_deref() == Some(request.as_str()) {
            "Request copied"
        } else {
            "Copy request"
        };
        if controls::action(
            ui,
            label,
            if suggestions {
                controls::ActionTone::Primary
            } else {
                controls::ActionTone::Secondary
            },
            !self.ai.draft.trim().is_empty(),
        )
        .clicked()
        {
            ui.ctx().copy_text(request.clone());
            self.ai.copied = Some(request);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn assistant_request(&self) -> String {
        let context = json!({"snapshot":self.session.snapshot(),"project":self.session.project().name,"proposal":self.ai.active,
            "sheet":self.active_sheet(),"selection":{"symbols":self.selection.symbols,"wires":self.selection.wires,"notes":self.selection.notes}});
        format!(
            "Help with this PCBcraft request using the live MCP tools. Inspect the current project before editing and prepare changes for review in Assistant.\n\nRequest:\n{}\n\nCurrent context (JSON data):\n{}",
            self.ai.draft.trim(),
            context
        )
    }

    fn assistant_settings(&mut self, ctx: &egui::Context) {
        if self.ai.setup {
            egui::Modal::new("assistant_connection".into()).show(ctx,|ui| {
                #[cfg(not(target_arch = "wasm32"))]
                let t=theme::current(ctx);
                ui.set_width(380.0);
                ui.heading("Assistant connections");
                self.assistant_agent_settings(ui);
                #[cfg(not(target_arch = "wasm32"))]
                ui.collapsing("External MCP app",|ui| {
                ui.label("Use an AI app that supports MCP. Enable the local connection, copy its configuration into that app, then paste your request there. Proposed edits return here for review.");
                ui.add_space(12.0);
                if let Some(bridge)=&self.ai.bridge {
                    let path=bridge.path();
                    ui.label("Local connection enabled");
                    ui.horizontal(|ui| {
                        if controls::action(ui,"Copy configuration",controls::ActionTone::Primary,true).clicked() {
                            let executable=std::env::current_exe().ok().map(|p|p.with_file_name("pcbcraft-mcp")).unwrap_or_else(||"pcbcraft-mcp".into());
                            match serde_json::to_string_pretty(&json!({"mcpServers":{"pcbcraft":{"command":executable,"args":["--connect",path]}}})) {
                                Ok(config)=>ui.ctx().copy_text(config),Err(e)=>self.notice(e),
                            }
                        }
                        if controls::action(ui,"Disable connection",controls::ActionTone::Secondary,true).clicked() {
                            self.ai.bridge=None; self.ai.allow_apply=false;
                        }
                    });
                    ui.add_space(8.0);
                    controls::checkbox(ui,&mut self.ai.allow_apply,"Allow the AI app to apply changes");
                    ui.label(RichText::new("When enabled, the connected app can commit proposals without your clicking Apply. This permission resets when the project changes.").small().color(t.ui.muted));
                } else if controls::action(ui,"Enable local connection",controls::ActionTone::Primary,true).clicked() {
                    self.enable_mcp(ctx);
                }
                ui.add_space(8.0);
                });
                ui.add_space(12.0);
                if controls::action(ui,"Done",controls::ActionTone::Secondary,true).clicked() || ui.ctx().input(|i|i.key_pressed(egui::Key::Escape)) {
                    self.ai.setup=false;
                }
            });
        }
        if self.ai.diagnostics {
            let mut open = true;
            egui::Window::new("Assistant diagnostics")
                .open(&mut open)
                .default_width(380.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            if self.ai.activity.is_empty() {
                                ui.label("No tool activity recorded.");
                            }
                            for line in self.ai.activity.iter().rev() {
                                ui.label(line);
                            }
                        });
                });
            self.ai.diagnostics = open;
        }
    }

    pub(super) fn reset_ai_permissions(&mut self) {
        self.stop_assistant();
        self.ai.chat.clear();
        self.ai.draft.clear();
        self.ai.allow_apply = false;
        self.ai.applied = None;
        self.ai.copied = None;
    }
}

struct ReviewSummary {
    rows: Vec<(&'static str, usize)>,
    connections_unchanged: bool,
}
impl ReviewSummary {
    fn between(before: &Document, after: &Document) -> Self {
        let mut added = 0;
        let mut moved = 0;
        let mut updated = 0;
        for s in &after.symbols {
            if let Some(old) = before.symbols.iter().find(|old| old.id == s.id) {
                moved += usize::from(old.at != s.at);
                updated += usize::from(
                    old.quarter_turns != s.quarter_turns
                        || before.component(s.id) != after.component(s.id),
                );
            } else {
                added += 1;
            }
        }
        let removed = before
            .symbols
            .iter()
            .filter(|old| !after.symbols.iter().any(|s| s.id == old.id))
            .count();
        let added_wires = after
            .wires
            .iter()
            .filter(|w| !before.wires.iter().any(|old| old.id == w.id))
            .count();
        let removed_wires = before
            .wires
            .iter()
            .filter(|old| !after.wires.iter().any(|w| w.id == old.id))
            .count();
        let rerouted = after
            .wires
            .iter()
            .filter(|w| {
                before.wires.iter().any(|old| {
                    old.id == w.id && (old.bends != w.bends || old.draw_path != w.draw_path)
                })
            })
            .count();
        let labels = after
            .wires
            .iter()
            .filter(|w| {
                before
                    .wires
                    .iter()
                    .any(|old| old.id == w.id && old.label != w.label)
            })
            .count();
        let notes = after
            .annotations
            .iter()
            .filter(|n| !before.annotations.iter().any(|old| old.id == n.id))
            .count();
        let links = |doc: &Document| {
            doc.wires
                .iter()
                .map(|w| {
                    if w.from <= w.to {
                        (w.from.clone(), w.to.clone())
                    } else {
                        (w.to.clone(), w.from.clone())
                    }
                })
                .collect::<std::collections::BTreeSet<_>>()
        };
        let parts = |doc: &Document| {
            doc.symbols
                .iter()
                .map(|s| (s.id, s.component))
                .collect::<std::collections::BTreeSet<_>>()
        };
        Self {
            rows: vec![
                ("Parts added", added),
                ("Parts removed", removed),
                ("Parts moved", moved),
                ("Parts updated", updated),
                ("Connections added", added_wires),
                ("Connections removed", removed_wires),
                ("Connections rerouted", rerouted),
                ("Connection labels updated", labels),
                ("Notes added", notes),
            ]
            .into_iter()
            .filter(|(_, count)| *count > 0)
            .collect(),
            connections_unchanged: links(before) == links(after) && parts(before) == parts(after),
        }
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
        ArrangeConnected {
            symbols,
            locked,
            region,
            ..
        } => {
            let fixed = if locked.is_empty() {
                "none".into()
            } else {
                locked.iter().map(reference).collect::<Vec<_>>().join(", ")
            };
            format!(
                "Arrange {} symbols and connections · fixed: {fixed}\nRegion: {:.1}, {:.1} to {:.1}, {:.1} mm",
                symbols.len(),
                region.min.x as f64 / 1e6,
                region.min.y as f64 / 1e6,
                region.max.x as f64 / 1e6,
                region.max.y as f64 / 1e6,
            )
        }
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
    if doc
        .wires
        .iter()
        .any(|w| w.bends.is_none() && (w.label.is_none() || w.draw_path))
    {
        ui.label(
            RichText::new("Overview uses endpoint links for automatic routes.")
                .small()
                .color(t.ui.muted),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pcbcraft_foundation::{authoring::Prepare, bridge, session::Session};

    struct PausedReply {
        started: std::sync::mpsc::Sender<assistant::Request>,
        release: std::sync::mpsc::Receiver<()>,
    }
    impl assistant::Backend for PausedReply {
        fn run(
            self: Box<Self>,
            request: assistant::Request,
            stream: &assistant::Stream,
        ) -> Result<()> {
            self.started
                .send(request)
                .map_err(|e| Error::Invalid(e.to_string()))?;
            stream.text("Inspecting your request.".into())?;
            while !stream.cancelled() {
                if self
                    .release
                    .recv_timeout(std::time::Duration::from_millis(10))
                    .is_ok()
                {
                    return stream.text(" Finished.".into());
                }
            }
            Ok(())
        }
    }
    fn paused_backend(
        app: &mut App,
    ) -> (
        std::sync::mpsc::Receiver<assistant::Request>,
        std::sync::mpsc::Sender<()>,
    ) {
        let (started, receive) = std::sync::mpsc::channel();
        let (release, wait) = std::sync::mpsc::channel();
        app.ai.test_backend = Some(Box::new(PausedReply {
            started,
            release: wait,
        }));
        (receive, release)
    }
    fn until(app: &mut App, ctx: &egui::Context, condition: impl Fn(&App) -> bool) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
        while !condition(app) && std::time::Instant::now() < deadline {
            frame(app, ctx, egui::vec2(1000.0, 700.0), vec![]);
            std::thread::yield_now();
        }
        assert!(condition(app), "Conversation condition did not settle");
    }

    #[test]
    fn conversation_send_stream_stop_manual_edit_and_project_switch() -> Result<()> {
        let (mut app, ctx) = crate::desktop::tests::library_fixture()?;
        app.library = false;
        app.ai.open = true;
        let before = app.session.project().clone();
        let (started, release) = paused_backend(&mut app);
        app.ai.draft = "Explain the selected part".into();
        let send = target(&mut app, &ctx, "Send")?;
        click(&mut app, &ctx, egui::vec2(1000.0, 700.0), send);
        let request = started
            .recv_timeout(std::time::Duration::from_secs(3))
            .map_err(|e| Error::Invalid(e.to_string()))?;
        assert_eq!(request.context.snapshot, app.session.snapshot());
        assert_eq!(request.context.sheet, app.active_sheet());
        assert_eq!(
            request.context.selection["symbols"],
            json!(app.selection.symbols)
        );
        assert_eq!(
            request.messages.last().map(|m| m.text.as_str()),
            Some("Explain the selected part")
        );
        until(&mut app, &ctx, |app| {
            app.ai
                .chat
                .messages()
                .last()
                .is_some_and(|m| !m.text.is_empty())
        });
        let stop = target(&mut app, &ctx, "Stop")?;
        assert!(app.layout.assistant.contains(stop));
        click(&mut app, &ctx, egui::vec2(1000.0, 700.0), stop);
        let _ = release.send(());
        assert!(app.ai.worker.is_none());
        assert_eq!(
            app.ai.chat.messages().last().map(|m| &m.status),
            Some(&assistant::Status::Stopped)
        );
        assert_eq!(app.session.project(), &before);

        let (started, _release) = paused_backend(&mut app);
        app.ai.draft = "Another request".into();
        app.send_assistant(&ctx)?;
        started
            .recv_timeout(std::time::Duration::from_secs(3))
            .map_err(|e| Error::Invalid(e.to_string()))?;
        app.session.apply(
            app.session.revision(),
            pcbcraft_foundation::session::Command::Place {
                kind: PartKind::Resistor,
                at: Point { x: 0, y: 0 },
            },
        )?;
        app.poll_assistant(&ctx);
        assert!(app.ai.worker.is_none());
        assert!(matches!(
            app.ai.chat.messages().last().map(|m| &m.status),
            Some(assistant::Status::Failed(_))
        ));
        assert_eq!(
            app.session.project().main.symbols.len(),
            before.main.symbols.len() + 1
        );
        app.act(crate::desktop::Action::New);
        assert!(app.ai.chat.messages().is_empty());
        assert!(app.ai.draft.is_empty());
        Ok(())
    }

    #[test]
    fn conversation_configuration_validates_without_starting_a_process() -> Result<()> {
        let (mut app, ctx) = crate::desktop::tests::library_fixture()?;
        app.library = false;
        app.ai.open = true;
        app.ai.setup = true;
        let before = app.session.project().clone();
        app.ai.agent_path = "relative-unconfigured-agent".into();
        let use_agent = target(&mut app, &ctx, "Use this agent")?;
        click(&mut app, &ctx, egui::vec2(1000.0, 700.0), use_agent);
        assert!(app.ai.setup_error.is_some());
        assert!(app.ai.agent.is_none());
        app.ai.agent_path = std::env::current_exe()?.to_string_lossy().into();
        app.ai.agent_arguments = "--acp\n--test-mode".into();
        let use_agent = target(&mut app, &ctx, "Use this agent")?;
        click(&mut app, &ctx, egui::vec2(1000.0, 700.0), use_agent);
        assert!(!app.ai.setup);
        assert_eq!(
            app.ai.agent.as_ref().map(|c| c.arguments.clone()),
            Some(vec!["--acp".into(), "--test-mode".into()])
        );
        assert!(app.ai.worker.is_none());
        assert!(app.ai.chat_bridge.is_none());
        assert!(app.ai.chat.messages().is_empty());
        assert_eq!(app.session.project(), &before);
        // Native capture automation cannot configure/send from an ordinary launch.
        assert!(
            app.capture_conversation(std::path::Path::new("unread-capture.json"))
                .is_err()
        );
        Ok(())
    }

    struct ProposeCircuit {
        path: std::path::PathBuf,
        prepared: std::sync::mpsc::Sender<()>,
        release: std::sync::mpsc::Receiver<()>,
    }
    impl assistant::Backend for ProposeCircuit {
        fn run(
            self: Box<Self>,
            request: assistant::Request,
            stream: &assistant::Stream,
        ) -> Result<()> {
            let connection = bridge::connect(&self.path)?;
            let call = |name: &str, arguments: serde_json::Value| {
                connection
                    .call(mcp::Call {
                        name: name.into(),
                        arguments,
                    })
                    .map_err(|e| Error::Invalid(e.message))
            };
            let inspected = call("inspect_project", json!({}))?;
            assert_eq!(inspected["snapshot"], json!(request.context.snapshot));
            assert_eq!(inspected["external_apply_enabled"], false);
            let proposal = call(
                "prepare_edit",
                json!({"expected":request.context.snapshot,"sheet":request.context.sheet,"summary":"Add two connected resistors","operations":[
                    {"op":"place","key":"a","kind":"resistor","at":{"x":0,"y":0}},
                    {"op":"place","key":"b","kind":"resistor","at":{"x":40*GRID_NM,"y":0}},
                    {"op":"connect","from":{"symbol":{"local":"a"},"terminal":"2"},"to":{"symbol":{"local":"b"},"terminal":"1"}}
                ]}),
            )?;
            assert!(call("apply_edit",json!({"expected":request.context.snapshot,"handle":proposal["handle"],"request_id":"must-review"})).is_err());
            stream.text("Two connected resistors are ready for review.".into())?;
            self.prepared
                .send(())
                .map_err(|e| Error::Invalid(e.to_string()))?;
            while !stream.cancelled() {
                if self
                    .release
                    .recv_timeout(std::time::Duration::from_millis(10))
                    .is_ok()
                {
                    break;
                }
            }
            Ok(())
        }
    }
    #[test]
    fn conversation_proposal_uses_live_bridge_then_review_apply_save_reopen_undo() -> Result<()> {
        let (mut app, ctx) = crate::desktop::tests::library_fixture()?;
        app.session = Session::new(Document::empty("Conversation circuit fixture")?)?;
        app.ensure_sheet();
        app.library = false;
        app.ai.open = true;
        app.ai.allow_apply = true; // External-MCP permission never grants chat permission.
        let before = app.session.project().clone();
        let (prepared, done) = std::sync::mpsc::channel();
        let (release, wait) = std::sync::mpsc::channel();
        let bridge = Bridge::start(|| {})?;
        let backend = ProposeCircuit {
            path: bridge.path(),
            prepared,
            release: wait,
        };
        app.ai.draft = "Add two connected resistors".into();
        app.start_assistant(&ctx, Box::new(backend), Some(bridge))?;
        until(&mut app, &ctx, |app| app.ai.chat_tools >= 3);
        until(&mut app, &ctx, |app| {
            app.ai
                .chat
                .messages()
                .last()
                .is_some_and(|m| m.text.contains("ready for review"))
        });
        done.recv_timeout(std::time::Duration::from_secs(3))
            .map_err(|e| Error::Invalid(e.to_string()))?;
        assert_eq!(app.session.project(), &before);
        assert_eq!(app.session.proposals().len(), 1);
        let review = target(&mut app, &ctx, "Review changes")?;
        click(&mut app, &ctx, egui::vec2(1000.0, 700.0), review);
        let apply = target(&mut app, &ctx, "Apply changes")?;
        click(&mut app, &ctx, egui::vec2(1000.0, 700.0), apply);
        assert_eq!(
            app.session.project(),
            &before,
            "Apply waits until response ends"
        );
        release
            .send(())
            .map_err(|e| Error::Invalid(e.to_string()))?;
        until(&mut app, &ctx, |app| app.ai.chat.active().is_none());
        let apply = target(&mut app, &ctx, "Apply changes")?;
        click(&mut app, &ctx, egui::vec2(1000.0, 700.0), apply);
        assert_eq!(app.document().symbols.len(), 2);
        assert_eq!(app.document().wires.len(), 1);
        assert_eq!(app.session.revision(), 1);
        let reopened = pcbcraft_foundation::persistence::decode_project(
            &pcbcraft_foundation::persistence::encode_project(app.session.project())?,
        )?;
        assert_eq!(&reopened, app.session.project());
        let undo = target(&mut app, &ctx, "Undo changes")?;
        click(&mut app, &ctx, egui::vec2(1000.0, 700.0), undo);
        assert_eq!(app.session.project(), &before);
        Ok(())
    }

    fn frame(app: &mut App, ctx: &egui::Context, size: egui::Vec2, events: Vec<egui::Event>) {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |ui| app.show(ui),
        )
        .drop_without_applying_deltas();
    }
    fn click(app: &mut App, ctx: &egui::Context, size: egui::Vec2, pos: egui::Pos2) {
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                size,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        for _ in 0..3 {
            frame(app, ctx, size, vec![]);
        }
    }

    #[test]
    fn assistant_entry_expands_to_workspace_height_and_collapses_without_edit_or_reframe()
    -> Result<()> {
        for (size, scale) in [
            (egui::vec2(1600.0, 1000.0), 1.0),
            (egui::vec2(1000.0, 700.0), 1.0),
            (egui::vec2(1000.0, 700.0), 1.5),
        ] {
            let (mut app, ctx) = crate::desktop::tests::library_fixture()?;
            ctx.set_pixels_per_point(scale);
            app.library = false;
            app.ai.draft = "Keep the supply fixed.".into();
            let pending: Prepare=serde_json::from_value(json!({"expected":app.session.snapshot(),"sheet":app.active_sheet(),"summary":"Keep pending review","operations":[{"op":"note","at":{"x":0,"y":0},"text":"Original proposal"}]})).map_err(|e|e.to_string())?;
            let proposal = app.session.prepare(pending)?;
            for _ in 0..4 {
                frame(&mut app, &ctx, size, vec![]);
            }
            let before = app.session.project().clone();
            let snapshot = app.session.snapshot();
            let camera = (app.canvas.zoom, app.canvas.offset);
            let properties = app.layout.properties;
            let filters = app.layout.filters;
            let entry = app.layout.assistant_entry;
            let output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                    ..Default::default()
                },
                |ui| app.show(ui),
            );
            let paint = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Rect(paint)
                        if paint.fill == theme::current(&ctx).ui.surface
                            && paint.rect.contains(entry.center()) =>
                    {
                        Some(paint.rect)
                    }
                    _ => None,
                })
                .ok_or("Assistant entry surface missing")?;
            assert!(
                (paint.min - entry.min).length() < 1.0 && (paint.max - entry.max).length() < 1.0,
                "Entry paint {paint:?} exceeds target {entry:?}"
            );
            output.drop_without_applying_deltas();
            assert_eq!(entry.right(), app.layout.bounds.right());
            assert_eq!(entry.bottom(), app.layout.bounds.bottom());
            assert!(app.layout.blocks_canvas(entry.center()));
            click(&mut app, &ctx, size, entry.center());
            assert!(app.ai.open);
            let panel = app.layout.assistant;
            assert_eq!(panel.top(), app.layout.bounds.top());
            assert_eq!(panel.bottom(), app.layout.bounds.bottom());
            assert_eq!(panel.right(), app.layout.bounds.right());
            assert!(!app.layout.assistant_entry.is_finite());
            let actual = ctx
                .memory(|m| m.area_rect("assistant_panel"))
                .ok_or("Assistant not drawn")?;
            assert!(
                (actual.min - panel.min).length() < 1.0 && (actual.max - panel.max).length() < 1.0,
                "{actual:?} differs from {panel:?}"
            );
            assert_eq!(app.layout.properties, properties);
            assert_eq!(app.layout.filters, filters);
            assert!(properties.intersects(panel) && filters.intersects(panel));
            assert!(!app.layout.navigation.intersects(panel));
            assert!(app.layout.blocks_canvas(panel.center()));
            app.tool = Tool::Place(PartKind::Resistor);
            click(
                &mut app,
                &ctx,
                size,
                panel.center_bottom() - egui::vec2(0.0, 35.0),
            );
            assert_eq!(app.session.project(), &before);
            click(
                &mut app,
                &ctx,
                size,
                panel.right_top() + egui::vec2(-24.0, 24.0),
            );
            assert!(!app.ai.open);
            assert!(app.layout.assistant_entry.is_finite());
            assert_eq!((app.canvas.zoom, app.canvas.offset), camera);
            assert_eq!(app.session.snapshot(), snapshot);
            assert_eq!(app.session.proposals()[0].handle, proposal.handle);
            assert_eq!(app.ai.draft, "Keep the supply fixed.");
        }
        Ok(())
    }

    #[test]
    fn assistant_opens_with_keyboard_and_resizes_only_within_workspace() -> Result<()> {
        let (mut app, ctx) = crate::desktop::tests::library_fixture()?;
        app.library = false;
        let size = egui::vec2(1600.0, 1000.0);
        for _ in 0..4 {
            frame(&mut app, &ctx, size, vec![]);
        }
        let camera = (app.canvas.zoom, app.canvas.offset);
        let before = app.session.project().clone();
        ctx.memory_mut(|m| m.request_focus("assistant_entry_toggle".into()));
        for pressed in [true, false] {
            frame(
                &mut app,
                &ctx,
                size,
                vec![egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
        }
        assert!(app.ai.open);
        for _ in 0..4 {
            frame(&mut app, &ctx, size, vec![]);
        }
        let panel = app.layout.assistant;
        let at = egui::pos2(panel.left() + 3.0, panel.center().y);
        frame(
            &mut app,
            &ctx,
            size,
            vec![
                egui::Event::PointerMoved(at),
                egui::Event::PointerButton {
                    pos: at,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        let to = at - egui::vec2(50.0, 0.0);
        frame(&mut app, &ctx, size, vec![egui::Event::PointerMoved(to)]);
        frame(
            &mut app,
            &ctx,
            size,
            vec![egui::Event::PointerButton {
                pos: to,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        for _ in 0..3 {
            frame(&mut app, &ctx, size, vec![]);
        }
        assert!(app.layout.assistant.width() > panel.width());
        for size in [egui::vec2(1000.0, 700.0), size] {
            frame(&mut app, &ctx, size, vec![]);
            assert_eq!(app.layout.assistant.bottom(), app.layout.bounds.bottom());
            assert!(app.layout.bounds.contains_rect(app.layout.assistant));
            assert!(app.layout.properties.intersects(app.layout.assistant));
            let launcher = egui::Rect::from_min_size(
                app.layout.bounds.left_bottom() - egui::vec2(0.0, 40.0),
                egui::vec2(292.0, 40.0),
            );
            assert!(!launcher.intersects(app.layout.navigation));
        }
        assert_eq!((app.canvas.zoom, app.canvas.offset), camera);
        assert_eq!(app.session.project(), &before);
        Ok(())
    }

    fn target(app: &mut App, ctx: &egui::Context, label: &str) -> Result<egui::Pos2> {
        for _ in 0..3 {
            crate::desktop::tests::workspace_frame(app, ctx, vec![]).drop_without_applying_deltas();
        }
        let output = crate::desktop::tests::workspace_frame(app, ctx, vec![]);
        let at = crate::desktop::tests::ribbon_text(&output, label)?
            .0
            .center();
        output.drop_without_applying_deltas();
        Ok(at)
    }

    #[test]
    fn assistant_request_copies_current_context_without_editing_or_sending() -> Result<()> {
        let (mut app, ctx) = crate::desktop::tests::library_fixture()?;
        app.library = false;
        app.ai.open = true;
        let size = egui::vec2(1000.0, 700.0);
        let before = app.session.project().clone();
        let input = target(&mut app, &ctx, "Ask about this sheet or describe a change…")?;
        click(&mut app, &ctx, size, input);
        frame(
            &mut app,
            &ctx,
            size,
            vec![egui::Event::Text("Keep the supply fixed.".into())],
        );
        assert_eq!(app.ai.draft, "Keep the supply fixed.");
        let suggestion = target(&mut app, &ctx, "Explain sheet")?;
        click(&mut app, &ctx, size, suggestion);
        assert_eq!(app.ai.draft, "Explain the circuit on this sheet.");
        let copy = target(&mut app, &ctx, "Copy request")?;
        frame(
            &mut app,
            &ctx,
            size,
            vec![
                egui::Event::PointerMoved(copy),
                egui::Event::PointerButton {
                    pos: copy,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        let output = crate::desktop::tests::workspace_frame(
            &mut app,
            &ctx,
            vec![egui::Event::PointerButton {
                pos: copy,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        let copied = output
            .platform_output
            .commands
            .iter()
            .find_map(|cmd| match cmd {
                egui::OutputCommand::CopyText(text) => Some(text),
                _ => None,
            })
            .ok_or("Request was not copied")?;
        assert!(copied.contains("Explain the circuit on this sheet."));
        let context: serde_json::Value = serde_json::from_str(
            copied
                .split("Current context (JSON data):\n")
                .nth(1)
                .ok_or("Missing context")?,
        )
        .map_err(|e| e.to_string())?;
        assert_eq!(context["sheet"], json!(app.active_sheet()));
        assert_eq!(
            context["selection"]["symbols"],
            json!(app.selection.symbols)
        );
        assert_eq!(context["snapshot"], json!(app.session.snapshot()));
        output.drop_without_applying_deltas();
        assert_eq!(app.session.project(), &before);
        assert!(app.session.proposals().is_empty());
        assert!(app.ai.bridge.is_none());
        // A long draft scrolls inside its own field, leaving the action visible.
        app.ai.draft = "Keep these existing connections. ".repeat(100);
        let at = target(&mut app, &ctx, "Copy request")?;
        assert!(app.layout.assistant.shrink(10.0).contains(at));
        assert_eq!(app.session.project(), &before);
        Ok(())
    }

    #[test]
    fn assistant_review_buttons_apply_undo_and_reject_stale_changes() -> Result<()> {
        let (mut app, ctx) = crate::desktop::tests::library_fixture()?;
        app.session = Session::new(Document::empty("Original review")?)?;
        app.ensure_sheet();
        app.library = false;
        app.ai.open = true;
        let size = egui::vec2(1000.0, 700.0);
        let before = app.session.project().clone();
        let prepare = |app: &App| -> Result<Prepare> {
            serde_json::from_value(json!({"expected":app.session.snapshot(),"sheet":app.active_sheet(),"summary":"Add a resistor","operations":[
                {"op":"place","key":"resistor","kind":"resistor","at":{"x":0,"y":0}}
            ]})).map_err(|e|e.to_string().into())
        };
        app.session.prepare(prepare(&app)?)?;
        let apply = target(&mut app, &ctx, "Apply changes")?;
        click(&mut app, &ctx, size, apply);
        assert_eq!(app.document().symbols.len(), 1);
        assert!(app.session.proposals().is_empty());
        let undo = target(&mut app, &ctx, "Undo changes")?;
        click(&mut app, &ctx, size, undo);
        assert_eq!(app.session.project(), &before);
        assert_eq!(app.session.revision(), 2);
        app.session.prepare(prepare(&app)?)?;
        app.session.apply(
            app.session.revision(),
            pcbcraft_foundation::session::Command::AddNote {
                at: Point::default(),
                text: "Manual edit".into(),
            },
        )?;
        let manual = app.session.project().clone();
        let apply = target(&mut app, &ctx, "Apply changes")?;
        click(&mut app, &ctx, size, apply);
        assert_eq!(app.session.project(), &manual);
        assert_eq!(app.session.proposals().len(), 1);
        let discard = target(&mut app, &ctx, "Discard")?;
        click(&mut app, &ctx, size, discard);
        assert!(app.session.proposals().is_empty());
        assert_eq!(app.session.project(), &manual);
        Ok(())
    }

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
        app.session = Session::new(Document::empty("Original arrangement review")?)?;
        app.ensure_sheet();
        app.ai.open = true;
        let point = |x: i64, y: i64| json!({"x":x*GRID_NM,"y":y*GRID_NM});
        let request:Prepare=serde_json::from_value(json!({"expected":app.session.snapshot(),"sheet":app.active_sheet(),"summary":"Reviewable circuit change","operations":[
            {"op":"place","key":"a","kind":"resistor","at":point(0,0)},
            {"op":"place","key":"b","kind":"led","at":point(80,0)},
            {"op":"connect","from":{"symbol":{"local":"a"},"terminal":"2"},"to":{"symbol":{"local":"b"},"terminal":"1"}},
            {"op":"arrange_connected","symbols":[{"local":"a"},{"local":"b"}],"locked":[{"local":"a"}],"origin":point(0,0),"columns":2,"spacing":point(40,40),"region":{"min":point(-20,-20),"max":point(80,20)}}
        ]})).map_err(|e|e.to_string())?;
        let proposal = app.session.prepare(request)?;
        let candidate = app.session.proposal_project(proposal.handle)?;
        let summary = ReviewSummary::between(app.document(), &candidate.main);
        assert!(summary.rows.contains(&("Parts added", 2)));
        assert!(summary.rows.contains(&("Connections added", 1)));
        assert!(!summary.connections_unchanged);
        app.ai.activity.push("inspect_project · completed".into());
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
            assert!(texts.iter().any(|(s, _)| *s == "Copy request"));
            let copy = texts
                .iter()
                .find(|(s, _)| *s == "Copy request")
                .ok_or("Missing request action")?
                .1;
            assert!(
                app.layout
                    .assistant
                    .shrink(10.0)
                    .contains(copy + egui::vec2(4.0, 16.0))
            );
            assert!(!texts.iter().any(|(s, _)| s.contains("inspect_project")
                || s.contains("revision")
                || s.contains("MCP connection")));
            let actions = texts
                .iter()
                .filter(|(s, _)| *s == "Apply changes" || *s == "Discard")
                .collect::<Vec<_>>();
            assert_eq!(actions.len(), 2);
            for (_, pos) in actions {
                assert!(
                    pos.x >= 0.0 && pos.y >= 0.0 && pos.x < size.x && pos.y < size.y,
                    "{pos:?} outside {size:?}"
                );
                assert!(
                    app.layout.assistant.shrink(10.0).contains(*pos),
                    "Action {pos:?} outside Assistant"
                );
            }
        }
        assert_eq!(app.session.revision(), 0);
        Ok(())
    }
}

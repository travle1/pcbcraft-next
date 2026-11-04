//! PCBcraft conversation composition and session-owned tool dispatch.
use super::*;
use assistant::{Context, Event, Role, Status};

impl App {
    /// Explicit native QA command. Normal launches neither read this file nor start an agent.
    pub(crate) fn capture_conversation(&mut self, path: &std::path::Path) -> Result<()> {
        use std::io::Read;
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Capture {
            agent: assistant::acp::Config,
            request: String,
            #[serde(default)]
            settings: bool,
            #[serde(default)]
            review: bool,
            #[serde(default)]
            streaming: bool,
        }
        if self.options.capture.is_none() {
            return Err(
                "--capture-conversation requires --capture; it is a native QA action.".into(),
            );
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(32769)
            .read_to_end(&mut bytes)?;
        if bytes.len() > 32768 {
            return Err("Conversation capture configuration exceeds 32 KiB.".into());
        }
        let capture: Capture =
            serde_json::from_slice(&bytes).map_err(|e| Error::Invalid(e.to_string()))?;
        capture.agent.validate()?;
        if capture.request.len() > assistant::MAX_PROMPT {
            return Err("Capture request exceeds the prompt limit.".into());
        }
        self.ai.agent_path = capture.agent.executable.to_string_lossy().into();
        self.ai.agent_arguments = capture.agent.arguments.join("\n");
        self.ai.agent = Some(capture.agent);
        self.ai.draft = capture.request;
        self.ai.open = true;
        self.ai.setup = capture.settings;
        self.ai.capture_request = !capture.settings;
        self.ai.capture_wait = !capture.settings && !capture.streaming;
        self.ai.capture_review = capture.review;
        Ok(())
    }
    pub(super) fn has_chat_backend(&self) -> bool {
        #[cfg(test)]
        if self.ai.test_backend.is_some() {
            return true;
        }
        self.ai.agent.is_some()
    }
    pub(super) fn assistant_editor_busy(&self) -> bool {
        self.busy()
            || self.pending.is_some()
            || self.canvas.drag.is_some()
            || self.canvas.wire_drag.is_some()
            || self.canvas.gesture.is_some()
            || self.inspector.draft.is_some()
            || self.text_ui.draft.is_some()
            || self.sheets.dialog.is_some()
            || self.nets.modal()
    }
    pub(super) fn send_assistant(&mut self, ctx: &egui::Context) -> Result<()> {
        if self.ai.chat.active().is_some() {
            return Err("Stop the current response before sending another request.".into());
        }
        if self.ai.retiring.len() >= 4 {
            return Err("The previous agent connection is still closing.".into());
        }
        if self.assistant_editor_busy() {
            return Err("Finish the active editor operation before sending.".into());
        }
        #[cfg(test)]
        if let Some(backend) = self.ai.test_backend.take() {
            return self.start_assistant(ctx, backend, None);
        }
        let config = self
            .ai
            .agent
            .clone()
            .ok_or("Configure an agent before sending a request.")?;
        config.validate()?;
        let adapter = std::env::current_exe()?
            .with_file_name(format!("pcbcraft-mcp{}", std::env::consts::EXE_SUFFIX));
        if !adapter.is_file() {
            return Err("Build both PCBcraft executables before connecting an agent.".into());
        }
        let wake = ctx.clone();
        let bridge = Bridge::start(move || wake.request_repaint())?;
        let backend = assistant::acp::Acp {
            config,
            adapter,
            connection: bridge.path(),
        };
        self.start_assistant(ctx, Box::new(backend), Some(bridge))
    }
    pub(super) fn start_assistant(
        &mut self,
        ctx: &egui::Context,
        backend: Box<dyn assistant::Backend>,
        bridge: Option<Bridge>,
    ) -> Result<()> {
        let context = Context {
            snapshot: self.session.snapshot(),
            sheet: self.active_sheet(),
            selection: json!({"symbols":self.selection.symbols,"wires":self.selection.wires,"notes":self.selection.notes}),
        };
        let request = self.ai.chat.begin(&self.ai.draft, context)?;
        let turn = request.turn;
        let wake = ctx.clone();
        match assistant::Worker::start(backend, request, move || wake.request_repaint()) {
            Ok(worker) => {
                self.ai.worker = Some(worker);
                self.ai.chat_bridge = bridge;
                self.ai.chat_tools = 0;
                self.ai.conversation_view = true;
                self.ai.draft.clear();
                self.ai.copied = None;
                Ok(())
            }
            Err(error) => {
                self.ai.chat.receive(turn, Event::Failed(error.to_string()));
                Err(error)
            }
        }
    }
    pub(super) fn stop_assistant(&mut self) {
        self.ai.chat.stop();
        self.retire_assistant();
    }
    fn retire_assistant(&mut self) {
        if let Some(worker) = self.ai.worker.take() {
            worker.cancel();
            self.ai.retiring.push(worker); // Join at shutdown if cleanup is still running.
        }
        if let Some(bridge) = self.ai.chat_bridge.take() {
            bridge.stop();
            // Bridge Drop joins bounded socket IO; never wait for it in an editor frame.
            let _ = std::thread::Builder::new()
                .name("pcbcraft-assistant-close".into())
                .spawn(move || drop(bridge));
        }
    }
    pub(in crate::desktop) fn poll_assistant(&mut self, ctx: &egui::Context) {
        self.ai.retiring.retain(|worker| !worker.finished());
        if self.ai.capture_request && !self.assistant_editor_busy() && self.frames > 2 {
            self.ai.capture_request = false;
            if let Err(error) = self.send_assistant(ctx) {
                self.ai.capture_wait = false;
                self.notice(error);
            }
        }
        if self.ai.chat.invalidate(&self.session.snapshot()) {
            self.retire_assistant();
        }
        for _ in 0..32 {
            let Some(worker) = self.ai.worker.as_ref() else {
                break;
            };
            let turn = worker.turn;
            let Some(event) = worker.next() else { break };
            self.ai.chat.receive(turn, event);
            if self.ai.chat.active().is_none() {
                self.retire_assistant();
                if self.ai.capture_review {
                    self.ai.conversation_view = false;
                }
                break;
            }
        }
        let Some(pending) = self.ai.chat_bridge.as_ref().and_then(Bridge::next) else {
            return;
        };
        let context = self.ai.chat.active().map(|(_, context)| context.clone());
        let reply = if std::time::Instant::now() >= pending.deadline {
            Err(mcp::Failure {
                code: "expired".into(),
                message: "This tool request expired before execution.".into(),
            })
        } else if self.assistant_editor_busy() {
            Err(mcp::Failure {
                code: "busy".into(),
                message: "An editor operation is active. Retry after it finishes.".into(),
            })
        } else if self.ai.chat_tools >= 64 {
            Err(mcp::Failure {
                code: "limit".into(),
                message: "This response reached its 64-tool limit.".into(),
            })
        } else if let Some(context) = context {
            self.ai.chat_tools += 1;
            assistant::dispatch(&mut self.session, &context, pending.call.clone())
        } else {
            Err(mcp::Failure {
                code: "cancelled".into(),
                message: "This conversation turn is no longer active.".into(),
            })
        };
        if pending.call.name == "prepare_edit"
            && let Ok(value) = &reply
        {
            self.ai.active = value
                .get("handle")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
        }
        self.ai.activity.push(format!(
            "{} · {}",
            pending.call.name,
            if reply.is_ok() {
                "completed"
            } else {
                "rejected"
            }
        ));
        if self.ai.activity.len() > 12 {
            self.ai.activity.remove(0);
        }
        pending.reply(reply);
    }
    pub(super) fn assistant_messages(&self, ui: &mut egui::Ui) {
        let t = theme::current(ui.ctx());
        for message in self.ai.chat.messages() {
            ui.label(
                RichText::new(match message.role {
                    Role::User => "You",
                    Role::Assistant => "Assistant",
                })
                .strong(),
            );
            if !message.text.is_empty() {
                ui.add(egui::Label::new(&message.text).wrap());
            }
            match &message.status {
                Status::Streaming => {
                    ui.label(
                        RichText::new(self.ai.chat.activity())
                            .small()
                            .color(t.ui.muted),
                    );
                }
                Status::Stopped => {
                    ui.label(RichText::new("Stopped").small().color(t.ui.muted));
                }
                Status::Failed(error) => {
                    ui.label(RichText::new(error).color(t.ui.warning));
                }
                Status::Complete => {}
            }
            ui.add_space(12.0);
        }
    }
    pub(super) fn assistant_agent_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("In-app Assistant").strong());
        ui.label("Choose an installed ACP agent. Sending a request runs this command with its existing sign-in and permissions. This is not an operating-system sandbox.");
        ui.label("Executable");
        ui.add(
            controls::text_input(egui::TextEdit::singleline(&mut self.ai.agent_path))
                .desired_width(ui.available_width())
                .char_limit(4096),
        );
        ui.label("Arguments (one per line)");
        egui::ScrollArea::vertical()
            .id_salt("assistant_arguments")
            .min_scrolled_height(54.0)
            .max_height(54.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    controls::text_input(egui::TextEdit::multiline(&mut self.ai.agent_arguments))
                        .desired_width(ui.available_width())
                        .desired_rows(2)
                        .char_limit(8192),
                );
            });
        ui.horizontal(|ui| {
            if controls::action(ui, "Use this agent", controls::ActionTone::Primary, true).clicked()
            {
                let config = assistant::acp::Config {
                    executable: self.ai.agent_path.trim().into(),
                    arguments: self
                        .ai
                        .agent_arguments
                        .lines()
                        .filter(|line| !line.is_empty())
                        .map(String::from)
                        .collect(),
                };
                match config.validate() {
                    Ok(()) => {
                        self.stop_assistant();
                        self.ai.chat.clear();
                        self.ai.agent = Some(config);
                        self.ai.setup_error = None;
                        self.ai.setup = false;
                    }
                    Err(error) => self.ai.setup_error = Some(error.to_string()),
                }
            }
            if self.ai.agent.is_some()
                && controls::action(
                    ui,
                    "Disconnect agent",
                    controls::ActionTone::Secondary,
                    true,
                )
                .clicked()
            {
                self.stop_assistant();
                self.ai.agent = None;
            }
        });
        if let Some(error) = &self.ai.setup_error {
            ui.colored_label(theme::current(ui.ctx()).ui.warning, error);
        }
        ui.add_space(12.0);
    }
}

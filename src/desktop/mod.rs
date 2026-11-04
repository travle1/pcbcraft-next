mod ai;
mod canvas;
mod controls;
mod icons;
mod inspector;
mod interaction;
mod library;
mod nets;
mod page;
mod palette;
mod panels;
mod ribbon;
mod sheets;
#[cfg(test)]
mod tests;
mod text_editor;
mod theme;
mod wire_geometry;
mod workspace;

use eframe::egui::{self, RichText};
use icons::Icon;
use pcbcraft_foundation::{
    document::*,
    persistence,
    session::{Command, Session},
};
use std::{path::PathBuf, sync::mpsc, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Select,
    Pan,
    Wire,
    Place(PartKind),
    SheetPlacement,
    Text,
}
#[derive(Clone, Copy)]
enum Action {
    New,
    Open,
    Example,
    Quit,
}
pub struct Options {
    pub demo: bool,
    pub capture: Option<PathBuf>,
    pub exit_after_capture: bool,
    pub capture_workspace: bool,
    pub capture_library: bool,
    pub capture_panels: bool,
    pub capture_search: bool,
    pub capture_properties: bool,
    pub capture_nets: bool,
    pub capture_wire_menu: bool,
    pub capture_text: bool,
    pub capture_selection: bool,
    pub capture_view: bool,
    pub capture_text_placement: bool,
    pub open: Option<PathBuf>,
}
enum FileResult {
    Opened(PathBuf, pcbcraft_foundation::project::Project, Vec<u8>),
    Saved(PathBuf, pcbcraft_foundation::project::Project, Vec<u8>),
    Cancelled,
    Error(Error),
}
pub struct App {
    session: Session,
    ai: ai::State,
    sheets: sheets::State,
    selection: pcbcraft_foundation::selection::Selection,
    text_ui: text_editor::State,
    tool: Tool,
    ribbon_tab: ribbon::Tab,
    canvas: canvas::Canvas,
    reference: String,
    value: String,
    browser: bool,
    properties: bool,
    filters: bool,
    library: bool,
    library_ui: library::State,
    panel_ui: panels::State,
    inspector: inspector::State,
    nets: nets::State,
    layout: panels::Layout,
    palette: bool,
    palette_ui: palette::State,
    query: String,
    drawer: Option<&'static str>,
    notices: Vec<String>,
    path: Option<PathBuf>,
    disk_bytes: Option<Vec<u8>>,
    job: Option<mpsc::Receiver<FileResult>>,
    pending: Option<Action>,
    after_save: Option<Action>,
    close_allowed: bool,
    options: Options,
    frames: u32,
    started: std::time::Instant,
    capture_sent: bool,
    capture_pointer: Option<egui::Pos2>,
    capture_input_step: u8,
}
impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, options: Options) -> Result<Self> {
        theme::apply(&cc.egui_ctx);
        if options.capture_workspace {
            egui::Popup::open_id(&cc.egui_ctx, workspace::popup_id());
        }
        Self::create(options)
    }
    fn create(options: Options) -> Result<Self> {
        let session = if options.demo {
            Session::demo()?
        } else {
            Session::new(Document::empty("Untitled")?)?
        };
        let mut app = Self {
            session,
            ai: Default::default(),
            sheets: Default::default(),
            selection: Default::default(),
            text_ui: Default::default(),
            tool: Tool::Select,
            ribbon_tab: ribbon::Tab::default(),
            canvas: Default::default(),
            reference: String::new(),
            value: String::new(),
            browser: true,
            properties: true,
            filters: true,
            library: options.capture_library || options.capture_panels,
            library_ui: library::State::default(),
            panel_ui: panels::State::default(),
            inspector: inspector::State::default(),
            nets: nets::State::default(),
            layout: panels::Layout::default(),
            palette: options.capture_search,
            palette_ui: palette::State::default(),
            query: String::new(),
            drawer: None,
            notices: vec!["Schematic draft · original built-in symbols. ERC, footprints and manufacturing are not implemented.".into()],
            path: None,
            disk_bytes: None,
            job: None,
            pending: None,
            after_save: None,
            close_allowed: false,
            options,
            frames: 0,
            started: std::time::Instant::now(),
            capture_sent: false,
            capture_pointer: None,
            capture_input_step: 0,
        };
        if app.options.capture_panels {
            app.drawer = Some("Messages");
            app.tool = Tool::Wire;
            app.library_ui.dock = library::Dock::Left;
        }
        if app.options.demo {
            let id = app.document().symbols.get(1).map(|s| s.id);
            app.select(id);
        }
        if app.options.capture_nets {
            app.refresh_nets();
            let anchor = app
                .nets
                .snapshot
                .nets
                .iter()
                .find(|n| n.name.as_deref() == Some("+3V3"))
                .map(|n| n.anchor.clone());
            if let Some(anchor) = anchor {
                app.select_net(anchor);
            }
            app.panel_ui.nets_open.insert(app.active_sheet());
            app.open_sheet_index(true);
        }
        if app.options.capture_text
            && app.options.capture.is_some()
            && let Some(note) = app.document().annotations.first()
        {
            app.open_text(pcbcraft_foundation::text::TextTarget::Note(note.id));
        }
        if app.options.capture_view {
            app.ribbon_tab = ribbon::Tab::View;
        }
        if app.options.capture_text_placement {
            app.tool = Tool::Text;
        }
        if app.options.capture_properties {
            app.open_properties();
        }
        if let Some(path) = app.options.open.clone() {
            app.load(Some(path));
        }
        Ok(app)
    }
    fn busy(&self) -> bool {
        self.job.is_some()
    }
    fn notice(&mut self, text: impl ToString) {
        self.notices.push(text.to_string());
        if self.notices.len() > 40 {
            self.notices.remove(0);
        }
    }
    fn selected_symbol(&self) -> Option<SymbolId> {
        self.selection.primary_symbol()
    }
    fn select(&mut self, id: Option<SymbolId>) {
        self.sheets.selected = None;
        self.selection.clear();
        if let Some(id) = id {
            self.selection.symbols.insert(id);
        }
        self.text_ui = Default::default();
        self.nets.selected = None;
        self.nets.wire = None;
        self.refresh_selection();
    }
    fn set_selection(&mut self, items: pcbcraft_foundation::selection::Selection) {
        self.sheets.selected = None;
        self.selection = items;
        self.text_ui = Default::default();
        self.nets.selected = None;
        self.nets.wire = None;
        self.refresh_selection();
        self.properties = true;
    }
    fn refresh_selection(&mut self) {
        let doc = self.document().clone();
        self.selection.retain(&doc);
        if let Some(c) = self
            .selected_symbol()
            .and_then(|id| self.document().component(id))
            .cloned()
        {
            self.reference = c.reference.clone();
            self.value = c.value.clone();
        }
        if self
            .text_ui
            .draft
            .as_ref()
            .is_some_and(|d| !d.target.exists(self.document()))
        {
            self.text_ui = Default::default();
        }
    }
    fn command(&mut self, command: Command) {
        if self.busy() || self.pending.is_some() {
            return;
        }
        match self.apply(self.session.revision(), command) {
            Ok(Some(id)) => self.select(Some(id)),
            Ok(None) => self.refresh_selection(),
            Err(e) => {
                self.notice(e);
                self.drawer = Some("Messages");
            }
        }
    }
    fn history(&mut self, redo: bool) {
        if self.busy() || self.pending.is_some() {
            return;
        }
        let r = if redo {
            self.session.redo()
        } else {
            self.session.undo()
        };
        if let Err(e) = r {
            self.notice(e);
        }
        self.ensure_sheet();
        self.sheets.cancel_drag();
        self.refresh_selection();
        self.canvas.drag = None;
        self.canvas.wire_drag = None;
        self.canvas.wire_start = None;
        self.canvas.gesture = None;
        self.canvas.text_box_start = None;
        self.nets.selected = None;
        self.nets.wire = None;
        self.text_ui = Default::default();
    }
    fn request(&mut self, action: Action) {
        if self.busy() {
            return;
        }
        if self.session.dirty() {
            self.pending = Some(action);
        } else {
            self.act(action);
        }
    }
    fn act(&mut self, action: Action) {
        self.pending = None;
        match action {
            Action::New | Action::Example => {
                let next = match action {
                    Action::Example => Session::demo(),
                    _ => Document::empty("Untitled").and_then(Session::new),
                };
                match next {
                    Ok(s) => {
                        self.session = s;
                        self.reset_ai_permissions();
                        self.sheets = Default::default();
                        self.nets = Default::default();
                        self.path = None;
                        self.disk_bytes = None;
                        self.select(None);
                        self.canvas = Default::default();
                        self.tool = Tool::Select;
                    }
                    Err(e) => self.notice(e),
                }
            }
            Action::Open => self.load(None),
            Action::Quit => self.close_allowed = true,
        }
    }
    fn load(&mut self, path: Option<PathBuf>) {
        let (tx, rx) = mpsc::channel();
        self.job = Some(rx);
        std::thread::spawn(move || {
            let path = path.or_else(|| {
                rfd::FileDialog::new()
                    .set_title("Open PCBcraft schematic draft")
                    .add_filter("PCBcraft draft", &["pc"])
                    .pick_file()
            });
            let result = match path {
                None => FileResult::Cancelled,
                Some(path) => match persistence::read_project(&path) {
                    Ok((doc, bytes)) => FileResult::Opened(path, doc, bytes),
                    Err(e) => FileResult::Error(e),
                },
            };
            let _ = tx.send(result);
        });
    }
    fn save(&mut self, save_as: bool) {
        if self.busy() {
            return;
        }
        let doc = self.session.project().clone();
        let current = if save_as { None } else { self.path.clone() };
        let expected = if current.is_some() {
            self.disk_bytes.clone()
        } else {
            None
        };
        let (tx, rx) = mpsc::channel();
        self.job = Some(rx);
        std::thread::spawn(move || {
            let path = current.or_else(|| {
                rfd::FileDialog::new()
                    .set_title("Save experimental PCBcraft draft")
                    .add_filter("PCBcraft draft", &["pc"])
                    .set_file_name(format!("{}.pc", doc.name))
                    .save_file()
            });
            let result = match path {
                None => FileResult::Cancelled,
                Some(mut path) => {
                    if path.extension().is_none() {
                        path.set_extension("pc");
                    }
                    match persistence::write_project(&path, &doc, expected.as_deref()) {
                        Ok(bytes) => FileResult::Saved(path, doc, bytes),
                        Err(e) => FileResult::Error(e),
                    }
                }
            };
            let _ = tx.send(result);
        });
    }
    fn poll(&mut self, ctx: &egui::Context) {
        let result = self.job.as_ref().map(|rx| rx.try_recv());
        match result {
            Some(Ok(result)) => {
                self.job = None;
                match result {
                    FileResult::Opened(path, doc, bytes) => match Session::from_project(doc) {
                        Ok(s) => {
                            self.session = s;
                            self.reset_ai_permissions();
                            self.sheets = Default::default();
                            self.sheets.folded = self
                                .session
                                .project()
                                .occurrences
                                .iter()
                                .filter(|o| o.id != self.session.project().root)
                                .map(|o| o.id)
                                .collect();
                            self.nets = Default::default();
                            self.path = Some(path);
                            self.disk_bytes = Some(bytes);
                            self.select(None);
                            self.canvas = Default::default();
                            self.tool = Tool::Select;
                            self.notice("Project opened. ERC has not been run.");
                        }
                        Err(e) => self.notice(e),
                    },
                    FileResult::Saved(path, doc, bytes) => {
                        self.session.mark_project_saved(doc);
                        self.path = Some(path);
                        self.disk_bytes = Some(bytes);
                        self.notice("Schematic draft saved.");
                        if let Some(action) = self.after_save.take() {
                            self.act(action);
                        }
                    }
                    FileResult::Cancelled => self.after_save = None,
                    FileResult::Error(e) => {
                        self.after_save = None;
                        self.notice(e);
                        self.drawer = Some("Messages");
                    }
                }
            }
            Some(Err(mpsc::TryRecvError::Disconnected)) => {
                self.job = None;
                self.after_save = None;
                self.notice("File operation stopped unexpectedly; project retained.");
            }
            Some(Err(mpsc::TryRecvError::Empty)) => {
                ctx.request_repaint_after(Duration::from_millis(80))
            }
            None => {}
        }
    }
    fn shortcuts(&mut self, ctx: &egui::Context) {
        if self.busy()
            || self.pending.is_some()
            || self.palette
            || self.inspector.draft.is_some()
            || self.nets.modal()
            || self.sheets.dialog.is_some()
            || egui::Popup::is_any_open(ctx)
        {
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.tool = Tool::Select;
            self.sheets.cancel_drag();
            self.canvas.drag = None;
            self.canvas.wire_drag = None;
            self.canvas.gesture = None;
            self.canvas.text_box_start = None;
            self.canvas.wire_start = None;
            self.palette = false;
            self.library_ui.cancel_drag();
            self.inspector.cancel_drag();
        }
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let pressed = |key| ctx.input(|i| i.key_pressed(key));
        let modifiers = ctx.input(|i| i.modifiers);
        if modifiers.command {
            if pressed(egui::Key::S) {
                self.save(modifiers.shift);
            }
            if pressed(egui::Key::O) {
                self.request(Action::Open);
            }
            if pressed(egui::Key::N) {
                self.request(Action::New);
            }
            if pressed(egui::Key::Z) {
                self.history(modifiers.shift);
            }
            if pressed(egui::Key::Y) {
                self.history(true);
            }
        } else {
            if pressed(egui::Key::T) {
                self.tool = Tool::Text;
            }
            if pressed(egui::Key::A) {
                self.library = true;
            }
            if pressed(egui::Key::W) {
                self.tool = Tool::Wire;
                self.canvas.wire_start = None;
            }
            if pressed(egui::Key::V) {
                self.tool = Tool::Select;
            }
            if pressed(egui::Key::F) {
                self.canvas.fit = true;
                self.canvas.fit_page = ctx.input(|i| i.modifiers.shift);
            }
            if pressed(egui::Key::S) {
                self.palette = true;
            }
            if self.tool == Tool::Wire {
                return;
            }
            if pressed(egui::Key::Delete) || pressed(egui::Key::Backspace) {
                self.delete_selection();
            }
            if let Some(symbol) = self.selected_symbol()
                && pressed(egui::Key::R)
            {
                self.command(Command::Rotate { symbol });
            }
        }
    }
    fn delete_selection(&mut self) {
        if let Some(object) = self.sheets.selected {
            match object {
                sheets::Object::Block(id) => {
                    if let Some(child) = self.session.project().child(self.active_sheet(), id) {
                        self.start_sheet_dialog(sheets::Kind::Delete, child.id);
                    }
                }
                sheets::Object::Port(id) => {
                    self.start_sheet_dialog(sheets::Kind::DeletePort(id), self.active_sheet())
                }
            }
            return;
        }
        if !self.selection.is_empty() {
            self.command(Command::DeleteItems {
                items: self.selection.clone(),
            });
            return;
        }
        if let Some(wire) = self.nets.wire {
            self.command(Command::Disconnect { wire });
        } else if let Some(anchor) = self.nets.selected.clone() {
            self.command(Command::DeleteNet { anchor });
        } else if let Some(symbol) = self.selected_symbol() {
            self.command(Command::Delete { symbol });
        }
    }
    fn top_bar(&mut self, ui: &mut egui::Ui) {
        let t = theme::current(ui.ctx());
        egui::Panel::top("app_bar")
            .exact_size(t.metrics.app_bar_height)
            .frame(
                egui::Frame::new()
                    .fill(t.ui.chrome)
                    .inner_margin(egui::Margin::symmetric(12, 4)),
            )
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    theme::quiet_buttons(ui);
                    ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
                    ui.add(Icon::Brand.image(23.0, ui.ctx()));
                    ui.label(RichText::new("PCBcraft").font(theme::semibold(14.0)));
                    ui.separator();
                    if icons::button(ui, Icon::Home, "Open example", !self.busy()).clicked() {
                        self.request(Action::Example);
                    }
                    controls::menu_button(ui, "File", |ui| {
                        use controls::MenuItem;
                        for (label, key, icon, action) in [
                            ("New schematic", "Ctrl+N", Icon::Plus, Action::New),
                            ("Open draft…", "Ctrl+O", Icon::Folder, Action::Open),
                            ("Open example", "", Icon::Workspace, Action::Example),
                        ] {
                            if MenuItem::new(label, icon)
                                .shortcut(key)
                                .enabled(!self.busy())
                                .show(ui)
                                .clicked()
                            {
                                ui.close();
                                self.request(action);
                            }
                        }
                        controls::menu_separator(ui);
                        if MenuItem::new("Save draft", Icon::Save)
                            .shortcut("Ctrl+S")
                            .enabled(!self.busy())
                            .show(ui)
                            .clicked()
                        {
                            ui.close();
                            self.save(false);
                        }
                        if MenuItem::new("Save draft as…", Icon::Save)
                            .enabled(!self.busy())
                            .show(ui)
                            .clicked()
                        {
                            ui.close();
                            self.save(true);
                        }
                        controls::menu_separator(ui);
                        if MenuItem::new("Quit", Icon::Close).show(ui).clicked() {
                            ui.close();
                            self.request(Action::Quit);
                        }
                    });
                    if icons::button(ui, Icon::Save, "Save draft · Ctrl+S", !self.busy()).clicked()
                    {
                        self.save(false);
                    }
                    if icons::button(
                        ui,
                        Icon::Undo,
                        "Undo · Ctrl+Z",
                        self.session.can_undo() && !self.busy(),
                    )
                    .clicked()
                    {
                        self.history(false);
                    }
                    if icons::button(
                        ui,
                        Icon::Redo,
                        "Redo · Ctrl+Shift+Z",
                        self.session.can_redo() && !self.busy(),
                    )
                    .clicked()
                    {
                        self.history(true);
                    }
                    ui.separator();
                    let title = format!(
                        "{}{}",
                        self.session.project().name,
                        if self.session.dirty() { "  •" } else { "" }
                    );
                    ui.add_sized([200.0, 28.0], egui::Button::new(title).fill(t.ui.raised))
                        .on_hover_text("One project · all sheets share history and connectivity");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if icons::button(ui, Icon::Close, "Close PCBcraft", true).clicked() {
                            self.request(Action::Quit);
                        }
                        if ui.button("□").on_hover_text("Maximize / restore").clicked() {
                            let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                            ui.ctx()
                                .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                        }
                        if ui.button("−").on_hover_text("Minimize").clicked() {
                            ui.ctx()
                                .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                        if controls::action(ui, "AI / MCP", controls::ActionTone::Secondary, true)
                            .clicked()
                        {
                            self.ai.open = !self.ai.open;
                        }
                        ui.label(RichText::new("SCHEMATIC DRAFT").small().color(t.ui.muted));
                        let r = ui.allocate_response(ui.available_size(), egui::Sense::drag());
                        if r.drag_started() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
                    });
                });
            });
    }
    fn footer(&mut self, ui: &mut egui::Ui) {
        let t = theme::current(ui.ctx());
        egui::Panel::bottom("status")
            .exact_size(28.0)
            .frame(
                egui::Frame::new()
                    .fill(t.ui.chrome)
                    .inner_margin(egui::Margin::symmetric(14, 4)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let text = if self.busy() {
                        "File operation…"
                    } else if self.session.dirty() {
                        "●  Unsaved changes"
                    } else if self.path.is_some() {
                        "●  Saved locally"
                    } else {
                        "○  Not saved"
                    };
                    ui.label(RichText::new(text).small().color(if self.session.dirty() {
                        t.ui.warning
                    } else {
                        t.ui.muted
                    }));
                    ui.separator();
                    ui.label(
                        RichText::new("ERC  ·  Not implemented")
                            .small()
                            .color(t.ui.muted),
                    );
                });
            });
    }
    fn dialogs(&mut self, ctx: &egui::Context) {
        self.command_palette(ctx);
        if let Some(action) = self.pending {
            egui::Modal::new("unsaved".into()).show(ctx, |ui| {
                ui.set_width(380.0);
                ui.heading("Keep your changes?");
                ui.label("This schematic has unsaved changes.");
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if controls::action(ui, "Save draft", controls::ActionTone::Primary, true)
                        .clicked()
                    {
                        self.pending = None;
                        self.after_save = Some(action);
                        self.save(false);
                    }
                    if controls::action(ui, "Discard changes", controls::ActionTone::Danger, true)
                        .clicked()
                    {
                        self.act(action);
                    }
                    if controls::action(ui, "Cancel", controls::ActionTone::Secondary, true)
                        .clicked()
                    {
                        self.pending = None;
                    }
                });
            });
        }
    }
}
impl eframe::App for App {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        // Capture-only input holds a pointer gesture in an inactive proof window.
        if (self.options.capture_selection || self.options.capture_text_placement)
            && self.options.capture.is_some()
        {
            raw.focused = true;
        }
        // Selection uses secondary drag; text-box sizing uses primary drag.
        if (self.options.capture_selection || self.options.capture_text_placement)
            && self.options.capture.is_some()
            && !self.busy()
            && self.frames >= 6
            && self.capture_input_step < 2
        {
            let center = self.layout.bounds.center();
            let pos = center
                + if self.capture_input_step == 0 {
                    egui::vec2(180.0, 130.0)
                } else {
                    egui::vec2(-180.0, -130.0)
                };
            raw.events.push(egui::Event::PointerMoved(pos));
            if self.capture_input_step == 0 {
                raw.events.push(egui::Event::PointerButton {
                    pos,
                    button: if self.options.capture_text_placement {
                        egui::PointerButton::Primary
                    } else {
                        egui::PointerButton::Secondary
                    },
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                });
            }
            self.capture_input_step += 1;
        }
        // Dedicated inactive capture only: exercise the actual pointer/menu path.
        if self.options.capture_wire_menu
            && self.options.capture.is_some()
            && !self.busy()
            && self.frames >= 6
            && self.capture_input_step < 2
            && let Some(pos) = self.capture_pointer.or(self.canvas.capture_wire_hit)
        {
            self.capture_pointer = Some(pos);
            raw.events.push(egui::Event::PointerMoved(pos));
            raw.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Secondary,
                pressed: self.capture_input_step == 0,
                modifiers: egui::Modifiers::NONE,
            });
            self.capture_input_step += 1;
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}
impl App {
    fn show(&mut self, ui: &mut egui::Ui) {
        let t = theme::current(ui.ctx());
        let ctx = ui.ctx().clone();
        let popup_was_open = egui::Popup::is_any_open(&ctx)
            || self.palette
            || self.inspector.draft.is_some()
            || self.nets.modal()
            || self.sheets.dialog.is_some();
        self.poll(&ctx);
        self.poll_mcp();
        self.ensure_sheet();
        self.refresh_nets();
        self.shortcuts(&ctx);
        if ctx.input(|i| i.viewport().close_requested()) && !self.close_allowed {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            if !self.busy() {
                self.request(Action::Quit);
            } else {
                self.notice("Finish the file operation before closing.");
            }
        }
        if self.properties
            && self.inspector.dock == library::Dock::Left
            && self.library
            && self.library_ui.dock == library::Dock::Left
        {
            self.inspector.dock = library::Dock::Floating;
        }
        self.top_bar(ui);
        self.ribbon(ui);
        self.footer(ui);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(t.canvas.background))
            .show(ui, |ui| {
                self.layout = panels::Layout::new(self, ui.max_rect());
                // A click that dismisses a popup belongs to that popup for the whole frame.
                let input_enabled =
                    !popup_was_open && !self.palette && !egui::Popup::is_any_open(&ctx);
                canvas::show(self, ui, input_enabled);
            });
        self.panels(&ctx);
        self.dialogs(&ctx);
        self.ai_panel(&ctx);
        self.more_properties(&ctx);
        self.net_dialogs(&ctx);
        self.sheet_dialog(&ctx);
        if self.close_allowed {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.frames = self.frames.saturating_add(1);
        if self.options.capture.is_some()
            && !self.capture_sent
            && (!self.options.capture_wire_menu || self.canvas.context_open)
            && (!self.options.capture_selection
                || matches!(self.canvas.gesture, Some(interaction::Gesture::Box { .. })))
            && self.frames >= 12
            && self.started.elapsed() >= Duration::from_millis(800)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            self.capture_sent = true;
        }
        if self.options.capture.is_some() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        let screenshot = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = screenshot
            && let Some(path) = self.options.capture.take()
        {
            let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
            match image::save_buffer(
                &path,
                &bytes,
                image.size[0] as u32,
                image.size[1] as u32,
                image::ColorType::Rgba8,
            ) {
                Ok(()) => eprintln!("Captured PCBcraft: {}", path.display()),
                Err(e) => eprintln!("Screenshot failed: {e}"),
            }
            if self.options.exit_after_capture {
                self.close_allowed = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

//! Selection-driven text inspector and paint projection; commits use the Electronics session.
use super::{App, controls, icons::Icon, theme};
use eframe::egui::{self, Align2, Pos2, RichText, Stroke, vec2};
use pcbcraft_foundation::{
    document::*,
    session::Command,
    text::{self, Appearance, Font, TextTarget},
};
#[derive(Default)]
pub(super) struct State {
    pub draft: Option<Draft>,
}
pub(super) struct Draft {
    pub target: TextTarget,
    incarnation: ProjectId,
    revision: u64,
    content: String,
    appearance: Appearance,
    error: Option<String>,
}
impl App {
    pub(super) fn open_text(&mut self, target: TextTarget) {
        let Some(content) = target.content(self.session.document()) else {
            return;
        };
        let appearance = text::appearance(self.session.document(), target);
        self.select(None);
        match target {
            TextTarget::Note(id) => {
                self.selection.notes.insert(id);
            }
            TextTarget::Reference(id) | TextTarget::Value(id) => {
                self.selection.symbols.insert(id);
            }
            TextTarget::Net(id) => {
                self.selection.wires.insert(id);
            }
        }
        self.text_ui.draft = Some(Draft {
            target,
            incarnation: self.session.incarnation,
            revision: self.session.revision(),
            content,
            appearance,
            error: None,
        });
        self.properties = true;
        self.panel_ui.properties_collapsed = false;
    }
    pub(super) fn add_note(&mut self, at: Point) {
        if self.busy() {
            return;
        }
        let result = self.session.apply(
            self.session.revision(),
            Command::AddNote {
                at,
                text: "New text".into(),
            },
        );
        match result {
            Ok(_) => {
                if let Some(note) = self.session.document().annotations.last() {
                    self.open_text(TextTarget::Note(note.id));
                }
            }
            Err(e) => self.notice(e),
        }
    }
    pub(super) fn text_properties(&mut self, ui: &mut egui::Ui, height: f32) {
        let Some(mut d) = self.text_ui.draft.take() else {
            return;
        };
        let t = theme::current(ui.ctx());
        egui::ScrollArea::vertical()
            .id_salt("text_inspector_body")
            .min_scrolled_height(0.0)
            .max_height((height - 100.0).max(30.0))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(Icon::Annotate.image(22.0, ui.ctx()));
                    ui.label(
                        RichText::new(match d.target {
                            TextTarget::Note(_) => "Sheet text",
                            TextTarget::Reference(_) => "Reference text",
                            TextTarget::Value(_) => "Value text",
                            TextTarget::Net(_) => "Net label",
                        })
                        .font(theme::semibold(t.metrics.heading)),
                    );
                });
                controls::section(ui, "CONTENT");
                if matches!(d.target, TextTarget::Note(_)) {
                    ui.add_sized(
                        [ui.available_width(), 68.0],
                        egui::TextEdit::multiline(&mut d.content)
                            .margin(vec2(9.0, 7.0))
                            .desired_rows(3)
                            .char_limit(4096),
                    );
                } else {
                    ui.add_sized(
                        [ui.available_width(), 28.0],
                        controls::text_input(
                            egui::TextEdit::singleline(&mut d.content).char_limit(128),
                        ),
                    );
                }
                controls::section(ui, "APPEARANCE");
                controls::field_row(ui, "Font", |ui| {
                    let name = |font| match font {
                        Font::Sans => "IBM Plex Sans",
                        Font::Semibold => "IBM Plex Semibold",
                        Font::Monospace => "Monospace",
                    };
                    egui::ComboBox::from_id_salt("text_font")
                        .selected_text(name(d.appearance.font))
                        .width(ui.available_width())
                        .show_ui(ui, |ui| {
                            controls::menu_content(ui);
                            for font in [Font::Sans, Font::Semibold, Font::Monospace] {
                                if controls::MenuItem::new(name(font), Icon::Annotate)
                                    .checked(font == d.appearance.font)
                                    .show(ui)
                                    .clicked()
                                {
                                    d.appearance.font = font;
                                    ui.close();
                                }
                            }
                        });
                });
                controls::field_row(ui, "Height", |ui| {
                    let mut mm = d.appearance.height_nm as f64 / 1e6;
                    if ui
                        .add(
                            egui::DragValue::new(&mut mm)
                                .range(0.25..=12.0)
                                .speed(0.05)
                                .suffix(" mm"),
                        )
                        .changed()
                    {
                        d.appearance.height_nm = (mm * 1e6).round() as i64;
                    }
                });
                ui.horizontal(|ui| {
                    controls::checkbox(ui, &mut d.appearance.italic, "Italic");
                    controls::checkbox(ui, &mut d.appearance.underline, "Underline");
                });
                for (name, coordinate) in [
                    ("Offset X", &mut d.appearance.offset.x),
                    ("Offset Y", &mut d.appearance.offset.y),
                ] {
                    controls::field_row(ui, name, |ui| {
                        let mut mm = *coordinate as f64 / 1e6;
                        if ui
                            .add(
                                egui::DragValue::new(&mut mm)
                                    .range(-1000.0..=1000.0)
                                    .speed(0.1)
                                    .suffix(" mm"),
                            )
                            .changed()
                        {
                            *coordinate = (mm * 1e6).round() as i64;
                        }
                    });
                }
                controls::section(ui, "WEB LINK");
                ui.add_sized(
                    [ui.available_width(), 28.0],
                    controls::text_input(
                        egui::TextEdit::singleline(&mut d.appearance.link)
                            .hint_text("https://…")
                            .char_limit(2048),
                    ),
                );
                if !d.appearance.link.is_empty()
                    && controls::action(
                        ui,
                        "Open link ↗",
                        controls::ActionTone::Secondary,
                        d.appearance.validate().is_ok(),
                    )
                    .clicked()
                {
                    ui.ctx()
                        .open_url(egui::OpenUrl::new_tab(&d.appearance.link));
                }
            });
        ui.separator();
        let changed = d.target.content(self.session.document()).as_ref() != Some(&d.content)
            || text::appearance(self.session.document(), d.target) != d.appearance;
        let mut applied = false;
        ui.horizontal(|ui| {
            if controls::action(
                ui,
                "Apply",
                controls::ActionTone::Primary,
                changed && !self.busy(),
            )
            .clicked()
            {
                let result = if d.incarnation != self.session.incarnation {
                    Err("Project changed. Reselect the text.".into())
                } else {
                    self.session.apply(
                        d.revision,
                        Command::SetText {
                            target: d.target,
                            content: d.content.clone(),
                            appearance: d.appearance.clone(),
                        },
                    )
                };
                match result {
                    Ok(_) => applied = true,
                    Err(e) => d.error = Some(e.to_string()),
                }
            }
            if controls::action(ui, "Reset", controls::ActionTone::Secondary, true).clicked() {
                applied = true;
            }
        });
        if let Some(error) = &d.error {
            ui.add(egui::Label::new(RichText::new(error).color(t.ui.danger)).truncate())
                .on_hover_text(error);
        }
        ui.label(
            RichText::new(
                "Click text to edit. Drag it to reposition. Ctrl-click opens a saved web link.",
            )
            .small()
            .color(t.ui.muted),
        );
        if applied {
            self.open_text(d.target);
        } else {
            self.text_ui.draft = Some(d);
        }
    }
}
pub(super) struct Run<'a> {
    pub target: TextTarget,
    pub at: Pos2,
    pub align: Align2,
    pub content: &'a str,
    pub size: f32,
    pub color: egui::Color32,
}
pub(super) fn paint(app: &mut App, doc: &Document, painter: &egui::Painter, run: Run<'_>) {
    let appearance = text::appearance(doc, run.target);
    let overridden = doc.text_overrides.iter().any(|o| o.target == run.target)
        || appearance != run.target.defaults();
    let size = if overridden {
        appearance.height_nm as f32 / 1e6 * 8.0 * app.canvas.zoom
    } else {
        run.size
    };
    let font = match appearance.font {
        Font::Sans => egui::FontId::proportional(size),
        Font::Semibold => theme::semibold(size),
        Font::Monospace => egui::FontId::monospace(size),
    };
    let color = if matches!(run.target,TextTarget::Note(id) if app.selection.notes.contains(&id)) {
        theme::current(painter.ctx()).canvas.selection
    } else {
        run.color
    };
    let mut job = egui::text::LayoutJob::default();
    job.append(
        run.content,
        0.0,
        egui::TextFormat {
            font_id: font,
            color,
            italics: appearance.italic,
            underline: if appearance.underline || !appearance.link.is_empty() {
                Stroke::new(0.8, color)
            } else {
                Stroke::NONE
            },
            ..Default::default()
        },
    );
    let galley = painter.layout_job(job);
    let offset =
        vec2(appearance.offset.x as f32, appearance.offset.y as f32) / 1e6 * 8.0 * app.canvas.zoom;
    let rect = run.align.anchor_size(run.at + offset, galley.size());
    painter.galley(rect.min, galley, color);
    app.canvas.text_hits.push((run.target, rect));
    if app
        .text_ui
        .draft
        .as_ref()
        .is_some_and(|d| d.target == run.target)
    {
        painter.rect_stroke(
            rect.expand(3.0),
            2,
            Stroke::new(0.8, theme::current(painter.ctx()).ui.accent),
            egui::StrokeKind::Outside,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{library_fixture, ribbon_text, workspace_click, workspace_frame};
    use super::*;
    #[test]
    fn compact_text_inspector_applies_once_and_rejects_stale_or_reopened_drafts() -> Result<()> {
        let (mut app, ctx) = library_fixture()?;
        app.library = false;
        let target = TextTarget::Note(app.session.document().annotations[0].id);
        app.open_text(target);
        let before = app.session.document().clone();
        let revision = app.session.revision();
        let draft = app.text_ui.draft.as_mut().ok_or("Missing draft")?;
        draft.content = "Power stage\nDocumentation".into();
        draft.appearance.height_nm = 3_500_000;
        draft.appearance.font = Font::Semibold;
        draft.appearance.link = "https://example.com/".into();
        for _ in 0..3 {
            workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
        }
        let output = workspace_frame(&mut app, &ctx, vec![]);
        let (apply, clip) = ribbon_text(&output, "Apply")?;
        assert!(
            clip.contains_rect(apply),
            "Apply must remain visible in compact window"
        );
        output.drop_without_applying_deltas();
        workspace_click(&mut app, &ctx, apply.center());
        assert_eq!(app.session.revision(), revision + 1);
        assert_eq!(
            target.content(app.session.document()).as_deref(),
            Some("Power stage\nDocumentation")
        );
        app.session.undo()?;
        assert_eq!(app.session.document(), &before);
        // The still-open inspector is stamped before undo and cannot overwrite it.
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
        workspace_click(&mut app, &ctx, apply.center());
        assert_eq!(app.session.document(), &before);
        assert!(
            app.text_ui
                .draft
                .as_ref()
                .and_then(|d| d.error.as_ref())
                .is_some()
        );
        app.open_text(target);
        app.text_ui.draft.as_mut().ok_or("No draft")?.content = "Old window".into();
        app.session = pcbcraft_foundation::session::Session::new(before.clone())?;
        workspace_frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
        workspace_click(&mut app, &ctx, apply.center());
        assert_eq!(app.session.document(), &before);
        assert!(
            app.text_ui
                .draft
                .as_ref()
                .and_then(|d| d.error.as_ref())
                .is_some()
        );
        Ok(())
    }
}

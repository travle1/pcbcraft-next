//! Uncommitted cursor previews. Only a canvas click creates document objects.
use super::*;
use eframe::egui::{Align2, FontId, Pos2, Rect, Stroke};

pub(super) struct Placement {
    draft: Dialog,
    base_name: String,
}
impl Dialog {
    fn placement_command(&self, at: Point) -> project::Command {
        match self.kind {
            Kind::Create { parent } => project::Command::Create {
                name: self.name.trim().into(),
                parent,
                existing: self.existing,
                at,
            },
            _ => project::Command::SetPort {
                occurrence: self.occurrence,
                id: None,
                name: self.name.trim().into(),
                direction: self.direction,
                at,
            },
        }
    }
}
impl App {
    pub(super) fn begin_sheet_placement(&mut self, draft: &Dialog) -> Result<()> {
        if draft.incarnation != self.session.incarnation
            || draft.revision != self.session.revision()
            || draft.occurrence != self.active_sheet()
        {
            return Err("Project or sheet changed. Review placement again.".into());
        }
        // Validate names, definition reuse and hierarchy before entering the tool.
        // Preview operates on a candidate copy, never on the authoritative project.
        self.session
            .project()
            .preview(draft.placement_command(draft.at))?;
        self.sheets.placement = Some(Placement {
            draft: draft.clone(),
            base_name: draft.name.trim().into(),
        });
        self.tool = Tool::SheetPlacement;
        self.canvas.wire_start = None;
        self.canvas.drag = None;
        self.select(None);
        Ok(())
    }
    pub(in crate::desktop) fn paint_sheet_placement(
        &mut self,
        ui: &egui::Ui,
        canvas: Rect,
        pointer: Option<Pos2>,
    ) {
        if self.tool != Tool::SheetPlacement {
            self.sheets.placement = None;
            return;
        }
        let Some(placement) = &self.sheets.placement else {
            self.tool = Tool::Select;
            return;
        };
        let d = &placement.draft;
        if d.incarnation != self.session.incarnation
            || d.revision != self.session.revision()
            || d.occurrence != self.active_sheet()
        {
            self.sheets.placement = None;
            self.tool = Tool::Select;
            self.notice("Project or sheet changed. Choose the placement tool again.");
            return;
        }
        if pointer.is_none() {
            return;
        }
        let at = self.canvas.last_world;
        let pos = self.canvas.screen(at, canvas);
        let t = theme::current(ui.ctx());
        let p = ui.painter();
        let stroke = Stroke::new(1.2, t.canvas.selection);
        if matches!(d.kind, Kind::Create { .. }) {
            let ports = d
                .existing
                .and_then(|id| self.session.project().definition(id))
                .map(|doc| doc.ports.as_slice())
                .unwrap_or_default();
            let bottom = Point {
                x: at.x + 24 * GRID_NM,
                y: at.y + (5 + ports.len() as i64 * 3) * GRID_NM,
            };
            p.rect(
                Rect::from_min_max(pos, self.canvas.screen(bottom, canvas)),
                3,
                t.ui.surface.gamma_multiply(0.75),
                stroke,
                egui::StrokeKind::Inside,
            );
            p.text(
                pos - egui::vec2(0.0, 4.0),
                Align2::LEFT_BOTTOM,
                &d.name,
                FontId::proportional((13.0 * self.canvas.zoom).max(9.0)),
                t.canvas.selection,
            );
            for (i, port) in ports.iter().enumerate() {
                let pin = Point {
                    x: at.x
                        + if port.direction == Direction::Output {
                            24 * GRID_NM
                        } else {
                            0
                        },
                    y: at.y + (3 + i as i64 * 3) * GRID_NM,
                };
                p.circle_stroke(self.canvas.screen(pin, canvas), 2.5, stroke);
            }
        } else {
            p.circle_stroke(pos, 3.0, stroke);
            p.line_segment([pos, pos + egui::vec2(7.0, 0.0)], stroke);
            p.text(
                pos + egui::vec2(9.0, 0.0),
                Align2::LEFT_CENTER,
                &d.name,
                FontId::proportional((12.0 * self.canvas.zoom).max(8.0)),
                t.canvas.selection,
            );
        }
        ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    }
    pub(in crate::desktop) fn commit_sheet_placement(&mut self, at: Point) {
        let Some(mut placement) = self.sheets.placement.take() else {
            return;
        };
        let d = &placement.draft;
        if self.busy()
            || self.pending.is_some()
            || d.incarnation != self.session.incarnation
            || d.occurrence != self.active_sheet()
        {
            self.tool = Tool::Select;
            return;
        }
        match self
            .session
            .project_command(d.revision, d.placement_command(at))
        {
            Ok(effect) => {
                self.select(None);
                self.sheets.selected = if let Some(id) = effect.created {
                    self.panel_ui.sheets_folded = false;
                    self.panel_ui.project_folded = false;
                    self.sheets.folded.remove(&self.active_sheet());
                    self.session
                        .project()
                        .occurrence(id)
                        .and_then(|o| o.via)
                        .map(Object::Block)
                } else {
                    self.document().ports.last().map(|p| Object::Port(p.id))
                };
                self.properties = true;
                self.nets = Default::default();
                placement.draft.revision = self.session.revision();
                // Repeated clicks create distinct names. Existing-definition
                // selection remains explicit; fresh sheets remain independent.
                let mut base = placement.base_name.clone();
                while base.len() > 72 {
                    base.pop();
                }
                let next = (2..=512).map(|n| format!("{base}_{n}")).find(|name| {
                    match placement.draft.kind {
                        Kind::Create { parent } => !self
                            .session
                            .project()
                            .occurrences
                            .iter()
                            .any(|o| o.parent == parent && o.name.eq_ignore_ascii_case(name)),
                        _ => !self
                            .document()
                            .ports
                            .iter()
                            .any(|p| p.name.eq_ignore_ascii_case(name)),
                    }
                });
                if let Some(name) = next {
                    placement.draft.name = name;
                    self.sheets.placement = Some(placement);
                } else {
                    self.tool = Tool::Select;
                    self.notice("Placement finished. Choose a new name for more objects.");
                }
            }
            Err(e) => {
                self.tool = Tool::Select;
                self.notice(e);
            }
        }
    }
}

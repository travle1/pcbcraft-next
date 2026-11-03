//! Bounded adaptations of SolidCraft ui_kit panel/fields/layout/menu/actions, c553365a2197a689.
//! Copyright (c) 2026 SolidCraft contributors. MIT OR Apache-2.0.
//! Retained notices: assets/family/. Scope: docs/panel-controls-provenance.md.
use super::icons::Icon;
use super::theme;
use eframe::egui::{self, Align2, Rect, RichText, Sense, Stroke, vec2};

/// Shared popup surface. Context, ribbon, File and panel menus use the same frame.
pub(super) fn menu_frame(ctx: &egui::Context) -> egui::Frame {
    let t = theme::current(ctx);
    egui::Frame::NONE
        .fill(t.ui.surface)
        .stroke(Stroke::new(1.0, t.ui.border))
        .corner_radius(t.metrics.radius.saturating_add(4))
        .inner_margin(6)
        .shadow(egui::Shadow {
            offset: [0, 14],
            blur: 36,
            spread: 0,
            color: t.ui.chrome.gamma_multiply(0.18),
        })
}

pub(super) fn menu_content(ui: &mut egui::Ui) {
    let t = theme::current(ui.ctx());
    // Fixed columns, scaled with type. No padding a label with spaces for shortcuts.
    ui.set_width((236.0 * t.metrics.body / 12.0).max(236.0));
    ui.spacing_mut().item_spacing.y = 0.0;
}

pub(super) fn menu_button<R>(
    ui: &mut egui::Ui,
    label: &str,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<Option<R>> {
    let previous = ui.style().clone();
    let frame = menu_frame(ui.ctx());
    let style = ui.style_mut();
    style.spacing.menu_margin = frame.inner_margin;
    style.visuals.menu_corner_radius = frame.corner_radius;
    style.visuals.popup_shadow = frame.shadow;
    style.visuals.window_fill = frame.fill;
    style.visuals.window_stroke = frame.stroke;
    let response = ui.menu_button(label, |ui| {
        menu_content(ui);
        add(ui)
    });
    ui.set_style(previous);
    response
}

pub(super) fn menu_separator(ui: &mut egui::Ui) {
    let t = theme::current(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 11.0), Sense::hover());
    ui.painter()
        .hline(r.x_range(), r.center().y, Stroke::new(1.0, t.ui.border));
}

/// Licensed, bounded adaptation of SolidCraft's MenuItem. Dispatch belongs to callers.
pub(super) struct MenuItem<'a> {
    label: &'a str,
    icon: Option<Icon>,
    shortcut: &'a str,
    enabled: bool,
    danger: bool,
    checked: Option<bool>,
}
impl<'a> MenuItem<'a> {
    pub fn new(label: &'a str, icon: Icon) -> Self {
        Self {
            label,
            icon: Some(icon),
            shortcut: "",
            enabled: true,
            danger: false,
            checked: None,
        }
    }
    pub fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = shortcut;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }
    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        ui.add_enabled_ui(self.enabled, |ui| self.row(ui)).inner
    }
    fn row(self, ui: &mut egui::Ui) -> egui::Response {
        let t = theme::current(ui.ctx());
        let enabled = self.enabled && ui.is_enabled();
        let height = (t.metrics.control_height + 6.0).max(t.metrics.body + 18.0);
        let (r, response) = ui.allocate_exact_size(
            vec2(ui.available_width(), height),
            if enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        let hot = enabled && (response.hovered() || response.has_focus());
        let ink = if self.danger {
            t.ui.danger
        } else if hot {
            t.ui.accent
        } else {
            t.ui.text
        };
        let dim = if self.danger {
            t.ui.danger
        } else if hot {
            t.ui.accent
        } else {
            t.ui.muted
        };
        let p = ui.painter().with_clip_rect(ui.clip_rect().intersect(r));
        if hot {
            p.rect_filled(r, t.metrics.radius, t.ui.selection);
        }
        let icon_rect = Rect::from_center_size(r.left_center() + vec2(14.0, 0.0), vec2(16.0, 16.0));
        if let Some(checked) = self.checked {
            if checked {
                let c = icon_rect.center();
                p.add(egui::Shape::line(
                    vec![
                        c + vec2(-5.0, 0.0),
                        c + vec2(-1.0, 4.0),
                        c + vec2(6.0, -4.0),
                    ],
                    Stroke::new(1.5, t.ui.accent),
                ));
            }
        } else if let Some(icon) = self.icon {
            let image = if self.danger {
                icon.danger_image(16.0, ui.ctx())
            } else {
                icon.image(16.0, ui.ctx())
            };
            image.paint_at(ui, icon_rect);
        }
        let mut right = r.right() - 8.0;
        if !self.shortcut.is_empty() {
            let key = p.layout_no_wrap(
                self.shortcut.into(),
                egui::FontId::monospace(t.metrics.small),
                dim,
            );
            right -= key.size().x;
            p.galley(
                egui::pos2(right, r.center().y - key.size().y / 2.0),
                key,
                dim,
            );
            right -= 8.0;
        }
        let left = r.left() + 30.0;
        let mut job = egui::text::LayoutJob::simple_singleline(
            self.label.into(),
            egui::FontId::proportional(t.metrics.body + 0.5),
            ink,
        );
        job.wrap.max_width = (right - left).max(1.0);
        job.wrap.max_rows = 1;
        let galley = p.layout_job(job);
        p.galley(
            egui::pos2(left, r.center().y - galley.size().y / 2.0),
            galley,
            ink,
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, self.label)
        });
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    }
}

#[derive(Clone, Copy)]
pub(super) enum ActionTone {
    Primary,
    Secondary,
    Danger,
}

/// SolidCraft checks.rs geometry and state treatment, without its preferences registry.
pub(super) fn checkbox(ui: &mut egui::Ui, value: &mut bool, label: &str) -> egui::Response {
    let t = theme::current(ui.ctx());
    let font = egui::FontId::proportional(t.metrics.body);
    let text = ui.painter().layout_no_wrap(label.into(), font, t.ui.text);
    let size = vec2(
        20.0 + text.size().x,
        t.metrics.control_height.max(text.size().y),
    );
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    let ppp = ui.ctx().pixels_per_point();
    let origin = egui::pos2(rect.left(), rect.center().y - 7.0);
    let square = Rect::from_min_size(
        egui::pos2(
            (origin.x * ppp).round() / ppp,
            (origin.y * ppp).round() / ppp,
        ),
        vec2(14.0, 14.0),
    );
    let hot = response.hovered() || response.has_focus();
    ui.painter().rect(
        square,
        3,
        if *value {
            t.ui.accent
        } else if hot {
            t.ui.hover
        } else {
            t.ui.field
        },
        Stroke::new(
            1.0,
            if *value || hot {
                t.ui.accent
            } else {
                t.ui.border
            },
        ),
        egui::StrokeKind::Inside,
    );
    if *value {
        let mark = square.shrink(3.0);
        ui.painter().add(egui::Shape::line(
            vec![mark.left_center(), mark.center_bottom(), mark.right_top()],
            Stroke::new(2.0, t.ui.on_accent),
        ));
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            square.expand(2.0),
            3,
            Stroke::new(1.0, t.ui.accent),
            egui::StrokeKind::Outside,
        );
    }
    ui.painter()
        .with_clip_rect(ui.clip_rect().intersect(rect))
        .galley(
            egui::pos2(square.right() + 6.0, rect.center().y - text.size().y / 2.0),
            text,
            t.ui.text,
        );
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *value, label)
    });
    response
}

/// Shared form actions: accent for commit, quiet secondary, red for destructive actions.
/// Like upstream actions.rs, interaction shades derive from semantic theme colors.
pub(super) fn action(
    ui: &mut egui::Ui,
    label: &str,
    tone: ActionTone,
    enabled: bool,
) -> egui::Response {
    let t = theme::current(ui.ctx());
    ui.scope(|ui| {
        let (fill, ink) = match tone {
            ActionTone::Primary => (t.ui.accent, t.ui.on_accent),
            ActionTone::Secondary => (t.ui.field, t.ui.text),
            ActionTone::Danger => (t.ui.field, t.ui.danger),
        };
        {
            let state = &mut ui.style_mut().visuals.widgets.inactive;
            state.bg_fill = fill;
            state.weak_bg_fill = state.bg_fill;
            state.fg_stroke = Stroke::new(1.0, ink);
            state.bg_stroke = Stroke::new(
                1.0,
                if matches!(tone, ActionTone::Primary) {
                    fill
                } else {
                    t.ui.border
                },
            );
        }
        let visuals = &mut ui.style_mut().visuals;
        for (amount, state) in [
            (0.12, &mut visuals.widgets.hovered),
            (0.24, &mut visuals.widgets.active),
        ] {
            state.bg_fill = fill.lerp_to_gamma(t.ui.text, amount);
            state.weak_bg_fill = state.bg_fill;
            state.fg_stroke = Stroke::new(1.0, ink);
            state.bg_stroke = Stroke::new(
                1.0,
                if matches!(tone, ActionTone::Danger) {
                    t.ui.danger
                } else {
                    t.ui.accent
                },
            );
        }
        ui.visuals_mut().override_text_color = Some(ink);
        ui.add_enabled(
            enabled,
            egui::Button::new(label).min_size(vec2(72.0, t.metrics.control_height.max(28.0))),
        )
    })
    .inner
}

pub(super) fn card<R>(
    ctx: &egui::Context,
    id: &'static str,
    rect: Rect,
    order: egui::Order,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Area::new(id.into())
        .fixed_pos(rect.min)
        .movable(false)
        .order(order)
        .fade_in(false)
        .show(ctx, |ui| {
            let frame = theme::current(ctx)
                .panel_frame()
                .inner_margin(10)
                .corner_radius(12);
            frame
                .show(ui, |ui| {
                    // Allocate the host once. A child may scroll, but cannot grow the card.
                    let size = (rect.size() - frame.total_margin().sum()).max(egui::Vec2::ZERO);
                    let (bounds, _) = ui.allocate_exact_size(size, Sense::hover());
                    let mut child =
                        ui.new_child(egui::UiBuilder::new().max_rect(bounds).id_salt("card_body"));
                    child.set_clip_rect(ui.clip_rect().intersect(bounds));
                    body(&mut child)
                })
                .inner
        })
        .inner
}

pub(super) fn text_input(edit: egui::TextEdit<'_>) -> egui::TextEdit<'_> {
    edit.vertical_align(egui::Align::Center)
        .margin(vec2(9.0, 0.0))
        .min_size(vec2(0.0, 28.0))
}

pub(super) fn catalog_footer(ui: &mut egui::Ui, text: &str) {
    let t = theme::current(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::hover());
    ui.painter()
        .hline(rect.x_range(), rect.top(), Stroke::new(1.0, t.ui.border));
    let mut job = egui::text::LayoutJob::simple(
        text.to_owned(),
        egui::FontId::proportional(t.metrics.small),
        t.ui.muted,
        rect.width().max(1.0),
    );
    job.wrap.max_rows = 1;
    let galley = ui.painter().layout_job(job);
    ui.painter()
        .galley(rect.min + vec2(0.0, 8.0), galley, t.ui.muted);
    response.on_hover_text(text);
}

pub(super) fn field_row(ui: &mut egui::Ui, label: &str, field: impl FnOnce(&mut egui::Ui)) {
    let t = theme::current(ui.ctx());
    ui.horizontal(|ui| {
        ui.add_sized(
            [70.0, 28.0],
            egui::Label::new(RichText::new(label).color(t.ui.muted)),
        );
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), field);
    });
}

pub(super) fn grip(ui: &egui::Ui, rect: Rect) {
    let t = theme::current(ui.ctx());
    for x in [4.0, 8.0] {
        for y in [-4.0, 0.0, 4.0] {
            ui.painter()
                .circle_filled(rect.left_center() + vec2(x, y), 1.0, t.ui.muted);
        }
    }
}

pub(super) fn section(ui: &mut egui::Ui, label: &str) {
    let t = theme::current(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        theme::semibold(t.metrics.small),
        t.ui.muted,
    );
    let width = galley.size().x;
    ui.painter().galley(
        Align2::LEFT_CENTER
            .anchor_size(r.left_center(), galley.size())
            .min,
        galley,
        t.ui.muted,
    );
    ui.painter().hline(
        r.left() + width + 12.0..=r.right(),
        r.center().y,
        Stroke::new(1.0, t.ui.border),
    );
}

/// Selection changes ink/weight only. The row and icon never change their insets.
pub(super) fn tree_row(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    tree_row_icon(ui, label, selected, super::icons::Icon::Symbol)
}
pub(super) fn tree_row_icon(
    ui: &mut egui::Ui,
    label: &str,
    selected: bool,
    icon: super::icons::Icon,
) -> egui::Response {
    let t = theme::current(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 27.0), Sense::click());
    if selected || response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, 4, if selected { t.ui.selection } else { t.ui.hover });
    }
    icon.image(15.0, ui.ctx()).paint_at(
        ui,
        Rect::from_center_size(rect.left_center() + vec2(13.0, 0.0), vec2(15.0, 15.0)),
    );
    let font = if selected {
        theme::semibold(t.metrics.body)
    } else {
        egui::FontId::proportional(t.metrics.body)
    };
    let mut job = egui::text::LayoutJob::simple(
        label.to_owned(),
        font,
        t.ui.text,
        (rect.width() - 34.0).max(1.0),
    );
    job.wrap.max_rows = 1;
    let text = ui.painter().layout_job(job);
    ui.painter().galley(
        rect.left_center() + vec2(28.0, -text.size().y * 0.5),
        text,
        t.ui.text,
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            label,
        )
    });
    response.on_hover_text(label)
}

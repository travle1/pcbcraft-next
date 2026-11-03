//! Workspace picker adapted from SolidCraft's `ui_kit::workspace_menu` at c553365.
//! Copyright (c) 2026 SolidCraft contributors. MIT OR Apache-2.0.
//! See docs/workspace-picker-provenance.md and assets/family/SOLIDCRAFT-LICENSE-*.
//! PCBcraft owns the entry data and lifecycle; this widget owns no document state.
use super::{icons::Icon, theme};
use eframe::egui::{self, Align2, Color32, FontId, Rect, RichText, Sense, pos2, vec2};

pub(super) fn popup_id() -> egui::Id {
    egui::Id::new("pcbcraft_workspace_picker")
}

struct Entry {
    title: &'static str,
    description: &'static str,
    tabs: Option<&'static str>,
    icon: Icon,
    current: bool,
}
const ENTRIES: [Entry; 4] = [
    Entry {
        title: "Schematic",
        description: "Capture circuits, place symbols and connect pins",
        tabs: Some("Capture · Analyze · Libraries · Manage · View"),
        icon: Icon::Workspace,
        current: true,
    },
    Entry {
        title: "PCB layout",
        description: "Place components, route copper and define the board",
        tabs: None,
        icon: Icon::Layout,
        current: false,
    },
    Entry {
        title: "3D render",
        description: "Inspect the assembled board, layers and components",
        tabs: None,
        icon: Icon::Render,
        current: false,
    },
    Entry {
        title: "Simulate",
        description: "Set up circuit models, analyses and waveform probes",
        tabs: None,
        icon: Icon::Simulate,
        current: false,
    },
];

pub(super) fn picker(ui: &mut egui::Ui, card: Rect) {
    let t = theme::current(ui.ctx());
    let response = ui.interact(card, ui.id().with("workspace_switcher"), Sense::click());
    let open = egui::Popup::is_id_open(ui.ctx(), popup_id());
    if response.hovered() || response.has_focus() || open {
        ui.painter()
            .rect_filled(card.shrink(1.0), t.metrics.radius, t.ui.hover);
    }
    ui.painter().rect_filled(
        Rect::from_min_max(
            card.min + vec2(10.0, 0.0),
            pos2(card.right() - 10.0, card.top() + 3.0),
        ),
        0,
        t.ui.accent,
    );
    let content = card.shrink(t.metrics.ribbon().card_inset);
    Icon::Workspace.image(30.0, ui.ctx()).paint_at(
        ui,
        Rect::from_center_size(
            pos2(content.center().x, content.top() + content.height() * 0.33),
            vec2(30.0, 30.0),
        ),
    );
    ui.painter().text(
        pos2(content.center().x, content.top() + content.height() * 0.60),
        Align2::CENTER_CENTER,
        "Schematic",
        theme::semibold(t.metrics.body),
        t.ui.text,
    );
    let label_y = content.bottom() - t.metrics.ribbon().tabs_height * 0.5 - 10.0;
    let label = ui.painter().layout_no_wrap(
        "Workspace".into(),
        FontId::proportional(t.metrics.small),
        t.ui.muted,
    );
    let label_center = pos2(content.center().x - 5.0, label_y);
    let arrow = pos2(label_center.x + label.size().x * 0.5 + 7.0, label_y);
    ui.painter()
        .galley(label_center - label.size() * 0.5, label, t.ui.muted);
    let direction = if open { -1.0 } else { 1.0 };
    ui.painter().add(egui::Shape::line(
        vec![
            arrow + vec2(-3.0, -1.5 * direction),
            arrow + vec2(0.0, 1.5 * direction),
            arrow + vec2(3.0, -1.5 * direction),
        ],
        egui::Stroke::new(1.2, t.ui.muted),
    ));
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Choose workspace")
    });
    let response = response.on_hover_text("Choose workspace");
    let scale = (t.metrics.body / 12.0).max(1.0);
    let available_height = (ui.ctx().content_rect().bottom() - card.bottom() - 4.0).max(160.0);
    let width = (450.0 * scale).min(ui.ctx().content_rect().width() - 24.0);
    egui::Popup::from_toggle_button_response(&response)
        .id(popup_id())
        .kind(egui::PopupKind::Menu)
        .align(egui::RectAlign::BOTTOM_START)
        .align_alternatives(&[])
        .gap(0.0)
        .width(width)
        .layout(egui::Layout::top_down(egui::Align::Min))
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .frame(
            egui::Frame::new()
                .fill(t.ui.surface)
                .stroke(egui::Stroke::new(1.0, t.ui.border))
                .corner_radius(t.metrics.radius + 4)
                .inner_margin(9),
        )
        .show(|ui| menu(ui, width, available_height, scale));
}

fn menu(ui: &mut egui::Ui, width: f32, height: f32, scale: f32) {
    let t = theme::current(ui.ctx());
    ui.set_width(width);
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.add_space(4.0 * scale);
    ui.label(
        RichText::new("WORKSPACE")
            .font(theme::semibold(t.metrics.small))
            .color(t.ui.muted),
    );
    ui.add_space(8.0 * scale);
    let navigate = ui.input_mut(|i| {
        i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)
            || i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
    });
    // Schematic is the sole available destination. Planned entries are never
    // keyboard targets; menu keys must not leak into the underlying editor.
    let activate_current = ui.input_mut(|i| {
        i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
            || i.consume_key(egui::Modifiers::NONE, egui::Key::Space)
    });
    egui::ScrollArea::vertical()
        .id_salt("workspace_rows")
        .max_height((height - 110.0 * scale).max(60.0))
        .show(ui, |ui| {
            for entry in &ENTRIES {
                let response = row(ui, entry, width, scale);
                if entry.current {
                    if navigate {
                        response.request_focus();
                    }
                    if response.clicked() || activate_current {
                        ui.close();
                    }
                }
            }
        });
    ui.add_space(6.0 * scale);
    ui.separator();
    ui.add_space(6.0 * scale);
    ui.add(egui::Label::new(RichText::new(
        "One electronics project across workspaces. PCB layout, 3D rendering and simulation are planned, not implemented."
    ).size(t.metrics.small).color(t.ui.muted)).wrap());
    ui.add_space(4.0 * scale);
}

fn row(ui: &mut egui::Ui, entry: &Entry, width: f32, scale: f32) -> egui::Response {
    let t = theme::current(ui.ctx());
    let current = entry.current;
    let accent = if current { t.ui.accent } else { t.ui.muted };
    let soft = t.ui.surface.lerp_to_gamma(accent, 0.10);
    let (rect, response) = ui.allocate_exact_size(
        vec2(width, if current { 72.0 } else { 60.0 } * scale),
        if current {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    if current {
        painter.rect_filled(rect, t.metrics.radius, soft);
        if response.hovered() || response.has_focus() {
            painter.rect_stroke(
                rect.shrink(0.5),
                t.metrics.radius,
                egui::Stroke::new(1.0, accent),
                egui::StrokeKind::Inside,
            );
        }
    }
    painter.rect_filled(
        Rect::from_min_max(
            rect.min + vec2(2.0, 7.0) * scale,
            pos2(rect.left() + 6.0 * scale, rect.bottom() - 7.0 * scale),
        ),
        2,
        accent,
    );
    let tile = Rect::from_center_size(
        pos2(rect.left() + 40.0 * scale, rect.center().y),
        vec2(44.0, 44.0) * scale,
    );
    painter.rect_filled(
        tile,
        t.metrics.radius + 3,
        if current { soft } else { t.ui.raised },
    );
    entry
        .icon
        .image(30.0 * scale, ui.ctx())
        .tint(Color32::from_white_alpha(if current { 255 } else { 89 }))
        .paint_at(ui, tile.shrink(7.0 * scale));
    let text = if current { t.ui.text } else { t.ui.muted };
    let x = rect.left() + 74.0 * scale;
    let y = rect.top() + 17.0 * scale;
    let title =
        painter.layout_no_wrap(entry.title.into(), theme::semibold(t.metrics.heading), text);
    let badge_x = x + title.size().x + 9.0 * scale;
    painter.galley(pos2(x, y - title.size().y * 0.5), title, text);
    let badge_ink = if current { t.ui.on_accent } else { accent };
    let badge = painter.layout_no_wrap(
        if current { "Current" } else { "Planned" }.into(),
        FontId::proportional(t.metrics.small),
        badge_ink,
    );
    let badge_rect = Rect::from_min_size(
        pos2(badge_x, y - 8.0 * scale),
        badge.size() + vec2(12.0, 3.0) * scale,
    );
    painter.rect_filled(
        badge_rect,
        t.metrics.radius + 2,
        if current { accent } else { soft },
    );
    painter.galley(badge_rect.min + vec2(6.0, 1.5) * scale, badge, badge_ink);
    let line = |content: &str, y: f32| {
        let mut job = egui::text::LayoutJob::simple_singleline(
            content.into(),
            FontId::proportional(t.metrics.body),
            text,
        );
        job.wrap.max_width = (rect.right() - x - 6.0 * scale).max(0.0);
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = false;
        painter.galley(pos2(x, y), painter.layout_job(job), text);
    };
    line(entry.description, rect.top() + 29.0 * scale);
    if let Some(tabs) = entry.tabs {
        line(tabs, rect.top() + 47.0 * scale);
    }
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, current, entry.title));
    response
}

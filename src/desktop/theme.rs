//! PCBcraft presentation boundary. Palette/metrics map verified SolidCraft data;
//! see docs/theme-foundation.md and the retained family notices. No host dependency.
use eframe::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
};

#[derive(Clone, Copy, Debug)]
pub struct UiColors {
    pub chrome: Color32,
    pub surface: Color32,
    pub raised: Color32,
    pub field: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub disabled: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub selection: Color32,
    pub hover: Color32,
    pub warning: Color32,
    pub danger: Color32,
}
#[derive(Clone, Copy, Debug)]
pub struct CanvasColors {
    pub background: Color32,
    pub grid: Color32,
    pub grid_major: Color32,
    pub symbol: Color32,
    pub wire: Color32,
    pub reference: Color32,
    pub value: Color32,
    pub selection: Color32,
}
#[derive(Clone, Copy, Debug)]
pub struct SchematicMetrics {
    pub wire_width: f32,
    pub symbol_width: f32,
    pub reference_size: f32,
    pub value_size: f32,
    pub pin_size: f32,
    pub grid_min_spacing: f32,
}
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct IconColors {
    pub line: Color32,
    pub ink: Color32,
    pub passive: [Color32; 3],
    pub active: [Color32; 3],
    pub guide: Color32,
}
#[derive(Clone, Copy, Debug)]
pub struct Metrics {
    pub radius: u8,
    pub panel_margin: i8,
    pub body: f32,
    pub small: f32,
    pub heading: f32,
    pub control_height: f32,
    pub gap: f32,
    pub app_bar_height: f32,
    pub ribbon_min_height: f32,
}
impl Metrics {
    pub fn ribbon(self) -> RibbonMetrics {
        RibbonMetrics {
            outer_margin: 6.0,
            card_inset: 4.0,
            workspace_width: (90.0 * self.body / 12.0).max(90.0),
            tabs_height: (self.control_height - 2.0).max(self.body + 10.0),
            header_gap: 5.0,
            tool_height: (24.0 + self.body + 16.0).max(52.0),
            tool_width: (56.0 * self.body / 12.0).max(56.0),
            caption_height: self.small + 4.0,
            caption_gap: 10.0,
            scroll_gutter: 5.0,
            min_height: self.ribbon_min_height,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct RibbonMetrics {
    pub outer_margin: f32,
    pub card_inset: f32,
    pub workspace_width: f32,
    pub tabs_height: f32,
    pub header_gap: f32,
    pub tool_height: f32,
    pub tool_width: f32,
    pub caption_height: f32,
    pub caption_gap: f32,
    pub scroll_gutter: f32,
    min_height: f32,
}
impl RibbonMetrics {
    pub fn group_height(self) -> f32 {
        self.tool_height + self.caption_gap + self.caption_height
    }
    pub fn height(self) -> f32 {
        (2.0 * (self.outer_margin + self.card_inset)
            + self.tabs_height
            + self.header_gap
            + self.group_height()
            + self.scroll_gutter)
            .ceil()
            .max(self.min_height)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub base: egui::Theme,
    pub ui: UiColors,
    pub canvas: CanvasColors,
    pub icons: IconColors,
    pub metrics: Metrics,
    pub schematic: SchematicMetrics,
}
impl Default for Theme {
    fn default() -> Self {
        let rgb = Color32::from_rgb;
        Self {
            base: egui::Theme::Dark,
            // SolidCraft c553365: assets/styles/ui/palettes.rs, dark palette.
            ui: UiColors {
                chrome: rgb(28, 31, 38),
                surface: rgb(39, 43, 52),
                raised: rgb(47, 52, 62),
                field: rgb(30, 34, 41),
                border: rgb(63, 69, 82),
                text: rgb(222, 226, 232),
                muted: rgb(148, 155, 168),
                disabled: rgb(110, 118, 132),
                accent: rgb(58, 136, 238),
                on_accent: Color32::WHITE,
                selection: rgb(42, 70, 110),
                hover: rgb(54, 61, 74),
                warning: rgb(236, 170, 70),
                danger: rgb(240, 100, 96),
            },
            // Original EDA roles: electrical meaning is independent of UI accent.
            canvas: CanvasColors {
                background: rgb(46, 52, 64),
                grid: rgb(51, 57, 68),
                grid_major: rgb(56, 62, 73),
                symbol: rgb(143, 176, 205),
                wire: rgb(119, 154, 141),
                reference: rgb(222, 226, 232),
                value: rgb(196, 183, 139),
                selection: rgb(92, 160, 255),
            },
            // Stable SVG source roles resolved through the upstream dark icon palette.
            icons: IconColors {
                line: rgb(219, 228, 238),
                ink: rgb(31, 32, 33),
                passive: [rgb(141, 163, 195), rgb(109, 129, 162), rgb(84, 103, 130)],
                active: [rgb(78, 161, 220), rgb(50, 135, 209), rgb(41, 118, 184)],
                guide: rgb(128, 147, 164),
            },
            metrics: Metrics {
                radius: 6,
                panel_margin: 10,
                body: 12.0,
                small: 11.0,
                heading: 14.0,
                control_height: 24.0,
                gap: 6.0,
                app_bar_height: 32.0,
                ribbon_min_height: 128.0,
            },
            schematic: SchematicMetrics {
                wire_width: 1.0,
                symbol_width: 1.1,
                reference_size: 11.0,
                value_size: 10.0,
                pin_size: 9.0,
                grid_min_spacing: 14.0,
            },
        }
    }
}
impl Theme {
    pub fn panel_frame(self) -> egui::Frame {
        egui::Frame::new()
            .fill(self.ui.surface)
            .corner_radius(self.metrics.radius)
            .inner_margin(self.metrics.panel_margin)
            .stroke(Stroke::new(1.0, self.ui.border))
    }
}
fn key() -> egui::Id {
    egui::Id::new("pcbcraft.presentation.theme")
}
/// Theme state is per egui context and never part of an electronics document.
pub fn current(ctx: &egui::Context) -> Theme {
    ctx.data(|data| data.get_temp::<Theme>(key()))
        .unwrap_or_default()
}
/// Quiet app/navigation controls retain hover and selection feedback.
pub fn quiet_buttons(ui: &mut egui::Ui) {
    ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
}
/// The future preferences/theme loader uses this same publication path.
/// Call between frames. Icon cache keys derive from resolved palette values.
pub fn set(ctx: &egui::Context, theme: Theme) {
    let c = theme.ui;
    let m = theme.metrics;
    ctx.set_theme(match theme.base {
        egui::Theme::Dark => egui::ThemePreference::Dark,
        egui::Theme::Light => egui::ThemePreference::Light,
    });
    let mut style = (*ctx.style_of(theme.base)).clone();
    style.visuals = match theme.base {
        egui::Theme::Dark => egui::Visuals::dark(),
        egui::Theme::Light => egui::Visuals::light(),
    };
    style.animation_time = 0.12;
    style.visuals.panel_fill = c.surface;
    style.visuals.window_fill = c.surface;
    style.visuals.extreme_bg_color = c.field;
    style.visuals.faint_bg_color = c.raised;
    style.visuals.override_text_color = Some(c.text);
    style.visuals.selection.bg_fill = c.selection;
    style.visuals.selection.stroke = Stroke::new(1.0, c.accent);
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, c.text);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, c.border);
    for state in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.open,
    ] {
        state.bg_fill = c.raised;
        state.weak_bg_fill = c.raised;
        state.fg_stroke = Stroke::new(1.0, c.text);
        state.bg_stroke = Stroke::new(0.5, c.border);
        state.corner_radius = egui::CornerRadius::same(m.radius);
    }
    style.visuals.widgets.hovered.bg_fill = c.hover;
    style.visuals.widgets.hovered.weak_bg_fill = c.hover;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, c.text);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, c.border);
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(m.radius);
    style.visuals.widgets.active.bg_fill = c.selection;
    style.visuals.widgets.active.weak_bg_fill = c.selection;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, c.text);
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, c.accent);
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(m.radius);
    style.visuals.window_stroke = Stroke::new(1.0, c.border);
    style.visuals.window_corner_radius = egui::CornerRadius::same(m.radius);
    style.visuals.menu_corner_radius = egui::CornerRadius::same(m.radius.saturating_add(4));
    style.spacing.menu_margin = egui::Margin::same(6);
    style.visuals.popup_shadow = egui::Shadow {
        offset: [0, 14],
        blur: 36,
        spread: 0,
        color: c.chrome.gamma_multiply(0.18),
    };
    style.spacing.item_spacing = egui::vec2(m.gap, m.gap);
    style.spacing.button_padding = egui::vec2(8.0, 3.0);
    style.spacing.interact_size.y = m.control_height;
    style.spacing.window_margin = egui::Margin::same(m.panel_margin);
    for (kind, size) in [
        (TextStyle::Small, m.small),
        (TextStyle::Body, m.body),
        (TextStyle::Button, m.body),
        (TextStyle::Heading, m.heading),
    ] {
        style.text_styles.insert(kind, FontId::proportional(size));
    }
    ctx.set_style_of(theme.base, style);
    ctx.data_mut(|data| data.insert_temp(key(), theme));
    ctx.request_repaint();
}
pub fn apply(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "Plex".into(),
        FontData::from_static(include_bytes!(
            "../../assets/family/IBMPlexSans-Variable.ttf"
        ))
        .into(),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "Plex".into());
    let mut semibold = FontData::from_static(include_bytes!(
        "../../assets/family/IBMPlexSans-Variable.ttf"
    ));
    semibold.tweak.coords = egui::epaint::text::VariationCoords::new([(b"wght", 600.0)]);
    fonts
        .font_data
        .insert("PlexSemibold".into(), semibold.into());
    let mut faces = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    faces.insert(0, "PlexSemibold".into());
    fonts
        .families
        .insert(FontFamily::Name("pcbcraft-semibold".into()), faces);
    ctx.set_fonts(fonts);
    egui_extras::install_image_loaders(ctx);
    set(ctx, Theme::default());
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("pcbcraft-semibold".into()))
}

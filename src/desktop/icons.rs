//! Temporary SVG consumer: shared masters and original EDA glyphs, resolved by theme role.
//! Replace with the upstream independent catalog adapter when available.
use super::theme;
use eframe::egui;
use std::hash::{Hash, Hasher};
#[derive(Clone, Copy)]
pub enum Icon {
    Save,
    Undo,
    Redo,
    Home,
    Folder,
    Sheet,
    Zoom,
    Close,
    Pan,
    Rotate,
    Symbol,
    Brand,
    Workspace,
    Layout,
    Render,
    Simulate,
    Wire,
    Move,
    Power,
    Port,
    Bus,
    Label,
    Mirror,
    Rules,
    Erc,
    Annotate,
    Bom,
    Sync,
    Select,
    Plus,
    Delete,
    Properties,
    Fit,
}
impl Icon {
    fn source(self) -> (&'static str, &'static str) {
        macro_rules! svg {
            ($n:literal) => {
                (
                    $n,
                    include_str!(concat!("../../assets/family/icons/", $n, ".svg")),
                )
            };
        }
        match self {
            Self::Save => svg!("save"),
            Self::Undo => svg!("undo"),
            Self::Redo => svg!("redo"),
            Self::Home => svg!("home"),
            Self::Folder => svg!("folder"),
            Self::Sheet => ("sheet", include_str!("../../assets/original/sheet.svg")),
            Self::Zoom => svg!("zoom"),
            Self::Close => svg!("close"),
            Self::Pan => svg!("pan"),
            Self::Rotate => svg!("spin"),
            Self::Symbol => (
                "place-symbol",
                include_str!("../../assets/original/place-symbol.svg"),
            ),
            Self::Brand => ("brand", include_str!("../../assets/original/chip.svg")),
            Self::Wire => ("wire", include_str!("../../assets/original/wire.svg")),
            Self::Workspace => svg!("workspace-schematic"),
            Self::Layout => (
                "workspace-layout",
                include_str!("../../assets/original/workspace-layout.svg"),
            ),
            Self::Render => svg!("workspace-render"),
            Self::Simulate => (
                "simulate",
                include_str!("../../assets/original/simulate.svg"),
            ),
            Self::Move => svg!("move"),
            Self::Mirror => svg!("mirror"),
            Self::Annotate => svg!("list"),
            Self::Bom => svg!("configuration_table"),
            Self::Select => svg!("select"),
            Self::Plus => svg!("plus"),
            Self::Delete => svg!("delete"),
            Self::Properties => svg!("measure"),
            Self::Fit => svg!("fit"),
            Self::Power => ("power", include_str!("../../assets/original/power.svg")),
            Self::Port => ("port", include_str!("../../assets/original/port.svg")),
            Self::Bus => ("bus", include_str!("../../assets/original/bus.svg")),
            Self::Label => (
                "net-label",
                include_str!("../../assets/original/net-label.svg"),
            ),
            Self::Rules => ("rules", include_str!("../../assets/original/rules.svg")),
            Self::Erc => ("erc", include_str!("../../assets/original/erc.svg")),
            Self::Sync => ("sync", include_str!("../../assets/original/sync.svg")),
        }
    }
    fn cache_uri(self, palette: theme::IconColors) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        palette.hash(&mut hasher);
        format!(
            "bytes://pcbcraft/icons/{}-{:016x}.svg",
            self.source().0,
            hasher.finish()
        )
    }
    pub fn image(self, size: f32, ctx: &egui::Context) -> egui::Image<'static> {
        self.image_with_palette(size, theme::current(ctx).icons)
    }
    pub fn danger_image(self, size: f32, ctx: &egui::Context) -> egui::Image<'static> {
        let t = theme::current(ctx);
        let mut palette = t.icons;
        // Keep the master outline and cutout paints; shade faces from the danger role.
        palette.line = t.ui.danger;
        palette.passive = [
            t.ui.danger.lerp_to_gamma(t.ui.text, 0.18),
            t.ui.danger,
            t.ui.danger.lerp_to_gamma(t.ui.chrome, 0.22),
        ];
        palette.active = palette.passive;
        self.image_with_palette(size, palette)
    }
    fn image_with_palette(self, size: f32, palette: theme::IconColors) -> egui::Image<'static> {
        egui::Image::from_bytes(
            self.cache_uri(palette),
            resolve(self.source().1, palette).into_bytes(),
        )
        .fit_to_exact_size(egui::vec2(size, size))
    }
}
// Original masters remain unchanged. Delete has white slot strokes, preserved verbatim;
// these qualified masters do not require generic mask/knockout rewriting.
// Two-stage substitution prevents a destination colour being interpreted as a source role.
fn resolve(source: &str, palette: theme::IconColors) -> String {
    let roles = [
        ("currentColor", palette.line),
        ("#34465b", palette.ink),
        ("#e2e8ef", palette.passive[0]),
        ("#b8c3cf", palette.passive[1]),
        ("#8e9faf", palette.passive[2]),
        ("#b9ddf8", palette.active[0]),
        ("#328bd3", palette.active[1]),
        ("#236ead", palette.active[2]),
        ("#6f879c", palette.guide),
    ];
    let mut result = source.to_owned();
    for (index, (source, _)) in roles.iter().enumerate() {
        result = result.replace(source, &format!("__pcbcraft_role_{index}__"));
    }
    for (index, (_, colour)) in roles.iter().enumerate() {
        let hex = format!("#{:02x}{:02x}{:02x}", colour.r(), colour.g(), colour.b());
        result = result.replace(&format!("__pcbcraft_role_{index}__"), &hex);
    }
    result
}

pub fn button(ui: &mut egui::Ui, icon: Icon, label: &str, enabled: bool) -> egui::Response {
    let response = ui
        .add_enabled(
            enabled,
            egui::Button::image(icon.image(16.0, ui.ctx())).frame(false),
        )
        .on_hover_text(label);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_palette_changes_invalidate_cached_icons_without_rewriting_art() {
        let base = theme::Theme::default().icons;
        let mut changed = base;
        // This target is another source role: resolution must not cascade.
        changed.line = egui::Color32::from_rgb(226, 232, 239);
        changed.passive[0] = egui::Color32::from_rgb(1, 2, 3);
        let source = "<path stroke=\"currentColor\" fill=\"#e2e8ef\"/>";
        let resolved = resolve(source, changed);
        assert!(resolved.contains("stroke=\"#e2e8ef\""));
        assert!(resolved.contains("fill=\"#010203\""));
        assert_ne!(
            Icon::Workspace.cache_uri(base),
            Icon::Workspace.cache_uri(changed)
        );
        assert_eq!(source, "<path stroke=\"currentColor\" fill=\"#e2e8ef\"/>");
    }
}

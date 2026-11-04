//! Native drawing-page geometry. It guides composition; content may remain outside.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Size {
    A4,
    #[default]
    A3,
    A2,
    Letter,
}
impl Size {
    pub const ALL: [Self; 4] = [Self::A4, Self::A3, Self::A2, Self::Letter];
    pub fn label(self) -> &'static str {
        match self {
            Self::A4 => "A4",
            Self::A3 => "A3",
            Self::A2 => "A2",
            Self::Letter => "Letter",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub size: Size,
    pub portrait: bool,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            size: Size::A3,
            portrait: false,
        }
    }
}
impl Page {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
    pub fn dimensions_nm(self) -> (i64, i64) {
        let (w, h) = match self.size {
            Size::A4 => (297_000_000, 210_000_000),
            Size::A3 => (420_000_000, 297_000_000),
            Size::A2 => (594_000_000, 420_000_000),
            Size::Letter => (279_400_000, 215_900_000),
        };
        if self.portrait { (h, w) } else { (w, h) }
    }
}

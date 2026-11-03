//! Original document text formatting. Text never establishes electrical connectivity.
use crate::document::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum TextTarget {
    Note(AnnotationId),
    Reference(SymbolId),
    Value(SymbolId),
    Net(WireId),
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Font {
    #[default]
    Sans,
    Semibold,
    Monospace,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    pub height_nm: i64,
    pub font: Font,
    pub italic: bool,
    pub underline: bool,
    pub offset: Point,
    pub link: String,
}
impl TextTarget {
    pub fn exists(self, doc: &Document) -> bool {
        match self {
            Self::Note(id) => doc.annotations.iter().any(|a| a.id == id),
            Self::Reference(id) | Self::Value(id) => doc.component(id).is_some(),
            Self::Net(id) => doc.wires.iter().any(|w| w.id == id),
        }
    }
    pub fn content(self, doc: &Document) -> Option<String> {
        match self {
            Self::Note(id) => doc
                .annotations
                .iter()
                .find(|a| a.id == id)
                .map(|a| a.text.clone()),
            Self::Reference(id) => doc.component(id).map(|c| c.reference.clone()),
            Self::Value(id) => doc.component(id).map(|c| c.value.clone()),
            Self::Net(id) => doc
                .wires
                .iter()
                .find(|w| w.id == id)
                .map(|w| w.label.clone().unwrap_or_default()),
        }
    }
    pub fn defaults(self) -> Appearance {
        Appearance {
            height_nm: match self {
                Self::Note(_) => 2_750_000,
                Self::Reference(_) => 1_375_000,
                Self::Value(_) => 1_250_000,
                Self::Net(_) => 1_875_000,
            },
            font: Font::Sans,
            italic: false,
            underline: false,
            offset: Point::default(),
            link: String::new(),
        }
    }
}
impl Appearance {
    pub fn validate(&self) -> Result<()> {
        if !(250_000..=12_000_000).contains(&self.height_nm) || !self.offset.valid() {
            return Err("Text height must be 0.25–12 mm with a bounded offset.".into());
        }
        if !self.link.is_empty() {
            let host = self
                .link
                .strip_prefix("https://")
                .or_else(|| self.link.strip_prefix("http://"))
                .ok_or("Links must start with https:// or http://.")?;
            if self.link.len() > 2048
                || self
                    .link
                    .chars()
                    .any(|c| c.is_control() || c.is_whitespace())
                || host
                    .split(['/', '?', '#'])
                    .next()
                    .is_none_or(|h| h.is_empty() || h.contains(['@', '\\']))
            {
                return Err(
                    "Use a valid web address without embedded credentials (2048 bytes max).".into(),
                );
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Override {
    pub target: TextTarget,
    pub appearance: Appearance,
}
pub fn appearance(doc: &Document, target: TextTarget) -> Appearance {
    if let Some(item) = doc.text_overrides.iter().find(|t| t.target == target) {
        return item.appearance.clone();
    }
    if doc
        .text_overrides
        .iter()
        .any(|o| matches!(o.target, TextTarget::Net(_)))
        && let TextTarget::Net(id) = target
        && let Some(w) = doc.wires.iter().find(|w| w.id == id)
        && let Some(net) = crate::netlist::groups(doc).containing(&w.from)
    {
        for item in &doc.text_overrides {
            if let TextTarget::Net(id) = item.target
                && net.links.contains(&id)
            {
                return item.appearance.clone();
            }
        }
    }
    target.defaults()
}
pub(crate) fn prune(doc: &mut Document) {
    let keep = doc
        .text_overrides
        .iter()
        .filter(|o| o.target.exists(doc))
        .cloned()
        .collect();
    doc.text_overrides = keep;
}
pub(crate) fn edit(
    doc: &mut Document,
    target: TextTarget,
    content: String,
    appearance: Appearance,
) -> Result<()> {
    appearance.validate()?;
    match target {
        TextTarget::Note(id) => {
            doc.annotations
                .iter_mut()
                .find(|a| a.id == id)
                .ok_or("Text no longer exists.")?
                .text = content
        }
        TextTarget::Reference(id) | TextTarget::Value(id) => {
            let cid = doc
                .symbols
                .iter()
                .find(|s| s.id == id)
                .ok_or("Symbol no longer exists.")?
                .component;
            let c = doc
                .components
                .iter_mut()
                .find(|c| c.id == cid)
                .ok_or("Component no longer exists.")?;
            if matches!(target, TextTarget::Reference(_)) {
                c.reference = content.trim().into();
            } else {
                c.value = content;
            }
        }
        TextTarget::Net(id) => {
            let w = doc
                .wires
                .iter()
                .find(|w| w.id == id)
                .ok_or("Wire no longer exists.")?;
            let anchor = w.from.clone();
            let ids = crate::netlist::groups(doc)
                .containing(&anchor)
                .ok_or("Net no longer exists.")?
                .links
                .clone();
            crate::netlist::rename(doc, &anchor, &content)?;
            doc.text_overrides
                .retain(|o| !matches!(o.target,TextTarget::Net(id) if ids.contains(&id)));
            // A formatting choice belongs to the whole explicit net, never its name alone.
            for id in ids {
                doc.text_overrides.push(Override {
                    target: TextTarget::Net(id),
                    appearance: appearance.clone(),
                });
            }
            return Ok(());
        }
    }
    doc.text_overrides.retain(|o| o.target != target);
    doc.text_overrides.push(Override { target, appearance });
    Ok(())
}

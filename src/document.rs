//! Original, deliberately bounded schematic draft model. No external EDA data.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub use crate::error::Error;
pub type Result<T> = std::result::Result<T, Error>;
pub const GRID_NM: i64 = 1_270_000;
pub const LIMIT_NM: i64 = 1_000_000_000;
pub const MAX_ITEMS: usize = 4096;

macro_rules! identity {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name([u8; 16]);
        impl $name {
            pub fn fresh() -> Result<Self> {
                let mut bytes = [0; 16];
                getrandom::fill(&mut bytes)
                    .map_err(|e| format!("Could not allocate identity: {e}"))?;
                Ok(Self(bytes))
            }
        }
    };
}
identity!(ProjectId);
identity!(SheetId);
identity!(ComponentId);
identity!(SymbolId);
identity!(WireId);
identity!(JunctionId);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}
impl Point {
    pub fn snapped(self) -> Self {
        let snap = |n: i64| {
            let cells = (n as f64 / GRID_NM as f64).round() as i64;
            cells.clamp(-LIMIT_NM / GRID_NM, LIMIT_NM / GRID_NM) * GRID_NM
        };
        Self {
            x: snap(self.x.clamp(-LIMIT_NM, LIMIT_NM)),
            y: snap(self.y.clamp(-LIMIT_NM, LIMIT_NM)),
        }
    }
    pub fn valid(self) -> bool {
        (-LIMIT_NM..=LIMIT_NM).contains(&self.x) && (-LIMIT_NM..=LIMIT_NM).contains(&self.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartKind {
    Resistor,
    Capacitor,
    Led,
    Connector,
    FanController,
    FanConnector,
    Regulator,
    TemperatureSensor,
}
impl PartKind {
    pub const ALL: [Self; 8] = [
        Self::Resistor,
        Self::Capacitor,
        Self::Led,
        Self::Connector,
        Self::FanController,
        Self::FanConnector,
        Self::Regulator,
        Self::TemperatureSensor,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Resistor => "Resistor",
            Self::Capacitor => "Capacitor",
            Self::Led => "LED",
            Self::Connector => "2-pin connector",
            Self::FanController => "Controller module",
            Self::FanConnector => "4-wire fan connector",
            Self::Regulator => "Regulator module",
            Self::TemperatureSensor => "Analog temperature sensor",
        }
    }
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Resistor => "R",
            Self::Capacitor => "C",
            Self::Led => "D",
            Self::Connector => "J",
            Self::FanConnector => "J",
            Self::FanController | Self::Regulator | Self::TemperatureSensor => "U",
        }
    }
    pub fn value(self) -> &'static str {
        match self {
            Self::Resistor => "1k",
            Self::Capacitor => "100nF",
            Self::Led => "Green LED",
            Self::Connector => "Input",
            Self::FanController => "Fan controller",
            Self::FanConnector => "12 V PWM fan",
            Self::Regulator => "12 V to 3.3 V",
            Self::TemperatureSensor => "Analog temperature",
        }
    }
    pub fn pins(self) -> &'static [PinDefinition] {
        crate::definitions::pins(self)
    }
    pub fn is_block(self) -> bool {
        matches!(
            self,
            Self::FanController | Self::FanConnector | Self::Regulator | Self::TemperatureSensor
        )
    }
}

#[derive(Clone, Copy)]
pub struct PinDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub x: i64,
    pub y: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub id: ComponentId,
    pub kind: PartKind,
    pub reference: String,
    pub value: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Symbol {
    pub id: SymbolId,
    pub component: ComponentId,
    pub at: Point,
    pub quarter_turns: u8,
}
impl Symbol {
    pub fn pin(&self, kind: PartKind, terminal: &str) -> Option<Point> {
        let p = kind.pins().iter().find(|p| p.id == terminal)?;
        let (x, y) = (p.x * GRID_NM, p.y * GRID_NM);
        let (x, y) = match self.quarter_turns {
            0 => (x, y),
            1 => (-y, x),
            2 => (-x, -y),
            3 => (y, -x),
            _ => return None,
        };
        Some(Point {
            x: self.at.x + x,
            y: self.at.y + y,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinRef {
    pub symbol: SymbolId,
    pub terminal: String,
}
/// A pin or an explicit, independently identified wire endpoint/junction.
/// Untagged pin serialization preserves the v1–v3 endpoint representation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Endpoint {
    Pin(PinRef),
    Junction(JunctionRef),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JunctionRef {
    pub junction: JunctionId,
}
impl From<PinRef> for Endpoint {
    fn from(p: PinRef) -> Self {
        Self::Pin(p)
    }
}
impl Endpoint {
    pub fn pin(&self) -> Option<&PinRef> {
        if let Self::Pin(p) = self {
            Some(p)
        } else {
            None
        }
    }
    pub fn on_symbol(&self, id: SymbolId) -> bool {
        self.pin().is_some_and(|p| p.symbol == id)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Junction {
    pub id: JunctionId,
    pub at: Point,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wire {
    pub id: WireId,
    pub from: Endpoint,
    pub to: Endpoint,
    /// Display alias for this explicit terminal link; names never create connectivity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Keep a drawn path when a named net is rendered. Older named links are aliases.
    #[serde(default, skip_serializing_if = "is_false")]
    pub draw_path: bool,
    /// Authored interior corners. None retains the legacy automatic display route.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bends: Option<Vec<Point>>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Annotation {
    pub at: Point,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub id: ProjectId,
    pub sheet_id: SheetId,
    pub name: String,
    /// Original built-in definitions are versioned, not resolved from a live catalog.
    pub builtin_definitions_version: u32,
    pub components: Vec<Component>,
    pub symbols: Vec<Symbol>,
    pub wires: Vec<Wire>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub junctions: Vec<Junction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<Annotation>,
}
impl Document {
    pub fn empty(name: &str) -> Result<Self> {
        let doc = Self {
            id: ProjectId::fresh()?,
            sheet_id: SheetId::fresh()?,
            name: name.into(),
            builtin_definitions_version: 2,
            components: Vec::new(),
            symbols: Vec::new(),
            wires: Vec::new(),
            junctions: Vec::new(),
            annotations: Vec::new(),
        };
        doc.validate()?;
        Ok(doc)
    }
    pub fn component(&self, symbol: SymbolId) -> Option<&Component> {
        let placement = self.symbols.iter().find(|s| s.id == symbol)?;
        self.components.iter().find(|c| c.id == placement.component)
    }
    pub fn pin(&self, pin: &PinRef) -> Option<Point> {
        self.symbols
            .iter()
            .find(|s| s.id == pin.symbol)?
            .pin(self.component(pin.symbol)?.kind, &pin.terminal)
    }
    pub fn endpoint(&self, endpoint: &Endpoint) -> Option<Point> {
        match endpoint {
            Endpoint::Pin(p) => self.pin(p),
            Endpoint::Junction(j) => self
                .junctions
                .iter()
                .find(|n| n.id == j.junction)
                .map(|n| n.at),
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty()
            || self.name.len() > 120
            || self.name.chars().any(char::is_control)
        {
            return Err("Project name must contain 1–120 printable bytes.".into());
        }
        if ![1, 2].contains(&self.builtin_definitions_version) {
            return Err("Unsupported built-in definition version.".into());
        }
        if self.symbols.len() > MAX_ITEMS
            || self.wires.len() > MAX_ITEMS
            || self.junctions.len() > MAX_ITEMS * 2
            || self.components.len() != self.symbols.len()
        {
            return Err("Draft supports up to 4096 single-unit components and wires.".into());
        }
        let mut components = BTreeSet::new();
        let mut refs = BTreeSet::new();
        for c in &self.components {
            if self.builtin_definitions_version == 1 && c.kind.is_block() {
                return Err("This symbol requires built-in definitions version 2.".into());
            }
            if !components.insert(c.id) || !refs.insert(c.reference.to_ascii_uppercase()) {
                return Err("Duplicate component identity or reference.".into());
            }
            let mut chars = c.reference.chars();
            if !chars.next().is_some_and(|c| c.is_ascii_alphabetic())
                || !chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
                || c.reference.len() > 32
            {
                return Err("Reference must start with a letter and use letters, digits or underscores (32 max).".into());
            }
            if c.value.len() > 128 || c.value.chars().any(char::is_control) {
                return Err("Value must be at most 128 printable bytes.".into());
            }
        }
        let mut symbols = BTreeSet::new();
        let mut placed = BTreeSet::new();
        for s in &self.symbols {
            if !symbols.insert(s.id)
                || !components.contains(&s.component)
                || !placed.insert(s.component)
                || !s.at.valid()
                || s.quarter_turns > 3
            {
                return Err("Invalid symbol identity, component, coordinate or rotation.".into());
            }
        }
        let mut junctions = BTreeSet::new();
        for j in &self.junctions {
            if !junctions.insert(j.id) || !j.at.valid() {
                return Err("Invalid or duplicate wire junction.".into());
            }
            let endpoint = Endpoint::Junction(JunctionRef { junction: j.id });
            if !self
                .wires
                .iter()
                .any(|w| w.from == endpoint || w.to == endpoint)
            {
                return Err("Unused wire junction.".into());
            }
        }
        let mut wires = BTreeSet::new();
        let mut pairs = BTreeSet::new();
        let mut named_pins = std::collections::BTreeMap::new();
        for w in &self.wires {
            if let Some(label) = &w.label {
                if label.is_empty() || label.len() > 32 || label.chars().any(char::is_control) {
                    return Err("Connection label must contain 1–32 printable bytes.".into());
                }
                for pin in [&w.from, &w.to] {
                    if let Some(previous) = named_pins.insert(pin.clone(), label)
                        && previous != label
                    {
                        return Err("Conflicting connection labels on one terminal.".into());
                    }
                }
            }
            if let Some(bends) = &w.bends {
                if bends.len() > 256 || bends.iter().any(|p| !p.valid()) {
                    return Err("Wire supports at most 256 bounded corners.".into());
                }
                let path = crate::wiring::path(self, w)?;
                crate::wiring::validate_path(&path)?;
                if w.label.is_some() && !w.draw_path {
                    return Err("An authored wire must remain visible.".into());
                }
            }
            let a = w.from.clone();
            let b = w.to.clone();
            let pair = if a <= b { (a, b) } else { (b, a) };
            if !wires.insert(w.id)
                || !pairs.insert(pair)
                || w.from == w.to
                || self.endpoint(&w.from).is_none()
                || self.endpoint(&w.to).is_none()
            {
                return Err("Invalid or duplicate terminal connection.".into());
            }
        }
        // Display aliases may abbreviate existing links, never create hidden connectivity.
        // A name identifies one explicit connected group and a group has at most one name.
        let mut adjacency = std::collections::BTreeMap::<_, Vec<_>>::new();
        for wire in &self.wires {
            let a = wire.from.clone();
            let b = wire.to.clone();
            adjacency.entry(a.clone()).or_default().push(b.clone());
            adjacency.entry(b).or_default().push(a);
        }
        let mut visited = BTreeSet::new();
        let mut names = BTreeSet::new();
        for start in adjacency.keys() {
            if visited.contains(start) {
                continue;
            }
            let mut stack = vec![start.clone()];
            let mut name = None;
            while let Some(pin) = stack.pop() {
                if !visited.insert(pin.clone()) {
                    continue;
                }
                if let Some(label) = named_pins.get(&pin) {
                    if name.is_some_and(|n| n != *label) {
                        return Err("Conflicting labels on one connected group.".into());
                    }
                    name = Some(*label);
                }
                if let Some(neighbors) = adjacency.get(&pin) {
                    stack.extend(neighbors.iter().cloned());
                }
            }
            if let Some(name) = name
                && !names.insert(name)
            {
                return Err("A display label cannot describe disconnected groups.".into());
            }
        }
        if self.annotations.len() > 256
            || self.annotations.iter().any(|a| {
                !a.at.valid()
                    || a.text.is_empty()
                    || a.text.len() > 128
                    || a.text.chars().any(char::is_control)
            })
        {
            return Err("Invalid sheet annotation.".into());
        }
        Ok(())
    }
}

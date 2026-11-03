//! Project aggregate, explicit hierarchy and occurrence-qualified cross-probe identities.
//! Display names never join nets or identify physical parts.
use crate::document::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_SHEETS: usize = 64;
pub const MAX_OCCURRENCES: usize = 256;
pub const MAX_DEPTH: usize = 16;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortRef {
    pub port: PortId,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SheetPinRef {
    pub sheet_symbol: SheetSymbolId,
    pub port: PortId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Input,
    Output,
    Bidirectional,
    Passive,
}
impl Direction {
    pub const ALL: [Self; 4] = [
        Self::Input,
        Self::Output,
        Self::Bidirectional,
        Self::Passive,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Input => "Input",
            Self::Output => "Output",
            Self::Bidirectional => "Bidirectional",
            Self::Passive => "Passive",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    pub id: PortId,
    pub name: String,
    pub direction: Direction,
    pub at: Point,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SheetPin {
    pub id: PortId,
    pub name: String,
    pub direction: Direction,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SheetSymbol {
    pub id: SheetSymbolId,
    pub definition: SheetId,
    pub at: Point,
    pub pins: Vec<SheetPin>,
}
impl SheetSymbol {
    pub fn pin(&self, id: PortId) -> Option<Point> {
        let i = self.pins.iter().position(|p| p.id == id)?;
        let output = self.pins[i].direction == Direction::Output;
        Some(Point {
            x: self
                .at
                .x
                .saturating_add(if output { 24 * GRID_NM } else { 0 }),
            y: self.at.y.saturating_add((3 + i as i64 * 3) * GRID_NM),
        })
    }
    pub fn bottom_right(&self) -> Point {
        Point {
            x: self.at.x.saturating_add(24 * GRID_NM),
            y: self
                .at
                .y
                .saturating_add((5 + self.pins.len() as i64 * 3) * GRID_NM),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Occurrence {
    pub id: SheetOccurrenceId,
    pub definition: SheetId,
    pub name: String,
    pub parent: Option<SheetOccurrenceId>,
    pub via: Option<SheetSymbolId>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub main: Document,
    pub sheets: Vec<Document>,
    pub root: SheetOccurrenceId,
    pub occurrences: Vec<Occurrence>,
}
#[derive(Clone, Debug)]
pub enum Command {
    /// None parent creates a flat sheet; Some creates a sheet symbol in that occurrence's definition.
    Create {
        name: String,
        parent: Option<SheetOccurrenceId>,
        existing: Option<SheetId>,
        at: Point,
    },
    Rename {
        occurrence: SheetOccurrenceId,
        name: String,
    },
    Delete {
        occurrence: SheetOccurrenceId,
    },
    SetPort {
        occurrence: SheetOccurrenceId,
        id: Option<PortId>,
        name: String,
        direction: Direction,
        at: Point,
    },
    DeletePort {
        occurrence: SheetOccurrenceId,
        id: PortId,
    },
    MoveSheet {
        occurrence: SheetOccurrenceId,
        symbol: SheetSymbolId,
        at: Point,
    },
}
#[derive(Clone, Debug, Default)]
pub struct Effect {
    pub created: Option<SheetOccurrenceId>,
}
fn name_valid(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 80 && !s.chars().any(char::is_control)
}
fn interface(d: &Document) -> Vec<SheetPin> {
    d.ports
        .iter()
        .map(|p| SheetPin {
            id: p.id,
            name: p.name.clone(),
            direction: p.direction,
        })
        .collect()
}
pub(crate) fn validate_sheet(d: &Document) -> Result<()> {
    if d.ports.len() > 32 || d.sheet_symbols.len() > 64 {
        return Err("Sheet supports up to 32 ports and 64 child symbols.".into());
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for p in &d.ports {
        if !ids.insert(p.id)
            || !names.insert(p.name.to_lowercase())
            || !name_valid(&p.name)
            || p.name.len() > 32
            || !p.at.valid()
        {
            return Err("Invalid or duplicate sheet port.".into());
        }
    }
    let mut symbols = BTreeSet::new();
    for s in &d.sheet_symbols {
        let mut pins = BTreeSet::new();
        if !symbols.insert(s.id)
            || !s.at.valid()
            || s.pins.len() > 32
            || !s.bottom_right().valid()
            || s.pins
                .iter()
                .any(|p| !pins.insert(p.id) || !name_valid(&p.name))
        {
            return Err("Invalid sheet symbol or port mapping.".into());
        }
    }
    Ok(())
}
impl Project {
    pub fn from_document(doc: Document) -> Result<Self> {
        // A legacy draft cannot carry hierarchy that would otherwise be lost.
        if !doc.sheet_symbols.is_empty() || !doc.ports.is_empty() {
            return Err("Use a project envelope for hierarchical sheets.".into());
        }
        doc.validate()?;
        let root = SheetOccurrenceId::fresh()?;
        Ok(Self {
            id: doc.id,
            name: doc.name.clone(),
            occurrences: vec![Occurrence {
                id: root,
                definition: doc.sheet_id,
                name: "Main".into(),
                parent: None,
                via: None,
            }],
            main: doc,
            sheets: vec![],
            root,
        })
    }
    pub fn definitions(&self) -> impl Iterator<Item = &Document> {
        std::iter::once(&self.main).chain(self.sheets.iter())
    }
    pub fn definition(&self, id: SheetId) -> Option<&Document> {
        self.definitions().find(|d| d.sheet_id == id)
    }
    pub(crate) fn definition_mut(&mut self, id: SheetId) -> Option<&mut Document> {
        if self.main.sheet_id == id {
            Some(&mut self.main)
        } else {
            self.sheets.iter_mut().find(|d| d.sheet_id == id)
        }
    }
    pub fn occurrence(&self, id: SheetOccurrenceId) -> Option<&Occurrence> {
        self.occurrences.iter().find(|o| o.id == id)
    }
    pub fn document(&self, id: SheetOccurrenceId) -> Option<&Document> {
        self.definition(self.occurrence(id)?.definition)
    }
    pub fn child(&self, parent: SheetOccurrenceId, via: SheetSymbolId) -> Option<&Occurrence> {
        self.occurrences
            .iter()
            .find(|o| o.parent == Some(parent) && o.via == Some(via))
    }
    pub fn path(&self, id: SheetOccurrenceId) -> String {
        let mut names = vec![];
        let mut current = Some(id);
        for _ in 0..=MAX_DEPTH {
            let Some(o) = current.and_then(|id| self.occurrence(id)) else {
                break;
            };
            names.push(o.name.as_str());
            current = o.parent;
        }
        names.reverse();
        names.join(" / ")
    }
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty()
            || self.name.len() > 120
            || self.name.chars().any(char::is_control)
            || self.sheets.len() >= MAX_SHEETS
            || self.occurrences.is_empty()
            || self.occurrences.len() > MAX_OCCURRENCES
        {
            return Err("Invalid project name or sheet count.".into());
        }
        let mut defs = BTreeSet::new();
        for d in self.definitions() {
            if !defs.insert(d.sheet_id) || d.id != self.id {
                return Err("Duplicate definition or wrong project identity.".into());
            }
            d.validate()?;
            for s in &d.sheet_symbols {
                let child = self
                    .definition(s.definition)
                    .ok_or("Missing child definition.")?;
                if s.pins != interface(child) {
                    return Err("Sheet pin mapping does not match the child interface.".into());
                }
            }
            self.check_graph(d.sheet_id, &mut BTreeSet::new(), 0, &mut 0)?;
        }
        let root = self.occurrence(self.root).ok_or("Missing primary sheet.")?;
        if root.parent.is_some() || root.definition != self.main.sheet_id {
            return Err("Invalid primary sheet.".into());
        }
        let mut ids = BTreeSet::new();
        let mut children = BTreeSet::new();
        let mut names = BTreeSet::new();
        for o in &self.occurrences {
            if !ids.insert(o.id)
                || !name_valid(&o.name)
                || !names.insert((o.parent, o.name.to_lowercase()))
            {
                return Err("Invalid or duplicate sheet occurrence/name in a branch.".into());
            }
            let d = self.document(o.id).ok_or("Occurrence has no definition.")?;
            match (o.parent, o.via) {
                (None, None) => {}
                (Some(parent), Some(via)) => {
                    let p = self.document(parent).ok_or("Missing parent occurrence.")?;
                    if !children.insert((parent, via))
                        || !p
                            .sheet_symbols
                            .iter()
                            .any(|s| s.id == via && s.definition == o.definition)
                    {
                        return Err("Invalid parent sheet mapping.".into());
                    }
                }
                _ => return Err("Incomplete parent sheet mapping.".into()),
            }
            let mut lineage = BTreeSet::new();
            let mut current = Some(o.id);
            while let Some(id) = current {
                if !lineage.insert(id) || lineage.len() > MAX_DEPTH + 1 {
                    return Err("Cyclic or too deep sheet hierarchy.".into());
                }
                current = self.occurrence(id).ok_or("Missing occurrence.")?.parent;
            }
            for s in &d.sheet_symbols {
                if self.child(o.id, s.id).is_none() {
                    return Err("Missing child occurrence.".into());
                }
            }
        }
        if self
            .definitions()
            .any(|d| !self.occurrences.iter().any(|o| o.definition == d.sheet_id))
        {
            return Err("Unreferenced sheet definition.".into());
        }
        Ok(())
    }
    fn check_graph(
        &self,
        id: SheetId,
        path: &mut BTreeSet<SheetId>,
        depth: usize,
        visited: &mut usize,
    ) -> Result<()> {
        *visited += 1;
        if *visited > MAX_OCCURRENCES || depth > MAX_DEPTH || !path.insert(id) {
            return Err("Cyclic or too deep sheet hierarchy.".into());
        }
        for s in &self
            .definition(id)
            .ok_or("Missing sheet definition.")?
            .sheet_symbols
        {
            self.check_graph(s.definition, path, depth + 1, visited)?;
        }
        path.remove(&id);
        Ok(())
    }
    fn sync_occurrences(&mut self) -> Result<()> {
        for d in self.definitions() {
            self.check_graph(d.sheet_id, &mut BTreeSet::new(), 0, &mut 0)?;
        }
        let old = self.occurrences.clone();
        self.occurrences.retain(|o| o.parent.is_none());
        let mut i = 0;
        while i < self.occurrences.len() {
            if self.occurrences.len() > MAX_OCCURRENCES {
                return Err("Too many sheet occurrences.".into());
            }
            let parent = self.occurrences[i].clone();
            let blocks = self
                .definition(parent.definition)
                .ok_or("Missing sheet.")?
                .sheet_symbols
                .clone();
            for s in blocks {
                let child =
                    if let Some(o) = old
                        .iter()
                        .find(|o| o.parent == Some(parent.id) && o.via == Some(s.id))
                    {
                        o.clone()
                    } else {
                        let base = self
                            .definition(s.definition)
                            .ok_or("Missing child sheet.")?
                            .name
                            .clone();
                        let mut name = base.clone();
                        let mut n = 2;
                        while self.occurrences.iter().any(|o| {
                            o.parent == Some(parent.id) && o.name.eq_ignore_ascii_case(&name)
                        }) {
                            name = format!("{base} {n}");
                            n += 1;
                        }
                        Occurrence {
                            id: SheetOccurrenceId::fresh()?,
                            definition: s.definition,
                            name,
                            parent: Some(parent.id),
                            via: Some(s.id),
                        }
                    };
                self.occurrences.push(child);
            }
            i += 1;
        }
        let used: BTreeSet<_> = self.occurrences.iter().map(|o| o.definition).collect();
        self.sheets.retain(|d| used.contains(&d.sheet_id));
        Ok(())
    }
    pub fn preview(&self, command: Command) -> Result<Self> {
        let mut next = self.clone();
        next.edit(command)?;
        Ok(next)
    }
    pub(crate) fn edit(&mut self, command: Command) -> Result<Effect> {
        let mut effect = Effect::default();
        match command {
            Command::Create {
                name,
                parent,
                existing,
                at,
            } => {
                if !name_valid(&name) {
                    return Err("Sheet name must use 1–80 printable bytes.".into());
                }
                if self
                    .occurrences
                    .iter()
                    .any(|o| o.parent == parent && o.name.eq_ignore_ascii_case(&name))
                {
                    return Err("A sheet with this name already exists in this branch.".into());
                }
                let definition = if let Some(id) = existing {
                    self.definition(id)
                        .ok_or("Sheet definition no longer exists.")?;
                    id
                } else {
                    let mut d = Document::empty(&name)?;
                    d.id = self.id;
                    let id = d.sheet_id;
                    self.sheets.push(d);
                    id
                };
                if let Some(parent) = parent {
                    let pins = interface(self.definition(definition).ok_or("Missing child.")?);
                    let parent_def = self
                        .occurrence(parent)
                        .ok_or("Parent no longer exists.")?
                        .definition;
                    let symbol = SheetSymbolId::fresh()?;
                    self.definition_mut(parent_def)
                        .ok_or("Missing parent.")?
                        .sheet_symbols
                        .push(SheetSymbol {
                            id: symbol,
                            definition,
                            at,
                            pins,
                        });
                    self.sync_occurrences()?;
                    let o = self
                        .occurrences
                        .iter_mut()
                        .find(|o| o.parent == Some(parent) && o.via == Some(symbol))
                        .ok_or("Missing new child.")?;
                    o.name = name;
                    effect.created = Some(o.id);
                } else {
                    let id = SheetOccurrenceId::fresh()?;
                    self.occurrences.push(Occurrence {
                        id,
                        definition,
                        name,
                        parent: None,
                        via: None,
                    });
                    self.sync_occurrences()?;
                    effect.created = Some(id);
                }
            }
            Command::Rename { occurrence, name } => {
                self.occurrences
                    .iter_mut()
                    .find(|o| o.id == occurrence)
                    .ok_or("Sheet no longer exists.")?
                    .name = name;
            }
            Command::Delete { occurrence } => {
                if occurrence == self.root {
                    return Err("The primary sheet cannot be removed.".into());
                }
                let o = self
                    .occurrence(occurrence)
                    .ok_or("Sheet no longer exists.")?
                    .clone();
                if let (Some(parent), Some(via)) = (o.parent, o.via) {
                    let id = self.occurrence(parent).ok_or("Missing parent.")?.definition;
                    let d = self
                        .definition_mut(id)
                        .ok_or("Missing parent definition.")?;
                    d.sheet_symbols.retain(|s| s.id != via);
                    d.wires.retain(|w| {
                        ![&w.from, &w.to]
                            .iter()
                            .any(|e| matches!(e, Endpoint::SheetPin(p) if p.sheet_symbol == via))
                    });
                    clean(d);
                } else {
                    self.occurrences.retain(|s| s.id != occurrence);
                }
                self.sync_occurrences()?;
            }
            Command::SetPort {
                occurrence,
                id,
                name,
                direction,
                at,
            } => {
                let def = self
                    .occurrence(occurrence)
                    .ok_or("Sheet no longer exists.")?
                    .definition;
                let d = self.definition_mut(def).ok_or("Missing definition.")?;
                let before = d.clone();
                if let Some(id) = id {
                    let p = d
                        .ports
                        .iter_mut()
                        .find(|p| p.id == id)
                        .ok_or("Port no longer exists.")?;
                    p.name = name;
                    p.direction = direction;
                    p.at = at;
                } else {
                    d.ports.push(Port {
                        id: PortId::fresh()?,
                        name,
                        direction,
                        at,
                    });
                }
                crate::wiring::repair_moved_endpoints(&before, d)?;
                self.sync_ports(def)?;
            }
            Command::DeletePort { occurrence, id } => {
                let def = self
                    .occurrence(occurrence)
                    .ok_or("Missing sheet.")?
                    .definition;
                let d = self.definition_mut(def).ok_or("Missing definition.")?;
                if !d.ports.iter().any(|p| p.id == id) {
                    return Err("Port no longer exists.".into());
                }
                d.ports.retain(|p| p.id != id);
                d.wires.retain(|w| {
                    ![&w.from, &w.to]
                        .iter()
                        .any(|e| matches!(e, Endpoint::Port(p) if p.port == id))
                });
                clean(d);
                self.sync_ports(def)?;
            }
            Command::MoveSheet {
                occurrence,
                symbol,
                at,
            } => {
                let def = self
                    .occurrence(occurrence)
                    .ok_or("Missing sheet.")?
                    .definition;
                let d = self.definition_mut(def).ok_or("Missing definition.")?;
                let before = d.clone();
                d.sheet_symbols
                    .iter_mut()
                    .find(|s| s.id == symbol)
                    .ok_or("Missing sheet symbol.")?
                    .at = at;
                crate::wiring::repair_moved_endpoints(&before, d)?;
            }
        }
        self.validate()?;
        Ok(effect)
    }
    fn sync_ports(&mut self, def: SheetId) -> Result<()> {
        let pins = interface(self.definition(def).ok_or("Missing definition.")?);
        for d in std::iter::once(&mut self.main).chain(self.sheets.iter_mut()) {
            let before = d.clone();
            let affected: BTreeSet<_> = d
                .sheet_symbols
                .iter()
                .filter(|s| s.definition == def)
                .map(|s| s.id)
                .collect();
            for s in &mut d.sheet_symbols {
                if s.definition == def {
                    s.pins = pins.clone();
                }
            }
            d.wires.retain(|w| ![&w.from, &w.to].iter().any(|e| matches!(e, Endpoint::SheetPin(p) if affected.contains(&p.sheet_symbol) && !pins.iter().any(|n| n.id == p.port))));
            clean(d);
            crate::wiring::repair_moved_endpoints(&before, d)?;
        }
        Ok(())
    }
}
fn clean(d: &mut Document) {
    crate::wiring::prune_junctions(d);
    crate::wiring::normalize_names(d);
    crate::text::prune(d);
}

/// Stable physical identity: one definition component in one sheet occurrence.
/// Future footprint and pad mappings must reference this identity, not a refdes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentTarget {
    pub project: ProjectId,
    pub occurrence: SheetOccurrenceId,
    pub component: ComponentId,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalTarget {
    pub component: ComponentTarget,
    pub terminal: String,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub occurrence: SheetOccurrenceId,
    pub endpoint: Endpoint,
}
#[derive(Clone, Debug, Serialize)]
pub struct Connectivity {
    pub project: ProjectId,
    pub nets: Vec<Vec<Node>>,
}
impl Project {
    pub fn component_target(
        &self,
        occurrence: SheetOccurrenceId,
        symbol: SymbolId,
    ) -> Option<ComponentTarget> {
        Some(ComponentTarget {
            project: self.id,
            occurrence,
            component: self.document(occurrence)?.component(symbol)?.id,
        })
    }
    pub fn terminal_target(&self, node: &Node) -> Option<TerminalTarget> {
        let p = node.endpoint.pin()?;
        self.document(node.occurrence)?.pin(p)?;
        Some(TerminalTarget {
            component: self.component_target(node.occurrence, p.symbol)?,
            terminal: p.terminal.clone(),
        })
    }
    pub fn resolve_component(&self, target: &ComponentTarget) -> Option<SymbolId> {
        if target.project != self.id {
            return None;
        }
        self.document(target.occurrence)?
            .symbols
            .iter()
            .find(|s| s.component == target.component)
            .map(|s| s.id)
    }
    pub fn connectivity(&self) -> Result<Connectivity> {
        self.validate()?;
        let mut graph = BTreeMap::<Node, BTreeSet<Node>>::new();
        fn link(g: &mut BTreeMap<Node, BTreeSet<Node>>, a: Node, b: Node) {
            g.entry(a.clone()).or_default().insert(b.clone());
            g.entry(b).or_default().insert(a);
        }
        for o in &self.occurrences {
            let d = self.document(o.id).ok_or("Missing sheet.")?;
            for s in &d.symbols {
                if let Some(c) = d.component(s.id) {
                    for pin in c.kind.pins() {
                        graph
                            .entry(Node {
                                occurrence: o.id,
                                endpoint: PinRef {
                                    symbol: s.id,
                                    terminal: pin.id.into(),
                                }
                                .into(),
                            })
                            .or_default();
                    }
                }
            }
            for p in &d.ports {
                graph
                    .entry(Node {
                        occurrence: o.id,
                        endpoint: Endpoint::Port(PortRef { port: p.id }),
                    })
                    .or_default();
            }
            for w in &d.wires {
                link(
                    &mut graph,
                    Node {
                        occurrence: o.id,
                        endpoint: w.from.clone(),
                    },
                    Node {
                        occurrence: o.id,
                        endpoint: w.to.clone(),
                    },
                );
            }
            for s in &d.sheet_symbols {
                let child = self.child(o.id, s.id).ok_or("Missing child occurrence.")?;
                for p in &s.pins {
                    link(
                        &mut graph,
                        Node {
                            occurrence: o.id,
                            endpoint: Endpoint::SheetPin(SheetPinRef {
                                sheet_symbol: s.id,
                                port: p.id,
                            }),
                        },
                        Node {
                            occurrence: child.id,
                            endpoint: Endpoint::Port(PortRef { port: p.id }),
                        },
                    );
                }
            }
        }
        let mut visited = BTreeSet::new();
        let mut nets = vec![];
        for start in graph.keys() {
            if visited.contains(start) {
                continue;
            }
            let mut stack = vec![start.clone()];
            let mut group = BTreeSet::new();
            while let Some(n) = stack.pop() {
                if !visited.insert(n.clone()) {
                    continue;
                }
                group.insert(n.clone());
                if let Some(next) = graph.get(&n) {
                    stack.extend(next.iter().cloned());
                }
            }
            nets.push(group.into_iter().collect());
        }
        Ok(Connectivity {
            project: self.id,
            nets,
        })
    }
}

/// Whole-project connectivity report. Aliases remain scoped display data, including
/// different aliases on one hierarchically connected net; no arbitrary winning name.
pub fn report(project: &Project, incarnation: ProjectId, revision: u64) -> Result<String> {
    let connectivity = project.connectivity()?;
    let mut components = vec![];
    for o in &project.occurrences {
        if let Some(d) = project.document(o.id) {
            for s in &d.symbols {
                if let Some(c) = d.component(s.id) {
                    components.push(serde_json::json!({ "target": project.component_target(o.id, s.id), "reference": c.reference, "value": c.value, "sheet_path": project.path(o.id) }));
                }
            }
        }
    }
    let nets: Vec<_> = connectivity.nets.iter().map(|nodes| {
        let mut aliases = BTreeSet::new();
        for node in nodes { if let Some(d) = project.document(node.occurrence) { for w in &d.wires { if (w.from == node.endpoint || w.to == node.endpoint) && let Some(name) = &w.label { aliases.insert((node.occurrence, name)); } } } }
        serde_json::json!({ "nodes": nodes, "terminals": nodes.iter().filter_map(|n| project.terminal_target(n)).collect::<Vec<_>>(), "local_aliases": aliases.into_iter().collect::<Vec<_>>() })
    }).collect();
    serde_json::to_string_pretty(&serde_json::json!({ "format": "pcbcraft-project-connectivity-report/2", "project_id": project.id, "session_incarnation": incarnation, "revision": revision, "sheets": project.occurrences, "components": components, "nets": nets })).map_err(|e| Error::Io(e.to_string()))
}

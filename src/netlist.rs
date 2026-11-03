//! Derived connectivity for the bounded draft. Crossings/names never join pins.
//! Membership is rebuilt from explicit links; an anchor is a revision-local key,
//! not a persistent net identity across splits/merges.
use crate::document::*;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Net {
    pub anchor: Endpoint,
    pub nodes: Vec<Endpoint>,
    pub name: Option<String>,
    pub terminals: Vec<PinRef>,
    pub links: Vec<WireId>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Netlist {
    pub nets: Vec<Net>,
    pub unconnected: Vec<PinRef>,
}
impl Netlist {
    pub fn compile(doc: &Document) -> Result<Self> {
        doc.validate()?;
        Ok(groups(doc))
    }
    pub fn containing<T: Clone + Into<Endpoint>>(&self, node: &T) -> Option<&Net> {
        let node: Endpoint = node.clone().into();
        self.nets.iter().find(|n| n.nodes.contains(&node))
    }
}

pub(crate) fn groups(doc: &Document) -> Netlist {
    let mut adjacency = BTreeMap::<Endpoint, Vec<Endpoint>>::new();
    for w in &doc.wires {
        adjacency
            .entry(w.from.clone())
            .or_default()
            .push(w.to.clone());
        adjacency
            .entry(w.to.clone())
            .or_default()
            .push(w.from.clone());
    }
    let mut visited = BTreeSet::new();
    let mut nets = Vec::new();
    for start in adjacency.keys() {
        if visited.contains(start) {
            continue;
        }
        let mut stack = vec![start.clone()];
        let mut terminals = BTreeSet::new();
        while let Some(pin) = stack.pop() {
            if !visited.insert(pin.clone()) {
                continue;
            }
            terminals.insert(pin.clone());
            if let Some(neighbors) = adjacency.get(&pin) {
                stack.extend(neighbors.iter().cloned());
            }
        }
        let links: Vec<_> = doc
            .wires
            .iter()
            .filter(|w| terminals.contains(&w.from))
            .collect();
        let name = links.iter().find_map(|w| w.label.clone());
        let mut links: Vec<_> = links.into_iter().map(|w| w.id).collect();
        links.sort();
        nets.push(Net {
            anchor: start.clone(),
            name,
            terminals: terminals.iter().filter_map(|n| n.pin().cloned()).collect(),
            nodes: terminals.into_iter().collect(),
            links,
        });
    }
    let mut unconnected = Vec::new();
    for s in &doc.symbols {
        if let Some(c) = doc.component(s.id) {
            for p in c.kind.pins() {
                let pin = PinRef {
                    symbol: s.id,
                    terminal: p.id.into(),
                };
                if !visited.contains(&Endpoint::from(pin.clone())) {
                    unconnected.push(pin);
                }
            }
        }
    }
    unconnected.sort();
    Netlist { nets, unconnected }
}
pub fn terminal_name(doc: &Document, pin: &PinRef) -> String {
    format!(
        "{}.{}",
        doc.component(pin.symbol)
            .map_or("?", |c| c.reference.as_str()),
        pin.terminal
    )
}
pub fn endpoint_name(doc: &Document, endpoint: &Endpoint) -> String {
    match endpoint {
        Endpoint::Pin(pin) => terminal_name(doc, pin),
        Endpoint::Junction(j) => doc
            .junctions
            .iter()
            .position(|n| n.id == j.junction)
            .map_or_else(
                || "Missing junction".into(),
                |i| format!("Wire point {}", i + 1),
            ),
    }
}
impl Net {
    pub fn title(&self, doc: &Document) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| format!("Unnamed · {}", endpoint_name(doc, &self.anchor)))
    }
}

/// Human-readable report, deliberately distinct from native projects or external netlists.
pub fn report(doc: &Document, incarnation: ProjectId, revision: u64) -> Result<String> {
    let netlist = Netlist::compile(doc)?;
    let terminal = |p: &PinRef| serde_json::json!({"symbol_id":p.symbol,"reference":doc.component(p.symbol).map(|c|&c.reference),"pin":p.terminal});
    let mut components: Vec<_> = doc.components.iter().collect();
    components.sort_by_key(|c| c.id);
    let nets:Vec<_>=netlist.nets.iter().map(|n|serde_json::json!({"name":n.name,"terminals":n.terminals.iter().map(terminal).collect::<Vec<_>>(),"link_ids":n.links,"wire_points":n.nodes.iter().filter(|e| e.pin().is_none()).collect::<Vec<_>>()})).collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "format":"pcbcraft-connectivity-report/1","project_id":doc.id,
        "session_incarnation":incarnation,"revision":revision,"components":components,
        "nets":nets,"unconnected":netlist.unconnected.iter().map(terminal).collect::<Vec<_>>()
    }))
    .map_err(|e| Error::Io(e.to_string()))
}

pub(crate) fn rename(doc: &mut Document, anchor: &Endpoint, name: &str) -> Result<()> {
    let net = groups(doc)
        .containing(anchor)
        .cloned()
        .ok_or("Net no longer exists.")?;
    let name = name.trim();
    if net.name.as_deref().unwrap_or_default() == name {
        return Ok(());
    }
    if name.len() > 32 || name.chars().any(char::is_control) {
        return Err("Net name must be at most 32 printable bytes.".into());
    }
    for w in &mut doc.wires {
        if net.links.contains(&w.id) {
            // Keep the existing path/alias presentation when naming the logical group.
            if w.label.is_none() {
                w.draw_path = true;
            }
            w.label = (!name.is_empty()).then(|| name.to_owned());
        }
    }
    Ok(())
}

pub struct DisconnectPreview {
    pub document: Document,
    pub fragments: usize,
    pub unnamed_fragments: usize,
    pub name_removed: bool,
}
/// Removing an edge may split a named group. Retain its name on the largest
/// linked fragment (stable anchor tie-break), clearing display aliases elsewhere.
/// UI must present this consequence before applying the revision-checked command.
pub fn preview_disconnect(doc: &Document, wire: WireId) -> Result<DisconnectPreview> {
    doc.validate()?;
    let edge = doc
        .wires
        .iter()
        .find(|w| w.id == wire)
        .ok_or("Connection no longer exists.")?;
    let old = groups(doc)
        .containing(&edge.from)
        .cloned()
        .ok_or("Net no longer exists.")?;
    let mut next = doc.clone();
    next.wires.retain(|w| w.id != wire);
    crate::wiring::prune_junctions(&mut next);
    let groups = groups(&next);
    let mut fragments: Vec<_> = groups
        .nets
        .iter()
        .filter(|n| old.nodes.contains(&n.anchor))
        .collect();
    fragments.sort_by(|a, b| {
        b.nodes
            .len()
            .cmp(&a.nodes.len())
            .then(a.anchor.cmp(&b.anchor))
    });
    let isolated = groups
        .unconnected
        .iter()
        .filter(|p| old.terminals.contains(p))
        .count();
    if let (Some(name), Some(primary)) = (&old.name, fragments.first()) {
        for w in &mut next.wires {
            if primary.links.contains(&w.id) {
                if w.label.is_none() {
                    w.draw_path = true;
                }
                w.label = Some(name.clone());
            }
        }
    }
    let name_removed = old.name.is_some() && fragments.is_empty();
    let mut unnamed_fragments = 0;
    for n in fragments.iter().skip(1) {
        if old.name.is_some() {
            unnamed_fragments += 1;
        }
        for w in &mut next.wires {
            if n.links.contains(&w.id) {
                w.label = None;
                w.draw_path = true;
            }
        }
    }
    next.validate()?;
    Ok(DisconnectPreview {
        document: next,
        fragments: fragments.len() + isolated,
        unnamed_fragments,
        name_removed,
    })
}

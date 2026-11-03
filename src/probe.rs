//! Renderer-independent selection exchange seam for a future layout view.
//! A projection consumes a snapshot; it must not echo it as a new user action.
use crate::{
    document::*,
    project::{ComponentTarget, Node, TerminalTarget},
    selection::Selection,
    session::Session,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub project: ProjectId,
    pub incarnation: ProjectId,
    pub revision: u64,
    pub occurrence: SheetOccurrenceId,
    pub components: Vec<ComponentTarget>,
    pub terminals: Vec<TerminalTarget>,
}
impl Session {
    /// Explicit selected component and net terminal targets. Net membership is revision-local.
    pub fn probe_selection(
        &self,
        occurrence: SheetOccurrenceId,
        selection: &Selection,
        net: Option<&Endpoint>,
    ) -> Result<Snapshot> {
        let project = self.project();
        let doc = project
            .document(occurrence)
            .ok_or("Sheet no longer exists.")?;
        let mut retained = selection.clone();
        retained.retain(doc);
        if retained != *selection {
            return Err("Selection contains missing items.".into());
        }
        if net.is_some_and(|e| doc.endpoint(e).is_none()) {
            return Err("Net anchor no longer exists.".into());
        }
        let components = selection
            .symbols
            .iter()
            .filter_map(|id| project.component_target(occurrence, *id))
            .collect();
        let mut anchors: BTreeSet<_> = doc
            .wires
            .iter()
            .filter(|w| selection.wires.contains(&w.id))
            .map(|w| Node {
                occurrence,
                endpoint: w.from.clone(),
            })
            .collect();
        if let Some(e) = net {
            anchors.insert(Node {
                occurrence,
                endpoint: e.clone(),
            });
        }
        let mut terminals = BTreeSet::new();
        if !anchors.is_empty() {
            for group in project.connectivity()?.nets {
                if group.iter().any(|n| anchors.contains(n)) {
                    terminals.extend(group.iter().filter_map(|n| project.terminal_target(n)));
                }
            }
        }
        Ok(Snapshot {
            project: project.id,
            incarnation: self.incarnation,
            revision: self.revision(),
            occurrence,
            components,
            terminals: terminals.into_iter().collect(),
        })
    }
    pub fn validate_probe(&self, snapshot: &Snapshot) -> Result<()> {
        if snapshot.project != self.project().id || snapshot.incarnation != self.incarnation {
            return Err("Selection belongs to a different project session.".into());
        }
        if snapshot.revision != self.revision() {
            return Err(Error::StaleRevision {
                expected: snapshot.revision,
                actual: self.revision(),
            });
        }
        if self.project().occurrence(snapshot.occurrence).is_none()
            || snapshot
                .components
                .iter()
                .any(|c| self.project().resolve_component(c).is_none())
        {
            return Err("Selection contains missing targets.".into());
        }
        for p in &snapshot.terminals {
            let symbol = self
                .project()
                .resolve_component(&p.component)
                .ok_or("Missing physical component.")?;
            if self
                .project()
                .document(p.component.occurrence)
                .and_then(|d| {
                    d.pin(&PinRef {
                        symbol,
                        terminal: p.terminal.clone(),
                    })
                })
                .is_none()
            {
                return Err("Missing terminal.".into());
            }
        }
        Ok(())
    }
}

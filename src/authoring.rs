//! Original schematic proposals. Transport and GUI share this session authority.
use crate::{
    document::*,
    project::Project,
    session::{Command, Session},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAX_OPERATIONS: usize = 64;
pub const MAX_PLANS: usize = 8;
pub const MAX_RECEIPTS: usize = 256;

/// Bound editor-thread work, including repeated sheet expansion.
pub(crate) fn check_budget(project: &Project) -> Result<()> {
    let objects: usize = std::iter::once(&project.main)
        .chain(&project.sheets)
        .map(|d| {
            d.symbols.len()
                + d.wires.len()
                + d.annotations.len()
                + d.junctions.len()
                + d.ports.len()
                + d.sheet_symbols.len()
        })
        .sum();
    let expanded: usize = project
        .occurrences
        .iter()
        .filter_map(|o| project.document(o.id))
        .map(|d| {
            d.components
                .iter()
                .map(|c| c.kind.pins().len())
                .sum::<usize>()
                + 2 * d.wires.len()
                + d.ports.len()
        })
        .sum();
    if objects > 4096 || expanded > 32768 {
        return Err("This synchronous profile supports 4096 project objects and 32768 expanded terminals/link endpoints. Larger projects need background preparation/analysis.".into());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub project: ProjectId,
    pub incarnation: ProjectId,
    pub revision: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolRef {
    Id(SymbolId),
    Local(String),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terminal {
    pub symbol: SymbolRef,
    pub terminal: String,
}

/// Proposal-local limits, not persisted document locks or page settings.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrangementRegion {
    pub min: Point,
    pub max: Point,
}

/// Coordinates are integer nanometres in the schematic, never screen pixels.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Place {
        key: String,
        kind: PartKind,
        at: Point,
        reference: Option<String>,
        value: Option<String>,
        #[serde(default)]
        quarter_turns: u8,
    },
    Move {
        symbol: SymbolRef,
        at: Point,
    },
    Edit {
        symbol: SymbolRef,
        reference: String,
        value: String,
        quarter_turns: u8,
    },
    Delete {
        symbol: SymbolRef,
    },
    Connect {
        from: Terminal,
        to: Terminal,
        bends: Option<Vec<Point>>,
    },
    Disconnect {
        wire: WireId,
    },
    RenameNet {
        anchor: Terminal,
        name: String,
    },
    Note {
        at: Point,
        text: String,
    },
    /// Ordered rows/columns. It never invents connectivity or moves unlisted parts.
    Arrange {
        symbols: Vec<SymbolRef>,
        origin: Point,
        columns: usize,
        spacing: Point,
    },
    /// Replan direct connections inside a bounded, explicitly ordered selection.
    ArrangeConnected {
        symbols: Vec<SymbolRef>,
        locked: Vec<SymbolRef>,
        region: ArrangementRegion,
        origin: Point,
        columns: usize,
        spacing: Point,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prepare {
    pub expected: Snapshot,
    pub sheet: SheetOccurrenceId,
    pub summary: String,
    pub operations: Vec<Operation>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Proposal {
    pub handle: ProjectId,
    pub base: Snapshot,
    pub sheet: SheetOccurrenceId,
    pub summary: String,
    pub operations: Vec<Operation>,
    pub created: BTreeMap<String, SymbolId>,
    pub affected_occurrences: Vec<SheetOccurrenceId>,
    pub findings: Vec<String>,
}
pub(crate) struct Plan {
    pub info: Proposal,
    candidate: Project,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Apply {
    pub request_id: String,
    pub expected: Snapshot,
    pub handle: ProjectId,
}
#[derive(Clone, Debug, Serialize)]
pub struct Receipt {
    pub request_id: String,
    pub before: Snapshot,
    pub after: Snapshot,
}
#[derive(Default)]
pub(crate) struct State {
    plans: Vec<Plan>,
    receipts: Vec<(Apply, Receipt)>,
}

impl Session {
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            project: self.project().id,
            incarnation: self.incarnation,
            revision: self.revision(),
        }
    }
    pub fn require_snapshot(&self, expected: &Snapshot) -> Result<()> {
        if expected != &self.snapshot() {
            return Err(
                "Conflict: project, session or revision changed. Inspect and prepare again.".into(),
            );
        }
        Ok(())
    }
    pub fn proposals(&self) -> Vec<Proposal> {
        self.authoring
            .plans
            .iter()
            .map(|p| p.info.clone())
            .collect()
    }
    pub fn proposal_project(&self, handle: ProjectId) -> Result<&Project> {
        self.authoring
            .plans
            .iter()
            .find(|p| p.info.handle == handle)
            .map(|p| &p.candidate)
            .ok_or_else(|| "Unknown or expired proposal.".into())
    }
    pub fn discard_proposal(&mut self, handle: ProjectId) {
        self.authoring.plans.retain(|p| p.info.handle != handle);
    }
    pub fn receipt(&self, request_id: &str) -> Option<&Receipt> {
        self.authoring
            .receipts
            .iter()
            .find(|(a, _)| a.request_id == request_id)
            .map(|(_, r)| r)
    }
    pub fn prepare(&mut self, request: Prepare) -> Result<Proposal> {
        self.require_snapshot(&request.expected)?;
        check_budget(self.project())?;
        if request.summary.trim().is_empty()
            || request.summary.len() > 240
            || request.operations.is_empty()
            || request.operations.len() > MAX_OPERATIONS
        {
            return Err("Provide a summary (1–240 bytes) and 1–64 operations.".into());
        }
        self.authoring
            .plans
            .retain(|p| p.info.base == request.expected);
        if self.authoring.plans.len() >= MAX_PLANS {
            return Err(
                "Eight proposals are pending. Discard one before preparing another.".into(),
            );
        }
        let before = self
            .project()
            .document(request.sheet)
            .ok_or("Unknown sheet occurrence.")?;
        let definition = before.sheet_id;
        let mut candidate = self.project().clone();
        let mut created = BTreeMap::new();
        for operation in &request.operations {
            apply_operation(&mut candidate, request.sheet, operation, &mut created)?;
        }
        candidate.validate()?;
        check_budget(&candidate)?;
        let mut findings = vec!["Structural validation passed. ERC, voltage/current checks and manufacturing validation are not implemented.".into()];
        if request
            .operations
            .iter()
            .any(|op| matches!(op, Operation::ArrangeConnected { .. }))
        {
            findings.push("Each connected-grid step preserves terminal membership, fixed symbols and outside geometry. Internal drawn pin-to-pin paths are replanned inside the region, avoiding catalog bodies, unrelated pins, junctions and shared wire segments. Crossings can remain; exact text clearance and functional-group inference are not implemented.".into());
        } else {
            findings.push("Arrangement uses ordered grid spacing and conservative symbol bounds. Full text clearance, net-crossing optimization and external libraries are not implemented; visually review before applying.".into());
        }
        let info = Proposal {
            handle: ProjectId::fresh()?,
            base: self.snapshot(),
            sheet: request.sheet,
            summary: request.summary,
            operations: request.operations,
            created,
            affected_occurrences: candidate
                .occurrences
                .iter()
                .filter(|o| o.definition == definition)
                .map(|o| o.id)
                .collect(),
            findings,
        };
        self.authoring.plans.push(Plan {
            info: info.clone(),
            candidate,
        });
        Ok(info)
    }
    pub fn apply_proposal(&mut self, request: Apply) -> Result<Receipt> {
        if request.request_id.is_empty() || request.request_id.len() > 128 {
            return Err("request_id must contain 1–128 bytes.".into());
        }
        if let Some((old, result)) = self
            .authoring
            .receipts
            .iter()
            .find(|(a, _)| a.request_id == request.request_id)
        {
            return if old == &request {
                Ok(result.clone())
            } else {
                Err("request_id was already used with different arguments.".into())
            };
        }
        self.require_snapshot(&request.expected)?;
        if self.authoring.receipts.len() >= MAX_RECEIPTS {
            return Err("Session reached 256 AI commits. Save and reopen before further AI edits; receipts are never silently evicted.".into());
        }
        let plan = self
            .authoring
            .plans
            .iter()
            .find(|p| p.info.handle == request.handle)
            .ok_or("Unknown or discarded proposal.")?;
        if plan.info.base != request.expected {
            return Err("Proposal base does not match expected snapshot.".into());
        }
        let candidate = plan.candidate.clone();
        let before = self.snapshot();
        self.commit(candidate)?;
        let receipt = Receipt {
            request_id: request.request_id.clone(),
            before,
            after: self.snapshot(),
        };
        self.authoring
            .receipts
            .push((request.clone(), receipt.clone()));
        self.discard_proposal(request.handle);
        Ok(receipt)
    }
}

fn symbol(r: &SymbolRef, created: &BTreeMap<String, SymbolId>) -> Result<SymbolId> {
    match r {
        SymbolRef::Id(id) => Ok(*id),
        SymbolRef::Local(key) => created
            .get(key)
            .copied()
            .ok_or_else(|| format!("Unknown local symbol '{key}'; place it first.").into()),
    }
}
fn terminal(t: &Terminal, created: &BTreeMap<String, SymbolId>) -> Result<PinRef> {
    Ok(PinRef {
        symbol: symbol(&t.symbol, created)?,
        terminal: t.terminal.clone(),
    })
}
fn apply_operation(
    project: &mut Project,
    sheet: SheetOccurrenceId,
    operation: &Operation,
    created: &mut BTreeMap<String, SymbolId>,
) -> Result<()> {
    let command = match operation {
        Operation::Place {
            key,
            kind,
            at,
            reference,
            value,
            quarter_turns,
        } => {
            if key.is_empty() || key.len() > 64 || created.contains_key(key) {
                return Err("Place keys must be unique and contain 1–64 bytes.".into());
            }
            let id = Session::edit_candidate(
                project,
                sheet,
                Command::Place {
                    kind: *kind,
                    at: *at,
                },
            )?
            .ok_or("Placement produced no symbol.")?;
            let c = project
                .document(sheet)
                .and_then(|d| d.component(id))
                .ok_or("Missing placed component.")?;
            let edit = Command::SetProperties {
                symbol: id,
                reference: reference.clone().unwrap_or_else(|| c.reference.clone()),
                value: value.clone().unwrap_or_else(|| c.value.clone()),
                quarter_turns: *quarter_turns,
            };
            Session::edit_candidate(project, sheet, edit)?;
            created.insert(key.clone(), id);
            return Ok(());
        }
        Operation::Move { symbol: r, at } => Command::Move {
            symbol: symbol(r, created)?,
            at: *at,
        },
        Operation::Edit {
            symbol: r,
            reference,
            value,
            quarter_turns,
        } => Command::SetProperties {
            symbol: symbol(r, created)?,
            reference: reference.clone(),
            value: value.clone(),
            quarter_turns: *quarter_turns,
        },
        Operation::Delete { symbol: r } => Command::Delete {
            symbol: symbol(r, created)?,
        },
        Operation::Connect { from, to, bends } => {
            let from = terminal(from, created)?;
            let to = terminal(to, created)?;
            let doc = project.document(sheet).ok_or("Missing sheet.")?;
            let bends = match bends {
                Some(points) => points.clone(),
                None => crate::authoring::layout::route(doc, &from, &to)?,
            };
            Command::DrawWire {
                from: crate::wiring::Anchor::Existing(from.into()),
                to: crate::wiring::Anchor::Existing(to.into()),
                bends,
            }
        }
        Operation::Disconnect { wire } => Command::Disconnect { wire: *wire },
        Operation::RenameNet { anchor, name } => Command::RenameNet {
            anchor: terminal(anchor, created)?.into(),
            name: name.clone(),
        },
        Operation::Note { at, text } => Command::AddNote {
            at: *at,
            text: text.clone(),
        },
        Operation::Arrange {
            symbols,
            origin,
            columns,
            spacing,
        } => {
            let ids = symbols
                .iter()
                .map(|r| symbol(r, created))
                .collect::<Result<Vec<_>>>()?;
            layout::arrange(project, sheet, &ids, *origin, *columns, *spacing)?;
            return Ok(());
        }
        Operation::ArrangeConnected {
            symbols,
            locked,
            region,
            origin,
            columns,
            spacing,
        } => {
            let resolve = |refs: &[SymbolRef]| {
                refs.iter()
                    .map(|r| symbol(r, created))
                    .collect::<Result<Vec<_>>>()
            };
            connected::arrange(
                project,
                sheet,
                &resolve(symbols)?,
                &resolve(locked)?,
                *region,
                *origin,
                *columns,
                *spacing,
            )?;
            return Ok(());
        }
    };
    Session::edit_candidate(project, sheet, command)?;
    Ok(())
}

mod connected;
mod layout;
#[cfg(test)]
mod tests;

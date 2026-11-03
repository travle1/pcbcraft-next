//! The sole authoring/undo authority, usable without a display or application host.
use crate::document::*;

#[derive(Clone, Debug)]
pub enum Command {
    AddNote {
        at: Point,
        text: String,
    },
    SetText {
        target: crate::text::TextTarget,
        content: String,
        appearance: crate::text::Appearance,
    },
    DeleteItems {
        items: crate::selection::Selection,
    },
    MoveItems {
        items: crate::selection::Selection,
        delta: Point,
        routes: Vec<crate::wiring::Projection>,
    },
    Place {
        kind: PartKind,
        at: Point,
    },
    Move {
        symbol: SymbolId,
        at: Point,
    },
    Rotate {
        symbol: SymbolId,
    },
    Edit {
        symbol: SymbolId,
        reference: String,
        value: String,
    },
    SetProperties {
        symbol: SymbolId,
        reference: String,
        value: String,
        quarter_turns: u8,
    },
    Delete {
        symbol: SymbolId,
    },
    RenameNet {
        anchor: Endpoint,
        name: String,
    },
    Disconnect {
        wire: WireId,
    },
    EditWire {
        edit: crate::wiring::WireEdit,
    },
    DrawWire {
        from: crate::wiring::Anchor,
        to: crate::wiring::Anchor,
        bends: Vec<Point>,
    },
    DeleteNet {
        anchor: Endpoint,
    },
    Connect {
        from: PinRef,
        to: PinRef,
    },
}
pub struct Session {
    doc: crate::project::Project,
    saved: crate::project::Project,
    undo: Vec<crate::project::Project>,
    redo: Vec<crate::project::Project>,
    revision: u64,
    pub incarnation: ProjectId,
}
impl Session {
    pub fn new(doc: Document) -> Result<Self> {
        Self::from_project(crate::project::Project::from_document(doc)?)
    }
    pub fn from_project(doc: crate::project::Project) -> Result<Self> {
        doc.validate()?;
        Ok(Self {
            saved: doc.clone(),
            doc,
            undo: Vec::new(),
            redo: Vec::new(),
            revision: 0,
            incarnation: ProjectId::fresh()?,
        })
    }
    /// Legacy convenience accessor for the primary definition, never the GUI active sheet.
    pub fn document(&self) -> &Document {
        &self.doc.main
    }
    pub fn project(&self) -> &crate::project::Project {
        &self.doc
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn dirty(&self) -> bool {
        self.doc != self.saved
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    /// Legacy single-sheet save marker. Project IO must use `mark_project_saved`.
    pub fn mark_saved(&mut self, snapshot: Document) {
        self.saved.main = snapshot;
    }
    pub fn mark_project_saved(&mut self, snapshot: crate::project::Project) {
        self.saved = snapshot;
    }
    fn check_revision(&self, expected: u64) -> Result<()> {
        if expected != self.revision {
            return Err(Error::StaleRevision {
                expected,
                actual: self.revision,
            });
        }
        Ok(())
    }
    fn commit(&mut self, next: crate::project::Project) -> Result<()> {
        next.validate()?;
        if next != self.doc {
            let revision = self
                .revision
                .checked_add(1)
                .ok_or(Error::RevisionExhausted)?;
            self.undo.push(std::mem::replace(&mut self.doc, next));
            if self.undo.len() > 32 {
                self.undo.remove(0);
            }
            self.redo.clear();
            self.revision = revision;
        }
        Ok(())
    }
    pub fn project_command(
        &mut self,
        expected: u64,
        command: crate::project::Command,
    ) -> Result<crate::project::Effect> {
        self.check_revision(expected)?;
        let mut next = self.doc.clone();
        let effect = next.edit(command)?;
        self.commit(next)?;
        Ok(effect)
    }
    /// Legacy primary-sheet edit. New callers should supply an occurrence to `apply_on`.
    pub fn apply(&mut self, expected: u64, command: Command) -> Result<Option<SymbolId>> {
        self.apply_on(expected, self.doc.root, command)
    }
    pub fn apply_on(
        &mut self,
        expected: u64,
        occurrence: SheetOccurrenceId,
        command: Command,
    ) -> Result<Option<SymbolId>> {
        if expected != self.revision {
            return Err(Error::StaleRevision {
                expected,
                actual: self.revision,
            });
        }
        let repair = matches!(
            &command,
            Command::Move { .. } | Command::Rotate { .. } | Command::SetProperties { .. }
        );
        let before = self
            .doc
            .document(occurrence)
            .ok_or("Sheet no longer exists.")?;
        let mut next = before.clone();
        let mut created = None;
        match command {
            Command::AddNote { at, text } => next.annotations.push(Annotation {
                id: AnnotationId::fresh()?,
                at,
                text,
            }),
            Command::SetText {
                target,
                content,
                appearance,
            } => crate::text::edit(&mut next, target, content, appearance)?,
            Command::DeleteItems { items } => crate::selection::delete(&mut next, &items)?,
            Command::MoveItems {
                items,
                delta,
                routes,
            } => next = crate::selection::moved(&next, &items, delta, &routes)?,
            Command::Place { kind, at } => {
                if kind.is_block() {
                    next.builtin_definitions_version = 2;
                }
                let id = ComponentId::fresh()?;
                let sid = SymbolId::fresh()?;
                let reference = (1..=MAX_ITEMS + 1)
                    .map(|n| format!("{}{n}", kind.prefix()))
                    .find(|r| {
                        !next
                            .components
                            .iter()
                            .any(|c| c.reference.eq_ignore_ascii_case(r))
                    })
                    .ok_or("No free reference.")?;
                next.components.push(Component {
                    id,
                    kind,
                    reference,
                    value: kind.value().into(),
                });
                next.symbols.push(Symbol {
                    id: sid,
                    component: id,
                    at,
                    quarter_turns: 0,
                });
                created = Some(sid);
            }
            Command::Move { symbol, at } => {
                next.symbols
                    .iter_mut()
                    .find(|s| s.id == symbol)
                    .ok_or("Symbol no longer exists.")?
                    .at = at
            }
            Command::Rotate { symbol } => {
                let s = next
                    .symbols
                    .iter_mut()
                    .find(|s| s.id == symbol)
                    .ok_or("Symbol no longer exists.")?;
                s.quarter_turns = (s.quarter_turns + 1) % 4;
            }
            Command::Edit {
                symbol,
                reference,
                value,
            } => {
                let cid = next
                    .symbols
                    .iter()
                    .find(|s| s.id == symbol)
                    .ok_or("Symbol no longer exists.")?
                    .component;
                let c = next
                    .components
                    .iter_mut()
                    .find(|c| c.id == cid)
                    .ok_or("Component no longer exists.")?;
                c.reference = reference.trim().into();
                c.value = value.trim().into();
            }
            Command::SetProperties {
                symbol,
                reference,
                value,
                quarter_turns,
            } => {
                let placement = next
                    .symbols
                    .iter_mut()
                    .find(|s| s.id == symbol)
                    .ok_or("Symbol no longer exists.")?;
                placement.quarter_turns = quarter_turns;
                let component = next
                    .components
                    .iter_mut()
                    .find(|c| c.id == placement.component)
                    .ok_or("Component no longer exists.")?;
                component.reference = reference.trim().into();
                component.value = value.trim().into();
            }
            Command::Delete { symbol } => {
                let cid = next
                    .symbols
                    .iter()
                    .find(|s| s.id == symbol)
                    .ok_or("Symbol no longer exists.")?
                    .component;
                next.symbols.retain(|s| s.id != symbol);
                next.components.retain(|c| c.id != cid);
                next.wires
                    .retain(|w| !w.from.on_symbol(symbol) && !w.to.on_symbol(symbol));
                crate::wiring::prune_junctions(&mut next);
                crate::wiring::normalize_names(&mut next);
            }
            Command::RenameNet { anchor, name } => {
                crate::netlist::rename(&mut next, &anchor, &name)?
            }
            Command::Disconnect { wire } => {
                next = crate::netlist::preview_disconnect(&next, wire)?.document
            }
            Command::EditWire { edit } => next = crate::wiring::preview_edit(&next, &edit)?,
            Command::DrawWire { from, to, bends } => {
                crate::wiring::draw(&mut next, from, to, bends)?;
                crate::wiring::heal_overlaps(before, &mut next)?;
            }
            Command::DeleteNet { anchor } => {
                let net = crate::netlist::groups(&next)
                    .containing(&anchor)
                    .cloned()
                    .ok_or("Net no longer exists.")?;
                next.wires.retain(|w| !net.links.contains(&w.id));
                crate::wiring::prune_junctions(&mut next);
            }
            Command::Connect { from, to } => next.wires.push(Wire {
                id: WireId::fresh()?,
                from: from.into(),
                to: to.into(),
                label: None,
                draw_path: false,
                bends: None,
            }),
        }
        if repair {
            crate::wiring::repair_moved_endpoints(before, &mut next)?;
        }
        crate::text::prune(&mut next);
        next.validate()?;
        let definition = next.sheet_id;
        let mut project = self.doc.clone();
        *project
            .definition_mut(definition)
            .ok_or("Missing sheet definition.")? = next;
        self.commit(project)?;
        Ok(created)
    }
    pub fn undo(&mut self) -> Result<()> {
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(Error::RevisionExhausted)?;
        if let Some(doc) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.doc, doc));
            self.revision = revision;
        }
        Ok(())
    }
    pub fn redo(&mut self) -> Result<()> {
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(Error::RevisionExhausted)?;
        if let Some(doc) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.doc, doc));
            self.revision = revision;
        }
        Ok(())
    }
    pub fn demo() -> Result<Self> {
        Self::new(crate::demo::fan_controller()?)
    }
}

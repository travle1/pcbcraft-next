use pcbcraft_foundation::{
    document::*,
    persistence,
    project::{self, Direction, Node, PortRef, SheetPinRef},
    selection::Selection,
    session::{Command, Session},
    wiring::Anchor,
};
fn create(
    s: &mut Session,
    name: &str,
    parent: Option<SheetOccurrenceId>,
    existing: Option<SheetId>,
) -> Result<SheetOccurrenceId> {
    s.project_command(
        s.revision(),
        project::Command::Create {
            name: name.into(),
            parent,
            existing,
            at: Point::default(),
        },
    )?
    .created
    .ok_or("No created sheet.".into())
}
fn port(s: &mut Session, occurrence: SheetOccurrenceId, name: &str) -> Result<PortId> {
    s.project_command(
        s.revision(),
        project::Command::SetPort {
            occurrence,
            id: None,
            name: name.into(),
            direction: Direction::Input,
            at: Point {
                x: -10 * GRID_NM,
                y: 0,
            },
        },
    )?;
    Ok(s.project()
        .document(occurrence)
        .ok_or("Missing sheet")?
        .ports
        .last()
        .ok_or("Missing port")?
        .id)
}
fn place(s: &mut Session, o: SheetOccurrenceId) -> Result<SymbolId> {
    s.apply_on(
        s.revision(),
        o,
        Command::Place {
            kind: PartKind::Resistor,
            at: Point::default(),
        },
    )?
    .ok_or("Missing symbol".into())
}
fn wire(s: &mut Session, o: SheetOccurrenceId, from: Endpoint, to: Endpoint) -> Result<()> {
    let d = s.project().document(o).ok_or("Missing sheet")?;
    let a = d.endpoint(&from).ok_or("Missing from")?;
    let b = d.endpoint(&to).ok_or("Missing to")?;
    let bends = if a.x != b.x && a.y != b.y {
        vec![Point { x: a.x, y: b.y }]
    } else {
        vec![]
    };
    s.apply_on(
        s.revision(),
        o,
        Command::DrawWire {
            from: Anchor::Existing(from),
            to: Anchor::Existing(to),
            bends,
        },
    )?;
    Ok(())
}
#[test]
fn flat_sheets_scope_edits_and_equal_net_names_without_joining() -> Result<()> {
    let mut s = Session::new(Document::empty("Multi sheet")?)?;
    let root = s.project().root;
    let second = create(&mut s, "Power", None, None)?;
    for o in [root, second] {
        let r = place(&mut s, o)?;
        wire(
            &mut s,
            o,
            PinRef {
                symbol: r,
                terminal: "1".into(),
            }
            .into(),
            PinRef {
                symbol: r,
                terminal: "2".into(),
            }
            .into(),
        )?;
        s.apply_on(
            s.revision(),
            o,
            Command::RenameNet {
                anchor: PinRef {
                    symbol: r,
                    terminal: "1".into(),
                }
                .into(),
                name: "LOCAL".into(),
            },
        )?;
    }
    assert_eq!(s.project().connectivity()?.nets.len(), 2);
    assert_eq!(s.document().components[0].reference, "R1");
    assert_eq!(
        s.project()
            .document(second)
            .ok_or("Missing fixture item")?
            .components[0]
            .reference,
        "R1"
    );
    let before = s.project().clone();
    s.project_command(
        s.revision(),
        project::Command::Delete { occurrence: second },
    )?;
    assert_eq!(s.project().occurrences.len(), 1);
    s.undo()?;
    assert_eq!(s.project(), &before);
    s.redo()?;
    assert_eq!(s.project().occurrences.len(), 1);
    Ok(())
}
#[test]
fn repeated_subsheets_bridge_only_explicit_ports_and_keep_physical_identity() -> Result<()> {
    let mut s = Session::new(Document::empty("Hierarchy")?)?;
    let root = s.project().root;
    let a = create(&mut s, "Channel A", Some(root), None)?;
    let p = port(&mut s, a, "INPUT")?;
    let r = place(&mut s, a)?;
    wire(
        &mut s,
        a,
        Endpoint::Port(PortRef { port: p }),
        PinRef {
            symbol: r,
            terminal: "1".into(),
        }
        .into(),
    )?;
    let definition = s
        .project()
        .document(a)
        .ok_or("Missing fixture item")?
        .sheet_id;
    let b = create(&mut s, "Channel B", Some(root), Some(definition))?;
    let at = s
        .project()
        .component_target(a, r)
        .ok_or("Missing fixture item")?;
    let bt = s
        .project()
        .component_target(b, r)
        .ok_or("Missing fixture item")?;
    assert_ne!(at, bt);
    assert_eq!(at.component, bt.component);
    let block_a = s
        .project()
        .occurrence(a)
        .ok_or("Missing fixture item")?
        .via
        .ok_or("Missing fixture item")?;
    let input = place(&mut s, root)?;
    wire(
        &mut s,
        root,
        PinRef {
            symbol: input,
            terminal: "2".into(),
        }
        .into(),
        Endpoint::SheetPin(SheetPinRef {
            sheet_symbol: block_a,
            port: p,
        }),
    )?;
    let snapshot = s.probe_selection(
        root,
        &Selection::default(),
        Some(
            &PinRef {
                symbol: input,
                terminal: "2".into(),
            }
            .into(),
        ),
    )?;
    assert!(snapshot.terminals.iter().any(|t| t.component == at));
    assert!(!snapshot.terminals.iter().any(|t| t.component == bt));
    s.validate_probe(&snapshot)?;
    let before = s.project().clone();
    s.project_command(
        s.revision(),
        project::Command::Rename {
            occurrence: a,
            name: "Renamed".into(),
        },
    )?;
    assert_eq!(
        s.project()
            .component_target(a, r)
            .ok_or("Missing fixture item")?,
        at
    );
    assert!(s.validate_probe(&snapshot).is_err());
    s.undo()?;
    assert_eq!(s.project(), &before);
    assert!(s.validate_probe(&snapshot).is_err());
    let fresh = s.probe_selection(
        a,
        &Selection {
            symbols: [r].into(),
            ..Default::default()
        },
        None,
    )?;
    let reopened = Session::from_project(persistence::decode_project(
        &persistence::encode_project(s.project())?,
    )?)?;
    assert!(reopened.validate_probe(&fresh).is_err());
    assert_eq!(reopened.project().resolve_component(&at), Some(r));
    Ok(())
}
#[test]
fn ports_update_all_parent_bindings_move_with_wires_and_remove_atomically() -> Result<()> {
    let mut s = Session::new(Document::empty("Ports")?)?;
    let root = s.project().root;
    let a = create(&mut s, "Child", Some(root), None)?;
    let p = port(&mut s, a, "IN")?;
    let r = place(&mut s, a)?;
    wire(
        &mut s,
        a,
        Endpoint::Port(PortRef { port: p }),
        PinRef {
            symbol: r,
            terminal: "1".into(),
        }
        .into(),
    )?;
    let def = s
        .project()
        .document(a)
        .ok_or("Missing fixture item")?
        .sheet_id;
    let b = create(&mut s, "Second", Some(root), Some(def))?;
    let parent_r = place(&mut s, root)?;
    let block = s
        .project()
        .occurrence(a)
        .ok_or("Missing fixture item")?
        .via
        .ok_or("Missing fixture item")?;
    wire(
        &mut s,
        root,
        PinRef {
            symbol: parent_r,
            terminal: "1".into(),
        }
        .into(),
        Endpoint::SheetPin(SheetPinRef {
            sheet_symbol: block,
            port: p,
        }),
    )?;
    s.project_command(
        s.revision(),
        project::Command::SetPort {
            occurrence: a,
            id: Some(p),
            name: "SENSE".into(),
            direction: Direction::Output,
            at: Point {
                x: -10 * GRID_NM,
                y: 5 * GRID_NM,
            },
        },
    )?;
    assert!(
        s.document()
            .sheet_symbols
            .iter()
            .all(|b| b.pins[0].name == "SENSE")
    );
    let before = s.project().clone();
    s.project_command(
        s.revision(),
        project::Command::MoveSheet {
            occurrence: root,
            symbol: block,
            at: Point {
                x: 10 * GRID_NM,
                y: 15 * GRID_NM,
            },
        },
    )?;
    assert_eq!(
        s.project().connectivity()?.nets,
        before.connectivity()?.nets
    );
    s.undo()?;
    assert_eq!(s.project(), &before);
    s.project_command(
        s.revision(),
        project::Command::DeletePort {
            occurrence: b,
            id: p,
        },
    )?;
    assert!(s.document().wires.is_empty());
    assert!(
        s.project()
            .document(a)
            .ok_or("Missing fixture item")?
            .wires
            .is_empty()
    );
    assert!(s.document().sheet_symbols.iter().all(|b| b.pins.is_empty()));
    s.undo()?;
    assert_eq!(s.project(), &before);
    Ok(())
}
#[test]
fn shared_parent_edits_create_distinct_recursive_occurrences_and_delete_consistently() -> Result<()>
{
    let mut s = Session::new(Document::empty("Repeated parent")?)?;
    let root = s.project().root;
    let a = create(&mut s, "A", Some(root), None)?;
    let def = s
        .project()
        .document(a)
        .ok_or("Missing fixture item")?
        .sheet_id;
    let b = create(&mut s, "B", Some(root), Some(def))?;
    let nested = create(&mut s, "Nested", Some(a), None)?;
    let block = s
        .project()
        .occurrence(nested)
        .ok_or("Missing fixture item")?
        .via
        .ok_or("Missing fixture item")?;
    let other = s
        .project()
        .child(b, block)
        .ok_or("Missing fixture item")?
        .id;
    assert_ne!(nested, other);
    let r = place(&mut s, nested)?;
    assert_ne!(
        s.project().component_target(nested, r),
        s.project().component_target(other, r)
    );
    let before = s.project().clone();
    s.project_command(
        s.revision(),
        project::Command::Delete { occurrence: nested },
    )?;
    assert!(s.project().child(b, block).is_none());
    assert!(
        s.project()
            .document(a)
            .ok_or("Missing fixture item")?
            .sheet_symbols
            .is_empty()
    );
    s.undo()?;
    assert_eq!(s.project(), &before);
    // Removing only A retains B's shared definition, contents and nested occurrence IDs.
    s.project_command(s.revision(), project::Command::Delete { occurrence: a })?;
    assert!(s.project().occurrence(nested).is_none());
    assert_eq!(
        s.project()
            .child(b, block)
            .ok_or("Missing fixture item")?
            .id,
        other
    );
    assert_eq!(
        s.project().resolve_component(
            &before
                .component_target(other, r)
                .ok_or("Missing fixture item")?
        ),
        Some(r)
    );
    Ok(())
}
#[test]
fn legacy_migration_and_complete_project_save_preserve_all_ids_and_guard_external_changes()
-> Result<()> {
    let original = Session::demo()?.document().clone();
    let legacy = persistence::encode(&original)?;
    let mut s = Session::from_project(persistence::decode_project(&legacy)?)?;
    assert_eq!(s.document(), &original);
    let second = create(&mut s, "Second", None, None)?;
    place(&mut s, second)?;
    let bytes = persistence::encode_project(s.project())?;
    assert!(
        persistence::decode(&bytes).is_err(),
        "Single-sheet API cannot silently drop other sheets"
    );
    assert_eq!(persistence::decode_project(&bytes)?, *s.project());
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("test.pc");
    let saved = persistence::write_project(&path, s.project(), None)?;
    std::fs::write(&path, &legacy)?;
    assert_eq!(
        persistence::write_project(&path, s.project(), Some(&saved)),
        Err(Error::FileChanged)
    );
    assert_eq!(std::fs::read(&path)?, legacy);
    Ok(())
}
#[test]
fn stale_cycles_wrong_scope_and_malformed_mappings_fail_without_mutation() -> Result<()> {
    let mut s = Session::new(Document::empty("Validation")?)?;
    let root = s.project().root;
    let a = create(&mut s, "Child", Some(root), None)?;
    let def = s.document().sheet_id;
    let p = port(&mut s, a, "IN")?;
    let r = place(&mut s, a)?;
    let before = s.project().clone();
    let revision = s.revision();
    assert!(create(&mut s, "Cycle", Some(a), Some(def)).is_err());
    assert!(
        s.project_command(
            revision - 1,
            project::Command::Rename {
                occurrence: a,
                name: "Stale".into()
            }
        )
        .is_err()
    );
    assert!(
        s.project_command(revision, project::Command::Delete { occurrence: root })
            .is_err()
    );
    assert!(
        s.apply_on(revision, root, Command::Rotate { symbol: r })
            .is_err()
    );
    assert!(
        s.project_command(
            revision,
            project::Command::SetPort {
                occurrence: a,
                id: Some(p),
                name: "IN".into(),
                direction: Direction::Input,
                at: Point { x: i64::MAX, y: 0 }
            }
        )
        .is_err()
    );
    assert_eq!(s.project(), &before);
    assert_eq!(s.revision(), revision);
    let mut broken = before.clone();
    broken.main.sheet_symbols[0].pins.clear();
    assert!(broken.validate().is_err());
    let mut broken = before.clone();
    broken.occurrences.retain(|o| o.id != a);
    assert!(broken.validate().is_err());
    let mut raw = serde_json::to_value(&before).map_err(|e| e.to_string())?;
    raw["main"]["sheet_symbols"][0]["at"]["x"] = serde_json::json!(i64::MAX);
    let broken: project::Project = serde_json::from_value(raw).map_err(|e| e.to_string())?;
    assert!(broken.validate().is_err());
    let node = Node {
        occurrence: a,
        endpoint: PinRef {
            symbol: r,
            terminal: "missing".into(),
        }
        .into(),
    };
    assert!(s.project().terminal_target(&node).is_none());
    Ok(())
}

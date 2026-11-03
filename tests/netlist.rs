use pcbcraft_foundation::{
    document::*,
    netlist::{self, Netlist},
    persistence,
    session::{Command, Session},
};
fn chain() -> Result<(Session, Vec<PinRef>)> {
    let mut s = Session::new(Document::empty("Original connectivity fixture")?)?;
    let mut pins = Vec::new();
    for _ in 0..4 {
        let id = s
            .apply(
                s.revision(),
                Command::Place {
                    kind: PartKind::Resistor,
                    at: Point::default(),
                },
            )?
            .ok_or("No symbol")?;
        pins.push(PinRef {
            symbol: id,
            terminal: "1".into(),
        });
    }
    for p in pins.windows(2) {
        s.apply(
            s.revision(),
            Command::Connect {
                from: p[0].clone(),
                to: p[1].clone(),
            },
        )?;
    }
    Ok((s, pins))
}
#[test]
fn compile_is_deterministic_explicit_and_reports_unconnected_pins() -> Result<()> {
    let (s, _) = chain()?;
    let n = Netlist::compile(s.document())?;
    assert_eq!(n.nets.len(), 1);
    assert_eq!(n.nets[0].terminals.len(), 4);
    assert_eq!(n.unconnected.len(), 4);
    // All symbols overlap geometrically; their other pins remain separate.
    let mut reordered = s.document().clone();
    reordered.wires.reverse();
    reordered.symbols.reverse();
    reordered.components.reverse();
    assert_eq!(n, Netlist::compile(&reordered)?);
    assert_eq!(
        netlist::report(s.document(), s.incarnation, s.revision())?,
        netlist::report(&reordered, s.incarnation, s.revision())?
    );
    let report: serde_json::Value =
        serde_json::from_str(&netlist::report(s.document(), s.incarnation, s.revision())?)
            .map_err(|e| e.to_string())?;
    assert_eq!(report["format"], "pcbcraft-connectivity-report/1");
    assert_eq!(
        report["unconnected"]
            .as_array()
            .ok_or("No unconnected list")?
            .len(),
        4
    );
    Ok(())
}
#[test]
fn naming_preserves_paths_rejects_conflicts_and_is_undoable() -> Result<()> {
    let (mut s, pins) = chain()?;
    let original = s.document().clone();
    s.apply(
        s.revision(),
        Command::RenameNet {
            anchor: pins[0].clone().into(),
            name: "CONTROL".into(),
        },
    )?;
    assert!(
        s.document()
            .wires
            .iter()
            .all(|w| w.draw_path && w.label.as_deref() == Some("CONTROL"))
    );
    let named = s.document().clone();
    let rev = s.revision();
    s.apply(
        rev,
        Command::RenameNet {
            anchor: pins[0].clone().into(),
            name: "CONTROL".into(),
        },
    )?;
    assert_eq!(s.revision(), rev);
    assert_eq!(named, persistence::decode(&persistence::encode(&named)?)?);
    // Old envelopes still parse old data, but cannot silently accept new path semantics.
    let text = String::from_utf8(persistence::encode(&named)?).map_err(|e| e.to_string())?;
    assert!(
        persistence::decode(
            text.replace(persistence::FORMAT, "pcbcraft-schematic-draft/2")
                .as_bytes()
        )
        .is_err()
    );
    s.undo()?;
    assert_eq!(s.document(), &original);
    s.redo()?;
    assert_eq!(s.document(), &named);
    let a = PinRef {
        terminal: "2".into(),
        ..pins[0].clone()
    };
    let b = PinRef {
        terminal: "2".into(),
        ..pins[1].clone()
    };
    s.apply(
        s.revision(),
        Command::Connect {
            from: a.clone(),
            to: b,
        },
    )?;
    let before = s.document().clone();
    let rev = s.revision();
    assert!(
        s.apply(
            rev,
            Command::RenameNet {
                anchor: a.into(),
                name: "CONTROL".into()
            }
        )
        .is_err()
    );
    assert_eq!(s.document(), &before);
    assert_eq!(s.revision(), rev);
    Ok(())
}
#[test]
fn disconnect_preview_split_apply_and_undo_agree() -> Result<()> {
    let (mut s, pins) = chain()?;
    s.apply(
        s.revision(),
        Command::RenameNet {
            anchor: pins[0].clone().into(),
            name: "SIGNAL".into(),
        },
    )?;
    let before = s.document().clone();
    let wire = before.wires[1].id;
    let preview = netlist::preview_disconnect(&before, wire)?;
    assert_eq!(preview.fragments, 2);
    assert_eq!(preview.unnamed_fragments, 1);
    assert_eq!(s.document(), &before, "Preview must not mutate");
    s.apply(s.revision(), Command::Disconnect { wire })?;
    assert_eq!(s.document(), &preview.document);
    let result = Netlist::compile(s.document())?;
    assert_eq!(result.nets.len(), 2);
    assert_eq!(result.nets.iter().filter(|n| n.name.is_some()).count(), 1);
    s.undo()?;
    assert_eq!(s.document(), &before);
    let unchanged = s.document().clone();
    let rev = s.revision();
    assert!(s.apply(rev - 1, Command::Disconnect { wire }).is_err());
    assert_eq!(s.document(), &unchanged);
    Ok(())
}
#[test]
fn removing_cycle_link_does_not_split_and_v2_read_preserves_aliases() -> Result<()> {
    let (mut s, pins) = chain()?;
    s.apply(
        s.revision(),
        Command::Connect {
            from: pins[0].clone(),
            to: pins[3].clone(),
        },
    )?;
    let preview = netlist::preview_disconnect(s.document(), s.document().wires[1].id)?;
    assert_eq!(preview.fragments, 1);
    assert_eq!(
        Netlist::compile(&preview.document)?.nets[0].terminals.len(),
        4
    );
    // A genuine v2 fixture: the new demo uses newer symbols, junctions and text styles.
    s.apply(
        s.revision(),
        Command::RenameNet {
            anchor: pins[0].clone().into(),
            name: "SIGNAL".into(),
        },
    )?;
    let mut legacy = s.document().clone();
    legacy.annotations.push(Annotation {
        id: AnnotationId::fresh()?,
        at: Point::default(),
        text: "Legacy note".into(),
    });
    for w in &mut legacy.wires {
        w.draw_path = false;
    }
    let mut raw = serde_json::to_value(&legacy).map_err(|e| e.to_string())?;
    if let Some(notes) = raw["annotations"].as_array_mut() {
        for note in notes {
            note.as_object_mut().ok_or("Invalid note")?.remove("id");
        }
    }
    let bytes = serde_json::to_vec(&serde_json::json!({
        "format": "pcbcraft-schematic-draft/2", "document": raw
    }))
    .map_err(|e| e.to_string())?;
    let mut decoded = persistence::decode(&bytes)?;
    for (old, migrated) in legacy.annotations.iter_mut().zip(&mut decoded.annotations) {
        old.id = migrated.id;
    }
    assert_eq!(decoded, legacy);
    Ok(())
}

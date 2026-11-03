use pcbcraft_foundation::{
    document::*,
    persistence,
    session::{Command, Session},
};

fn place(s: &mut Session, kind: PartKind, at: Point) -> Result<SymbolId> {
    s.apply(s.revision(), Command::Place { kind, at })?
        .ok_or_else(|| "No placed symbol".into())
}

#[test]
fn invalid_or_stale_edits_are_atomic() -> Result<()> {
    let mut s = Session::demo()?;
    let symbol = s.document().symbols.first().ok_or("No symbol")?.id;
    let before = s.document().clone();
    let revision = s.revision();
    assert!(matches!(
        s.apply(revision + 1, Command::Delete { symbol }),
        Err(Error::StaleRevision { .. })
    ));
    assert!(
        s.apply(
            revision,
            Command::Edit {
                symbol,
                reference: "R1".into(),
                value: "changed".into()
            }
        )
        .is_err()
    );
    assert!(
        s.apply(
            revision,
            Command::Move {
                symbol,
                at: Point { x: i64::MAX, y: 0 }
            }
        )
        .is_err()
    );
    assert_eq!(s.document(), &before);
    assert_eq!(s.revision(), revision);
    assert!(!s.can_undo());
    assert!(!s.dirty());
    Ok(())
}

#[test]
fn undo_redo_preserve_ids_and_advance_revision() -> Result<()> {
    let mut s = Session::new(Document::empty("Original fixture")?)?;
    let first = place(&mut s, PartKind::Resistor, Point::default())?;
    let placed = s.document().clone();
    s.mark_saved(placed.clone());
    s.apply(
        s.revision(),
        Command::Edit {
            symbol: first,
            reference: "R17".into(),
            value: "4.7k".into(),
        },
    )?;
    let edited = s.document().clone();
    s.undo()?;
    assert_eq!(s.document(), &placed);
    assert!(!s.dirty());
    s.redo()?;
    assert_eq!(s.document(), &edited);
    assert_eq!(s.revision(), 4);
    s.undo()?;
    s.undo()?;
    let second = place(&mut s, PartKind::Resistor, Point::default())?;
    assert_ne!(first, second);
    assert!(!s.can_redo());
    assert_eq!(s.revision(), 7);
    Ok(())
}

#[test]
fn deletion_removes_connections_and_undo_restores_whole_aggregate() -> Result<()> {
    let mut s = Session::demo()?;
    let before = s.document().clone();
    let symbol = before.symbols.first().ok_or("No connector")?.id;
    s.apply(s.revision(), Command::Delete { symbol })?;
    assert!(
        s.document()
            .wires
            .iter()
            .all(|w| !w.from.on_symbol(symbol) && !w.to.on_symbol(symbol))
    );
    s.document().validate()?;
    s.undo()?;
    assert_eq!(s.document(), &before);
    Ok(())
}

#[test]
fn connections_reject_missing_pins_and_reversed_duplicates() -> Result<()> {
    let mut s = Session::demo()?;
    let wire = s.document().wires.first().ok_or("No wire")?.clone();
    let before = s.document().clone();
    assert!(
        s.apply(
            s.revision(),
            Command::Connect {
                from: wire.to.pin().ok_or("Not a pin")?.clone(),
                to: wire.from.pin().ok_or("Not a pin")?.clone()
            }
        )
        .is_err()
    );
    let mut invalid = wire.from.pin().ok_or("Not a pin")?.clone();
    invalid.terminal = "99".into();
    assert!(
        s.apply(
            s.revision(),
            Command::Connect {
                from: invalid,
                to: wire.to.pin().ok_or("Not a pin")?.clone()
            }
        )
        .is_err()
    );
    assert_eq!(s.document(), &before);
    Ok(())
}

#[test]
fn no_op_does_not_dirty_or_consume_history() -> Result<()> {
    let mut s = Session::demo()?;
    let symbol = s.document().symbols.first().ok_or("No symbol")?.clone();
    let revision = s.revision();
    s.apply(
        revision,
        Command::Move {
            symbol: symbol.id,
            at: symbol.at,
        },
    )?;
    assert_eq!(s.revision(), revision);
    assert!(!s.can_undo());
    assert!(!s.dirty());
    for n in [i64::MIN, -LIMIT_NM, 0, LIMIT_NM, i64::MAX] {
        let point = Point { x: n, y: n }.snapped();
        assert!(point.valid());
        assert_eq!(point.x % GRID_NM, 0);
    }
    Ok(())
}

#[test]
fn native_round_trip_preserves_identity_but_reopens_a_new_session() -> Result<()> {
    let s = Session::demo()?;
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("original.pc");
    let bytes = persistence::write(&path, s.document(), None)?;
    let (doc, reopened_bytes) = persistence::read(&path)?;
    assert_eq!(doc, *s.document());
    assert_eq!(bytes, reopened_bytes);
    let reopened = Session::new(doc)?;
    assert_ne!(s.incarnation, reopened.incarnation);
    assert_eq!(reopened.revision(), 0);
    assert!(!reopened.can_undo());
    assert!(!reopened.dirty());
    Ok(())
}

#[test]
fn unknown_data_and_invalid_structure_fail_without_silent_loss() -> Result<()> {
    let s = Session::demo()?;
    let bytes = persistence::encode(s.document())?;
    let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    assert!(matches!(
        persistence::decode(text.replace(persistence::FORMAT, "future/2").as_bytes()),
        Err(Error::UnsupportedFormat(_))
    ));
    assert!(
        persistence::decode(text.replacen('{', "{\"future_field\":true,", 1).as_bytes()).is_err()
    );
    let mut doc = s.document().clone();
    doc.builtin_definitions_version = 999;
    assert!(persistence::encode(&doc).is_err());
    doc = s.document().clone();
    doc.symbols.pop();
    assert!(persistence::encode(&doc).is_err());
    assert!(persistence::decode(&vec![b' '; persistence::MAX_BYTES + 1]).is_err());
    Ok(())
}

#[test]
fn save_detects_external_changes_and_failed_save_preserves_file() -> Result<()> {
    let mut s = Session::demo()?;
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("safe.pc");
    let original = persistence::write(&path, s.document(), None)?;
    let mut external = s.document().clone();
    external.name = "External edit".into();
    let external_bytes = persistence::write(&path, &external, Some(&original))?;
    place(&mut s, PartKind::Capacitor, Point::default())?;
    assert!(matches!(
        persistence::write(&path, s.document(), Some(&original)),
        Err(Error::FileChanged)
    ));
    assert_eq!(std::fs::read(&path)?, external_bytes);
    external.symbols.clear();
    assert!(persistence::write(&path, &external, Some(&external_bytes)).is_err());
    assert_eq!(std::fs::read(&path)?, external_bytes);
    Ok(())
}

#[test]
fn fan_demo_has_distinct_supply_rails_and_a_series_led_path() -> Result<()> {
    let s = Session::demo()?;
    let d = s.document();
    assert_eq!(d.symbols.len(), 16);
    let pin = |reference: &str, terminal: &str| -> Result<PinRef> {
        let symbol = d
            .symbols
            .iter()
            .find(|s| d.component(s.id).is_some_and(|c| c.reference == reference))
            .ok_or("Missing demo part")?;
        Ok(PinRef {
            symbol: symbol.id,
            terminal: terminal.into(),
        })
    };
    let connected = |a: PinRef, b: PinRef| {
        let mut visited = Vec::new();
        let mut pending = vec![Endpoint::from(a)];
        while let Some(p) = pending.pop() {
            if p == Endpoint::from(b.clone()) {
                return true;
            }
            if visited.contains(&p) {
                continue;
            }
            for w in &d.wires {
                if w.from == p {
                    pending.push(w.to.clone());
                } else if w.to == p {
                    pending.push(w.from.clone());
                }
            }
            visited.push(p);
        }
        false
    };
    for (a, b) in [
        (("J1", "1"), ("J3", "2")),
        (("J1", "1"), ("C1", "1")),
        (("U2", "3"), ("C2", "1")),
        (("U2", "3"), ("C3", "1")),
        (("U2", "3"), ("C4", "1")),
        (("R4", "2"), ("D1", "1")),
        (("J1", "2"), ("D1", "2")),
        (("U1", "5"), ("R2", "1")),
        (("R2", "2"), ("J3", "4")),
    ] {
        assert!(connected(pin(a.0, a.1)?, pin(b.0, b.1)?));
    }
    for (a, b) in [
        (("J1", "1"), ("J1", "2")),
        (("J1", "1"), ("U2", "3")),
        (("J1", "2"), ("U2", "3")),
        (("D1", "1"), ("C3", "1")),
        (("J1", "1"), ("J3", "4")),
        (("U1", "7"), ("D1", "1")),
    ] {
        assert!(!connected(pin(a.0, a.1)?, pin(b.0, b.1)?));
    }
    assert_eq!(
        d.component(pin("U1", "1")?.symbol)
            .ok_or("Missing controller")?
            .kind
            .pins()
            .len(),
        8
    );
    let mut invalid = d.clone();
    let mut bad_pin = invalid.wires[0].from.pin().ok_or("Not a pin")?.clone();
    bad_pin.terminal = "99".into();
    invalid.wires[0].from = bad_pin.into();
    assert!(invalid.validate().is_err());
    Ok(())
}

#[test]
fn legacy_drafts_reopen_and_definition_upgrade_undo_is_atomic() -> Result<()> {
    let mut d = Document::empty("Legacy fixture")?;
    d.builtin_definitions_version = 1;
    let bytes = serde_json::to_vec(
        &serde_json::json!({"format":"pcbcraft-schematic-draft/1","document":d}),
    )
    .map_err(|e| e.to_string())?;
    let mut s = Session::new(persistence::decode(&bytes)?)?;
    let before = s.document().clone();
    let id = place(&mut s, PartKind::FanController, Point::default())?;
    assert_eq!(s.document().builtin_definitions_version, 2);
    assert!(
        s.document()
            .pin(&PinRef {
                symbol: id,
                terminal: "8".into()
            })
            .is_some()
    );
    s.undo()?;
    assert_eq!(&before, s.document());
    s.redo()?;
    let original = s.document().clone();
    s.apply(
        s.revision(),
        Command::SetProperties {
            symbol: id,
            reference: "U7".into(),
            value: "Edited module".into(),
            quarter_turns: 3,
        },
    )?;
    s.undo()?;
    assert_eq!(s.document(), &original);
    let revision = s.revision();
    assert!(
        s.apply(
            revision,
            Command::SetProperties {
                symbol: id,
                reference: "invalid space".into(),
                value: "bad".into(),
                quarter_turns: 1
            }
        )
        .is_err()
    );
    assert_eq!(s.revision(), revision);
    assert_eq!(s.document(), &original);
    let spoof = serde_json::to_vec(
        &serde_json::json!({"format":"pcbcraft-schematic-draft/1","document":original}),
    )
    .map_err(|e| e.to_string())?;
    assert!(persistence::decode(&spoof).is_err());
    Ok(())
}

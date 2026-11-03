use pcbcraft_foundation::{
    document::*,
    netlist, persistence,
    selection::{self, Selection},
    session::{Command, Session},
    text::{self, Font, TextTarget},
    wiring::{self, Anchor},
};
fn p(x: i64, y: i64) -> Point {
    Point {
        x: x * GRID_NM,
        y: y * GRID_NM,
    }
}
fn fixture() -> Result<(Session, Selection)> {
    let mut s = Session::new(Document::empty("Selection fixture")?)?;
    let mut items = Selection::default();
    for x in [-30, 30] {
        let id = s
            .apply(
                s.revision(),
                Command::Place {
                    kind: PartKind::Resistor,
                    at: p(x, 0),
                },
            )?
            .ok_or("Missing symbol")?;
        items.symbols.insert(id);
    }
    s.apply(
        s.revision(),
        Command::AddNote {
            at: p(-30, -20),
            text: "Original note".into(),
        },
    )?;
    items.notes.insert(s.document().annotations[0].id);
    s.apply(
        s.revision(),
        Command::DrawWire {
            from: Anchor::Free(p(-20, 20)),
            to: Anchor::Free(p(20, 20)),
            bends: vec![],
        },
    )?;
    items.wires.insert(s.document().wires[0].id);
    Ok((Session::new(s.document().clone())?, items))
}
#[test]
fn mixed_group_move_delete_undo_and_save_are_atomic() -> Result<()> {
    let (mut s, items) = fixture()?;
    let before = s.document().clone();
    let preview = selection::moved(&before, &items, p(5, 5), &[])?;
    assert_eq!(s.document(), &before);
    s.apply(
        0,
        Command::MoveItems {
            items: items.clone(),
            delta: p(5, 5),
            routes: vec![],
        },
    )?;
    assert_eq!(s.document(), &preview);
    for (a, b) in before.symbols.iter().zip(&s.document().symbols) {
        assert_eq!(
            b.at,
            Point {
                x: a.at.x + p(5, 5).x,
                y: a.at.y + p(5, 5).y
            }
        );
    }
    assert_eq!(s.document().annotations[0].at, p(-25, -15));
    assert_eq!(s.revision(), 1);
    s.undo()?;
    assert_eq!(s.document(), &before);
    s.redo()?;
    assert_eq!(s.document(), &preview);
    assert_eq!(
        persistence::decode(&persistence::encode(s.document())?)?,
        preview
    );
    s.apply(s.revision(), Command::DeleteItems { items })?;
    assert!(
        s.document().symbols.is_empty()
            && s.document().wires.is_empty()
            && s.document().annotations.is_empty()
    );
    s.undo()?;
    assert_eq!(s.document(), &preview);
    Ok(())
}
#[test]
fn group_validation_and_zero_distance_preserve_revision_and_history() -> Result<()> {
    let (mut s, items) = fixture()?;
    let before = s.document().clone();
    s.apply(
        0,
        Command::MoveItems {
            items: items.clone(),
            delta: Point::default(),
            routes: vec![],
        },
    )?;
    assert_eq!(s.revision(), 0);
    let mut missing = items.clone();
    missing.symbols.insert(SymbolId::fresh()?);
    for (revision, command) in [
        (
            1,
            Command::DeleteItems {
                items: items.clone(),
            },
        ),
        (0, Command::DeleteItems { items: missing }),
        (
            0,
            Command::MoveItems {
                items: items.clone(),
                delta: Point { x: i64::MAX, y: 0 },
                routes: vec![],
            },
        ),
        (
            0,
            Command::MoveItems {
                items,
                delta: Point { x: LIMIT_NM, y: 0 },
                routes: vec![],
            },
        ),
    ] {
        assert!(s.apply(revision, command).is_err());
    }
    assert_eq!(s.document(), &before);
    assert_eq!(s.revision(), 0);
    assert!(!s.can_undo());
    Ok(())
}
#[test]
fn moving_selected_wire_preserves_external_pin_anchors_and_connectivity() -> Result<()> {
    let (s, mut items) = fixture()?;
    let mut s = Session::new(s.document().clone())?;

    // Use geometry order, independent of opaque random identity ordering.
    let mut symbols = s.document().symbols.clone();
    symbols.sort_by_key(|x| x.at.x);
    s.apply(
        s.revision(),
        Command::Connect {
            from: PinRef {
                symbol: symbols[0].id,
                terminal: "2".into(),
            },
            to: PinRef {
                symbol: symbols[1].id,
                terminal: "1".into(),
            },
        },
    )?;
    let wire = s.document().wires.last().ok_or("No wire")?.clone();
    items.clear();
    items.wires.insert(wire.id);
    let before = s.document().clone();
    let a = before.endpoint(&wire.from).ok_or("Missing pin")?;
    let b = before.endpoint(&wire.to).ok_or("Missing pin")?;
    let projection = wiring::Projection {
        wire: wire.id,
        path: vec![a, b],
    };
    let moved = selection::moved(&before, &items, p(0, 5), std::slice::from_ref(&projection))?;
    assert_eq!(moved.endpoint(&wire.from), Some(a));
    assert_eq!(moved.endpoint(&wire.to), Some(b));
    assert_eq!(
        netlist::Netlist::compile(&before)?,
        netlist::Netlist::compile(&moved)?
    );
    assert_eq!(before.symbols, moved.symbols);
    assert_eq!(
        selection::moved(&before, &items, Point::default(), &[projection])?,
        before
    );
    Ok(())
}
#[test]
fn text_labels_notes_links_and_styles_round_trip_and_undo() -> Result<()> {
    let (mut s, _) = fixture()?;
    let symbol = s.document().symbols[0].id;
    let targets = [
        (
            TextTarget::Note(s.document().annotations[0].id),
            "Power stage\nSee documentation",
        ),
        (TextTarget::Reference(symbol), "R100"),
        (TextTarget::Value(symbol), "47k"),
        (TextTarget::Net(s.document().wires[0].id), "POWER"),
    ];
    for (target, content) in targets {
        let before = s.document().clone();
        let mut appearance = target.defaults();
        appearance.height_nm = 3_000_000;
        appearance.font = Font::Semibold;
        appearance.italic = true;
        appearance.underline = true;
        appearance.offset = p(1, -1);
        appearance.link = "https://example.com/component?rev=1#pins".into();
        s.apply(
            s.revision(),
            Command::SetText {
                target,
                content: content.into(),
                appearance: appearance.clone(),
            },
        )?;
        assert_eq!(text::appearance(s.document(), target), appearance);
        assert_eq!(target.content(s.document()).as_deref(), Some(content));
        let edited = s.document().clone();
        s.undo()?;
        assert_eq!(s.document(), &before);
        s.redo()?;
        assert_eq!(s.document(), &edited);
        assert_eq!(
            persistence::decode(&persistence::encode(s.document())?)?,
            edited
        );
    }
    let wire = s.document().wires[0].id;
    s.apply(s.revision(), Command::Disconnect { wire })?;
    assert!(
        !s.document()
            .text_overrides
            .iter()
            .any(|o| o.target == TextTarget::Net(wire))
    );
    s.undo()?;
    assert!(
        s.document()
            .text_overrides
            .iter()
            .any(|o| o.target == TextTarget::Net(wire))
    );
    Ok(())
}
#[test]
fn invalid_text_edits_and_stale_requests_leave_zero_mutation() -> Result<()> {
    let (mut s, _) = fixture()?;
    let before = s.document().clone();
    let target = TextTarget::Note(before.annotations[0].id);
    for url in [
        "javascript:alert(1)",
        "file:///tmp/data",
        "https://user:pass@example.com",
        "https://",
        "https://?query=value",
        "https://\\example.com",
        "https://example.com/\n",
    ] {
        let mut appearance = target.defaults();
        appearance.link = url.into();
        assert!(
            s.apply(
                0,
                Command::SetText {
                    target,
                    content: "Changed".into(),
                    appearance
                }
            )
            .is_err()
        );
    }
    for size in [0, 249_999, 12_000_001, i64::MAX] {
        let mut appearance = target.defaults();
        appearance.height_nm = size;
        assert!(
            s.apply(
                0,
                Command::SetText {
                    target,
                    content: "Changed".into(),
                    appearance
                }
            )
            .is_err()
        );
    }
    for content in ["", "bad\u{0}"] {
        assert!(
            s.apply(
                0,
                Command::SetText {
                    target,
                    content: content.into(),
                    appearance: target.defaults()
                }
            )
            .is_err()
        );
    }
    let duplicate = TextTarget::Reference(before.symbols[0].id);
    assert!(
        s.apply(
            0,
            Command::SetText {
                target: duplicate,
                content: before.components[1].reference.clone(),
                appearance: duplicate.defaults()
            }
        )
        .is_err()
    );
    assert!(
        s.apply(
            1,
            Command::SetText {
                target,
                content: "stale".into(),
                appearance: target.defaults()
            }
        )
        .is_err()
    );
    assert_eq!(s.document(), &before);
    assert_eq!(s.revision(), 0);
    assert!(!s.can_undo());
    Ok(())
}
#[test]
fn v4_notes_gain_identity_once_and_v5_data_cannot_masquerade_as_legacy() -> Result<()> {
    let (s, _) = fixture()?;
    let mut raw: serde_json::Value =
        serde_json::from_slice(&persistence::encode(s.document())?).map_err(|e| e.to_string())?;
    raw["format"] = "pcbcraft-schematic-draft/4".into();
    assert!(persistence::decode(&serde_json::to_vec(&raw).map_err(|e| e.to_string())?).is_err());
    for note in raw["document"]["annotations"]
        .as_array_mut()
        .ok_or("No notes")?
    {
        note.as_object_mut().ok_or("Bad note")?.remove("id");
    }
    let migrated = persistence::decode(&serde_json::to_vec(&raw).map_err(|e| e.to_string())?)?;
    assert_eq!(
        migrated.annotations[0].text,
        s.document().annotations[0].text
    );
    assert_eq!(
        persistence::decode(&persistence::encode(&migrated)?)?,
        migrated
    );
    raw["document"]["text_overrides"] = serde_json::json!([]);
    assert!(persistence::decode(&serde_json::to_vec(&raw).map_err(|e| e.to_string())?).is_err());
    Ok(())
}

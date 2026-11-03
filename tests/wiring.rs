use pcbcraft_foundation::{
    document::*,
    netlist::Netlist,
    persistence,
    session::{Command, Session},
    wiring::{self, Anchor},
};
fn p(x: i64, y: i64) -> Point {
    Point {
        x: x * GRID_NM,
        y: y * GRID_NM,
    }
}
fn draw(s: &mut Session, from: Anchor, to: Anchor, bends: Vec<Point>) -> Result<()> {
    s.apply(s.revision(), Command::DrawWire { from, to, bends })?;
    Ok(())
}
#[test]
fn free_wires_bends_branch_delete_and_round_trip_are_real_connectivity() -> Result<()> {
    let mut s = Session::new(Document::empty("Wire fixture")?)?;
    draw(
        &mut s,
        Anchor::Free(p(0, 0)),
        Anchor::Free(p(20, 10)),
        vec![p(20, 0)],
    )?;
    let w = s.document().wires[0].clone();
    assert_eq!(Netlist::compile(s.document())?.nets.len(), 1);
    assert!(Netlist::compile(s.document())?.nets[0].terminals.is_empty());
    let before = s.document().clone();
    let route = wiring::path(s.document(), &w)?;
    draw(
        &mut s,
        Anchor::Split {
            wire: w.id,
            at: p(10, 0),
            path: route,
        },
        Anchor::Free(p(10, -10)),
        vec![],
    )?;
    let n = Netlist::compile(s.document())?;
    assert_eq!(n.nets.len(), 1);
    assert_eq!(n.nets[0].links.len(), 3);
    assert_eq!(s.document().junctions.len(), 4);
    assert_eq!(
        persistence::decode(&persistence::encode(s.document())?)?,
        *s.document()
    );
    let branched = s.document().clone();
    s.undo()?;
    assert_eq!(*s.document(), before);
    s.redo()?;
    assert_eq!(*s.document(), branched);
    let branch = s.document().wires.last().ok_or("No branch")?.id;
    s.apply(s.revision(), Command::Disconnect { wire: branch })?;
    assert_eq!(s.document().wires.len(), 2);
    assert_eq!(s.document().junctions.len(), 3);
    s.undo()?;
    assert_eq!(*s.document(), branched);
    s.apply(
        s.revision(),
        Command::DeleteNet {
            anchor: n.nets[0].anchor.clone(),
        },
    )?;
    assert!(s.document().wires.is_empty() && s.document().junctions.is_empty());
    s.undo()?;
    assert_eq!(*s.document(), branched);
    Ok(())
}
#[test]
fn crossings_stay_separate_until_an_explicit_branch_and_invalid_edits_are_atomic() -> Result<()> {
    let mut s = Session::new(Document::empty("Crossings")?)?;
    draw(
        &mut s,
        Anchor::Free(p(-10, 0)),
        Anchor::Free(p(10, 0)),
        vec![],
    )?;
    draw(
        &mut s,
        Anchor::Free(p(0, -10)),
        Anchor::Free(p(0, 10)),
        vec![],
    )?;
    assert_eq!(Netlist::compile(s.document())?.nets.len(), 2);
    let a = s.document().wires[0].clone();
    let b = s.document().wires[1].clone();
    for (wire, name) in [(&a, "A"), (&b, "B")] {
        s.apply(
            s.revision(),
            Command::RenameNet {
                anchor: wire.from.clone(),
                name: name.into(),
            },
        )?;
    }
    let before = s.document().clone();
    let revision = s.revision();
    // Candidate splits the first wire, then would merge conflicting named nets.
    let command = Command::DrawWire {
        from: Anchor::Split {
            wire: a.id,
            at: p(5, 0),
            path: wiring::path(s.document(), &a)?,
        },
        to: Anchor::Existing(b.to.clone()),
        bends: vec![p(5, 10)],
    };
    assert!(s.apply(revision, command.clone()).is_err());
    assert_eq!(*s.document(), before);
    assert_eq!(s.revision(), revision);
    assert!(s.apply(revision - 1, command).is_err());
    s.apply(
        s.revision(),
        Command::RenameNet {
            anchor: b.from.clone(),
            name: String::new(),
        },
    )?;
    let path = wiring::path(s.document(), &a)?;
    draw(
        &mut s,
        Anchor::Split {
            wire: a.id,
            at: p(5, 0),
            path,
        },
        Anchor::Existing(b.to),
        vec![p(5, 10)],
    )?;
    assert_eq!(Netlist::compile(s.document())?.nets.len(), 1);
    assert_eq!(
        Netlist::compile(s.document())?.nets[0].name.as_deref(),
        Some("A")
    );
    Ok(())
}
#[test]
fn free_wire_can_connect_to_pin_and_stretches_when_component_moves() -> Result<()> {
    let mut s = Session::new(Document::empty("Attachment")?)?;
    let id = s
        .apply(
            s.revision(),
            Command::Place {
                kind: PartKind::Resistor,
                at: p(0, 0),
            },
        )?
        .ok_or("Missing resistor")?;
    let pin = PinRef {
        symbol: id,
        terminal: "1".into(),
    };
    let at = s.document().pin(&pin).ok_or("Missing pin")?;
    draw(
        &mut s,
        Anchor::Existing(pin.clone().into()),
        Anchor::Free(p(-30, 15)),
        vec![Point {
            x: -30 * GRID_NM,
            y: at.y,
        }],
    )?;
    let before = s.document().clone();
    s.apply(
        s.revision(),
        Command::Move {
            symbol: id,
            at: p(3, 6),
        },
    )?;
    let w = &s.document().wires[0];
    wiring::validate_path(&wiring::path(s.document(), w)?)?;
    assert_eq!(Netlist::compile(s.document())?.nets[0].terminals, vec![pin]);
    s.undo()?;
    assert_eq!(*s.document(), before);
    s.apply(s.revision(), Command::Rotate { symbol: id })?;
    wiring::validate_path(&wiring::path(s.document(), &s.document().wires[0])?)?;
    s.undo()?;
    assert_eq!(*s.document(), before);
    Ok(())
}
#[test]
fn v4_rejects_invalid_geometry_and_fake_legacy_envelopes() -> Result<()> {
    let mut s = Session::new(Document::empty("Format")?)?;
    let empty = s.document().clone();
    assert!(draw(&mut s, Anchor::Free(p(0, 0)), Anchor::Free(p(1, 1)), vec![]).is_err());
    assert_eq!(*s.document(), empty);
    draw(&mut s, Anchor::Free(p(0, 0)), Anchor::Free(p(1, 0)), vec![])?;
    let text = String::from_utf8(persistence::encode(s.document())?).map_err(|e| e.to_string())?;
    for version in 1..=3 {
        assert!(
            persistence::decode(
                text.replace(
                    persistence::FORMAT,
                    &format!("pcbcraft-schematic-draft/{version}")
                )
                .as_bytes()
            )
            .is_err()
        );
    }
    let mut invalid = s.document().clone();
    invalid.junctions[0].at.x = LIMIT_NM + 1;
    assert!(persistence::encode(&invalid).is_err());
    let mut invalid = s.document().clone();
    invalid.wires[0].bends = Some(vec![p(1, 1)]);
    assert!(persistence::encode(&invalid).is_err());
    // v3 pin-only drawings still read unchanged.
    let demo = Session::demo()?;
    let old = String::from_utf8(persistence::encode(demo.document())?)
        .map_err(|e| e.to_string())?
        .replace(persistence::FORMAT, "pcbcraft-schematic-draft/3");
    assert_eq!(persistence::decode(old.as_bytes())?, *demo.document());
    Ok(())
}

#[test]
fn segment_corner_and_junction_moves_preserve_membership_and_fixed_pin_anchors() -> Result<()> {
    use pcbcraft_foundation::wiring::WireEdit;
    let mut s = Session::new(Document::empty("Reposition")?)?;
    draw(
        &mut s,
        Anchor::Free(p(0, 0)),
        Anchor::Free(p(20, 0)),
        vec![],
    )?;
    let wire = s.document().wires[0].clone();
    let route = wiring::path(s.document(), &wire)?;
    draw(
        &mut s,
        Anchor::Split {
            wire: wire.id,
            at: p(10, 0),
            path: route,
        },
        Anchor::Free(p(10, 10)),
        vec![],
    )?;
    let before = s.document().clone();
    let connectivity = Netlist::compile(&before)?;
    let junction = s.document().wires[0].to.clone();
    let Endpoint::Junction(j) = junction else {
        return Err("No junction".into());
    };
    s.apply(
        s.revision(),
        Command::EditWire {
            edit: WireEdit::Junction {
                id: j.junction,
                at: p(12, 5),
            },
        },
    )?;
    assert_eq!(Netlist::compile(s.document())?, connectivity);
    assert_eq!(
        s.document().endpoint(&s.document().wires[0].from),
        Some(p(0, 0))
    );
    for w in &s.document().wires {
        wiring::validate_path(&wiring::path(s.document(), w)?)?;
    }
    s.undo()?;
    assert_eq!(*s.document(), before);
    let w = s.document().wires[0].clone();
    let route = wiring::path(s.document(), &w)?;
    s.apply(
        s.revision(),
        Command::EditWire {
            edit: WireEdit::Segment {
                wire: w.id,
                path: route,
                index: 0,
                at: p(5, -6),
            },
        },
    )?;
    let shifted = s.document().clone();
    assert_eq!(Netlist::compile(&shifted)?, connectivity);
    assert_eq!(shifted.endpoint(&w.from), before.endpoint(&w.from));
    assert_eq!(shifted.endpoint(&w.to), before.endpoint(&w.to));
    let route = wiring::path(&shifted, &shifted.wires[0])?;
    assert_eq!(route, vec![p(0, 0), p(0, -6), p(10, -6), p(10, 0)]);
    s.apply(
        s.revision(),
        Command::EditWire {
            edit: WireEdit::Corner {
                wire: w.id,
                path: route,
                index: 1,
                at: p(-3, -9),
            },
        },
    )?;
    assert_eq!(Netlist::compile(s.document())?, connectivity);
    assert_eq!(
        persistence::decode(&persistence::encode(s.document())?)?,
        *s.document()
    );
    s.undo()?;
    assert_eq!(*s.document(), shifted);
    s.undo()?;
    assert_eq!(*s.document(), before);
    Ok(())
}

#[test]
fn overlapping_runs_heal_without_duplicate_edges_and_conflicts_are_atomic() -> Result<()> {
    let mut s = Session::new(Document::empty("Overlaps")?)?;
    draw(
        &mut s,
        Anchor::Free(p(0, 0)),
        Anchor::Free(p(20, 0)),
        vec![],
    )?;
    let before = s.document().clone();
    draw(
        &mut s,
        Anchor::Free(p(5, 0)),
        Anchor::Free(p(25, 0)),
        vec![],
    )?;
    assert_eq!(
        s.document().wires.len(),
        1,
        "Overlapping runs collapse to one wire"
    );
    assert_eq!(s.document().junctions.len(), 2);
    let route = wiring::path(s.document(), &s.document().wires[0])?;
    assert!(route == vec![p(0, 0), p(25, 0)] || route == vec![p(25, 0), p(0, 0)]);
    s.undo()?;
    assert_eq!(*s.document(), before);
    // Geometry dragging must reject a short between differently named nets.
    draw(
        &mut s,
        Anchor::Free(p(5, 10)),
        Anchor::Free(p(25, 10)),
        vec![],
    )?;
    let a = s.document().wires[0].clone();
    let b = s.document().wires[1].clone();
    for (wire, name) in [(&a, "POWER"), (&b, "SIGNAL")] {
        s.apply(
            s.revision(),
            Command::RenameNet {
                anchor: wire.from.clone(),
                name: name.into(),
            },
        )?;
    }
    let before = s.document().clone();
    let route = wiring::path(&before, &b)?;
    assert!(
        s.apply(
            s.revision(),
            Command::EditWire {
                edit: wiring::WireEdit::Segment {
                    wire: b.id,
                    path: route,
                    index: 0,
                    at: p(10, 0)
                }
            }
        )
        .is_err()
    );
    assert_eq!(*s.document(), before);
    Ok(())
}
#[test]
fn dropped_endpoint_attaches_with_a_real_junction_and_backtracking_disappears() -> Result<()> {
    let mut s = Session::new(Document::empty("Heal a T")?)?;
    draw(
        &mut s,
        Anchor::Free(p(0, 0)),
        Anchor::Free(p(20, 0)),
        vec![],
    )?;
    draw(
        &mut s,
        Anchor::Free(p(10, 10)),
        Anchor::Free(p(10, 20)),
        vec![],
    )?;
    let before = s.document().clone();
    let main = before.wires[0].clone();
    let Endpoint::Junction(j) = before.wires[1].from.clone() else {
        return Err("No point".into());
    };
    s.apply(
        s.revision(),
        Command::EditWire {
            edit: wiring::WireEdit::Attach {
                junction: j.junction,
                target: Anchor::Split {
                    wire: main.id,
                    at: p(10, 0),
                    path: wiring::path(&before, &main)?,
                },
            },
        },
    )?;
    assert_eq!(Netlist::compile(s.document())?.nets.len(), 1);
    assert_eq!(s.document().wires.len(), 3);
    s.undo()?;
    assert_eq!(*s.document(), before);
    let mut points = vec![p(0, 0), p(10, 0), p(5, 0), p(5, 10)];
    wiring::simplify(&mut points);
    assert_eq!(points, vec![p(0, 0), p(5, 0), p(5, 10)]);
    Ok(())
}

#[test]
fn overlap_with_legacy_display_route_materializes_only_affected_geometry() -> Result<()> {
    use pcbcraft_foundation::wiring::{Projection, WireEdit};
    let mut s = Session::new(Document::empty("Legacy route overlap")?)?;
    let mut routes = Vec::new();
    for y in [0, 30] {
        let left = s
            .apply(
                s.revision(),
                Command::Place {
                    kind: PartKind::Resistor,
                    at: p(-20, y),
                },
            )?
            .ok_or("Missing part")?;
        let right = s
            .apply(
                s.revision(),
                Command::Place {
                    kind: PartKind::Resistor,
                    at: p(20, y),
                },
            )?
            .ok_or("Missing part")?;
        s.apply(
            s.revision(),
            Command::Connect {
                from: PinRef {
                    symbol: left,
                    terminal: "2".into(),
                },
                to: PinRef {
                    symbol: right,
                    terminal: "1".into(),
                },
            },
        )?;
        let w = s.document().wires.last().ok_or("No legacy link")?;
        routes.push(Projection {
            wire: w.id,
            path: vec![p(-15, y), p(15, y)],
        });
    }
    let before = s.document().clone();
    s.apply(
        s.revision(),
        Command::EditWire {
            edit: WireEdit::WithRoutes {
                routes: routes.clone(),
                edit: Box::new(WireEdit::Draw {
                    from: Anchor::Free(p(0, 10)),
                    to: Anchor::Free(p(5, 0)),
                    bends: vec![p(0, 0)],
                }),
            },
        },
    )?;
    assert_eq!(s.document().wires.len(), 4);
    assert!(s.document().wires[0].bends.is_some());
    assert_eq!(
        s.document().wires[1],
        before.wires[1],
        "Unrelated legacy route remains byte-for-byte unchanged"
    );
    let n = Netlist::compile(s.document())?;
    assert_eq!(n.nets.len(), 2);
    assert!(n.nets.iter().all(|n| n.terminals.len() == 2));
    s.undo()?;
    assert_eq!(*s.document(), before);
    routes[0].path[0].y += GRID_NM;
    assert!(
        s.apply(
            s.revision(),
            Command::EditWire {
                edit: WireEdit::WithRoutes {
                    routes,
                    edit: Box::new(WireEdit::Draw {
                        from: Anchor::Free(p(-5, 0)),
                        to: Anchor::Free(p(5, 0)),
                        bends: vec![]
                    })
                }
            }
        )
        .is_err()
    );
    assert_eq!(*s.document(), before);
    Ok(())
}

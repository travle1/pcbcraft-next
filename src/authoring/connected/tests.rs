use super::*;
use crate::{
    authoring::*,
    session::{Command, Session},
};

fn p(x: i64, y: i64) -> Point {
    Point {
        x: x * GRID_NM,
        y: y * GRID_NM,
    }
}
fn fixture() -> Result<(Session, Vec<SymbolId>)> {
    let mut s = Session::new(Document::empty("Original eight-part chain")?)?;
    let mut ids = Vec::new();
    for i in 0..8 {
        ids.push(
            s.apply(
                s.revision(),
                Command::Place {
                    kind: PartKind::Resistor,
                    at: if i == 0 { p(-40, 0) } else { p(i * 40, -80) },
                },
            )?
            .ok_or("Missing symbol")?,
        );
    }
    for pair in ids.windows(2) {
        s.apply(
            s.revision(),
            Command::Connect {
                from: PinRef {
                    symbol: pair[0],
                    terminal: "2".into(),
                },
                to: PinRef {
                    symbol: pair[1],
                    terminal: "1".into(),
                },
            },
        )?;
    }
    Ok((s, ids))
}
fn operation(ids: &[SymbolId]) -> Operation {
    Operation::ArrangeConnected {
        symbols: ids.iter().copied().map(SymbolRef::Id).collect(),
        locked: vec![SymbolRef::Id(ids[0])],
        region: ArrangementRegion {
            min: p(-60, -20),
            max: p(160, 80),
        },
        origin: p(0, 0),
        columns: 4,
        spacing: p(40, 40),
    }
}
fn request(s: &Session, op: Operation) -> Prepare {
    Prepare {
        expected: s.snapshot(),
        sheet: s.project().root,
        summary: "Arrange original connected chain, keep input fixed".into(),
        operations: vec![op],
    }
}
fn apply(s: &mut Session, proposal: Proposal, name: &str) -> Result<()> {
    s.apply_proposal(Apply {
        expected: proposal.base,
        handle: proposal.handle,
        request_id: name.into(),
    })?;
    Ok(())
}

#[test]
fn connected_grid_preserves_locks_topology_history_and_repeats_as_noop() -> Result<()> {
    let (mut s, ids) = fixture()?;
    s.apply(
        s.revision(),
        Command::Place {
            kind: PartKind::Led,
            at: p(200, 200),
        },
    )?;
    let before = s.project().clone();
    let proposal = s.prepare(request(&s, operation(&ids)))?;
    assert_eq!(s.project(), &before);
    let candidate = s.proposal_project(proposal.handle)?.clone();
    assert_eq!(candidate.connectivity()?.nets, before.connectivity()?.nets);
    assert_eq!(candidate.main.components, before.main.components);
    assert_eq!(candidate.main.symbols[0], before.main.symbols[0]);
    assert_eq!(candidate.main.symbols[8], before.main.symbols[8]);
    for (i, symbol) in candidate.main.symbols[1..8].iter().enumerate() {
        assert_eq!(
            symbol.at,
            p(((i + 1) % 4) as i64 * 40, ((i + 1) / 4) as i64 * 40)
        );
    }
    let paths = candidate
        .main
        .wires
        .iter()
        .map(|w| wiring::path(&candidate.main, w))
        .collect::<Result<Vec<_>>>()?;
    for (i, (old, wire)) in before
        .main
        .wires
        .iter()
        .zip(&candidate.main.wires)
        .enumerate()
    {
        assert_eq!(
            (old.id, &old.from, &old.to, &old.label),
            (wire.id, &wire.from, &wire.to, &wire.label)
        );
        assert!(wire.bends.is_some());
        wiring::validate_path(&paths[i])?;
        for symbol in &candidate.main.symbols {
            let body = layout::bounds(&candidate.main, symbol.id, 0)?;
            assert!(!paths[i].windows(2).any(|s| body.crosses(s[0], s[1])));
        }
        assert!(
            !paths[..i]
                .iter()
                .any(|other| shares_segment(&paths[i], other))
        );
    }
    apply(&mut s, proposal, "arrange")?;
    let revision = s.revision();
    let repeat = s.prepare(request(&s, operation(&ids)))?;
    assert_eq!(s.proposal_project(repeat.handle)?, &candidate);
    apply(&mut s, repeat, "repeat")?;
    assert_eq!(s.revision(), revision);
    s.undo()?;
    assert_eq!(s.project(), &before);
    s.redo()?;
    assert_eq!(s.project(), &candidate);
    let reopened =
        crate::persistence::decode_project(&crate::persistence::encode_project(&candidate)?)?;
    assert_eq!(reopened, candidate);
    let mut reopened = Session::from_project(reopened)?;
    let repeat = reopened.prepare(request(&reopened, operation(&ids)))?;
    assert_eq!(reopened.proposal_project(repeat.handle)?, &candidate);
    Ok(())
}

#[test]
fn impossible_constraints_reject_without_mutation_or_pending_proposal() -> Result<()> {
    let (mut s, ids) = fixture()?;
    let before = s.project().clone();
    let revision = s.revision();
    for case in 0..8 {
        let mut op = operation(&ids);
        if let Operation::ArrangeConnected {
            symbols,
            locked,
            region,
            spacing,
            origin,
            ..
        } = &mut op
        {
            match case {
                0 => symbols.push(SymbolRef::Id(ids[0])),
                1 => locked.push(SymbolRef::Id(ids[0])),
                2 => locked.push(SymbolRef::Id(SymbolId::fresh()?)),
                3 => region.max = p(50, 50),
                4 => *spacing = p(16, 16),
                5 => origin.x += 1,
                6 => origin.x = LIMIT_NM,
                _ => region.max = region.min,
            }
        }
        assert!(s.prepare(request(&s, op)).is_err(), "case {case}");
        assert_eq!(s.project(), &before);
        assert_eq!(s.revision(), revision);
        assert!(s.proposals().is_empty());
    }
    Ok(())
}

#[test]
fn outside_endpoint_and_branch_movement_reject_atomically() -> Result<()> {
    for branch in [false, true] {
        let (mut s, ids) = fixture()?;
        let to = if branch {
            crate::wiring::Anchor::Free(p(60, -80))
        } else {
            let outside = s
                .apply(
                    s.revision(),
                    Command::Place {
                        kind: PartKind::Resistor,
                        at: p(80, -120),
                    },
                )?
                .ok_or("Missing symbol")?;
            crate::wiring::Anchor::Existing(
                PinRef {
                    symbol: outside,
                    terminal: "1".into(),
                }
                .into(),
            )
        };
        let from = PinRef {
            symbol: ids[1],
            terminal: "2".into(),
        };
        let end = to.point(s.document()).ok_or("Missing endpoint")?;
        s.apply(
            s.revision(),
            Command::DrawWire {
                from: crate::wiring::Anchor::Existing(from.into()),
                to,
                bends: if branch {
                    vec![]
                } else {
                    vec![Point {
                        x: end.x,
                        y: -80 * GRID_NM,
                    }]
                },
            },
        )?;
        let before = s.project().clone();
        let error = s
            .prepare(request(&s, operation(&ids)))
            .err()
            .ok_or("Expected rejection")?;
        assert!(error.to_string().contains("outside the selection"));
        assert_eq!(s.project(), &before);
        assert!(s.proposals().is_empty());
    }
    Ok(())
}

#[test]
fn locked_boundary_wire_and_unselected_geometry_are_preserved() -> Result<()> {
    let (mut s, ids) = fixture()?;
    let outside = s
        .apply(
            s.revision(),
            Command::Place {
                kind: PartKind::Resistor,
                at: p(-80, 0),
            },
        )?
        .ok_or("Missing outside symbol")?;
    s.apply(
        s.revision(),
        Command::DrawWire {
            from: crate::wiring::Anchor::Existing(
                PinRef {
                    symbol: outside,
                    terminal: "2".into(),
                }
                .into(),
            ),
            to: crate::wiring::Anchor::Existing(
                PinRef {
                    symbol: ids[0],
                    terminal: "1".into(),
                }
                .into(),
            ),
            bends: vec![],
        },
    )?;
    let before = s.project().clone();
    let proposal = s.prepare(request(&s, operation(&ids)))?;
    let candidate = s.proposal_project(proposal.handle)?;
    assert_eq!(candidate.main.wires.last(), before.main.wires.last());
    assert_eq!(candidate.main.symbols.last(), before.main.symbols.last());
    assert_eq!(candidate.main.symbols[0], before.main.symbols[0]);
    Ok(())
}

#[test]
fn preserved_wire_obstructed_by_new_position_rejects() -> Result<()> {
    let (mut s, ids) = fixture()?;
    s.apply(
        s.revision(),
        Command::DrawWire {
            from: crate::wiring::Anchor::Free(p(30, 0)),
            to: crate::wiring::Anchor::Free(p(50, 0)),
            bends: vec![],
        },
    )?;
    let before = s.project().clone();
    let error = s
        .prepare(request(&s, operation(&ids)))
        .err()
        .ok_or("Expected rejection")?;
    assert!(error.to_string().contains("obstruct preserved wire"));
    assert_eq!(s.project(), &before);
    Ok(())
}

#[test]
fn unsupported_text_clearance_is_explicit() -> Result<()> {
    let (mut s, ids) = fixture()?;
    s.apply(
        s.revision(),
        Command::AddNote {
            at: p(400, 300),
            text: "Long note".into(),
        },
    )?;
    let error = s
        .prepare(request(&s, operation(&ids)))
        .err()
        .ok_or("Expected rejection")?;
    assert!(error.to_string().contains("clearance is not qualified"));
    Ok(())
}

#[test]
fn blocked_route_rejects_and_twenty_part_selection_stays_bounded() -> Result<()> {
    let mut s = Session::new(Document::empty("Original blocked route")?)?;
    let mut ids = Vec::new();
    for x in [-20, 60] {
        ids.push(
            s.apply(
                s.revision(),
                Command::Place {
                    kind: PartKind::Resistor,
                    at: p(x, -80),
                },
            )?
            .ok_or("Missing symbol")?,
        );
    }
    s.apply(
        s.revision(),
        Command::Place {
            kind: PartKind::Controller24,
            at: p(20, 0),
        },
    )?;
    s.apply(
        s.revision(),
        Command::Connect {
            from: PinRef {
                symbol: ids[0],
                terminal: "2".into(),
            },
            to: PinRef {
                symbol: ids[1],
                terminal: "1".into(),
            },
        },
    )?;
    let before = s.project().clone();
    let op = Operation::ArrangeConnected {
        symbols: ids.into_iter().map(SymbolRef::Id).collect(),
        locked: vec![],
        origin: p(-20, 0),
        columns: 2,
        spacing: p(80, 40),
        region: ArrangementRegion {
            min: p(-40, -10),
            max: p(80, 10),
        },
    };
    let error = s
        .prepare(request(&s, op))
        .err()
        .ok_or("Expected routing rejection")?;
    assert!(error.to_string().contains("Cannot arrange wire"));
    assert_eq!(s.project(), &before);
    assert!(s.proposals().is_empty());

    let mut s = Session::new(Document::empty("Original twenty-part array")?)?;
    let mut symbols = Vec::new();
    for i in 0..20 {
        symbols.push(SymbolRef::Id(
            s.apply(
                s.revision(),
                Command::Place {
                    kind: PartKind::Resistor,
                    at: p(i * 20, -80),
                },
            )?
            .ok_or("Missing symbol")?,
        ));
    }
    let op = Operation::ArrangeConnected {
        symbols,
        locked: vec![],
        origin: p(0, 0),
        columns: 5,
        spacing: p(40, 40),
        region: ArrangementRegion {
            min: p(-20, -20),
            max: p(180, 140),
        },
    };
    let proposal = s.prepare(request(&s, op))?;
    assert_eq!(s.proposal_project(proposal.handle)?.main.symbols.len(), 20);
    Ok(())
}

#[test]
fn segment_sharing_differs_from_crossings_and_endpoint_contact() {
    assert!(shares_segment(&[p(0, 0), p(10, 0)], &[p(4, 0), p(20, 0)]));
    assert!(!shares_segment(&[p(0, 0), p(10, 0)], &[p(10, 0), p(20, 0)]));
    assert!(!shares_segment(&[p(0, 0), p(10, 0)], &[p(5, -5), p(5, 5)]));
    assert!(shares_segment(&[p(0, 0), p(0, 10)], &[p(0, 4), p(0, 20)]));
}

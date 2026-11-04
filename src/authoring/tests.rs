use super::*;
fn p(x: i64, y: i64) -> Point {
    Point {
        x: x * GRID_NM,
        y: y * GRID_NM,
    }
}
fn local(key: &str) -> SymbolRef {
    SymbolRef::Local(key.into())
}
fn pin(key: &str, id: &str) -> Terminal {
    Terminal {
        symbol: local(key),
        terminal: id.into(),
    }
}
fn place(key: &str, kind: PartKind, at: Point) -> Operation {
    Operation::Place {
        key: key.into(),
        kind,
        at,
        reference: None,
        value: None,
        quarter_turns: 0,
    }
}
fn fixture(session: &Session) -> Prepare {
    Prepare {
        expected: session.snapshot(),
        sheet: session.project().root,
        summary: "Original indicator signal path".into(),
        operations: vec![
            place("input", PartKind::Connector, p(0, 0)),
            place("r", PartKind::Resistor, p(0, 20)),
            place("led", PartKind::Led, p(0, 40)),
            Operation::Arrange {
                symbols: vec![local("input"), local("r"), local("led")],
                origin: p(0, 0),
                columns: 3,
                spacing: p(36, 36),
            },
            Operation::Connect {
                from: pin("input", "2"),
                to: pin("r", "1"),
                bends: None,
            },
            Operation::Connect {
                from: pin("r", "2"),
                to: pin("led", "1"),
                bends: None,
            },
        ],
    }
}
#[test]
fn atomic_preview_apply_retry_undo_redo_and_save() -> Result<()> {
    let mut s = Session::new(Document::empty("Test")?)?;
    let before = s.project().clone();
    let proposal = s.prepare(fixture(&s))?;
    assert_eq!(s.project(), &before);
    assert!(!s.dirty());
    assert!(!s.can_undo());
    let candidate = s.proposal_project(proposal.handle)?.clone();
    assert_eq!(candidate.main.symbols.len(), 3);
    assert_eq!(candidate.main.wires.len(), 2);
    for w in &candidate.main.wires {
        crate::wiring::validate_path(&crate::wiring::path(&candidate.main, w)?)?;
    }
    let request = Apply {
        request_id: "apply-1".into(),
        expected: s.snapshot(),
        handle: proposal.handle,
    };
    let r = s.apply_proposal(request.clone())?;
    assert_eq!(s.project(), &candidate);
    assert_eq!(r.after.revision, 1);
    assert_eq!(s.apply_proposal(request.clone())?.after, r.after);
    assert_eq!(s.revision(), 1);
    s.undo()?;
    assert_eq!(s.project(), &before);
    assert!(!s.can_undo());
    assert_eq!(s.revision(), 2);
    // An uncertain network retry after GUI Undo must NOT redo the edit.
    assert_eq!(s.apply_proposal(request.clone())?.after, r.after);
    assert_eq!(s.project(), &before);
    let mut altered = request;
    altered.expected.revision = 2;
    assert!(s.apply_proposal(altered).is_err());
    s.redo()?;
    assert_eq!(s.project(), &candidate);
    assert_eq!(s.revision(), 3);
    let bytes = crate::persistence::encode_project(s.project())?;
    let reopened = crate::persistence::decode_project(&bytes)?;
    assert_eq!(reopened, candidate);
    Ok(())
}
#[test]
fn invalid_late_step_no_partial_edit_or_history() -> Result<()> {
    let mut s = Session::new(Document::empty("Test")?)?;
    let mut request = fixture(&s);
    request.operations.push(Operation::Connect {
        from: pin("r", "nonexistent"),
        to: pin("led", "1"),
        bends: None,
    });
    let before = s.project().clone();
    assert!(s.prepare(request).is_err());
    assert_eq!(s.project(), &before);
    assert_eq!(s.revision(), 0);
    assert!(!s.can_undo());
    assert!(s.proposals().is_empty());
    Ok(())
}
#[test]
fn stale_after_gui_edit_undo_and_reopen() -> Result<()> {
    let mut s = Session::new(Document::empty("Test")?)?;
    let info = s.prepare(fixture(&s))?;
    let request = Apply {
        request_id: "stale".into(),
        expected: info.base.clone(),
        handle: info.handle,
    };
    s.apply(
        s.revision(),
        Command::Place {
            kind: PartKind::Resistor,
            at: p(100, 100),
        },
    )?;
    assert!(s.apply_proposal(request.clone()).is_err());
    s.undo()?;
    assert!(s.apply_proposal(request.clone()).is_err());
    let mut reopened = Session::from_project(s.project().clone())?;
    assert!(reopened.apply_proposal(request).is_err());
    Ok(())
}
#[test]
fn shared_definition_preview_lists_all_occurrences() -> Result<()> {
    let mut s = Session::new(Document::empty("Main")?)?;
    let child = s
        .project_command(
            0,
            crate::project::Command::Create {
                name: "Child".into(),
                parent: Some(s.project().root),
                existing: None,
                at: p(0, 0),
            },
        )?
        .created
        .ok_or("Missing occurrence")?;
    let definition = s.project().document(child).ok_or("Missing child")?.sheet_id;
    s.project_command(
        s.revision(),
        crate::project::Command::Create {
            name: "Second".into(),
            parent: Some(s.project().root),
            existing: Some(definition),
            at: p(50, 0),
        },
    )?;
    let mut request = fixture(&s);
    request.sheet = child;
    let proposal = s.prepare(request)?;
    assert_eq!(proposal.affected_occurrences.len(), 2);
    s.apply_proposal(Apply {
        request_id: "shared".into(),
        expected: proposal.base,
        handle: proposal.handle,
    })?;
    let targets = s.project().connectivity()?.nets;
    assert!(targets.iter().flatten().any(|n| n.occurrence == child));
    Ok(())
}
#[test]
fn discard_limits_unknown_fields_and_overlap() -> Result<()> {
    let mut s = Session::new(Document::empty("Test")?)?;
    let mut request = fixture(&s);
    request.operations[3] = Operation::Arrange {
        symbols: vec![local("input"), local("r"), local("led")],
        origin: p(0, 0),
        columns: 3,
        spacing: p(16, 16),
    };
    assert!(s.prepare(request).is_err());
    let mut invalid = serde_json::to_value(fixture(&s)).map_err(|e| e.to_string())?;
    invalid["surprise"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Prepare>(invalid).is_err());
    for _ in 0..MAX_PLANS {
        s.prepare(fixture(&s))?;
    }
    assert!(s.prepare(fixture(&s)).is_err());
    let handle = s.proposals()[0].handle;
    s.discard_proposal(handle);
    s.prepare(fixture(&s))?;
    assert_eq!(s.revision(), 0);
    Ok(())
}
#[test]
fn arrange_preserves_nets_and_unlisted_parts() -> Result<()> {
    let mut s = Session::new(Document::empty("Test")?)?;
    let proposal = s.prepare(fixture(&s))?;
    let r = proposal.created["r"];
    let led = proposal.created["led"];
    s.apply_proposal(Apply {
        request_id: "create".into(),
        expected: proposal.base,
        handle: proposal.handle,
    })?;
    let before = s.project().connectivity()?.nets;
    let fixed = s.document().symbols[0].clone();
    let info = s.prepare(Prepare {
        expected: s.snapshot(),
        sheet: s.project().root,
        summary: "Move the output pair".into(),
        operations: vec![Operation::Arrange {
            symbols: vec![SymbolRef::Id(r), SymbolRef::Id(led)],
            origin: p(40, 40),
            columns: 2,
            spacing: p(36, 36),
        }],
    })?;
    let candidate = s.proposal_project(info.handle)?;
    assert_eq!(candidate.connectivity()?.nets, before);
    assert_eq!(candidate.main.symbols[0], fixed);
    Ok(())
}

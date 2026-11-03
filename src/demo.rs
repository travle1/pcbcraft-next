//! Original editable illustration, not a qualified hardware/reference design.
//! Generic modules deliberately make no manufacturer pinout or firmware claims.
use crate::document::*;

pub fn fan_controller() -> Result<Document> {
    let mut d = Document::empty("Fan controller")?;
    use PartKind::*;
    // Coordinates in grid cells; three clear columns, power above signal flow.
    let parts = [
        ("J1", Connector, "12 V input", -108, -64, 1),
        ("U1", FanController, "3.3 V controller", 0, 4, 0),
        ("U2", Regulator, "12 V → 3.3 V module", -52, -65, 0),
        ("C1", Capacitor, "10 µF", -86, -64, 1),
        ("C2", Capacitor, "4.7 µF", -20, -64, 1),
        ("C3", Capacitor, "100 nF", 6, -64, 1),
        ("U3", TemperatureSensor, "Analog temperature", -80, 0, 0),
        ("C4", Capacitor, "100 nF", -110, 1, 1),
        ("J2", Connector, "0–3.3 V setpoint", -80, 40, 2),
        ("J4", Connector, "Mode jumper", 80, 52, 0),
        ("R3", Resistor, "10k", 60, 34, 1),
        ("J3", FanConnector, "12 V · 4-wire fan", 110, 16, 0),
        ("R1", Resistor, "10k", 74, -14, 1),
        ("R2", Resistor, "100 Ω", 78, 22, 0),
        ("R4", Resistor, "1k", 65, -34, 0),
        ("D1", Led, "Green LED", 94, -34, 0),
    ];
    for (reference, kind, value, x, y, turns) in parts {
        let component = ComponentId::fresh()?;
        d.components.push(Component {
            id: component,
            kind,
            reference: reference.into(),
            value: value.into(),
        });
        d.symbols.push(Symbol {
            id: SymbolId::fresh()?,
            component,
            at: point(x, y),
            quarter_turns: turns,
        });
    }
    for (label, anchor, terminals) in [
        (
            "+12V",
            ("J1", "1"),
            vec![("U2", "1"), ("C1", "1"), ("J3", "2")],
        ),
        (
            "+3V3",
            ("U2", "3"),
            vec![
                ("C2", "1"),
                ("C3", "1"),
                ("U1", "1"),
                ("U3", "1"),
                ("C4", "1"),
                ("R1", "1"),
                ("R3", "1"),
            ],
        ),
        (
            "GND",
            ("J1", "2"),
            vec![
                ("U2", "2"),
                ("C1", "2"),
                ("C2", "2"),
                ("C3", "2"),
                ("U1", "2"),
                ("U3", "2"),
                ("C4", "2"),
                ("J2", "2"),
                ("J4", "2"),
                ("J3", "1"),
                ("D1", "2"),
            ],
        ),
        ("TEMP", ("U3", "3"), vec![("U1", "3")]),
        ("SETPOINT", ("J2", "1"), vec![("U1", "4")]),
        ("MODE", ("U1", "8"), vec![("J4", "1"), ("R3", "2")]),
        ("PWM", ("U1", "5"), vec![("R2", "1")]),
        ("TACH", ("U1", "6"), vec![("J3", "3"), ("R1", "2")]),
        ("STATUS", ("U1", "7"), vec![("R4", "1")]),
    ] {
        for terminal in terminals {
            link(&mut d, anchor, terminal, Some(label))?;
        }
    }
    link(&mut d, ("R2", "2"), ("J3", "4"), None)?;
    link(&mut d, ("R4", "2"), ("D1", "1"), None)?;
    for (a, b) in [
        (("J1", "1"), ("U2", "1")),
        (("J1", "1"), ("C1", "1")),
        (("J1", "2"), ("U2", "2")),
        (("J1", "2"), ("C1", "2")),
        (("U2", "3"), ("C2", "1")),
        (("U2", "3"), ("C3", "1")),
        (("U3", "3"), ("U1", "3")),
        (("U1", "5"), ("R2", "1")),
        (("U1", "6"), ("J3", "3")),
    ] {
        let from = pin(&d, a.0, a.1)?;
        let to = pin(&d, b.0, b.1)?;
        if let Some(wire) = d
            .wires
            .iter_mut()
            .find(|w| w.from == Endpoint::from(from.clone()) && w.to == Endpoint::from(to.clone()))
        {
            wire.draw_path = true;
        }
    }
    let old_from = pin(&d, "U2", "3")?;
    let cap = pin(&d, "C4", "1")?;
    let sensor = pin(&d, "U3", "1")?;
    if let Some(w) = d
        .wires
        .iter_mut()
        .find(|w| w.from == Endpoint::from(old_from.clone()) && w.to == Endpoint::from(cap.clone()))
    {
        w.from = sensor.into();
        w.draw_path = true;
    }
    for (a, b) in [
        (("U1", "7"), ("R4", "1")),
        (("J2", "1"), ("U1", "4")),
        (("U1", "8"), ("J4", "1")),
    ] {
        let from = pin(&d, a.0, a.1)?;
        let to = pin(&d, b.0, b.1)?;
        if let Some(w) = d
            .wires
            .iter_mut()
            .find(|w| w.from == Endpoint::from(from.clone()) && w.to == Endpoint::from(to.clone()))
        {
            w.draw_path = true;
        }
    }
    // Keep pull-ups visibly in their signal paths without crossing another rail.
    for (old_a, old_b, from_part, from_pin, to_part, to_pin) in [
        (("U1", "8"), ("R3", "2"), "R3", "2", "J4", "1"),
        (("U1", "6"), ("R1", "2"), "R1", "2", "J3", "3"),
    ] {
        let a = pin(&d, old_a.0, old_a.1)?;
        let b = pin(&d, old_b.0, old_b.1)?;
        let from = pin(&d, from_part, from_pin)?;
        let to = pin(&d, to_part, to_pin)?;
        if let Some(w) = d
            .wires
            .iter_mut()
            .find(|w| w.from == Endpoint::from(a.clone()) && w.to == Endpoint::from(b.clone()))
        {
            w.from = from.into();
            w.to = to.into();
            w.label = None;
        }
    }
    for (x, y, text) in [
        (-118, -90, "POWER  /  12 V input · 3.3 V logic"),
        (-118, -25, "TEMPERATURE"),
        (-118, 24, "SETPOINT INPUT"),
        (-22, -23, "CONTROLLER"),
        (98, -16, "FAN INTERFACE"),
        (48, -48, "STATUS INDICATOR"),
        (
            -118,
            88,
            "Original generic modules · illustrative draft · not electrically validated",
        ),
    ] {
        d.annotations.push(Annotation {
            id: AnnotationId::fresh()?,
            at: point(x, y),
            text: text.into(),
        });
    }
    d.validate()?;
    Ok(d)
}
fn point(x: i64, y: i64) -> Point {
    Point {
        x: x * GRID_NM,
        y: y * GRID_NM,
    }
}
fn pin(d: &Document, reference: &str, terminal: &str) -> Result<PinRef> {
    let symbol = d
        .symbols
        .iter()
        .find(|s| d.component(s.id).is_some_and(|c| c.reference == reference))
        .ok_or("Demo component missing")?;
    Ok(PinRef {
        symbol: symbol.id,
        terminal: terminal.into(),
    })
}
fn link(d: &mut Document, a: (&str, &str), b: (&str, &str), label: Option<&str>) -> Result<()> {
    let from = pin(d, a.0, a.1)?;
    let to = pin(d, b.0, b.1)?;
    d.wires.push(Wire {
        id: WireId::fresh()?,
        from: from.into(),
        to: to.into(),
        label: label.map(str::to_owned),
        draw_path: false,
        bends: None,
    });
    Ok(())
}

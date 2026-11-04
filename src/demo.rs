//! Original editable fan-controller study, not a qualified hardware design.
//! Functional pin names are illustrative; no manufacturer pinout is implied.
use crate::{
    document::*,
    text::{Override, TextTarget},
};

pub fn fan_controller() -> Result<Document> {
    let mut d = Document::empty("Fan controller")?;
    d.builtin_definitions_version = 3;
    use PartKind::*;
    // Authored local circuits around a central controller, following the user's
    // FigCraft composition. Routes and text are native editable document data.
    let parts = [
        ("J1", Connector, "12 V input", -116, -84, 1),
        ("U1", Controller24, "Generic MCU · 3.3 V", 0, 12, 0),
        ("U2", Regulator, "12 V → 3.3 V", -72, -85, 0),
        ("C1", Capacitor, "10 µF", -96, -84, 1),
        ("C2", Capacitor, "4.7 µF", -48, -84, 1),
        ("C3", Capacitor, "100 nF", -24, -84, 1),
        ("C7", Capacitor, "100 nF", 8, -50, 1),
        ("R5", Resistor, "10k", -48, -42, 1),
        ("C5", Capacitor, "100 nF", -48, -26, 1),
        ("R10", Resistor, "100k", -120, -38, 1),
        ("R11", Resistor, "22k", -120, -18, 1),
        ("J2", Connector, "0–3.3 V setpoint", -112, 4, 2),
        ("J4", Connector, "Mode select", -92, 24, 2),
        ("R3", Resistor, "10k", -134, 16, 1),
        ("Y1", Oscillator, "8 MHz", -86, 54, 0),
        ("C6", Capacitor, "100 nF", -124, 54, 1),
        ("J5", Header6, "SWD debug", -100, 94, 0),
        ("J6", Header6, "UART console", -28, 94, 0),
        ("J3", FanConnector, "12 V · 4-wire fan", 116, -18, 0),
        ("R1", Resistor, "10k", 80, -40, 1),
        ("R2", Resistor, "100 Ω", 80, -12, 0),
        ("R4", Resistor, "1k", 72, 12, 0),
        ("D1", Led, "Status LED", 104, 12, 0),
        (
            "U3",
            DigitalTemperatureSensor,
            "I²C temperature",
            112,
            54,
            0,
        ),
        ("C4", Capacitor, "100 nF", 144, 40, 1),
        ("R8", Resistor, "4.7k", 72, 44, 1),
        ("R9", Resistor, "4.7k", 84, 44, 1),
        ("J7", Header6, "I²C display", 112, 96, 0),
        ("C8", Capacitor, "10 µF", 80, 96, 1),
        ("R7", Resistor, "1k", 28, 94, 0),
        ("D2", Led, "Activity", 56, 94, 0),
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
    // Aliases are explicit electrical edges, not text-based connectivity.
    for (name, anchor, terminals) in [
        ("+12V", ("J1", "1"), vec![("J3", "2"), ("R10", "1")]),
        (
            "+3V3",
            ("U2", "3"),
            vec![
                ("U1", "1"),
                ("C7", "1"),
                ("R5", "1"),
                ("R3", "1"),
                ("Y1", "1"),
                ("C6", "1"),
                ("J5", "1"),
                ("J6", "1"),
                ("R1", "1"),
                ("U3", "1"),
                ("C4", "1"),
                ("R8", "1"),
                ("R9", "1"),
                ("J7", "1"),
                ("C8", "1"),
            ],
        ),
        (
            "GND",
            ("J1", "2"),
            vec![
                ("C2", "2"),
                ("C3", "2"),
                ("U1", "2"),
                ("U1", "24"),
                ("C7", "2"),
                ("C5", "2"),
                ("R11", "2"),
                ("J2", "2"),
                ("J4", "2"),
                ("Y1", "2"),
                ("C6", "2"),
                ("J5", "6"),
                ("J6", "6"),
                ("J3", "1"),
                ("D1", "2"),
                ("U3", "2"),
                ("C4", "2"),
                ("J7", "6"),
                ("C8", "2"),
                ("D2", "2"),
            ],
        ),
        ("SWDIO", ("U1", "11"), vec![("J5", "2")]),
        ("SWCLK", ("U1", "12"), vec![("J5", "4")]),
        ("NRST", ("U1", "9"), vec![("J5", "3")]),
        ("UART_TX", ("U1", "13"), vec![("J6", "2")]),
        ("UART_RX", ("U1", "14"), vec![("J6", "4")]),
        ("SDA", ("U3", "3"), vec![("J7", "2")]),
        ("SCL", ("U3", "4"), vec![("J7", "4")]),
        ("ALERT", ("U3", "5"), vec![("U1", "22")]),
        ("ACTIVITY", ("U1", "15"), vec![("R7", "1")]),
    ] {
        for b in terminals {
            wire(&mut d, anchor, b, Some(name), None)?;
        }
    }
    // Short, deliberately orthogonal local routes with real branch junctions.
    for (a, b, name, bends) in [
        (("J1", "1"), ("C1", "1"), Some("+12V"), vec![]),
        (("C1", "1"), ("U2", "1"), None, vec![]),
        (("J1", "2"), ("C1", "2"), None, vec![]),
        (("C1", "2"), ("U2", "2"), None, vec![(-84, -79)]),
        (("U2", "3"), ("C2", "1"), Some("+3V3"), vec![]),
        (("C2", "1"), ("C3", "1"), None, vec![]),
        (
            ("J2", "1"),
            ("U1", "4"),
            Some("SETPOINT"),
            vec![(-72, 4), (-72, 0)],
        ),
        (
            ("J4", "1"),
            ("U1", "8"),
            Some("MODE"),
            vec![(-58, 24), (-58, 8)],
        ),
        (
            ("Y1", "3"),
            ("U1", "10"),
            Some("CLK"),
            vec![(-46, 54), (-46, 16)],
        ),
        (
            ("U1", "6"),
            ("J3", "3"),
            Some("TACH"),
            vec![(48, -12), (48, -16)],
        ),
        (
            ("U1", "5"),
            ("R2", "1"),
            Some("PWM"),
            vec![(56, -8), (56, -12)],
        ),
        (("R2", "2"), ("J3", "4"), None, vec![]),
        (
            ("U1", "7"),
            ("R4", "1"),
            Some("STATUS"),
            vec![(40, 4), (40, 12)],
        ),
        (("R4", "2"), ("D1", "1"), None, vec![]),
        (
            ("U1", "16"),
            ("U3", "3"),
            Some("SDA"),
            vec![(48, 28), (48, 50)],
        ),
        (
            ("U1", "17"),
            ("U3", "4"),
            Some("SCL"),
            vec![(40, 32), (40, 58)],
        ),
        (("R7", "2"), ("D2", "1"), None, vec![]),
    ] {
        wire(&mut d, a, b, name, Some(&bends))?;
    }
    branch(
        &mut d,
        ("R5", "2"),
        ("C5", "1"),
        ("U1", "9"),
        (-48, -36),
        &[(-30, -36), (-30, -12)],
        "NRST",
    )?;
    branch(
        &mut d,
        ("R10", "2"),
        ("R11", "1"),
        ("U1", "3"),
        (-120, -28),
        &[(-62, -28), (-62, -8)],
        "VIN_SENSE",
    )?;
    // Split each signal at the physical tap, preserving a single shared vertex.
    tap(
        &mut d,
        ("U1", "6"),
        ("J3", "3"),
        ("R1", "2"),
        (80, -16),
        &[],
    )?;
    tap(
        &mut d,
        ("J4", "1"),
        ("U1", "8"),
        ("R3", "2"),
        (-76, 24),
        &[(-134, 32), (-76, 32)],
    )?;
    tap(
        &mut d,
        ("U1", "16"),
        ("U3", "3"),
        ("R8", "2"),
        (72, 50),
        &[],
    )?;
    tap(
        &mut d,
        ("U1", "17"),
        ("U3", "4"),
        ("R9", "2"),
        (84, 58),
        &[],
    )?;
    // Each heading is local to its circuit; it does not expand fit to distant margins.
    for (x, y, label) in [
        (-126, -100, "Power supply"),
        (-132, -54, "Input monitor"),
        (-58, -62, "Reset & decoupling"),
        (-110, -6, "Manual control"),
        (-108, 40, "Clock source"),
        (64, -60, "Fan interface"),
        (64, 30, "Temperature sensor"),
        (-126, 78, "SWD debug"),
        (-50, 78, "UART console"),
        (20, 78, "Activity"),
        (88, 80, "Display connector"),
    ] {
        let id = AnnotationId::fresh()?;
        d.annotations.push(Annotation {
            id,
            at: point(x, y),
            text: label.into(),
        });
        let target = TextTarget::Note(id);
        let mut appearance = target.defaults();
        appearance.height_nm = 4_000_000;
        d.text_overrides.push(Override { target, appearance });
    }
    let id = pin(&d, "R2", "1")?.symbol;
    for (target, y) in [(TextTarget::Reference(id), 9), (TextTarget::Value(id), 4)] {
        let mut appearance = target.defaults();
        appearance.height_nm = 3_000_000;
        appearance.offset = point(0, y);
        d.text_overrides.push(Override { target, appearance });
    }
    for reference in ["R5", "C5"] {
        let target = TextTarget::Value(pin(&d, reference, "1")?.symbol);
        let mut appearance = target.defaults();
        appearance.height_nm = 3_000_000;
        appearance.offset = point(-12, 0);
        d.text_overrides.push(Override { target, appearance });
    }
    // Keep values clear of the shared input return rail, including page overview.
    for reference in ["J1", "C1"] {
        let target = TextTarget::Value(pin(&d, reference, "1")?.symbol);
        let mut appearance = target.defaults();
        appearance.height_nm = 3_000_000;
        appearance.offset = if reference == "C1" {
            point(6, 9)
        } else {
            point(0, 5)
        };
        d.text_overrides.push(Override { target, appearance });
    }
    let target = TextTarget::Reference(pin(&d, "R9", "1")?.symbol);
    let mut appearance = target.defaults();
    appearance.height_nm = 3_000_000;
    appearance.offset = point(0, -6);
    d.text_overrides.push(Override { target, appearance });
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
fn wire(
    d: &mut Document,
    a: (&str, &str),
    b: (&str, &str),
    label: Option<&str>,
    bends: Option<&[(i64, i64)]>,
) -> Result<()> {
    let from = pin(d, a.0, a.1)?.into();
    let to = pin(d, b.0, b.1)?.into();
    d.wires.push(Wire {
        id: WireId::fresh()?,
        from,
        to,
        label: label.map(str::to_owned),
        draw_path: bends.is_some(),
        bends: bends.map(|p| p.iter().map(|&(x, y)| point(x, y)).collect()),
    });
    Ok(())
}
fn junction(d: &mut Document, at: (i64, i64)) -> Result<Endpoint> {
    let id = JunctionId::fresh()?;
    d.junctions.push(Junction {
        id,
        at: point(at.0, at.1),
    });
    Ok(Endpoint::Junction(JunctionRef { junction: id }))
}
fn branch(
    d: &mut Document,
    a: (&str, &str),
    b: (&str, &str),
    c: (&str, &str),
    at: (i64, i64),
    bends: &[(i64, i64)],
    name: &str,
) -> Result<()> {
    let tap = junction(d, at)?;
    for (part, bends, label) in [
        (a, Vec::new(), None),
        (b, Vec::new(), None),
        (
            c,
            bends.iter().map(|&(x, y)| point(x, y)).collect(),
            Some(name.into()),
        ),
    ] {
        let to = pin(d, part.0, part.1)?.into();
        d.wires.push(Wire {
            id: WireId::fresh()?,
            from: tap.clone(),
            to,
            label,
            draw_path: true,
            bends: Some(bends),
        });
    }
    Ok(())
}
fn tap(
    d: &mut Document,
    a: (&str, &str),
    b: (&str, &str),
    part: (&str, &str),
    at: (i64, i64),
    branch_bends: &[(i64, i64)],
) -> Result<()> {
    let from: Endpoint = pin(d, a.0, a.1)?.into();
    let to: Endpoint = pin(d, b.0, b.1)?.into();
    let index = d
        .wires
        .iter()
        .position(|w| w.from == from && w.to == to && w.draw_path)
        .ok_or("Demo tap route missing")?;
    let original = d.wires.remove(index);
    let tap = junction(d, at)?;
    let target = point(at.0, at.1);
    let start = d.endpoint(&from).ok_or("Missing tap start")?;
    let end = d.endpoint(&to).ok_or("Missing tap end")?;
    let mut path = vec![start];
    path.extend(original.bends.unwrap_or_default());
    path.push(end);
    let segment = path
        .windows(2)
        .position(|p| {
            (p[0].x == target.x
                && p[1].x == target.x
                && (p[0].y.min(p[1].y)..=p[0].y.max(p[1].y)).contains(&target.y))
                || (p[0].y == target.y
                    && p[1].y == target.y
                    && (p[0].x.min(p[1].x)..=p[0].x.max(p[1].x)).contains(&target.x))
        })
        .ok_or("Tap must lie on route")?;
    d.wires.push(Wire {
        id: original.id,
        from,
        to: tap.clone(),
        label: original.label,
        draw_path: true,
        bends: Some(path[1..=segment].to_vec()),
    });
    d.wires.push(Wire {
        id: WireId::fresh()?,
        from: tap.clone(),
        to,
        label: None,
        draw_path: true,
        bends: Some(path[segment + 1..path.len() - 1].to_vec()),
    });
    let from = pin(d, part.0, part.1)?.into();
    let bends = branch_bends.iter().map(|&(x, y)| point(x, y)).collect();
    d.wires.push(Wire {
        id: WireId::fresh()?,
        from,
        to: tap,
        label: None,
        draw_path: true,
        bends: Some(bends),
    });
    Ok(())
}

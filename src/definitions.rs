//! Original generic module drawings/terminals. No manufacturer pinout is implied.
use crate::document::{PartKind, PinDefinition as P};
pub fn pins(kind: PartKind) -> &'static [P] {
    match kind {
        PartKind::FanController => &[
            P {
                id: "1",
                name: "3V3",
                x: -20,
                y: -12,
            },
            P {
                id: "2",
                name: "GND",
                x: -20,
                y: 12,
            },
            P {
                id: "3",
                name: "TEMP",
                x: -20,
                y: -4,
            },
            P {
                id: "4",
                name: "SET",
                x: -20,
                y: 4,
            },
            P {
                id: "5",
                name: "PWM",
                x: 20,
                y: 4,
            },
            P {
                id: "6",
                name: "TACH",
                x: 20,
                y: -4,
            },
            P {
                id: "7",
                name: "STATUS",
                x: 20,
                y: -12,
            },
            P {
                id: "8",
                name: "MODE",
                x: 20,
                y: 12,
            },
        ],
        PartKind::FanConnector => &[
            P {
                id: "1",
                name: "GND",
                x: -12,
                y: -6,
            },
            P {
                id: "2",
                name: "+12V",
                x: -12,
                y: -2,
            },
            P {
                id: "3",
                name: "TACH",
                x: -12,
                y: 2,
            },
            P {
                id: "4",
                name: "PWM",
                x: -12,
                y: 6,
            },
        ],
        PartKind::Regulator => &[
            P {
                id: "1",
                name: "VIN",
                x: -12,
                y: -4,
            },
            P {
                id: "2",
                name: "GND",
                x: -12,
                y: 4,
            },
            P {
                id: "3",
                name: "3V3",
                x: 12,
                y: -4,
            },
        ],
        PartKind::TemperatureSensor => &[
            P {
                id: "1",
                name: "3V3",
                x: -12,
                y: -4,
            },
            P {
                id: "2",
                name: "GND",
                x: -12,
                y: 4,
            },
            P {
                id: "3",
                name: "OUT",
                x: 12,
                y: 0,
            },
        ],
        _ => &[
            P {
                id: "1",
                name: "",
                x: -5,
                y: 0,
            },
            P {
                id: "2",
                name: "",
                x: 5,
                y: 0,
            },
        ],
    }
}

//! Original generic module drawings/terminals. No manufacturer pinout is implied.
use crate::document::{PartKind, PinDefinition as P};
pub fn pins(kind: PartKind) -> &'static [P] {
    match kind {
        PartKind::Controller24 => &[
            P {
                id: "1",
                name: "VDD",
                x: -22,
                y: -28,
            },
            P {
                id: "9",
                name: "NRST",
                x: -22,
                y: -24,
            },
            P {
                id: "3",
                name: "VIN_SENSE",
                x: -22,
                y: -20,
            },
            P {
                id: "4",
                name: "SET",
                x: -22,
                y: -12,
            },
            P {
                id: "8",
                name: "MODE",
                x: -22,
                y: -4,
            },
            P {
                id: "10",
                name: "CLK",
                x: -22,
                y: 4,
            },
            P {
                id: "15",
                name: "GPIO",
                x: -22,
                y: 8,
            },
            P {
                id: "11",
                name: "SWDIO",
                x: -22,
                y: 16,
            },
            P {
                id: "12",
                name: "SWCLK",
                x: -22,
                y: 20,
            },
            P {
                id: "13",
                name: "UART_TX",
                x: -22,
                y: 24,
            },
            P {
                id: "14",
                name: "UART_RX",
                x: -22,
                y: 28,
            },
            P {
                id: "2",
                name: "GND",
                x: -22,
                y: 36,
            },
            P {
                id: "6",
                name: "TACH",
                x: 22,
                y: -24,
            },
            P {
                id: "5",
                name: "PWM",
                x: 22,
                y: -20,
            },
            P {
                id: "7",
                name: "STATUS",
                x: 22,
                y: -8,
            },
            P {
                id: "22",
                name: "ALERT",
                x: 22,
                y: 0,
            },
            P {
                id: "16",
                name: "SDA",
                x: 22,
                y: 16,
            },
            P {
                id: "17",
                name: "SCL",
                x: 22,
                y: 20,
            },
            P {
                id: "18",
                name: "SPI_MOSI",
                x: 22,
                y: 24,
            },
            P {
                id: "19",
                name: "SPI_MISO",
                x: 22,
                y: 28,
            },
            P {
                id: "20",
                name: "SPI_SCK",
                x: 22,
                y: 32,
            },
            P {
                id: "21",
                name: "SPI_CS",
                x: 22,
                y: 36,
            },
            P {
                id: "23",
                name: "ADC_AUX",
                x: 22,
                y: 4,
            },
            P {
                id: "24",
                name: "VSS",
                x: 22,
                y: -28,
            },
        ],
        PartKind::DigitalTemperatureSensor => &[
            P {
                id: "1",
                name: "VDD",
                x: -12,
                y: -12,
            },
            P {
                id: "2",
                name: "GND",
                x: -12,
                y: 12,
            },
            P {
                id: "3",
                name: "SDA",
                x: -12,
                y: -4,
            },
            P {
                id: "4",
                name: "SCL",
                x: -12,
                y: 4,
            },
            P {
                id: "5",
                name: "ALERT",
                x: 12,
                y: 0,
            },
        ],
        PartKind::Oscillator => &[
            P {
                id: "1",
                name: "VDD",
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
        PartKind::Header6 => &[
            P {
                id: "1",
                name: "1",
                x: -10,
                y: -4,
            },
            P {
                id: "3",
                name: "3",
                x: -10,
                y: 0,
            },
            P {
                id: "5",
                name: "5",
                x: -10,
                y: 4,
            },
            P {
                id: "2",
                name: "2",
                x: 10,
                y: -4,
            },
            P {
                id: "4",
                name: "4",
                x: 10,
                y: 0,
            },
            P {
                id: "6",
                name: "6",
                x: 10,
                y: 4,
            },
        ],

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

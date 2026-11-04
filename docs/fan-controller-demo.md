# Editable fan-controller study

`cargo run -- --demo` opens an original 31-part example. Its composition follows
the user's FigCraft schematic overview: a central controller, compact local
support circuits, and surrounding interfaces. The design reference remains
unchanged at `../designs/exports/workspaces-2026-10-30/01-schematic-overview.png`.
The app draws real editable symbols, wires, junctions and styled notes. It does not
embed the reference image or import its electrical design.

This is a generic editing illustration, not a qualified hardware design. Its
functional 24-pin controller, regulator, oscillator and sensor do not claim any
manufacturer's pinout. Footprints, firmware, full ERC and electrical qualification
remain absent.

- J1 supplies +12 V; U2 represents a 3.3 V regulator. C1 is input decoupling;
  C2/C3 are output decoupling. C4/C6/C7 decouple local logic/sensor supply.
- U1 is the central 24-pin controller. R10/R11 illustrate supply monitoring;
  R5/C5 illustrate reset; Y1 is a clock module. J2 is an external setpoint;
  J4/R3 provide a mode input.
- U3 represents an I²C temperature sensor with separate SDA/SCL pull-ups R8/R9.
  J7/C8 provide a display interface. J5 and J6 expose debug and serial signals.
- J3 illustrates the four-wire fan roles (supply, ground, tach and PWM), with R1
  as a tach pull-up and R2 as a series PWM resistor. R4/D1 and R7/D2 form series
  indicator paths. No capacitor parallels an LED.

The earlier fan-role research used the public
[Noctua PWM white paper](https://cdn.noctua.at/media/Noctua_PWM_specifications_white_paper.pdf)
and [microcontroller guide](https://www.noctua.at/en/support/faqs/microcontroller-guide-pwm-setup-and-rpm-monitoring).
No upstream diagram, implementation or preserved GPL-fork data was imported.
Actual parts/interfaces require independent electrical and timing qualification.

## Presentation and compatibility

The twelve original built-in symbol types include four additions in catalog
version 3: Controller24, DigitalTemperatureSensor, Oscillator and Header6. Existing
catalog version 1/2 definitions retain their geometry and pin identities. Placing
a newer type upgrades the document's catalog version in the same undo transaction.
Older files remain readable; older applications cannot open catalog-3 documents.
The native project envelope is now version 7; legacy version 6 remains readable.

Local routes are authored orthogonal segments with explicit junctions. Remote
signals/power use explicit alias edges; equal text alone never establishes a net.
Only intentional branches have junctions. Incidental crossings stay separate.
Section headings and stored text offsets leave space around wires and parts.
Net names prefer a sufficiently long horizontal run; GND names sit below it.
Reference/value positions for vertical components leave more space from rails.

Measured render tests check reference/value clearance from authored wires at
1000×700 and 1600×1000, plus text-to-text clearance at 1600×1000. Notes also clear the authored wires. Fit-page is an
overview: net names, values and pin detail appear from 25% zoom, avoiding crowded
minimum-size labels at compact widths. These checks
protect this curated example, not arbitrary user drawings. General automatic
placement, label avoidance and AI schematic arrangement are separate work.

Opening a new example uses this composition. Existing saved projects keep their
own placements and text settings. The renderer still supplies bounded display
routes for un-authored links; authored routes follow the manual wire editor's
behavior. See [wire editing](wire-editing.md).

The current composition uses an A3 landscape page with a quiet dashed boundary.
Functional groups have wider gaps and shorter local clusters around the controller;
the page supplies a stable composition frame instead of packing parts to the window.
The input values and sensor pull-up reference have explicit clearance offsets.
F fits content; Shift+F fits the page. Existing saved placements are unchanged.

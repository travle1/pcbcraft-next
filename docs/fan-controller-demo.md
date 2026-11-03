# Original fan-controller illustration

`--demo` / File → Open example now opens 16 editable components arranged into
power, temperature, manual control, controller, fan interface and status sections.
This replaces the unlabeled four-part LED loop. That earlier capacitor was across
the input supply, not the LED, but the layout did not communicate that clearly.

The new example is an original generic-module illustration, not a specific MCU,
regulator or sensor design and not a fabrication-ready reference. It has no
manufacturer pinout, footprints, qualified component values, firmware or ERC result.
Its drawings, connectivity and fixtures were authored here; no external schematic,
KiCad implementation or preserved GPL-fork material was used.

- J1 supplies +12V. U2 is a generic 12V-to-3.3V regulator module.
- C1 is across input supply/GND; C2 is across the regulator output/GND; C3 and C4
  are logic/sensor supply decoupling. No capacitor parallels the status LED.
- U1 is a generic eight-terminal control module. U3 illustrates analog temperature
  input; J2 illustrates a bounded external setpoint; J4 and R3 illustrate mode input.
- J3 uses the common four-wire fan roles: ground, supply, tachometer and PWM.
  R1 illustrates a tachometer pull-up, and R2 a PWM series resistor.
- R4 is in series with D1. STATUS does not bypass R4 to reach the LED.

The public [Noctua PWM white paper](https://cdn.noctua.at/media/Noctua_PWM_specifications_white_paper.pdf)
and [microcontroller guide](https://www.noctua.at/en/support/faqs/microcontroller-guide-pwm-setup-and-rpm-monitoring)
were consulted for the fan signal roles and CMOS PWM / open-collector tachometer
interface distinction. No implementation code or diagram was copied. The generic
module is not a claim that every fan can connect directly to an arbitrary controller.
Actual parts require their own data-sheet, electrical and timing qualification.

All connections are explicit terminal links. The revised drawing aligns the
temperature path, groups the sensor decoupler locally, and draws setpoint, mode,
status and fan connections continuously. Supply aliases remain for remote power.
Labels do not implicitly join objects. Native draft version 3 stores named-path
display separately from remote aliases; version 1/2 drafts remain readable.
The existing symbol definitions are unchanged.

The renderer chooses bounded orthogonal candidates using pin directions,
symbol-body and unrelated-pin obstacles, and prefers paths clear of other nets.
It can still show crossings when its limited candidates cannot avoid them; no
crossing creates connectivity. Labels appear once per visibly linked island;
ordinary endpoints/bends get no junction dot, while branches in the same logical
net do. Blocked display routes show warning stubs rather than drawing through a
symbol. Paint and picking use the same path. This is a display-routing improvement,
distinct from the newer [manual wire editor](wire-editing.md), a general automatic
arranger, or ERC. Manual edits can freeze the affected display route into authored
bends/junctions; unrelated display routes remain unchanged.

These presentation changes apply to freshly opened examples; a saved draft keeps
its authored placement. The route projection follows moves/rotations without
rewriting the document or changing terminal membership.

Focused tests check separated supply rails, the series LED/PWM paths, missing-pin
rejection, normal save/reopen, version 1 reading, atomic properties and undo.
These are model/topology tests, not ERC or electrical safety qualification.

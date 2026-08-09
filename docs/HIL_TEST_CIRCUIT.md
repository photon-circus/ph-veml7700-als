# VEML7700 HIL test circuit

## 1. Fixture goals

The fixture must separately support:

1. digital/protocol and relative optical behavior; and
2. calibrated optical characterization.

Do not promote a bench LED duty cycle to lux without a retained reference
measurement.

## 2. Core hardware

- managed ESP32 harness running the public driver crate;
- VEML7700 carrier with direct access to VDD, GND, SDA, and SCL;
- switchable VDD source with current limit and voltage witness;
- 100 nF local decoupling; optional documented supply filter only when its effect
  is characterized;
- external SDA/SCL pull-ups to a known logic rail;
- logic analyzer on SDA, SCL, VDD-enable, and optional light-source control;
- opaque enclosure controlling ambient leakage;
- stable white LED source with constant-current drive and thermal warm-up;
- at least two reproducible optical levels or a controllable attenuator.

There is no VEML7700 interrupt signal to wire.

## 3. Optical geometry

Record:

- side-view or top-view package variant and orientation;
- sensor-to-diffuser/window distance;
- aperture dimensions and alignment to the sensitive area;
- diffuser/window material and thickness;
- reference sensor position and angular relationship;
- enclosure reflectance and light leaks;
- source distance, angle, spectrum class, drive current, and warm-up time.

## 4. Core relative plan

A calibrated luxmeter is not mandatory for protocol, ratio, cadence, and
threshold-status tests. The source still needs repeatability and a dark baseline.
Use logic capture to prove low-byte-first I²C and exact sequencing.

## 5. Calibrated optical plan

Add a calibrated reference luxmeter or photometric head co-located as closely as
geometry permits. Retain identity, serial number, range, calibration date/state,
raw readings, uncertainty, source metadata, and fixture revision. A DMM reading
of LED current alone is not a lux witness.

## 6. Safing

Fixture exit and panic-stop must:

- stop source drive;
- release I²C pins;
- place sensor in shutdown when communication remains available;
- disable switchable VDD;
- stop/cancel captures;
- retain partial artifacts and mark the run interrupted.

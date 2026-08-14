# Test plan

## Level 1 — Pure value and codec tests

Cover every documented gain, integration time, persistence, power-saving mode,
configuration field combination, reserved encoding, nominal scale, threshold
ordering rule, identity value, and conservative timing bound.

## Level 2 — Strict protocol and failure tests

The scripted I²C transport asserts exact address, pointer, little-endian word
order, payload, transfer count, sequence, and complete script consumption.

Cover probe, observation, snapshot ordering, state-preserving updates,
threshold-monitor conflicts and arming, every fresh-measurement stage, primary
failure, cleanup failure, and post-capture restoration failure.

## Level 3 — Coupled autonomous-state fake

The existing test-only fake covers:

- shutdown retention and explicit wake edges;
- conservative integration deadlines;
- power-saving refresh cadence;
- independent ALS/white refresh timing;
- threshold persistence; and
- sensor state surviving an MCU reset.

These tests are useful but not independent cross-validation because the fake
uses driver types/timing constants and is not driven through I²C.

## Level 4 — Independent I²C-level behavioral model

The first slice is implemented in `ph-veml7700-als-model`. Model-only tests cover
reset, wake/conversion, latching, shutdown retention, stable reads, and duration
partitions. Two driver-versus-model tests cover `probe` and one successful
`measure_once` against the model's I²C boundary and relative-duration input.

The maintained declaration is
[`crates/veml7700-model/README.md`](../crates/veml7700-model/README.md). Agreement
establishes compatibility with that slice only.

Still unimplemented: threshold registers and status, power-saving cadence,
standalone configuration sequences, transport-fault injection, and the rest of
the driver API. Do not treat a green slice as full-device cross-validation.

## Physical evidence

No physical or calibrated-optical protocol is defined here. Physical
qualification is a separate future process that may compare silicon with the
accepted independent model. Host tests cannot justify calibrated lux, fixture,
board, electrical, or silicon claims.

## Canonical gate

`scripts/ci.sh` runs formatting, host tests/checks, clippy, rustdoc, doctests,
representative bare-metal targets, dependency policy, and package verification.
A green gate proves only the implemented host boundary.

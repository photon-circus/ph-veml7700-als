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

## Level 4 — Independent I²C-level behavioral mock

Not implemented yet.

Derive the mock separately from `HARDWARE_CONTRACT.md`. It must autonomously
model register-pointer and little-endian transaction semantics, configuration,
shutdown retention, wake/integration deadlines, ALS and white refresh, power-
saving cadence, threshold persistence/status, reset behavior, identity, and
injectable transport faults.

Driver-versus-mock tests must use public driver APIs and the mock's I²C boundary.
The mock must not import driver codecs or timing helpers as its oracle.

## Physical evidence

No physical or calibrated-optical protocol is defined here. Physical
qualification is a separate future process that may compare silicon with the
accepted independent model. Host tests cannot justify calibrated lux, fixture,
board, electrical, or silicon claims.

## Canonical gate

`scripts/ci.sh` runs formatting, host tests/checks, clippy, rustdoc, doctests,
representative bare-metal targets, dependency policy, and package verification.
A green gate proves only the implemented host boundary.

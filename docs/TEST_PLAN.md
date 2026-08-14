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

`crates/veml7700/src/testing/fake_device.rs` is a standalone state machine
sketching autonomous device behavior: shutdown retention and explicit wake
edges, conservative integration deadlines, power-saving refresh cadence,
independent ALS/white refresh timing, and threshold persistence.

Its tests drive the fake directly and assert on the fake. They never construct
`Veml7700`, so they establish nothing about the driver. The fake additionally
imports driver semantic types and timing constants, so it could not serve as an
independent oracle even if it were connected to one. Read it as exploratory
design notes in executable form, superseded by Level 4 as the model's slice
grows. Retirement or migration is tracked in
[issue #9](https://github.com/photon-circus/ph-veml7700-als/issues/9).

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

`scripts/ci.sh` runs claim checks, formatting, host tests/checks, clippy,
rustdoc, doctests, five bare-metal targets, dependency policy, package
verification, and tests against the unpacked distributable package. Each step is
announced, and the run ends with an explicit pass or failing-step line.

`CI_PROFILE=bounded` selects the subset GitHub Actions runs. It skips dependency
policy, four of the five targets, and packaging, naming each skip, and its
summary states that it covers only part of the release gate. A skipped check is
not a passed check.

The crate currently has no doctests, so the two doctest steps pass with zero
cases; see
[issue #10](https://github.com/photon-circus/ph-veml7700-als/issues/10).
A green gate proves only the implemented host boundary.

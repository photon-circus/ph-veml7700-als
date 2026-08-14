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

## Level 3 — Independent autonomous-state model

Model-only tests cover reset, wake and recurring conversion, shutdown retention,
duration partitions, the documented power-saving cadence table, independently
scheduled ALS/white refreshes, threshold registers, persistence, and stable
polled status. White-channel phase skew is an injected test input, not a claim
about silicon timing.

## Level 4 — Driver-versus-model I²C conformance

Driver-versus-model tests cover `probe`, successful `measure_once`, public
power/cadence configuration, threshold arming and observation, monitor disable,
and sequential channel reads across independently scheduled refreshes. Driver
configuration and observations cross the model's I²C boundary; explicit
relative duration and raw optical samples remain harness inputs.

The maintained declaration is
[`crates/veml7700-model/README.md`](../crates/veml7700-model/README.md). Agreement
establishes compatibility with that slice only.

Still unimplemented: transport-fault injection, arbitrary active
reconfiguration, threshold-flag clearing, source-undeclared reset values, and
unexercised driver operations. Do not treat a green slice as full-device
cross-validation.

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

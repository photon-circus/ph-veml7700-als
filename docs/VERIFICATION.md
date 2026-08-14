# Verification

> **Authority: contributor procedure.** Which test layer establishes what.
> Exact conformance coverage lives in the packaged README, not here.

Each level below establishes something different, and the differences are the
point. A test at one level must never be cited as evidence for another.

| Level | Establishes | Must not be cited for |
| --- | --- | --- |
| 1 — pure/codec | encodings, domains, units, validation | device behavior |
| 2 — scripted I²C | exact transactions, sequencing, injected failures | autonomous device behavior, or model conformance |
| 3 — model-only | the model's own declared behavior | agreement with the driver |
| 4 — driver-versus-model | that the public driver and an independent derivation agree, for named traces | operations, configurations, or initial states it does not exercise |

**The exact level-4 coverage is the packaged matrix in
[`crates/veml7700/README.md`](../crates/veml7700/README.md), not this file.** It
names every covered operation with its initial state and configuration, every
public operation with no conformance trace, and the configuration domain that is
never exercised. The canonical gate fails if it drifts from the test inventory in
either direction.

Two further boundaries that are easy to blur:

- `ph-veml7700-als-model` is repository-only and unpublished.
- Tests run against the unpacked package establish that the published crate
  builds and passes its own tests standalone. They exclude `tests/device_model.rs`
  and the path-only model dependency, so they establish **no** model conformance.

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

Model-only tests cover reset, including undeclared `0x06` remaining unavailable,
wake and recurring conversion, shutdown retention, duration partitions, the
documented power-saving cadence table, independently scheduled ALS/white
refreshes, threshold registers, persistence, and stable polled status.
White-channel phase skew is an injected test input, not a claim about silicon
timing.

## Level 4 — Driver-versus-model I²C conformance

Driver-versus-model tests cover `probe`, successful `measure_once`, public
power/cadence configuration, threshold arming and observation, monitor disable,
and sequential channel reads across independently scheduled refreshes. Driver
configuration and observations cross the model's I²C boundary; explicit
relative duration and raw optical samples remain harness inputs.

The maintained declaration, including unimplemented traces, is
[`crates/veml7700-model/README.md`](../crates/veml7700-model/README.md).

## Canonical gate

`scripts/ci.sh` runs claim checks, formatting, host tests/checks, clippy,
rustdoc, doctests, five bare-metal targets, dependency policy, package
verification, and tests against the unpacked distributable package. Each step is
announced, and the run ends with an explicit pass or failing-step line.

`CI_PROFILE=bounded` selects the subset GitHub Actions runs. It skips dependency
policy, four of the five targets, and packaging, naming each skip, and its
summary states that it covers only part of the release gate. A skipped check is
not a passed check.

A green gate proves only the implemented host boundary.

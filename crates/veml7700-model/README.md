# VEML7700 device behavioral model

This repository-only, unpublished model is an independent executable projection
of the shared VEML7700 evidence. It exists to challenge the driver derivation in
support of user trust, not to make the repository appear complete. A passing
trace establishes only agreement between two derivations within that trace; it
is not silicon evidence, hardware qualification, optical calibration, or
coverage of an untraced API. When evidence cannot support autonomous behavior,
the model uses explicit input, unknown state, or an `Unsupported` result.

The exact conformance inventory lives only in
[`docs/VERIFICATION.md`](../../docs/VERIFICATION.md). This file does not copy it.

## Shared evidence, independent behavior

[`docs/HARDWARE_CONTRACT.md`](../../docs/HARDWARE_CONTRACT.md) owns stable
`S-nn` propositions, evidence state, and provenance. The model cites those IDs
and states only its own consequence. It does not copy vendor prose, source
coordinates, or hardware artifacts.

The model does not depend on `ph-veml7700-als` and does not import driver codecs,
constants, timing helpers, transaction builders, or state machines. Shared
evidence identity is required; shared derivation is forbidden.

## Ideal behavior and injected variation

The model is deterministic and ideal unless a harness explicitly injects
variation:

- first conversion completes at the nominal integration point after the modeled
  wake interval (`S-15`, `S-23`);
- recurring conversion with power saving disabled uses the nominal integration
  interval (`S-15`);
- enabled power-saving cadence is modeled only in the exact `S-21` gain and
  integration domain; the undefined `S-22` gain relation is not assumed;
- no jitter, noise, drift, clock variation, optical physics, or transport delay
  is added internally;
- ALS/white phase offset and elapsed duration are explicit harness inputs.

The driver's tolerance allowance is its separate reaction to `S-24` and `S-55`.
It is intentionally not copied into the ideal model. Hardware variation may be
injected by a test profile without becoming a default device claim.

## Required construction inputs

`Veml7700Model::new` takes `RetainedInputs`; there is no `Default`.

- `als_counts` and `white_counts` are the raw pair available to a later ideal
  conversion. They are not ambient-light generation.
- `initial_power_saving` is a raw harness-selected word because `S-11` is
  undefined. The value selects a scenario; the model does not endorse it. The
  constructor rejects a mode field that contradicts `S-48`, and autonomous
  behavior remains unsupported when injected reserved bits have unknown effect.
- `white_phase_offset` is injected scheduling topology and defaults only through
  the explicit `RetainedInputs::new` constructor choice.

Requiring inputs makes missing scenario choices visible instead of turning a
plausible zero into false evidence.

## Behavioral surface

| Classification | Model consequence |
| --- | --- |
| Modeled | Selected address/identity (`S-05`, `S-43`); independently implemented word transfer and register map (`S-08`, `S-09`); reset configuration (`S-12`); strict codecs; ideal wake/conversion (`S-15`, `S-23`); represented recurring cadence (`S-21`); shutdown retention (`S-25`); threshold programming/readback (`S-16`, `S-38`). |
| Injected | Raw ALS/white pair, initial power-saving word, white-channel phase offset, and bounded relative elapsed duration. |
| Unknown | Unestablished initial threshold, status, and output words (`S-10`, `S-11`). |
| Unsupported | Threshold qualification at every protect number (`S-39`, `S-49`, `S-50`); flag clearing/history (`S-42`, `S-53`, `S-54`); enabled power saving outside the exact `S-21` domain (`S-22`, `S-44`); threshold writes while monitoring; arbitrary active reconfiguration; unrepresented reserved interactions. |
| Excluded | Lux/environment generation, calibration, optics, electrical faults, transport retries, MCU reset, HIL infrastructure, and silicon variation not supplied as input. |

An evidence-dependent `Unsupported` boundary is not resolved by adding more
traces. New source or scoped physical evidence must first address its stable
proposition; the model then reassesses independently and may remain unsupported.

## Transport and state boundary

The model accepts the declared complete-word `write` and combined `write_read`
shapes, and keeps model limitations distinct from modeled device responses.
Adapters must never translate `Unsupported` into a plausible device response.

State changes occur only through transport operations, `set_raw_sample`,
`set_white_phase_offset`, and `advance`. `inspect` is non-mutating. Bus calls
consume no invented time, and a repeated supported read at an unchanged
temporal frontier is stable.

Active reconfiguration is rejected as the model's independent reaction to
`S-56`. Setting shutdown without changing the remaining configuration and
disabling an enabled monitor are supported transitions, not evidence about
undefined status history.

Evidence changes follow
[`CONTRIBUTING.md`](../../CONTRIBUTING.md#updating-shared-evidence); this file
records only the model consequence.

# VEML7700 device behavioral model

This crate is the maintained behavioral declaration for the independent
VEML7700 device model. Passing tests establish compatibility with this declared
slice only. They do not establish correctness on silicon, calibrated optics, or
support for the rest of the driver API.

## Claim

- Device identity and behavioral selection: Vishay VEML7700 at fixed 7-bit I²C
  address `0x10`, datasheet baseline (no silicon variant).
- Purpose and current consumer: host tests of `ph-veml7700-als` `probe` and one
  successful `measure_once` flow.
- What agreement with this model establishes: the public driver and this
  independently derived interpretation agree for those exercised traces
  (address, ID, byte order, injected ALS/white pair, delay-driven conversion,
  and configuration/power-saving restoration).
- What agreement with this model does not establish: support for other driver
  features, undocumented silicon behavior, analog/electrical timing, optical
  physics, or physical-hardware qualification.

## Sources

- VEML7700 datasheet, Vishay document 84286, Rev. 1.8, 28-Nov-2024:
  <https://www.vishay.com/docs/84286/veml7700.pdf>
  - SHA-256: `f338cf7d5911828a2f2ac8ae8324049380c852e34aa5afa43ac92c98ffe827d1`
  - retrieved 2026-08-13; 295,562 bytes
- *Designing the VEML7700 Into an Application*, Vishay document 84323,
  06-Mar-2025: <https://www.vishay.com/docs/84323/designingveml7700.pdf>
  - SHA-256: `422f2bea390e145d0d082f40fdeaad4945c79beec159d6600d4007da0aaed558`
- Repository interpretation and nonclaims: [`docs/HARDWARE_CONTRACT.md`](../../docs/HARDWARE_CONTRACT.md)
  and [`docs/DECISIONS.md`](../../docs/DECISIONS.md).

Vendor PDFs remain untracked. Owner-verification boxes in the hardware contract
stay provisional and are not physical-support claims.

## Behavioral boundary

### Inputs

- Transport operations: complete I²C `write` of `[pointer, low, high]` and
  `write_read` of a one-byte pointer returning two data bytes, at address `0x10`,
  limited to configuration (`0x00`), power saving (`0x03`), ALS (`0x04`), white
  (`0x05`), and ID (`0x07`).
- Applied stimuli: a persistent raw pair `{ als_counts, white_counts }`. This is
  the result available to a completed conversion, not ambient lux.
- Relative-duration input: non-negative nanosecond-resolution elapsed duration.
  Driver `DelayNs` requests must reach this same input in conformance tests.
- Injected events: none. Construction starts from the documented reset/default
  state. A separately injectable POR event is not part of this slice.

### Outputs and observations

- Device responses: source-backed address NACK for a non-`0x10` 7-bit address;
  low-byte-first 16-bit register words for supported accesses.
- Device outputs visible outside the transport: none. INT/GPIO is not modeled.
- Pure inspection: `inspect()` returns frozen register and conversion-progress
  fields for model tests only and does not mutate the model.

### State and mutation

- State retained: configuration and power-saving words; active/shutdown;
  the held raw sample; the last completed ALS/white pair; remaining progress
  toward the current conversion.
- Inputs that permit mutation: ordered `write` / `write_read`, `set_raw_sample`,
  and `advance`.
- Documented transport side effects: none in this slice. Register reads do not
  clear flags or consume time.
- Stable behavior at an unchanged temporal frontier: repeated supported reads
  with no duration step return the same result.

## Fidelity

| Classification | Included behavior |
| --- | --- |
| Modeled | Fixed address and ID; reset/default configuration; low-byte-first access to configuration, power-saving, ALS, and white; supported measurement configuration; shutdown-to-active wake; conservative conversion completion; shutdown retention; configuration and power-saving restoration. |
| Abstracted | Analog conversion latches the currently held raw ALS/white pair at the conservative completion boundary. The timing bound is deterministic rather than an oscillator or tolerance simulation. |
| Injected | Raw ALS/white pair and relative elapsed duration. |
| Excluded | Lux/environment generation, optical physics, noise, jitter, drift, electrical timing, transport faults/retries, MCU or post-construction device reset, scheduler/topology, HIL evidence, and silicon calibration. |
| Unsupported | Threshold registers, persistence and status; power-saving-enabled cadence; standalone sequences beyond this slice; source-undeclared or reserved interactions; mid-conversion reconfiguration beyond the supported freeze; all other registers or sequences not needed by `probe` and successful `measure_once`. |

## Source decisions

- Completion boundary: after the shutdown-to-active wake edge, latch the held
  raw pair at 2.5 ms wake allowance plus 130% of the selected integration time.
  This is deterministic model behavior, not a claim that silicon always converts
  at that instant. The driver may wait longer (it adds software margin).
- Elapsed duration: retain remaining conversion progress so valid partitions of
  the same duration are observationally equivalent. No absolute time is stored.
- Result pair: latch ALS and white together. This does not claim a vendor
  atomic-pair primitive; it tests the driver's shutdown-before-read policy.
- Shutdown: entering shutdown prevents further conversion progress and preserves
  the last completed pair.
- Unsupported interactions: reject or leave unavailable. Do not fabricate a
  plausible NACK, register value, or transition.

## Independence and proportionality

- Derivation uses the pinned sources and hardware contract. This crate does not
  depend on `ph-veml7700-als` and does not import driver masks, codecs, timing
  helpers, transaction builders, or state machines.
- A separate workspace crate makes that dependency direction inspectable.
  Model-only tests run without the production driver.
- The only extra artifact is small test-side I²C/`DelayNs` glue in the driver
  crate's conformance tests. Those adapters are not this model's behavioral API
  and must not hide model limitations as device responses.

## Known limitations and change discipline

- Model limitations (`TransportError::Unsupported`) are distinct from device
  address NACK. Adapters must preserve that distinction.
- Later silicon evidence may correct this baseline or introduce a selected
  variant; it must not silently replace the datasheet interpretation.
- Shared duration types, transport-phase granularity, and multi-device
  coordination remain deferred organization questions and are not resolved here.
- Extend the claim only when a concrete test needs the behavior and its source
  ambiguities have been resolved.

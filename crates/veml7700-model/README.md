# VEML7700 device behavioral model

This crate is the maintained behavioral declaration for the independent
VEML7700 device model. Passing tests establish compatibility with this declared
slice only. They do not establish correctness on silicon, calibrated optics, or
support for the rest of the driver API.

## Claim

- Device identity and behavioral selection: Vishay VEML7700 at fixed 7-bit I²C
  address `0x10`, datasheet baseline (no silicon variant).
- Purpose and current consumer: host tests of `ph-veml7700-als` probe, fresh
  measurement, power-saving cadence, threshold monitoring, and sequential
  ALS/white observation traces.
- What agreement with this model establishes: the public driver and this
  independently derived interpretation agree for those exercised traces
  (address, ID, byte order, injected ALS/white samples and scheduling skew,
  delay-driven refresh, configuration/power-saving restoration, documented
  cadence, and threshold qualification).
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

Vendor PDFs remain untracked. Owner verification of the hardware contract is
partly complete: the electrical and bus boundary, the power-saving refresh
table, the resolution and maximum-range tables, the gain and integration
encodings, the linearity limits, and the reconfiguration sequence are verified
against the pinned sources. The remaining rows — including both §1
source-baseline entries — stay provisional. No verified row is a
physical-support claim: confirming that a recorded interpretation matches a
recorded document establishes nothing about silicon.

## Behavioral boundary

### Inputs

- Transport operations: complete I²C `write` of `[pointer, low, high]` and
  `write_read` of a one-byte pointer returning two data bytes, at address `0x10`,
  limited to configuration (`0x00`), high threshold (`0x01`), low threshold
  (`0x02`), power saving (`0x03`), ALS (`0x04`), white (`0x05`), threshold
  status (`0x06`), and ID (`0x07`).
- Address values outside `0x00..=0x7F` are model-input limitations, not device
  NACKs. Other valid 7-bit addresses receive the modeled address NACK.
- Applied stimuli: a persistent raw pair `{ als_counts, white_counts }`. This is
  the result available to a completed conversion, not ambient lux.
- Relative-duration input: non-negative nanosecond-resolution elapsed duration.
  Driver `DelayNs` requests must reach this same input in conformance tests.
- Injected white-channel phase offset: non-negative relative duration applied to
  future wake edges. It creates test scheduling topology and is not a silicon
  timing claim.
- Injected events: no POR, transport fault, or flag-clear event. Construction
  starts from the documented reset/default configuration state.

### Outputs and observations

- Device responses: source-backed address NACK for a non-`0x10` 7-bit address;
  low-byte-first 16-bit register words for supported accesses.
- Device outputs visible outside the transport: none. INT/GPIO is not modeled.
- Pure inspection: `inspect()` returns frozen register and conversion-progress
  fields for model tests only and does not mutate the model.

### State and mutation

- State retained: configuration, power-saving and programmed threshold words;
  threshold qualification/status; active/shutdown; the held raw sample; the
  independently completed ALS/white values; and each channel's remaining
  refresh progress.
- Inputs that permit mutation: ordered `write` / `write_read`, `set_raw_sample`,
  `set_white_phase_offset`, and `advance`.
- Documented transport side effects: none in this slice. Register reads do not
  clear flags or consume time.
- Stable behavior at an unchanged temporal frontier: repeated supported reads
  with no duration step return the same result.

## Fidelity

| Classification | Included behavior |
| --- | --- |
| Modeled | Fixed address and ID; reset/default configuration; low-byte-first access to every declared register; supported measurement configuration; shutdown-to-active wake; recurring refresh; shutdown retention; documented 100–800 ms power-saving cadence; threshold programming, persistence and polled status; configuration and power-saving restoration. |
| Abstracted | Refreshes deterministically latch held channel values at conservative boundaries. Qualified threshold flags remain set within the slice; no silicon clearing behavior is claimed. Construction represents a device with no prior threshold qualification, so the first monitored ALS refresh establishes the whole `0x06` word and an unqualified flag then reads clear. |
| Injected | Raw ALS/white pair, relative elapsed duration, and white-channel phase offset. |
| Excluded | Lux/environment generation, optical physics, noise, jitter, drift, electrical timing, transport faults/retries, MCU or post-construction device reset, HIL evidence, silicon calibration, and actual ALS/white phase behavior. |
| Unsupported | Enabled power saving at 25/50 ms; threshold, threshold-status (`0x06`), and output reset values not declared by sources; threshold-flag clearing/deassertion; threshold writes while monitoring; arbitrary active reconfiguration; source-undeclared or reserved interactions; and unexercised standalone sequences. |

## Source decisions

- Completion boundary: after the shutdown-to-active wake edge, latch the held
  raw pair at 2.5 ms wake allowance plus 130% of the selected integration time.
  This is deterministic model behavior, not a claim that silicon always converts
  at that instant. The driver may wait longer (it adds software margin).
- Elapsed duration: retain remaining conversion progress so valid partitions of
  the same duration are observationally equivalent. No absolute time is stored.
- Recurring refresh: with power saving disabled, use 130% of the selected
  integration time; with power saving enabled, use the exact vendor table for
  100, 200, 400, and 800 ms. Reject 25/50 ms rather than extrapolating.
- Channel scheduling: ALS and white have independent countdowns. An injected
  white offset permits cross-generation tests without inventing a fixed silicon
  phase relationship.
- Threshold qualification: evaluate strict below-low and above-high conditions
  on ALS refresh and assert status after 1, 2, 4, or 8 consecutive qualifying
  results. Reads do not clear status; later clearing semantics remain outside
  the slice.
- Initial threshold-status history: the sources declare no reset value for
  `0x06`, and this model asserts flags without ever clearing them, so a
  refresh that qualifies nothing cannot by itself prove a flag is clear. The
  model therefore *declares* that construction represents a device with no
  prior qualification, and the first monitored ALS refresh establishes the
  whole word from qualification alone. Observable consequence: after that
  refresh, an unqualified flag reads clear. This is a declared abstraction
  chosen to keep the polled-status path testable, not a source-backed reset
  value, and it is the only bit of `0x06` behavior not derived from a
  qualification transition.
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
- The pinned sources do not declare reset values for threshold, threshold-status
  (`0x06`), ALS, or white output registers. Reading an output before conversion,
  a threshold before programming, or threshold status before a monitored ALS
  refresh is an explicit model limitation rather than an invented value.
- Later silicon evidence may correct this baseline or introduce a selected
  variant; it must not silently replace the datasheet interpretation.
- Shared duration types, transport-phase granularity, and multi-device
  coordination remain deferred organization questions and are not resolved here.
- Extend the claim only when a concrete test needs the behavior and its source
  ambiguities have been resolved.

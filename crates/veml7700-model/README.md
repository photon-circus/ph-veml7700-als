# VEML7700 device behavioral model

This crate is the maintained behavioral declaration for the independent
VEML7700 device model.

**It is repository-only and unpublished.** It exists to cross-check
`ph-veml7700-als` during development and is excluded from that crate's published
package, so it cannot be depended on from a registry. Its manifest retains
`publish = false` deliberately, and nothing in the driver's public surface
exposes it. Passing tests establish compatibility with this declared
slice only. They do not establish correctness on silicon, calibrated optics, or
support for the rest of the driver API.

## Claim

- Device identity and behavioral selection: Vishay VEML7700 at fixed 7-bit I²C
  address `0x10`, datasheet baseline (no silicon variant).
- Purpose and current consumer: host tests of `ph-veml7700-als` probe, fresh
  measurement, power-saving cadence, threshold monitoring, and sequential
  ALS/white observation traces.
- What agreement with this model establishes: the public driver and this
  independently derived interpretation agree, for the traces that exist.
  **Which traces those are is not restated here.** The exact operation, initial
  state, and configuration coverage — including the public operations with no
  trace at all, and the configuration domain never exercised — lives in one
  place, the coverage matrix in
  [`crates/veml7700/README.md`](../veml7700/README.md), because that surface
  ships to a consumer and this one does not.

  A second list here would be a second thing to keep true. The canonical gate
  compares the matrix against the executable test inventory in both directions;
  it cannot police a prose copy in this file, which is exactly why there is not
  one.
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

Vendor PDFs remain untracked. Owner verification of the hardware contract has
been walked end to end: 41 rows are verified against the pinned sources,
including both §1 source-baseline entries, which the owner closed by recomputing
both SHA-256 digests over the retrieved copies. Three rows remain open and each
states its own obstacle in place.

No verified row is a physical-support claim: confirming that a recorded
interpretation matches a recorded document establishes nothing about silicon.

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
  the result available to a completed conversion, not ambient lux. It is
  **required at construction** via `RetainedInputs` and has no default — see
  below.
- Relative-duration input: non-negative nanosecond-resolution elapsed duration,
  bounded by `MAX_ADVANCE` (one hour of virtual time) per step. Driver `DelayNs`
  requests must reach this same input in conformance tests.
- Injected white-channel phase offset: non-negative relative duration applied to
  future wake edges. It creates test scheduling topology and is not a silicon
  timing claim.
- Injected events: no POR, transport fault, or flag-clear event. Construction
  starts from the documented reset/default configuration state.

#### The model does not invent stimuli

`Veml7700Model::new` takes `RetainedInputs` and there is no `Default`.
Construction previously zeroed the held sample and the white phase offset, so a
harness that woke the model without injecting a sample received a conversion
reporting **zero ambient light** — a reading it never supplied.

Nothing failed, which is the point. Zero is a plausible ALS value, so the
fabricated sample flowed through conversions, threshold comparisons and
driver-versus-model traces looking exactly like an injected one.

This is the third instance of one pattern in this crate, and the reason it is
stated here rather than in a changelog entry: **an invented value does not
produce a conformance failure, it produces agreement.** The persistence rule
(withdrawn), the register `0x03` reset value (declared as an assumption), and now
the retained sample (made required). The rule that follows is the one worth
carrying forward — where this model would otherwise guess, it must either declare
the guess or refuse to run.

Two smaller cases of the same shape were closed alongside it. `from_micros` used
to saturate, silently substituting roughly 584 years for whatever was asked; it
now rejects overflow. The white-channel wake edge used to saturate; it is now
provably non-overflowing because the offset is bounded where it enters.

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
| Modeled | Fixed address and ID; reset/default configuration; low-byte-first access to every declared register; supported measurement configuration; shutdown-to-active wake; recurring refresh; shutdown retention; documented 100–800 ms power-saving cadence; threshold programming at every protect number, and polled status at protect number one; configuration and power-saving restoration. |
| Abstracted | **Two declared assumptions, tabulated below.** Refreshes deterministically latch held channel values at the vendor-stated conservative boundary, which is an abstraction of application-note guidance rather than an independently chosen number. Qualified threshold flags remain set within the slice; no silicon clearing behavior is claimed. Construction represents a device with no prior threshold qualification, so the first monitored ALS refresh establishes the whole `0x06` word and an unqualified flag then reads clear. |
| Injected | Raw ALS/white pair and white-channel phase offset, both required at construction; relative elapsed duration, bounded per step. |
| Excluded | Lux/environment generation, optical physics, noise, jitter, drift, electrical timing, transport faults/retries, MCU or post-construction device reset, HIL evidence, silicon calibration, and actual ALS/white phase behavior. |
| Unsupported | Threshold qualification above protect number one, for which the sources state the counting condition (`S-39`) but neither its sufficiency nor what a non-qualifying measurement does to a partial run (`S-40`, `UndefinedQualificationRule`); enabled power saving at 25/50 ms (`S-44`); threshold, threshold-status (`0x06`), and output reset values not declared by sources (`S-10`); threshold-flag clearing/deassertion (`S-42`); threshold writes while monitoring; arbitrary active reconfiguration; source-undeclared or reserved interactions; and unexercised standalone sequences. |

### Declared assumptions

Two model behaviors rest on facts the pinned sources do not state (`S-11`,
`S-22`). They are
collected here rather than left in the code, because a reader deciding what
model agreement is worth needs to know where the model is guessing.

Each is recorded in `docs/HARDWARE_CONTRACT.md` beside the row it affects. Both
are declared **Assumptions** under D-029: unresolvable by any further reading,
closable only with a part on a bench. Each row names the observation that would
settle it; the procedures live in #58, because this repository does not carry
physical-evidence plans.

| Assumption | Where the model relies on it | Contract state | Observable consequence |
| --- | --- | --- | --- |
| Register `0x03` reads `0x0000` before it is written | `Veml7700Model::new` via `RESET_POWER_SAVING` | **Assumption** (D-029) | A harness that never writes `0x03` sees continuous-conversion cadence. The **driver** has no such assumption — it reads the register before acting on it. |
| Refresh time does not depend on ALS gain | `refresh_interval_ns`, which takes no gain argument | **Assumption** (D-029) | Every cadence the model predicts is gain-independent. If silicon disagreed, model and driver would agree with each other and both diverge from the part. |

The 130 % conversion boundary is **not** in this table. It is not a
source-silent assumption: application note 84323 states that a ±30 %
integration-time tolerance can be assumed, so `conversion_bound_ns` latching at
130 % of nominal is an abstraction of vendor guidance, not a number this
repository invented. What that guidance does not supply is a worst case
characterized across process, voltage, and temperature — so a real spread wider
than ±30 % would still make the model complete a conversion the device has not.
That is a fidelity limit of a deterministic model, recorded under
[Source decisions](#source-decisions), not a place where the model guesses at
silence.

### The assumption this model removed

There were four. Two left the table for different reasons, and the difference is
worth keeping straight.

The persistence qualification rule left because under D-030 a model assumes only
what it needs to run, and nothing here forced a rule: `update_threshold_status`
now qualifies at protect number one and reports
`Unsupported::UndefinedQualificationRule` above it. That is a model that stopped
guessing.

It stayed gone when the sources turned out to say more than this repository had
found. The application note gives the counting condition — a flag is set *only
when* `ALS_PERS` measurements stay above or below the threshold — but in
necessary form, and with nothing about what a non-qualifying measurement does to
a partial run. Counting needs both halves, so the model still declares the rule
undefined rather than supplying the missing one. See #73.

The ±30 % integration tolerance left for the opposite reason — nothing about the
model changed. The source turned out to state the figure, so the row was
misclassified, not over-assumed. Only the classification moved.

It is worth recording what that assumption had been doing, because it is the
failure the whole model-conformance apparatus exists to prevent, found in this
repository rather than in the field.

The driver only *programs* `ALS_PERS`; `Persistence::count()` is an accessor and
no driver logic reads it. So the conformance trace at persistence 4 advanced
four refreshes and asserted the flag — confirming a register write while reading
like confirmation of when the flag asserts. The model had invented a counting
rule, the driver never had one to disagree with, and the agreement between them
looked exactly like evidence.

The lesson generalizes past this row: **an invented model behavior does not
produce a conformance failure.** It produces agreement, because the driver is
usually silent wherever the sources are. Where the model guesses, it must guess
visibly or not at all.

The second is the one that most limits what conformance establishes. Driver and
model derive it independently but from the same silent source, so agreement
there is not corroboration — it is two derivations sharing an assumption. That
is a known limit of an independent model against a document, and only physical
evidence closes it.

### This slice is a stage, not a ceiling

The model is narrow, and that narrowness is the honest consequence of never
having measured the part. It is not a defect to be argued away or a placeholder
to be quietly widened.

The intended direction is progressive: as hardware evidence arrives, each
assumption above is either confirmed and promoted to a verified row, or refuted
and corrected in both the model and the driver — and the crate's level of
confirmation rises with it. The three rows are ordered work, not caveats. Each
already names the observation that would settle it — collected in #58 — so the
first hardware-in-the-loop session has a list rather than a research problem.

There is a fourth way an open question closes, and this crate has now used it:
the persistence rule was removed rather than settled. Where nothing forces the
model to have an opinion, withdrawing one is progress too, and it needs no
hardware.

Two constraints on that growth, so it stays honest as it happens. Physical
evidence must correct this baseline rather than silently replace it — a
measurement that contradicts a source is a recorded discrepancy, not an edit.
And the claim must never run ahead of the evidence: the slice widens *after* a
measurement lands, never in anticipation of one.

### Active reconfiguration is unsupported because the sources say so

The rejection of changed or repeated active configuration, and of active
power-saving changes, is source-backed rather than a modelling convenience. The
vendor's software flow sets `ALS_SD = 1` before any reconfiguration, changes
fields while shut down, and clears `ALS_SD` afterwards; `docs/HARDWARE_CONTRACT.md`
§5 records this as a verified row.

Only two writes are accepted while active, because both are transitions rather
than reconfigurations: setting the shutdown bit with every other field
unchanged, and disabling an enabled monitor with every other field unchanged.

This was the one place the driver and the model disagreed. It was resolved
against the driver: the driver now shuts down before reconfiguring, so the model
was not relaxed to admit the sequence it had been rejecting. Two
driver-versus-model traces in `tests/conformance/` start from
an active device and would fail against the previous driver with
`Unsupported::MidConversionReconfiguration`.

## Source decisions

- Completion boundary: after the shutdown-to-active wake edge, latch the held
  raw pair at 2.5 ms wake allowance plus 130% of the selected integration time.
  The 130% is an abstraction of the vendor's stated ±30% integration-time
  tolerance (application note 84323, page 4, *Command Code ALS_IT*), collapsed
  to a single deterministic instant. It is not a claim that silicon always
  converts at that instant, and the vendor states the tolerance as assumable
  rather than characterizing it. The driver may wait longer (it adds software
  margin).
- Elapsed duration: retain remaining conversion progress so valid partitions of
  the same duration are observationally equivalent. No absolute time is stored.
- Recurring refresh: with power saving disabled, use 130% of the selected
  integration time; with power saving enabled, use the exact vendor table for
  100, 200, 400, and 800 ms. Reject 25/50 ms rather than extrapolating.
- Channel scheduling: ALS and white have independent countdowns. An injected
  white offset permits cross-generation tests without inventing a fixed silicon
  phase relationship.
- Threshold qualification: evaluate strict below-low and above-high conditions
  on ALS refresh, and assert status **only at protect number one**, where a
  single qualifying refresh needs no counting rule. Above one the sources state
  no rule, so the model reports `UndefinedQualificationRule` rather than
  selecting one. Reads do not clear status; later clearing semantics remain
  outside the slice.
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
  (`0x06`), ALS, or white output registers (`S-10`). Reading an output before conversion,
  a threshold before programming, or threshold status before a monitored ALS
  refresh is an explicit model limitation rather than an invented value.
- Later silicon evidence may correct this baseline or introduce a selected
  variant; it must not silently replace the datasheet interpretation.
- Shared duration types, transport-phase granularity, and multi-device
  coordination remain deferred organization questions and are not resolved here.
- Extend the claim only when a concrete test needs the behavior and its source
  ambiguities have been resolved.

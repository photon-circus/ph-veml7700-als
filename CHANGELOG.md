# Changelog

All notable changes to this repository are documented here.

## [Unreleased]

This section will become the first release, `0.1.0-incubating.1`. Nothing has
been released or published, so every entry describes the initial surface rather
than a change from a prior version.

### Added

- Async, `no_std`, allocation-free VEML7700 driver over caller-provided
  `embedded-hal-async` I²C: explicit snapshot-versus-fresh semantics,
  conservative timing bound to the selected integration time, restoration-aware
  fresh capture, a typed threshold-monitor domain that rejects silent
  retargeting, raw ALS and white counts, and integer nominal micro-lux scaling
  from the vendor resolution table.
- Compiled crate-level usage example covering inert construction, `probe`, and
  the snapshot-versus-fresh distinction, mirrored in the packaged README and
  kept identical to it by the canonical gate.
- Concrete preserved bus errors carrying semantic operation, register, and stage
  context, including distinct primary and recovery failures when restoration
  also fails and a captured sample survives a failed restore. `probe` reports
  through its own `ProbeError`, because address NACK means absence only there.
  Every variant of every public error enum is reachable, enforced by the gate.
  Error types are `#[non_exhaustive]` so later variants stay additive, while the
  device value types are exhaustive so a caller still gets a compile error for
  an unhandled gain or integration time.
- Pure codec tests across every documented field combination and reserved
  encoding; exact scripted-I²C tests asserting address, pointer, little-endian
  word order, payload, and transaction count; per-stage failure injection for
  both fresh capture and threshold-monitor programming.
- Independent `ph-veml7700-als-model` crate: a quiescent, datasheet-derived
  device behavioral model covering `probe`, successful `measure_once`,
  autonomous power-saving cadence, threshold persistence/status, and injected
  ALS/white scheduling skew at the I²C boundary. Model-only and
  driver-versus-model tests exercise the declared slice; its README maintains
  the claim, fidelity, sources, and nonclaims.
- One canonical local verification gate, `scripts/ci.sh`, covering formatting,
  host tests, lints with warnings denied, rustdoc, five bare-metal targets,
  dependency and license policy, package construction and inspection, and tests
  against the unpacked distributable package.
- Two claim checks in that gate, so the repository's load-bearing promises fail
  loudly rather than drift: vendor documents must not be tracked, and the
  required status disclosure must be identical in the root README, the packaged
  crate README, and the crate documentation.
- One version across both workspace crates, `0.1.0-incubating.1`, with the gate
  failing on divergence and asserting the lifecycle-matching prerelease without
  storing a second copy of the literal.
- Bounded GitHub Actions workflow running the `bounded` profile of that same
  script, so there is no second implementation of the gate. It cancels
  superseded pull-request runs, pins its one third-party action to a commit
  SHA, uses read-only permissions, and exposes a stable aggregate `ci` result
  for branch protection. It is dispatch-only while the repository is private and
  gains its automatic triggers at the visibility change.
- Contributor bug-report form and pull-request template, both of which require
  an explicit evidence source so mock, model, and simulated results cannot
  quietly become hardware claims.
- Documented release procedure separating candidate preparation, repository
  visibility, and registry publication into explicit maintainer decisions.

### Documentation

- Removed stale coupled-fake references, deleted the `docs/` index that restated
  the contract table, and replaced repeated model nonclaims with links to
  `crates/veml7700-model/README.md`.
- Corrected the status disclosure, which still described model coverage as
  `probe` and one successful `measure_once` path after the model gained
  power-saving cadence, threshold monitoring, and sequential channel
  observation.
- The hardware-contract register map now records which registers have
  source-declared reset values and which do not.
- `SECURITY.md` now routes reporters to GitHub private vulnerability reporting,
  with a monitored email address for anyone who will not create an account,
  instead of directing them to "the repository owner" with no route at all. It
  states scope, supported-version posture, and disclosure preference without
  promising a response time.
- `CODE_OF_CONDUCT.md` gained scope, a confidential reporting route, a
  proportionate enforcement ladder, and an escalation path for reports
  concerning the maintainer.
- `CONTRIBUTING.md` is now self-sufficient for a human contributor: setup and
  pinned tool prerequisites, fast-versus-full verification, which test layer
  owns which claim, evidence-source language, per-document authority, the pull
  request workflow, and contribution licensing. It no longer sends contributors
  to `AGENTS.md`, and no longer describes every file under `docs/` as normative.
- Added a feature-proposal issue form, which the repository's own template
  directory had been suppressing, and pointed the issue-template contact links
  at specific contract and evidence documents rather than the raw `docs/` tree.
- The pull request template gained the organization's purpose, governing
  decision, contract/compatibility, evidence-table, documentation/licence/package
  and handoff fields, and now distinguishes scripted-I²C from pure unit evidence.
- Recorded D-025: no `CODEOWNERS` while the project has one maintainer, with the
  paths it should cover when a second joins.
- `RELEASING.md` now states that `cargo publish` repackages from the source
  tree, so the reviewed archive is evidence about a tree rather than the exact
  bytes the registry receives, and requires publication from that same unchanged
  clean pinned tree followed by download-and-verify of the registry artifact.
- **Corrected the I²C clock-frequency row.** It read "standard and fast mode are
  supported from 10 kHz through 400 kHz", one range spanning both modes. The
  source specifies `f(SMBCLK)` separately per mode: 10 kHz to 100 kHz standard,
  10 kHz to 400 kHz fast. The old wording would have permitted standard mode at
  400 kHz. The row also now records that the source marks these values as
  protocol-derived and not production tested.
- The hardware contract tracks verification per fact rather than per section,
  so a partly source-backed section can record both states. §6 and §8 are now
  both: the refresh table and the gain ×2 resolution column are verified, while
  the register field layout, the other gain columns, and the 25 ms and 50 ms
  rows are not.
- Recorded that the driver treats power-saving refresh time as independent of
  ALS gain. The source states the relation at gain ×2 only, so this is an
  inference; it is now visible in the contract rather than implicit in
  `nominal_refresh_time_ms`.
- Verified the complete twenty-four-entry resolution table and recorded the
  matching maximum-detection-range table beside it, so the full-scale range of
  every gain and integration pair is stated rather than derived at the call
  site. Gain ×1/8 at 25 ms reaches 140 926 lx; at 100 ms it reaches 35 232 lx.
- Recorded the source's linearity limits and correction guidance: gain ×1 and
  ×2 are confined to illumination below 100 lx, linear behavior spans 0.0042 lx
  to about 1 klx, and correction is called for with gain ×1/4 and ×1/8 and above
  1 000 lx. The driver still does not apply the polynomial — that remains D-007 —
  but the contract now states the consequence for `nominal_illuminance` instead
  of leaving it implied, and records the coefficients as a device fact rather
  than as work owed by this crate. Evaluating the quartic on target would mean
  floating point, which neither crate uses and several supported triples have no
  FPU for; the contract names `ph-curves` as the intended home, since it fits
  curves host-side and emits integer tables, and notes that `ph-temt6000-als`
  already pairs an illuminance layer with it.
- Recorded the source's starting-configuration guidance: begin at the lowest
  gain for unknown brightness, and use an integration time below 100 ms to cover
  the brightest conditions. This is the source basis a first-use preset needs.
- Recorded that the source places auto-ranging in application software, so
  automatic range selection remaining a non-claim follows the source's own
  framing rather than being only a scope decision.
- Logged two vendor prose-versus-table discrepancies: a stated range of "0 lx to
  230 lx" where the table gives 275 lx, and a ranging example that computes
  46 lx where its own arithmetic and stated logic give 54 lx.
- **Established that reconfiguration requires shutdown first.** The source's
  software flow sets `ALS_SD = 1` before any reconfiguration, changes gain or
  integration time while shut down, and clears `ALS_SD` afterwards. This is a
  positive requirement rather than an absent permission, and it resolves the
  driver-versus-model disagreement in #29 against the driver: the model's
  rejection of active reconfiguration is correct, and `set_measurement_config`
  currently writes without entering shutdown. Correcting that is a behavior
  change owned by #29.
- Verified the integration-time encodings and the 2.5 ms minimum wake-up delay.

### Changed

- Every host `cargo test` in the canonical gate now passes an explicit
  `--target`, resolved from `rustc -vV` rather than assumed. Without it Cargo
  writes test executables straight into `target/debug`, where a Windows
  Application Control policy intermittently refuses to launch them:
  `Couldn't run the test: An Application Control policy has blocked this file.
  (os error 4551)`.

  The tests passed; the harness could not start the binary it had just built.
  It surfaced as the gate failing at a *different* doctest step on each run,
  which is the worst shape a gate failure can take — it reads as flakiness in
  the code under test, and the natural response is to re-run until green. An
  authoritative gate that is sometimes wrong about its own subject is worse than
  a slow one.
- The model no longer invents its own stimuli. `Veml7700Model::new` takes a
  required `RetainedInputs` carrying the raw ALS/white pair and the white-channel
  phase offset, and `Default` is gone rather than reimplemented.

  Construction used to zero all three, so a harness that woke the model without
  calling `set_raw_sample` received a conversion reporting **zero ambient
  light** — a reading it never supplied. Nothing failed, which is the point:
  zero is a plausible ALS value, so the fabricated sample flowed through
  conversions, threshold comparisons and driver-versus-model traces looking
  exactly like an injected one.

  That is now the third instance of one pattern, and the model README states the
  rule rather than the incident: **an invented value does not produce a
  conformance failure, it produces agreement.** The persistence rule was
  withdrawn, register `0x03`'s reset value was declared as an assumption, and the
  retained sample is now required. Where the model would otherwise guess, it must
  declare the guess or refuse to run.
- `RelativeDuration::from_micros` rejects overflow instead of saturating, and
  gains `try_from_micros` for non-literal input. Saturation silently substituted
  roughly 584 years of virtual time for whatever was asked, and every later
  assertion was then made against a timeline nobody chose. `RelativeDuration`
  also gains `ZERO`, so "no offset" is written rather than defaulted.
- `advance` rejects steps beyond the new `MAX_ADVANCE` (one hour of virtual
  time), **before any mutation**, so a caller that catches the rejection observes
  an unchanged model. Recorded as D-031, which states that this is a
  model-domain constraint rather than a performance guard — the difference
  decides whether raising it is safe — and why the loop rejects instead of
  batching event-free recurrence. The loop runs once per refresh event and the shortest
  recurrence is about 32.5 ms, so a `u64::MAX` nanosecond input implied roughly
  568 billion iterations — not an error, just a hang, which is the worst way for
  a suite to report a bad argument.
- The white-channel wake edge is computed with a checked add rather than a
  saturating one. Bounding the phase offset where it enters the model makes
  overflow unrepresentable, so the check is unreachable today and stays loud if
  that bound is ever loosened.
- Model tests cover **all six integration times** immediately before and exactly
  at their first conversion boundary; only 100 ms was covered. A boundary
  computed from the wrong integration constant is exactly the defect this model
  exists to catch in the driver, so leaving five of six untested left the oracle
  unchecked at the value it is asked about most. Model tests 25 → 30.
- Driver-versus-model conformance moved out of the driver package into a third
  workspace package, `tests/conformance` (`ph-veml7700-als-conformance`,
  unpublished, `0.0.0`). The dependency arrow is now
  `conformance → driver + model`, with nothing pointing back.

  The point is that the independence claim became **checkable rather than
  stated**. Previously the driver dev-depended on the model and excluded one test
  path from the package; a driver unit test could have reached the model
  directly, and nothing would have failed. Now `cargo test -p ph-veml7700-als`
  does not build the model at all, so that mistake is unavailable rather than
  discouraged.

  The canonical gate runs the three layers as three visibly separate steps
  instead of one, because a combined invocation hides which layer failed. 15
  steps became 17.

  The adapters moved to the conformance package's library, where the rule that
  matters is stated at the top: a model limitation and a source-backed device
  NACK must never become the same error. Burying that in a test file made it
  advice; as a documented public boundary it is reviewable.
- `MeasurementConfig` and `PowerSavingConfig` encodings gained **literal
  contract vectors** alongside the exhaustive round trips. A round trip proves
  the encoder and decoder agree with each other, which they would keep doing
  with every field shifted one bit — the new tests are the only ones that would
  fail. They deliberately cover the two encodings where bit order and magnitude
  order disagree (gain `10` is ×1/8 while `11` is ×1/4; integration `1100` is the
  *shortest* time), because a table sorted the intuitive way encodes both
  backwards.
- Split `shutdown_before_the_bound_keeps_the_previous_completed_pair`, whose name
  claimed more than its body: nothing had completed, so there was no previous
  pair to keep. It is now
  `shutdown_before_the_first_bound_completes_no_conversion`, and a new test
  actually establishes a completed pair before shutting down and asserts it
  survives — the Auto-Memorization behavior §7 records as verified. A test whose
  name asserts more than its body is worse than a missing test, because the
  coverage matrix reads names.
- The conformance package pins `0.0.0` rather than inheriting the workspace
  version, and the gate asserts both that and its `publish = false`. D-022 now
  says "product crates" wherever it said "both crates". Letting the harness
  inherit would be the natural-looking edit and would quietly enrol a package no
  consumer can observe into the release lifecycle.
- The independent model no longer counts persistence streaks. It qualifies
  threshold status at protect number one — where a single refresh needs no
  counting rule — and reports the new
  `Unsupported::UndefinedQualificationRule` above it. Programming any protect
  number still works: Table 1 is verified, so only *qualification* is withdrawn,
  not the register field.

  The withdrawn behavior was an invented rule. No reviewed passage states
  whether the count runs over consecutive refreshes or whether a non-qualifying
  one resets it, and the model had chosen both. Under D-030 a model assumes only
  what it needs to run, and nothing here required a rule.

  What makes this worth more than a coverage note: **the invented rule could not
  have produced a conformance failure.** The driver only programs `ALS_PERS` —
  `Persistence::count()` is an accessor no driver logic reads — so the trace at
  persistence 4 advanced four refreshes, asserted the flag, and passed. It
  confirmed a register write while reading like confirmation of when the flag
  asserts. The model guessed, the driver had no rule to disagree with, and the
  agreement looked exactly like evidence. An independent model catches what one
  side invented only when the other side has an opinion.

  The model's covered surface is smaller as a result, and the packaged coverage
  matrix says so. That is the intended direction: a narrower oracle that is sound
  beats a broader one that manufactures agreement.

  `every_persistence_setting_qualifies_both_threshold_directions` became
  `protect_number_one_qualifies_both_threshold_directions` plus
  `protect_numbers_above_one_declare_the_qualification_rule_undefined`, and
  `disabling_the_monitor_resets_incomplete_qualification_without_clearing_status`
  lost its streak-reset half — the surviving half, that disabling does not clear
  an established flag, is still source-grounded. The conformance trace split the
  same way, so the write path stays covered.
- Both crates now inherit `version` from `[workspace.package]`, so a bump edits
  one manifest line and drift between the two crates is unrepresentable rather
  than merely gate-detected. The gate reads the resolved version back through
  `cargo pkgid` instead of parsing manifest text, which no longer contains a
  literal to parse.
- The canonical gate gained a `release` profile alongside `full` and `bounded`.
  It refuses a dirty worktree, packages without `--allow-dirty`, and records the
  commit, archive name and SHA-256, file inventory, normalized manifest, VCS
  metadata, and the repository-only model-test boundary to
  `target/release-evidence/evidence.md`. It performs no registry action.
- The gate asserts the supported `cargo-deny` version rather than accepting
  whatever is installed, so an advisory result no longer depends on the runner.
- The gate validates local Markdown links and `#heading` anchors. The check is
  offline by design: external URLs are not fetched, because a gate that fails
  when a vendor site is briefly down teaches contributors to ignore it.

- **Breaking:** every mutating operation now shuts the device down before
  reconfiguring it. The sources require `ALS_SD = 1` before any reconfiguration,
  so `set_measurement_config`, `set_power_saving`, `measure_once_with_timing`
  and `arm_threshold_monitor` write the shutdown bit first when they start from
  an active device, change fields only while shut down, and return to active
  last. Operations starting from a shut-down device are unchanged and still
  leave the device shut down. `set_power_state` changes only the shutdown bit
  and is not a reconfiguration.

  Two observable consequences. Transaction counts increase for active starts:
  `set_measurement_config` and `set_power_saving` take three writes instead of
  one. And a failure part way through can leave an originally active device shut
  down, so a caller must read back rather than assume — the cost of following
  the required sequence.

  This resolves the driver-versus-model disagreement against the driver. The
  model's rejection of active reconfiguration was correct and was not relaxed.
- `set_measurement_config` and `set_power_saving` now return successfully
  without writing when the requested value already matches. Previously an
  idempotent call still cycled power, which would interrupt an enabled monitor's
  active domain for a call that changes no field.
- `arm_threshold_monitor` re-arming an enabled monitor on an active device now
  shuts down with the monitored domain intact, then disables the monitor while
  shut down. The shutdown and monitor bits cannot move in one write: each is
  accepted alone as a transition, but together they are a reconfiguration.
- A failed shutdown write in `measure_once_with_timing` reports without
  attempting restoration. Nothing has been mutated at that point and the device
  may still be active, so the generic restoration sequence would have committed
  the very active write this change removes, turning one fault into
  `RecoveryFailed`.
- `MeasureStage::EnterShutdown` names the new pre-reconfiguration write, so a
  failure at that point is attributable rather than folded into a later stage.
  Entering shutdown first also makes the recovery path safe: every later stage
  now runs on a shut-down device, so restoration can no longer attempt a write
  while active — previously both the operation and its recovery failed together.

- **Breaking:** `ThresholdMonitorError` gained a `confirmed` field naming the
  last stage that definitely reached the device, separate from `stage`, whose
  commit status is unknown. A caller can now distinguish a known committed
  prefix from the uncertain write without being told something the bus never
  established. The type is `#[non_exhaustive]`, so this is additive for matching,
  but any code constructing it literally must add the field.
- Every async operation now documents that it is **not cancellation-safe**, with
  a table of the device state left at each await boundary and a deterministic
  read-back recovery procedure using public operations only. Restoration
  guarantees are qualified with "when polled to completion" — dropping a future
  does not undo what it has already done, because a driver cannot run async
  cleanup during a synchronous `Drop`.
- The measurement delay is called out as the boundary that matters: it is the
  longest suspension, so a timeout or `select!` lands there most often, and it
  leaves the sensor awake and converting in a domain the caller did not ask to
  persist. The guidance is to bound the operation with a shorter integration time
  rather than by racing the future.
- Recorded the general rule that **a failed write is not a rejected write.** An
  I²C error can mean the byte never arrived, or that it arrived, took effect, and
  the acknowledgement was lost; no error type in this crate reports a write as
  rolled back or not applied.
- `read_threshold_status` and `arm_threshold_monitor` now state that a set flag
  may be stale, and that arming does not clear status — a flag set under a
  previous set of thresholds can read as asserted against the new ones. There is
  no procedure that fixes it: with no read-to-clear contract, discarding a read
  changes nothing, so every asserted read is potentially stale. What a caller can
  rely on is the unasserted case, and corroborating a set flag against a fresh
  snapshot. This is a limitation of what the sources support, recorded where a
  caller meets it rather than only in repository contracts.
- `disable_threshold_monitor` now states that it clears the monitor bit only and
  does not restore the power state that preceded arming, so a device armed from
  shutdown stays active after disabling.
- Recorded D-027: why cancellation is documented rather than defended, why a
  cleanup guard or background task would trade an honest limitation for a hidden
  one, and why these tests assert sequencing rather than device state.

- **Replaced the model-conformance claim with an exact, packaged coverage
  matrix.** The old disclosure said the model "covers `probe`, fresh
  measurement, power-saving cadence, threshold monitoring, and sequential
  ALS/white observation", then excluded "unexercised public operations" without
  naming them. Both halves overstated it: the categories imply whole operations
  when the traces are much narrower, and an unnamed exclusion is not a
  disclosure. The matrix now names every covered operation with its accepted
  initial state and configuration, every public operation with no conformance
  trace, and — the part categories hide — the configuration domain that is never
  exercised at all.
- Named the five public operations with no conformance trace: `read_device_id`,
  `inspect`, `snapshot`, `set_measurement_config`, and custom-timing
  `measure_once_with_timing`. Each says why it is absent rather than only that
  it is.
- Disclosed that conformance exercises gain ×1/8 and 100 ms only, persistence 4
  only, cadence Modes 1 and 2 only, and **high-threshold qualification only** —
  no trace ever qualifies the low threshold. A claim outside that table has no
  conformance evidence behind it.
- The gate now fails if the matrix drifts from the test inventory in either
  direction: a named test that does not exist, or a conformance test the
  packaged claim does not disclose. The second is the honesty-critical one, and
  it is the one a reviewer would not notice.
- The gate also compares the matrix between the packaged README and the crate
  documentation, so the two cannot disagree about what is claimed.
- `ph-veml7700-als-model` now declares itself repository-only and unpublished in
  its own README.
- `docs/VERIFICATION.md` states what each test level may and may not be cited for,
  and defers to the packaged matrix for exact level-4 coverage rather than
  restating it. `crates/veml7700-model/README.md` defers the same way: it kept a
  category-level list of what conformance establishes, which the exact matrix
  immediately contradicted, and a second list is a second thing to keep true.

- **Breaking:** `MeasurementConfig::safe_bright_start()` is renamed
  `maximum_range_start()` and changed from gain ×1/8 / 100 ms to ×1/8 / **25 ms**.
  The old name promised range the configuration did not have: it saturates at
  ~35 232 lx, below direct sunlight, while being named for safety in bright
  light. The gain was always right — the sources recommend starting at ×1/8 or
  ×1/4 for unknown brightness — but they also say an integration time below
  100 ms may be needed to show such a value. ×1/8 at 25 ms reaches ~140 926 lx,
  the widest the part offers.
- **Breaking:** `FreshMeasurement.waited_us` is renamed `requested_wait_us`. It
  is the delay this driver requested, not measured elapsed time: `DelayNs`
  guarantees at least the request and may take arbitrarily longer, and the driver
  reads no clock.
- **Breaking:** `MeasurementConfig::default()` now returns `maximum_range_start()`
  and documents itself as this crate's software policy, explicitly not the device
  reset domain. `silicon_reset_default()` remains the device's own power-up state
  and now says it is not a recommendation — gain ×1 saturates at 4 404 lx.
- Added `NominalScale::full_scale_micro_lux`, so the saturation point of a
  gain/integration pair is a tested value rather than a number a reader derives.
  The spread across configurations is four orders of magnitude.
- `nominal_illuminance` now states that it is **invalid as any kind of estimate
  when the counts are saturated**: at maximum code the conversion clipped, so the
  value is the domain's ceiling rather than an observation. It bounds nothing
  about the actual illuminance either — a nominal figure, uncorrected and
  uncalibrated, cannot bound anything outside its own scale. What a clipped
  reading establishes is that the configuration was too narrow. Saturation is not
  an error, so an unchecked read looks like an ordinary value.
- The usage example now checks `fresh.als.is_saturated()` and shows the manual
  response, including that nothing wider exists at maximum range.
- Recorded why the 1 ms software margin is 1 ms — a driver policy value, not a
  source-derived one — and what it explicitly does not cover: integration
  tolerance beyond ±30 %, I²C transaction time, executor latency, or any silicon
  behavior.
- Documented the `defmt` boundary. It is target-firmware integration only: it
  references `_defmt_panic` and a global logger the firmware supplies, so
  `cargo test --all-features` cannot link a host test binary. The supported host
  test profile is `--no-default-features`; the gate still compiles, lints,
  documents and cross-builds the feature.
- Threshold conformance traces use 100 ms rather than the new preset, because
  25 ms has no vendor-documented power-saving refresh time — pairing it with an
  enabled cadence would ask for behavior no source establishes. The preset
  documents that constraint.

- **The packaged README is now the single consumer source of truth**, included
  verbatim into the crate documentation with `#![doc = include_str!]`. docs.rs
  and crates.io can no longer disagree about what this crate claims, because
  there is no second copy to disagree with. Two gate checks that existed only to
  keep the copies in step — the usage-example comparison and the coverage-matrix
  comparison — were deleted in the same change, and the status-disclosure check
  narrowed from three files to two.
- Added the consumer information the packaged README was missing: exact
  prerelease dependency syntax (a `0.1` requirement will not match
  `0.1.0-incubating.1`), MSRV 1.92.0 and Edition 2024, the `no_std` /
  allocation-free / unsafe-free posture, the runtime dependency, the five
  reference target triples, and what compiling on them does and does not
  establish.
- Removed every link to mutable `main` from the packaged README rather than
  deferring them to release-time verification. The content they pointed at is
  either inline now or unusable to a consumer: `ph-veml7700-als-model` is
  repository-only, so a link to it from the published crate offers nothing.
- The root README is a landing page for people working on the repository, not a
  second consumer manual. It carries the status disclosure, points consumers at
  the packaged README, and describes layout, verification profiles, and
  publication status.

- **Every maintained document now states its authority** — normative contract,
  evidence record, contributor procedure, or non-normative rationale. Previously
  `CONTRIBUTING.md` called everything under `docs/` normative, giving
  architecture prose and historical decisions the force of an acceptance
  contract.
- `docs/ARCHITECTURE.md` and `docs/API_CONTRACT.md` are replaced by
  `docs/DRIVER_CONTRACT.md`. They described the same subject from two angles, so
  a reader had no way to know which governed when they disagreed.
- **The handwritten public-signature inventory is deleted, not relocated.** The
  compiler and rustdoc own signatures; a prose copy can only drift, and it had.
  What survives is the semantic contract, which neither can express.
- `docs/DOCUMENTATION_STANDARDS.md` is removed. Six of its eleven lines were
  already in `CONTRIBUTING.md`; the seventh — examples compile as doctests — is
  review-blocking and became invariant I-28.
- `docs/TEST_PLAN.md` is renamed `docs/VERIFICATION.md`, describing what each
  test layer establishes rather than restating coverage that would then rot.
- **`docs/vendor/README.md` is now the provenance source of truth.** `AGENTS.md`
  previously named the model README canonical for source digests, which put
  source identity under a document whose subject is the model.
- Recorded D-028 with the reasoning for each consolidation.

- Public error types now implement `core::fmt::Display`, and `core::error::Error`
  where the bus error does too, so a driver failure can join a standard error
  chain. Both remain `no_std`, allocation-free and unsafe-free, and the rustdoc
  carries a reporting helper that walks a chain into a caller-owned buffer with
  no allocator.
- **`Display` is deliberately unbounded on the bus error.**
  `embedded_hal_async::i2c::Error` requires only `Debug`, so bounding on
  `Display` would have denied these impls to the very HAL error types this driver
  exists to carry. The message states the semantic context this crate owns and
  leaves the concrete error to `source()`. The `core::error::Error` bound sits on
  the impl rather than the type, so a bus error that does not implement it is
  still perfectly usable — only the chain is unavailable.
- `source()` is returned only where a cause exists. `ProbeError::NotPresent` and
  `WrongDevice` have none: they are conclusions the driver reached, not failures
  it forwarded.
- `MeasureOnceError::RecoveryFailed` reports the **primary** failure as its
  source. It carries two independent failures and a chain can express one;
  reporting the recovery failure as the cause would invert what happened. The
  recovery error stays an ordinary field.
- Model `TransportError` and `Unsupported` gained `core::error::Error`. Neither
  reports a source: an unsupported interaction is a limit the model declares, not
  a failure forwarded from elsewhere.

- Split the driver test corpus into six `#[cfg(test)]` modules by
  responsibility — probe and identity, observation, configuration and power,
  fresh measurement and recovery, threshold programming, and cancellation
  boundaries. `driver.rs` drops from 2 023 lines to 1 031, of which 978 are
  production code, so reviewing the driver no longer means scrolling past a
  thousand lines of tests to reach the next function.
- **Production code is byte-identical**, verified by diffing the pre-split file
  against the post-split production half. The test count is unchanged at 58; no
  test was dropped, merged, or weakened.
- The exact-transaction builders moved to `crate::testing::scripted_i2c`, so the
  wire format — pointer byte, then low, then high — has one definition rather
  than one per test module. A byte-order regression cannot be written into a
  test as though it were expected.
- Removed a duplicate delay stub. `RecordingDelay` did what
  `CancellableDelay::ready()` already did, and having two meant a test could
  record elapsed time through either.
- The package gains six files and loses none. It carries **2 017 lines of driver
  source where it previously carried 2 023** — the same tests, rearranged. They
  remain packaged deliberately: the gate tests the unpacked archive, and
  excluding them would make that step verify less.

- The hardware contract distinguishes a fourth verification state:
  **Assumption**. A row the sources do not settle is either unread — where more
  reading may close it — or unresolvable by reading at all, where the driver
  must still behave one way and rests on an assumption about silicon. Marking
  both the same way implied a search that would eventually succeed.
- Recorded refresh-time independence from ALS gain as the first such assumption.
  The sources publish the relation at gain ×2 only; `nominal_refresh_time_ms`
  takes no gain argument and the model inherits the same shape, so both behave
  as though it holds. The row states what would settle it — measuring the
  refresh interval across all four gains at one integration time and cadence, on
  silicon — because an assumption without a stated test is indistinguishable
  from a guess.
- Recorded D-029: assumptions about silicon are declared with their code site
  and their settling observation, so a future hardware-in-the-loop effort has
  somewhere to start.

- Declared two further assumptions under D-029, each naming the code that relies
  on it and the observation that would settle it: register `0x03` reading
  `0x0000` before it is written, and integration time falling within ±30 % of
  nominal.
- Reclassified the ±30 % row after briefly recording it as waiting on a passage.
  That was wrong about the kind of fact it is. Integration intervals are counted
  off the part's internal oscillator, so the spread is that oscillator's
  tolerance — a process-dependent silicon characteristic, not a timing parameter
  a further page would list — and Vishay publishes no oscillator accuracy for
  this part. Waiting for that passage was waiting for a document that was never
  going to exist. `INTEGRATION_TOLERANCE_PERCENT` and its two "documented ±30 %"
  doc comments now say *assumed*, which is what they always were.
- Kept the persistence rule out of that category. A qualification rule is
  functional behavior a datasheet can state in prose, unlike an oscillator
  tolerance, so reading could still close it. The four counts are verified; the
  word *consecutive* is not.
- Recorded D-030: undefined device behavior is **allocated** between driver and
  model rather than answered the same way in both. Each is asked whether it needs
  the fact to function. The driver acts defensively when it does not, because an
  unbackable promise is worse than none; it assumes and says so when it must act
  anyway. The model declares undefined by default, because a model that invents
  plausible behavior still produces agreement — and that agreement means nothing
  while looking exactly like evidence.
- Found the case that rule exists to catch. The model implements persistence as
  consecutive counting with reset, the driver only *programs* `ALS_PERS` and no
  driver logic reads the count, so
  `threshold_monitor_public_operations_qualify_after_configured_persistence`
  confirms a register write while reading like a behavioral result. The model
  will declare the qualification rule undefined and answer `Unsupported`; the
  driver will stop describing persistence in terms of *consecutive* measurements
  it cannot observe. The model half lands as its own issue, because changing what
  the model does is a behavior change rather than a documentation edit.
- Corrected the public rustdoc that still promised what these rows withdrew.
  Declaring an assumption in the contract and leaving the shipped API describing
  the old certainty is worse than not declaring it, because the surface a
  consumer actually reads is the one that keeps the promise.
  - `Persistence` and its four variants described *consecutive qualifying
    measurements*, and `ThresholdMonitorConfig::persistence` repeated it. They
    now name the protect number, state that the qualification rule is not
    source-backed, and direct callers to poll status rather than compute an
    assertion time. `Persistence::count()` says it feeds no driver calculation.
  - `measure_once` advertised "conservative vendor-derived timing" for a wait
    only partly vendor-derived. It now separates the three components — the
    vendor's 2.5 ms wake delay, the assumed 130 %, and the 1 ms policy margin —
    and names the failure: if the real spread exceeds ±30 %, a stale value comes
    back indistinguishable from a new one.
  - `measure_once_with_timing` and `MeasurementTiming::with_additional_margin_us`
    called the floor a "documented conservative minimum". Dropped, with the note
    that adding margin cannot convert an assumption into a guarantee.
- Moved the settling procedures for the three Assumptions to #58, leaving each
  row a one-sentence observation. Naming what would settle a row is D-029's
  requirement; carrying the procedure would make this document a
  physical-evidence plan, which the product boundary excludes.
- Resolved the §4 reset-value row. `0x00` and `0x07` are source-declared; every
  other register, including `0x03`, is **not**. The §4 table previously listed
  `0x0000` for `0x03` unqualified, in a column whose other entries said *not
  declared by sources*.
- The `0x03` assumption turns out to be narrower than it looked: **the driver
  does not rely on it at all**, because every path reads the register before
  acting on it. The dependency is confined to the model's construction.
- `crates/veml7700-model/README.md` gains a **Declared assumptions** table naming
  all four model behaviors that rest on unstated facts, where each is relied on
  in code, and what a reader observes as a result. Scattering them through the
  source-decision prose meant a reader deciding what model agreement is worth had
  no way to see where the model guesses.
- Recorded the sharpest limit that follows: driver and model derive
  refresh-gain independence independently, but from the same silent source, so
  agreement there is **not corroboration** — it is two derivations sharing an
  assumption. An independent model cannot catch a defect both inherited from the
  document; only physical evidence closes that.
- By contrast the ±30 % assumption is load-bearing. `INTEGRATION_TOLERANCE_PERCENT`
  is why the conservative wait is 130 % of the selected integration time, so the
  margin is conservative *given the assumption* rather than in general.

- **Corrected the source classification of the ±30 % integration-time
  tolerance.** The repository said no vendor document states it, called it
  third-party in origin, and argued that no such passage could exist. It does:
  application note 84323, Revision 06-Mar-2025, page 4, section *Command Code
  ALS_IT*, `Remark` — "For the integration time a tolerance of ± 30 % can be
  assumed. This tolerance should also be considered during the read out of the
  measurement results." That is in a pinned source, under the digest already
  recorded in `docs/vendor/README.md`. Found on an independent re-read during
  review of #63; see #65.
- The `HARDWARE_CONTRACT.md` §7 row is now verified with document, revision,
  page, and section, and the conservative 130 % wait is recorded as this driver
  acting on the Remark's second sentence rather than as a stand-in for a missing
  figure. `INTEGRATION_TOLERANCE_PERCENT`, `measure_once`,
  `conversion_bound_ns`, and the model's completion-boundary decision say
  vendor-stated where they said assumed.
- **Kept the distinction the citation does not close.** The vendor writes "can
  be assumed" in an application note; the datasheet specifies no
  integration-time tolerance and publishes no oscillator accuracy, showing the
  oscillator only in its block diagram. So the figure is sourced and still not a
  worst case characterized across process, voltage, and temperature — a spread
  wider than ±30 % still fails the freshness guarantee silently.
  `CONTRIBUTING.md` gains **specified** versus **vendor-stated guidance** as an
  evidence-language distinction.
- Recorded D-032: a claim that a source is silent needs a located negative —
  document, revision, sections read — not an argument that the figure is the
  kind of thing vendors do not publish. A D-029 Assumption asserts that further
  reading cannot help, which suppresses the reading that would refute it, so it
  is the one row type that must not rest on inference.
- Recomputed the counts that claim tracked: **38 rows verified, 3 open**, two of
  them D-029 Assumptions. The model's declared-assumption table drops to two
  rows; the 130 % boundary moves out of it and into the source decisions as an
  abstraction of vendor guidance. D-030's allocation table drops to three rows,
  with a note on why the fourth left.
- **No behavior changed.** The driver still waits 130 % of the selected
  integration time, the model still completes at 130 %, and no test moved. The
  repository had the right number and a false account of its provenance.
- Applied D-032's located-negative rule to the two Assumptions that already
  existed, so the rule is met on the day it is written rather than only by rows
  added later. §6 now names the *Refresh Time, I_DD, and Resolution Relation*
  table in both documents — all sixteen rows `ALS_GAIN = x2`, and the app note's
  `PSM`/`ALS_IT` table carries no gain term at all. §4 now names the datasheet's
  command-register overview, Table 4, and the register-format note that declares
  a power-on default for `0x00` only.
- That exercise immediately found a second overstatement: the `0x03` row said no
  passage declares its power-on value, and the application note does state that
  bits 2:1 come up as mode 1. Only `PSM_EN`, and with it the full word, is
  undeclared. The row's supporting sentence is corrected here and the §4
  reset-value row is made precise; narrowing the Assumption's subject is a
  contract-state change and is #71.
- `measure_once_with_timing` said the conservative minimum is "partly assumed
  rather than vendor-specified", which contradicted its sibling `measure_once`
  after the reclassification. Both entry points now classify the same 130 %
  bound the same way.

- **Restored the I²C bus voltage range alongside `V_ih` and `V_il`.** §2 said
  "1.7 V appears nowhere in the source". It appears in three: the datasheet's
  page-one `PRODUCT SUMMARY` carries a column named `I²C BUS VOLTAGE RANGE` =
  1.7 V to 3.6 V, beside `OPERATING VOLTAGE RANGE` = 2.5 V to 3.6 V; the
  datasheet's own application circuit labels the same rail; and application note
  84323 page 2 states in prose that the pull-up resistors may be connected to a
  1.7 V to 3.6 V supply. Recorded as its own verified row under Vishay's
  parameter name. See #67.
- The three quantities that were being merged are now named and separated in
  place: sensor supply `V_DD` (2.5–3.6 V), the bus rail the pull-ups may use
  (1.7–3.6 V), and the input thresholds `V_ih` / `V_il` (1.3–3.6 V and
  −0.3–0.4 V, both at `V_DD` = 3.3 V). Nothing is inferred beyond what Vishay
  states — no level shifting, no minimum bus voltage per clock rate, no
  relationship between the rail and `V_ih`.
- The row had been wrong in both directions: it once recorded a "high-level
  supply … 1.7 V to 3.6 V", merging rail and threshold. #54 separated them
  correctly and then overcorrected into the absence claim. #54 had itself
  enumerated the right answer — "1.7 V is from a different parameter" — and
  passed over it because the search stopped at the passage the dispute quoted.
- D-032 widened accordingly: the located-negative rule governs **any** absence
  claim, not only D-029 Assumption rows. This one sat inside a checked row,
  where the checkmark read as confirmation of the whole row rather than of the
  fact it recorded.
- Counts recomputed to **39 verified, 3 open**.

- **Audited every absence claim in `HARDWARE_CONTRACT.md` against both pinned
  documents** and backfilled located negatives, rather than waiting for a fourth
  false negative to surface after #65, #67, and #71. Each backfilled row names
  the sections read in each document, what they were found to contain, and that
  absence outside those sections is not claimed.
- Rows given located negatives: the §4 "not declared by sources" reset cells
  (the word *reset* appears in no register context in either document, and
  exactly two default statements exist); §5's "Table 1 does not establish this";
  §6's missing 25/50 ms refresh rows; §8's sign-bit absence; and §9's
  flag-clearing row, which now also names the application note's
  `INTERRUPT HANDLING` section — the other place such a rule would be stated.
- The audit found one false claim: §9 says no reviewed passage states the
  persistence qualification rule, and application note 84323 printed page 16,
  `INTERRUPT HANDLING`, states the counting condition — a flag is set only when
  the threshold is exceeded and a programmed number of measurements (`ALS_PERS`)
  *stay above / below* it. The reset behavior is still unstated. Correcting the
  row is a contract-state change, so it is #73; the row carries a pointer
  meanwhile rather than standing unqualified.
- Recorded a third vendor discrepancy in the standard place — beside the row it
  affects, counted in `docs/vendor/README.md`: *Basic Characteristics* gives
  `f(SCL)` as a flat 10 kHz to 400 kHz where *I²C Timing Characteristics* splits
  standard mode at 100 kHz from fast mode at 400 kHz. The first two recorded
  discrepancies are prose against a table; this one is table against table, and
  the more specific table governs. Nothing in this driver selects a bus mode.
- D-032 records what the audit found and two limits of it: presence is provable
  and absence is not, and the searches ran over machine-extracted PDF text that
  can drop glyphs set in figures.

### Known issues

- The independent model remains a bounded slice: transport faults, arbitrary
  active reconfiguration, threshold-flag clearing, unspecified register reset
  values, and unexercised public operations remain outside its claim.
- The hosted workflow has never executed a job, so it is unverified. It and
  default-branch protection both resolve at the visibility change. See issue #6.
- Vendor owner-verification has been walked end to end: **39 rows verified,
  3 open.** The open rows are two D-029 Assumptions that only hardware can
  close (refresh independence from ALS gain, register `0x03` reset value) and
  the persistence qualification rule, which reading could still close and which
  D-030 resolves without it. No verified row is a physical-support claim —
  matching a recorded interpretation to a recorded document establishes nothing
  about silicon. No open row blocks release work.
- A verified row is also not a characterized one. The ±30 % integration-time
  tolerance is vendor-stated design guidance (#65, D-032), so the conservative
  wait rests on a source rather than on a stand-in — but the vendor does not
  characterize the spread across process, voltage, and temperature, and a wider
  real spread would still break freshness silently.
- No reviewed physical or calibrated-optical evidence exists, and candidate
  version `0.1.0-incubating.1` remains unpublished with `publish = false`.

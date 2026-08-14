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

### Known issues

- The independent model remains a bounded slice: transport faults, arbitrary
  active reconfiguration, threshold-flag clearing, unspecified register reset
  values, and unexercised public operations remain outside its claim.
- The hosted workflow has never executed a job, so it is unverified. It and
  default-branch protection both resolve at the visibility change. See issue #6.
- Vendor owner-verification is well advanced but incomplete: the electrical and
  bus boundary, the power-saving refresh table, the complete resolution and
  maximum-range tables, the gain encodings, the linearity limits, and the
  vendor's starting-configuration guidance are owner-verified. Still
  provisional: both §1 source-baseline rows, the integration/persistence/
  shutdown encodings, the register map, word transfer order, wake-up timing,
  the threshold monitor, and the identity word. None of these are
  physical-support claims. One gap still blocks open work: the configuration
  active-write rule (#29).
- No reviewed physical or calibrated-optical evidence exists, and candidate
  version `0.1.0-incubating.1` remains unpublished with `publish = false`.

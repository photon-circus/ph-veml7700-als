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

### Known issues

- The independent model remains a bounded slice: transport faults, arbitrary
  active reconfiguration, threshold-flag clearing, unspecified register reset
  values, and unexercised public operations remain outside its claim.
- The hosted workflow has never executed a job, so it is unverified. It and
  default-branch protection both resolve at the visibility change. See issue #6.
- Vendor owner-verification is partly complete: 8 of 28 hardware-contract rows
  are owner-verified — the electrical and bus boundary, the sixteen power-saving
  refresh times, and the gain ×2 resolution column. The remaining 20 are
  provisional, including both §1 source-baseline rows. None of these are
  physical-support claims. Two gaps block open work: the configuration
  active-write rule (#29) and the gain ×1/8 and 25 ms resolution entries (#32).
- No reviewed physical or calibrated-optical evidence exists, and candidate
  version `0.1.0-incubating.1` remains unpublished with `publish = false`.

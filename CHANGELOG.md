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
- Concrete preserved bus errors carrying semantic operation, register, and stage
  context, including distinct primary and recovery failures when restoration
  also fails and a captured sample survives a failed restore.
- Pure codec tests across every documented field combination and reserved
  encoding; exact scripted-I²C tests asserting address, pointer, little-endian
  word order, payload, and transaction count; per-stage failure injection for
  both fresh capture and threshold-monitor programming.
- Independent `ph-veml7700-als-model` crate: a quiescent, datasheet-derived
  device behavioral model covering `probe` and one successful `measure_once`
  flow at the I²C boundary, driven by explicit relative duration. Ships with
  model-only tests and two driver-versus-model tests. Its README is the
  maintained declaration of claim, fidelity, sources, and nonclaims.
- One canonical local verification gate, `scripts/ci.sh`, covering formatting,
  host tests, lints with warnings denied, rustdoc, five bare-metal targets,
  dependency and license policy, package construction and inspection, and tests
  against the unpacked distributable package.
- Two claim checks in that gate, so the repository's load-bearing promises fail
  loudly rather than drift: vendor documents must not be tracked, and the
  required status disclosure must be identical in the root README, the packaged
  crate README, and the crate documentation.
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

### Known issues

- The independent model covers only `probe` and one successful `measure_once`
  flow. Every other public operation is outside the model claim.
- The coupled fake in `crates/veml7700/src/testing/fake_device.rs` is
  exploratory. Its tests drive the fake directly and never construct the driver,
  so they establish nothing about driver behavior. See issue #9.
- The crate has no doctests and no compiled usage example. See issue #10.
- The hosted workflow has never executed a job, so it is unverified. It and
  default-branch protection both resolve at the visibility change. See issue #11.
- The model crate's version convention is undecided relative to the driver's
  lifecycle-matching prerelease. See issue #12.
- Vendor owner-verification is incomplete: `docs/vendor/README.md` records the
  retrieved documents and their digests, but the hardware-contract verification
  boxes remain unchecked and are not physical-support claims.
- No reviewed physical or calibrated-optical evidence exists, and candidate
  version `0.1.0-incubating.1` remains unpublished with `publish = false`.

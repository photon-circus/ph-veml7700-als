# Changelog

All notable changes to this repository are documented here.

## [Unreleased]

Last updated: 2026-08-14 UTC

### Added

- Async, `no_std` VEML7700 driver with explicit snapshot/fresh semantics,
  conservative timing provenance, restoration-aware capture, typed threshold
  monitoring, and integer nominal illuminance scaling.
- Pure codec tests, exact scripted-I²C tests, failure injection, and a test-only
  autonomous-state fake covering refresh cadence, retention, and persistence.
- Independent `ph-veml7700-als-model` crate for the datasheet-derived `probe`
  and successful `measure_once` slice, with model-only tests and two
  driver-versus-model tests. The maintained declaration is the model crate
  README.

### Changed

- Reject addresses outside the 7-bit input domain as model limitations, exclude
  conformance-only tests from the driver package, and test the unpacked package.
- Reject repeated active configuration writes in the bounded device model
  instead of inventing conversion restart-or-continuation behavior.
- Normalize the unpublished driver version to `0.1.0-incubating.1`, retain
  `publish = false` during preparation, and separate preparation, repository
  visibility, and crates.io publication into explicit maintainer decisions.
- Align distribution and evidence disclosures with the organization profile:
  incomplete model coverage and absent physical qualification limit claims but
  do not impose a blanket publication gate.
- Add an explicit release procedure, Incubating version-state check, and the
  MIT license text to the packaged driver artifact.
- Consolidate routine verification into one bounded local Git Bash gate with
  thin Bash and PowerShell launchers.
- Reframe documentation around the implemented driver and the honest limitation
  of the current coupled, test-only fake.

### Removed

- Speculative PH-HIL runners, plans, policies, optical-fixture templates, mock
  firmware artifacts, transcripts, build shims, and evidence directories.
- Generated development-pack manifests, hash registries, validators, bootstrap
  roles, implementation work packets, and speculative release checklists.

### Known issues

- The independent model covers only `probe` and one successful `measure_once`
  flow. The coupled fake remains exploratory and is not an independent oracle.
- Vendor source verification remains owner-pending, no reviewed physical or
  calibrated-optical evidence exists, and candidate version
  `0.1.0-incubating.1` remains unpublished.

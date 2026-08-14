# Changelog

All notable changes to this repository are documented here.

## [Unreleased]

Last updated: 2026-08-13 UTC

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

- Lock crates.io publication with `publish = false` until independent model and
  reviewed physical evidence support the claimed scope.
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
  calibrated-optical evidence exists, and the crate remains unpublished.

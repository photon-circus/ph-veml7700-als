# Changelog

All notable changes to this repository are documented here.

## [Unreleased]

Last updated: 2026-08-12 UTC

### Added

- Async, `no_std` VEML7700 driver with explicit snapshot/fresh semantics,
  conservative timing provenance, restoration-aware capture, typed threshold
  monitoring, and integer nominal illuminance scaling.
- Pure codec tests, exact scripted-I²C tests, failure injection, and a test-only
  autonomous-state fake covering refresh cadence, retention, and persistence.

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

- The autonomous fake is not an independent I²C-level mock and does not yet
  cross-validate public driver operations.
- Vendor source verification remains owner-pending, no reviewed physical or
  calibrated-optical evidence exists, and the crate remains unpublished.

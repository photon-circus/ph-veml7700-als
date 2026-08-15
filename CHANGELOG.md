# Changelog

Notable user-visible changes are recorded here. Process history and evidence
debate belong in their authoritative records, not in the changelog.

## [Unreleased]

Nothing in this repository has been released or published. This section
describes the candidate's current surface, not changes from a prior version.

### Added

- Async, `no_std`, allocation-free VEML7700 driver over caller-provided
  `embedded-hal-async` I²C, with inert construction and exact resource release.
- Typed measurement, power-saving, threshold-monitor, identity, and error
  surfaces; exact scripted-transport tests; restoration-aware
  `MeasurementCapture`; and integer nominal micro-lux conversion.
- Independent repository-only device model and driver-versus-model conformance
  tests. The verification record owns the exact coverage inventory.

### Clarified

- `AlsCounts::is_max_code` reports the exact maximum-word observation. It does
  not establish physical clipping, overrange, or a scene lower bound (`S-51`,
  `S-52`), so no accessor on this type is named for saturation.
- Threshold-status reads are raw observations. The driver performs no explicit
  clearing operation and promises no flag history or freshness semantics (`S-38`,
  `S-42`).
- Nominal power-saving refresh time is available only in the documented gain
  ×2 domain; other gains return `None` rather than assuming independence
  (`S-21`, `S-22`).
- Shared evidence is identified by stable `S-nn` propositions. Driver and model
  consequences remain independently derived and are not evidence about silicon.

### Known limitations

- No physical-hardware or calibrated-optical evidence has been recorded.
- Current lifecycle and consumer limitations are disclosed in the packaged
  README; the verification record owns exact conformance coverage.

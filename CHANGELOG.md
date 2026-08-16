# Changelog

Notable user-visible changes are recorded here. Process history and evidence
debate belong in their authoritative records, not in the changelog.

## [Unreleased]

Nothing user-visible has changed since `0.1.0-incubating.1`.

## [0.1.0-incubating.1] — 2026-08-16

Embedded applications targeting the VEML7700 previously chose between blocking
drivers and ones that quietly present nominal arithmetic as calibrated light.
This release adds an async, allocation-free `no_std` driver whose value is that
it refuses to overstate what it knows: observational reads stay separate from a
controlled one-shot capture, partial-operation context survives in errors, and
integer scaling is labelled nominal rather than product lux. The cost is that
the caller retains calibration, optical policy, scheduling, and bus recovery,
and that several device behaviors return `Unsupported` or `None` where the
shared evidence cannot justify an answer.

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

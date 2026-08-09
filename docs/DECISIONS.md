# Decision log

## D-001 — Direct fixed-address I²C facade

`Veml7700<I2C>` owns the bus directly. There is no address field or internal
transport trait because the supported device has one fixed I²C address and no
transport variation.

## D-002 — Preserve concrete bus errors

`Error<I2C::Error>` retains the HAL error plus semantic operation and register
context. Address NACK is classified only in `probe()`.

## D-003 — Strict low-byte-first codec

All register words use little-endian byte conversion. Tests assert exact wire
payloads.

## D-004 — Safe bright-start value is a constructor, not driver state

`MeasurementConfig::safe_bright_start()` returns gain ×1/8 and 100 ms, reflecting
vendor guidance for unknown illumination. The driver does not automatically
apply or cache it.

## D-005 — Snapshot and fresh operations are different types

A snapshot may be old and its ALS/white pair may straddle refresh. A fresh
operation controls timing, freezes the data in shutdown, records provenance, and
restores the prior state.

## D-006 — Shutdown used as a pair-freeze mechanism

The vendor documents retained data in shutdown. The complete operation enters
shutdown after the conservative integration wait before reading the two result
registers. This is explicitly a driver coherence policy.

## D-007 — Nominal integer scaling only

The core exposes micro-lux from the vendor resolution table. It omits the
empirical polynomial and system calibration because their validity depends on
optics, source spectrum, and application geometry.

## D-008 — Threshold “interrupt” is a polled monitor

The VEML7700 has no dedicated interrupt pin. API naming uses monitor/status and
never owns a GPIO.

## D-009 — Monitor owns cadence and measurement domain

Threshold counts depend on gain/integration; persistence wall time depends on
power-saving cadence. These fields are configured atomically at the semantic
level and protected from ordinary retargeting.

## D-010 — No undocumented flag-clear promise

Official documentation describes status flags but not a reliable clearing
protocol. The API returns observations without inventing latch semantics.

## D-011 — No VEML6030 abstraction in v0.1

The application note describes functional similarity, but family extraction is
deferred until independent contracts and evidence exist for multiple devices.

## D-012 — External `ph-hil` boundary

The project supplies schema-1 plans, contracts, transcripts, modules, and policy.
`ph-hil` owns orchestration, flashing, instruments, safing, sealed artifacts, and
analysis. No runtime crate dependency is introduced.

## D-013 — Fresh timing is bound to one integration-time selection

`MeasurementTiming` is constructed from an `IntegrationTime`, keeps that
selection private, and cannot encode a shorter-than-conservative wait.
`measure_once_with_timing()` rejects timing derived for a different integration
time before any I²C transaction. This prevents an apparently explicit timing
value from becoming stale when the requested measurement domain changes.

## D-014 — Fresh measurement creates a known wake edge

A complete fresh measurement first disables power-saving mode, installs the
requested gain and integration time while shut down, and only then transitions
from shutdown to active. The driver does not rely on an unobserved prior active
interval or infer conversion age from a register snapshot.

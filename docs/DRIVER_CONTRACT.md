# Driver contract

> **Authority: normative.** This is the semantic contract for the public driver:
> what it owns, what the caller owns, and what each operation promises. A change
> that contradicts it changes this document first, in the same pull request, with
> rationale in [`DECISIONS.md`](DECISIONS.md).

Crate policy: `#![no_std]`, `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`,
async-first `embedded-hal-async`.

**Signatures are not recorded here.** The compiler and rustdoc own them, and a
handwritten inventory only drifts — it was one, and it did. For the public
surface, read the generated documentation or `crates/veml7700/src/lib.rs`. This
document records what the surface *means*, which neither of those can express.

Consumer-facing documentation lives in
[`crates/veml7700/README.md`](../crates/veml7700/README.md), which is also the
crate documentation. Distribution state — version, publication status — is
recorded there and in the manifest, and deliberately **not** repeated here, so a
version bump never has to touch this file.

## Product boundary

`ph-veml7700-als` is a fixed-address async I²C driver. The caller owns bus
construction/recovery, delays, scheduling, power, board wiring, optical design,
calibration, retries, and application policy. The crate owns VEML7700 register
encoding, complete operations, nominal scaling, and truthful result/error
reporting.

## Dependency direction

```text
application / optical policy
             |
             v
public driver operations
             |
             v
typed VEML7700 codecs and timing policy
             |
             v
embedded-hal-async I²C and delay traits
```

No layer depends on a concrete HAL, PAC, board, executor, allocator, operating
system, or physical-test framework.

## Driver state

`Veml7700<I2C>` stores only the I²C resource. Construction is inert and release
returns the exact resource. Configuration, power-saving state, threshold domain,
status, samples, and timing deadlines remain device-authoritative.

## Snapshot and fresh measurement

A snapshot reports observed configuration and sequential ALS/white register
values without claiming freshness. A complete fresh operation installs a known
domain in shutdown, creates a shutdown-to-active wake edge, waits a conservative
integration interval, freezes results in shutdown, reads both channels, and
restores prior state. Errors retain capture and restoration context.

## Threshold-monitor ownership

The monitored domain includes gain, integration time, thresholds, persistence,
power-saving cadence, and active state. Arming is disable-first and enable-last.
Ordinary methods reject changes that would silently retarget an enabled monitor.
No GPIO abstraction exists because status is polled over I²C.

## Optical boundary

Integer `MicroLux` uses the vendor's nominal resolution table. It is not
calibrated lux at a product aperture. Window transmission, geometry, spectrum,
cosine response, part tolerance, high-lux correction, and auto-ranging belong
to a separately reviewed integration layer or application.

## Independent model

The independent device behavioral model is `ph-veml7700-als-model`. It is derived
from `HARDWARE_CONTRACT.md` without driver codecs or timing helpers and is
observed through the I²C boundary. Driver-versus-model tests configure and
observe the device through public `Veml7700` operations; only raw samples,
relative time, and explicitly injected white-channel scheduling skew bypass that
boundary.

The maintained claim is
[`crates/veml7700-model/README.md`](../crates/veml7700-model/README.md).

## Explicit non-goals

- calibrated optical measurement or metrology
- MCU examples, board support, or physical fixtures
- automatic ranging or correction policy
- VEML6030 family abstraction
- raw-register API
- registry credentials or automatic publication

## Semantic contracts

- `probe` validates exact ID `0xC481`; it does not claim package or calibration.
  It reports through `ProbeError`, which carries no `Operation`, because address
  NACK means absence only there. `Operation` therefore has no `Probe` variant.
- Every variant of every public error enum is named by some driver path. A
  variant a caller can match but never reach is not part of this surface, and
  the canonical gate fails if one appears.
- Every public error type is `#[non_exhaustive]`, so later variants and fields
  stay additive. What that requires of a caller differs by shape: matching an
  error *enum* needs a wildcard arm, while destructuring the
  `ThresholdMonitorError` *struct* needs `..` in the pattern, which no wildcard
  arm can substitute for. Reading its `stage` and `source` fields is unaffected.
  The device value types — `Gain`,
  `IntegrationTime`, `Persistence`, `PowerState`, `ThresholdMonitorState`,
  `PowerSavingMode`, `MeasurementPairCoherence` — are exhaustive on purpose and
  may be matched without a wildcard. See D-024.
- `snapshot` never claims freshness or atomic ALS/white pairing.
- `measure_once` creates an explicit shutdown-to-active wake edge and restores
  the prior device state or returns explicit uncertainty.
- `MeasurementTiming` cannot represent a shorter-than-conservative wait and is
  rejected when derived for a different integration-time selection.
- `Thresholds` keeps its fields private, so `Thresholds::new` is the only way to
  build one and a reversed pair cannot reach the device. The driver therefore
  never writes threshold state that `read_thresholds` would reject on read-back.
- monitor configuration is disable-first, enable-last.
- **every mutating operation shuts the device down before reconfiguring it.**
  The sources require `ALS_SD = 1` before any reconfiguration, so an operation
  that starts from an active device writes the shutdown bit first, carrying the
  existing domain unchanged; changes measurement, persistence, monitor and
  power-saving fields only while shut down; and returns to active last. This
  applies to `set_measurement_config`, `set_power_saving`,
  `measure_once_with_timing` and `arm_threshold_monitor`. `set_power_state`
  changes only the shutdown bit and is not a reconfiguration.
- an operation that starts from a shut-down device performs no extra shutdown
  write. `set_measurement_config`, `set_power_saving` and
  `measure_once_with_timing` also leave it shut down, so neither silently wakes
  a device the caller left asleep.
- **`arm_threshold_monitor` is the exception, by design.** It always ends
  active, from either starting state, because a shut-down monitor cannot
  qualify anything — arming a device and leaving it asleep would be an
  operation that cannot do its job. `disable_threshold_monitor` does not
  restore the previous power state; a caller that wants the device asleep
  afterwards asks for that with `set_power_state`.
- `set_measurement_config` and `set_power_saving` return successfully without
  writing when the requested value already matches, so an idempotent call never
  costs a power cycle and never interrupts an enabled monitor.
- because shutdown comes first, a failure part way through a mutating operation
  can leave an originally active device shut down. Which fields were installed
  depends on how far the sequence reached, so read the relevant registers back
  rather than assuming. This is the cost of following the required sequence; the
  alternative is a write the sources do not sanction (`S-19`).
- **no async operation is cancellation-safe, and none claims to be.** Dropping a
  future does not undo what it has already done: the driver is not an executor
  and cannot run cleanup during a drop, so restoration happens only on paths that
  return. Every restoration guarantee in this contract holds *when polled to
  completion*. Every operation that writes — `set_measurement_config`,
  `set_power_state`, `set_power_saving`, `measure_once_with_timing`,
  `arm_threshold_monitor` and `disable_threshold_monitor` — tabulates in its
  rustdoc the state left at each await boundary, and gives a deterministic
  read-back procedure using public operations only. Read-only operations issue
  no writes, so a drop leaves the device unchanged.
- **a failed write is not a rejected write.** An `Err` establishes that the
  operation did not complete, not that the device is unchanged: an I²C error can
  mean the byte never arrived, or that it arrived, took effect, and the
  acknowledgement was lost. Nothing at the transport distinguishes those, so no
  error type here reports a write as rolled back or not applied. Dropping a
  future mid-write leaves the same uncertainty without an error to inspect.
- `ThresholdMonitorError` separates the two: `confirmed` is the last stage that
  definitely reached the device, `stage` is the write whose commit status is
  unknown, and every later stage was not attempted. The device is therefore in
  one of exactly two states, and no rollback is attempted.
- threshold status is never cleared by this driver, and arming does not clear it.
  A flag set under a previous domain can read as asserted after re-arming; the
  sources establish no clearing contract, so none is promised.
- no operation exposes a raw register pointer or owns an interrupt GPIO.
- **every public error type implements [`core::fmt::Display`] unbounded on the
  bus error, and [`core::error::Error`] when the bus error does too.** The
  `Display` bound is deliberate: `embedded_hal_async::i2c::Error` requires only
  `Debug`, so bounding on `Display` would deny these impls to the HAL error types
  this driver exists to carry. The message therefore states the semantic context
  this crate owns — operation, register, stage — and `source()` supplies the
  concrete bus error.
- `source()` is returned only where a cause genuinely exists. `ProbeError::NotPresent`
  and `ProbeError::WrongDevice` have none: they are conclusions this driver
  reached, not failures it forwarded, and inventing a source would misdescribe
  them.
- `MeasureOnceError::RecoveryFailed` carries two independent failures and a chain
  can express one. `source()` is the **primary** failure — why the operation
  stopped. The recovery failure remains an ordinary field, because a second
  independent failure is not a cause of the first.
- no error message repeats the message of its own source. Each level states what
  that level knows, so a chained report does not print the same sentence twice.
- `MicroLux` values are nominal, not calibrated, and are **invalid as a point
  estimate when the source counts are saturated**: at maximum code the true
  illuminance is at least the reported value and otherwise unknown. Saturation is
  not an error, so `AlsCounts::is_saturated` must be checked.
- `MeasurementConfig::default()` is this crate's software policy — the widest
  range — and is deliberately **not** the device reset domain, which is
  `silicon_reset_default`.
- `FreshMeasurement::requested_wait_us` is the delay this driver requested, not
  measured elapsed time. `DelayNs` guarantees at least the request and may take
  longer; the driver reads no clock.

# ph-veml7700-als

Incubating async, allocation-free `no_std` VEML7700 ambient-light driver.

> [!WARNING]
> **Lifecycle:** Incubating.
> **Distribution:** Unpublished; the candidate version is
> `0.1.0-incubating.1` and the manifest retains `publish = false`.
> **Model conformance:** An independent I²C-level model covers twelve public
> operations at gain ×1/8 and 100 ms only, from shut-down and active starts,
> with high-threshold qualification only. `read_device_id`, `inspect`,
> `snapshot`, `set_measurement_config`, and custom-timing
> `measure_once_with_timing` have no conformance trace. See the coverage matrix
> for the exact domain.
> **Physical evidence:** None. No reviewed physical or calibrated-optical
> evidence exists. Evidence applies only to the named operations, and eventual
> publication would not imply hardware qualification.

## Model conformance coverage

Every positive claim below maps to one named test that drives the **public
driver API** through an abstract I²C boundary against an independently derived
device model. Nothing here establishes behavior on silicon.

Two other test layers exist and must not be confused with this one. *Model-only*
tests exercise the model's own declared behavior, not the driver. *Scripted-I²C*
tests assert exact transactions, not device behavior. Neither is conformance.

### Covered

| Public operation | Accepted initial state | Configuration exercised | Conformance test |
| --- | --- | --- | --- |
| `probe` | any | — | `probe_accepts_the_fixed_address_little_endian_id` |
| `measure_once` | reset / shut down | Div8, 100 ms, cadence disabled | `measure_once_returns_the_injected_pair_after_the_driver_delay_and_restores_state` |
| `measure_once` | active | Div8, 100 ms, cadence disabled | `measure_once_from_an_active_start_agrees_with_the_model` |
| `arm_threshold_monitor` | reset / shut down | Div8, 100 ms, persistence 4, cadence disabled | `threshold_monitor_public_operations_qualify_after_configured_persistence` |
| `arm_threshold_monitor` | active, monitor disabled | Div8, 100 ms, persistence 4, Mode 2 | `arming_the_monitor_from_an_active_start_agrees_with_the_model` |
| `arm_threshold_monitor` | active, monitor **enabled** | Div8, 100 ms, persistence 4, Mode 2 | `re_arming_an_enabled_active_monitor_agrees_with_the_model` |
| `read_threshold_status` | armed | **high direction only** | `threshold_monitor_public_operations_qualify_after_configured_persistence` |
| `read_thresholds` | armed | — | same, and `re_arming_an_enabled_active_monitor_agrees_with_the_model` |
| `disable_threshold_monitor` | armed, active | — | `threshold_monitor_public_operations_qualify_after_configured_persistence` |
| `set_power_saving` | shut down | Mode 2 enabled | `public_power_operations_observe_the_documented_mode_2_refresh_boundary` |
| `set_power_state` | both | — | four traces |
| `read_als_snapshot` | active | — | `public_power_operations_observe_the_documented_mode_2_refresh_boundary`, `public_channel_reads_can_observe_independently_refreshed_generations` |
| `read_white_snapshot` | active | — | `public_channel_reads_can_observe_independently_refreshed_generations` |
| `read_configuration` | various | — | four traces |
| `read_power_saving` | after restoration | — | `measure_once_returns_the_injected_pair_after_the_driver_delay_and_restores_state` |

### Not covered

These public operations have **no** driver-versus-model trace. They are tested by
other layers, which establish different and weaker things.

| Public operation | Why it is absent |
| --- | --- |
| `read_device_id` | `probe` reads the same register internally, so the codec is exercised — but this operation is never called in a conformance trace. |
| `inspect` | Never called. |
| `snapshot` | Never called. Its component reads are covered separately. |
| `set_measurement_config` | Never called. Reconfiguration is exercised only as part of `measure_once` and `arm_threshold_monitor`. |
| `measure_once_with_timing` | Never called with caller-supplied timing. `measure_once` delegates to it with conservative timing only, so the custom-timing surface is unexercised. |

### Configuration domain not exercised

Coverage above is narrower than the operations imply. Within the covered
operations, conformance traces exercise only:

| Domain | Exercised | Not exercised |
| --- | --- | --- |
| Gain | ×1/8 | ×1, ×2, ×1/4 |
| Integration time | 100 ms | 25, 50, 200, 400, 800 ms |
| Persistence | 4 | 1, 2, 8 |
| Power-saving mode | Mode 1, Mode 2 | Mode 3, Mode 4 |
| Threshold direction | high qualification | **low qualification is never exercised** |

A claim about a gain, integration time, persistence value, cadence mode, or
threshold direction outside this table has no conformance evidence behind it.

### Boundaries

- `ph-veml7700-als-model` is a **repository-only, unpublished** test artifact. It
  is not part of this package and cannot be depended on.
- Conformance runs in the repository only. `tests/device_model.rs` and the
  path-only model dependency are excluded from the published package, so tests
  run against the unpacked archive establish that the crate builds and passes its
  own tests standalone — never model conformance.
- Model agreement establishes that two independent derivations of the same
  documents agree. It does not establish that either matches silicon.

## Usage

The application supplies its platform's async I²C bus and delay provider:

```rust,no_run
use embedded_hal_async::{delay::DelayNs, i2c::I2c};
use ph_veml7700_als::{MeasurementConfig, Veml7700};

async fn sample<I2C, D>(i2c: I2C, delay: &mut D)
where
    I2C: I2c,
    D: DelayNs,
{
    // Construction is inert: it performs no I²C transaction.
    let mut sensor = Veml7700::new(i2c);
    let _device_id = sensor.probe().await.expect("VEML7700 probe failed");

    // A snapshot may contain retained or stale data. ALS and white are read
    // sequentially and may straddle an autonomous refresh.
    let snapshot = sensor.snapshot().await.expect("snapshot failed");

    // A fresh measurement is deliberately configured, timed, and frozen
    // before the ALS and white registers are read.
    let fresh = sensor
        .measure_once(delay, MeasurementConfig::safe_bright_start())
        .await
        .expect("fresh measurement failed");

    let _counts = (snapshot.als.counts(), fresh.als.counts());
}
```

The crate distinguishes register snapshots from deliberately timed fresh
measurements, protects threshold-monitor domains, preserves restoration
failures, and converts ALS counts using integer nominal datasheet scales. It
does not claim calibrated lux or apply application-specific optical correction.

Driver verification consists of pure codec tests, exact scripted I²C with
failure injection, and the independent model whose claim is declared in
[`crates/veml7700-model/README.md`](https://github.com/photon-circus/ph-veml7700-als/blob/main/crates/veml7700-model/README.md).

The package is not published and retains `publish = false`. See the
[repository README](https://github.com/photon-circus/ph-veml7700-als#readme) and
[driver documentation](https://github.com/photon-circus/ph-veml7700-als/tree/main/docs)
for the complete scope.

Licensed under MIT.

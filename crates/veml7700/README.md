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

## Features

Default features are **empty**. The crate has one optional feature.

| Feature | Effect |
| --- | --- |
| `defmt` | Derives `defmt::Format` on public value and error types |

`defmt` is **target-firmware integration only.** It emits references to
`_defmt_panic` and a global logger, which the firmware supplies (typically
`defmt-rtt` plus a panic handler). A host test binary has neither, so
`cargo test --all-features` cannot link on a development machine. That is a
property of `defmt`, not a defect here.

The supported host test profile is therefore `--no-default-features`. The
canonical gate still compiles, lints and documents the feature with
`--all-features`, and builds it for the bare-metal targets, so the feature is
verified everywhere it can be — only the host *test binary* is out of scope.

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
| `measure_once` | reset / shut down | ×1/8, 25 ms, cadence disabled | `measure_once_returns_the_injected_pair_after_the_driver_delay_and_restores_state` |
| `measure_once` | active | ×1/8, 25 ms, cadence disabled | `measure_once_from_an_active_start_agrees_with_the_model` |
| `arm_threshold_monitor` | reset / shut down | ×1/8, 100 ms, persistence 1, cadence disabled | `threshold_monitor_public_operations_qualify_at_protect_number_one` |
| `arm_threshold_monitor` | reset / shut down | ×1/8, 100 ms, persistence 4, cadence disabled — **programming only, no qualification** | `arming_above_protect_number_one_programs_the_field_but_yields_no_modeled_status` |
| `arm_threshold_monitor` | active, monitor disabled | ×1/8, 100 ms, persistence 4, Mode 2 | `arming_the_monitor_from_an_active_start_agrees_with_the_model` |
| `arm_threshold_monitor` | active, monitor **enabled** | ×1/8, 100 ms, persistence 4, Mode 2 | `re_arming_an_enabled_active_monitor_agrees_with_the_model` |
| `read_threshold_status` | armed | **high direction only**, protect number one | `threshold_monitor_public_operations_qualify_at_protect_number_one`, and `arming_above_protect_number_one_programs_the_field_but_yields_no_modeled_status` for the unmodeled case |
| `read_thresholds` | armed | — | same, and `re_arming_an_enabled_active_monitor_agrees_with_the_model` |
| `disable_threshold_monitor` | armed, active | — | `threshold_monitor_public_operations_qualify_at_protect_number_one` |
| `set_power_saving` | shut down | Mode 2 enabled | `public_power_operations_observe_the_documented_mode_2_refresh_boundary` |
| `set_power_state` | reset / shut down | requests active only; no trace shuts an active device down | four traces |
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
| Integration time | 25 ms (fresh capture), 100 ms (threshold) | 50, 200, 400, 800 ms |
| Persistence | 1 (qualification), 4 (programming only) | 2, 8; and **qualification above protect number one is unmodeled, not merely unexercised** |
| Power-saving mode | Mode 1, Mode 2 | Mode 3, Mode 4 |
| Threshold direction | high qualification | **low qualification is never exercised** |

A claim about a gain, integration time, persistence value, cadence mode, or
threshold direction outside this table has no conformance evidence behind it.

The persistence row is the one exception worth reading twice, because it is not
a coverage gap that more traces would close. The sources establish the four
`ALS_PERS` encodings, and the vendor's application note states the counting
condition — a flag is set *only when* the threshold is exceeded and `ALS_PERS`
measurements stay above or below it. That form is **necessary, not stated to be
sufficient**, and nothing states what a measurement that fails to qualify does
to a partial run. Predicting an assertion needs both. This driver therefore
programs the field and promises nothing about *when* a flag asserts above
protect number one, and the model declares that rule undefined rather than
completing it. See `docs/DECISIONS.md` D-030.

Threshold traces deliberately use 100 ms rather than the
[`maximum_range_start`] preset: 25 ms has no vendor-documented power-saving
refresh time (`S-21`), so pairing it with an enabled cadence would ask for
behavior no source establishes.

[`maximum_range_start`]: https://docs.rs/ph-veml7700-als/latest/ph_veml7700_als/struct.MeasurementConfig.html#method.maximum_range_start

### Boundaries

- `ph-veml7700-als-model` is a **repository-only, unpublished** test artifact. It
  is not part of this package and cannot be depended on.
- Conformance runs in the repository only, from a separate unpublished workspace
  package (`ph-veml7700-als-conformance`) that depends on this crate exactly as a
  downstream consumer would. **This crate does not depend on the model at all**,
  so `cargo test -p ph-veml7700-als` cannot build it. Tests run against the
  unpacked archive therefore establish that the crate builds and passes its own
  tests standalone — never model conformance.
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

    // Start at the widest range. Unknown light can be direct sunlight, and a
    // narrower domain would saturate without saying so.
    let fresh = sensor
        .measure_once(delay, MeasurementConfig::maximum_range_start())
        .await
        .expect("fresh measurement failed");

    // Saturation is not an error, so it must be checked. At maximum code the
    // conversion clipped: `nominal_illuminance` is the domain's ceiling, not an
    // observation, and it bounds nothing about the actual light.
    if fresh.als.is_saturated() {
        // Nothing wider exists — this is already the maximum range. Attenuate
        // optically, or record that the reading clipped rather than treating the
        // number as a value.
    } else {
        // Once the ambient range is known, a longer integration time or higher
        // gain gives a finer reading over a narrower span.
        let _micro_lux = fresh.nominal_illuminance.as_micro_lux();
    }

    let _counts = (snapshot.als.counts(), fresh.als.counts());
}
```

The crate distinguishes register snapshots from deliberately timed fresh
measurements, protects threshold-monitor domains, preserves restoration
failures, and converts ALS counts using integer nominal datasheet scales. It
does not claim calibrated lux or apply application-specific optical correction.

## Installing

This is a prerelease, so the version must be written out in full — a `0.1`
requirement will not match it:

```toml
[dependencies]
ph-veml7700-als = "0.1.0-incubating.1"
```

It is **not yet published**; the manifest retains `publish = false`. The
snippet above is what the dependency will look like, not something that
resolves today.

## Requirements

| | |
| --- | --- |
| Rust | 1.92.0 (MSRV), Edition 2024 |
| Runtime dependency | `embedded-hal-async` 1.0 |
| Posture | `#![no_std]`, allocation-free, `#![forbid(unsafe_code)]` |
| Bus | Caller-provided async I²C; the driver owns no HAL, executor, board, or clock |

### Supported targets

The full local gate compiles all five on every change. Hosted CI runs the
`bounded` profile, which compiles `thumbv7em-none-eabihf` only and reports the
other four as explicit skips — so the automated evidence a pull request carries
is one triple, not five.

```text
thumbv6m-none-eabi          thumbv8m.main-none-eabihf   riscv32imac-unknown-none-elf
thumbv7em-none-eabihf       riscv32imc-unknown-none-elf
```

Compiling on a triple establishes that the documented `no_std` surface builds
there. It establishes nothing about board wiring, bus timing, concurrency, or
silicon.

## Verification and scope

Driver verification consists of pure codec tests, exact scripted I²C with
failure injection, an independent behavioral model, and driver-versus-model
conformance for the traces named above.

`ph-veml7700-als-model` is repository-only and unpublished. It is excluded from
this package, so it cannot be depended on and no link to it is given here — a
consumer of the published crate has no way to use it.

Licensed under MIT.

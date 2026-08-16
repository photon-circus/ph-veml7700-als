# ph-veml7700-als

An async, allocation-free `no_std` VEML7700 ambient-light driver over a
caller-provided `embedded-hal-async` I²C bus.

[![Lifecycle: Incubating](https://img.shields.io/badge/lifecycle-incubating-orange.svg)](https://github.com/photon-circus/.github/blob/main/REPOSITORY_STANDARDS.md#31-lifecycle-values)
[![MSRV](https://img.shields.io/badge/MSRV-1.92.0-blue.svg)](https://github.com/photon-circus/ph-veml7700-als/blob/main/Cargo.toml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/photon-circus/ph-veml7700-als/blob/main/crates/veml7700/LICENSE)

> [!WARNING]
> **Lifecycle:** Incubating. **Distribution:** Published on crates.io as the
> `0.1.0-incubating.1` prerelease. **Verification:** Driver-versus-model
> agreement is bounded to the traces in the [verification record]. No
> physical-hardware or calibrated-optical evidence has been recorded.

The driver keeps board policy with the application and avoids hidden cached
device state. It distinguishes observational snapshots from a controlled
one-shot capture, preserves partial-operation context in errors, and exposes
nominal integer scaling without presenting it as calibrated system lux.

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
    let mut sensor = Veml7700::new(i2c);
    sensor.probe().await.expect("VEML7700 probe failed");

    let measurement = sensor
        .measure_once(delay, MeasurementConfig::maximum_range_start())
        .await
        .expect("measurement failed");

    let _counts = measurement.als.counts();
}
```

The generated API documentation owns exact operation behavior, errors, timing,
and cancellation semantics. Data-sheet and hardware provenance remain in the
[shared evidence registry] and are cited from the API by stable `S-nn` rather
than copied here.

Important current limits: nominal micro-lux is not product calibration; maximum
code alone does not prove optical overrange; and threshold flag qualification
has no timing guarantee. The independent model returns an unsupported result
where shared evidence cannot justify an oracle instead of manufacturing
coverage.

## Features

Default features are empty. `defmt` adds `defmt::Format` implementations to
public values and errors; firmware supplies its own logger and panic symbols.

## Installing

```sh
cargo add ph-veml7700-als@0.1.0-incubating.1
```

Name the prerelease explicitly. Cargo will not select `0.1.0-incubating.1` from
a plain `0.1` requirement, so a dependency without a prerelease in its own
version requirement resolves to nothing.

Availability on crates.io is a distribution fact and nothing more. It does not
imply complete model conformance, physical observation, hardware qualification,
or promotion out of Incubating; the status disclosure above still governs.

## Requirements

- Rust 1.92.0, Edition 2024
- `embedded-hal-async` 1.0

Model-conformance and package-verification details live in the repository's
[verification record]. A successful build or model trace does not establish
board wiring, bus integrity, optical performance, or silicon qualification.

[verification record]: https://github.com/photon-circus/ph-veml7700-als/blob/main/docs/VERIFICATION.md#model-conformance-coverage
[shared evidence registry]: https://github.com/photon-circus/ph-veml7700-als/blob/main/docs/HARDWARE_CONTRACT.md

Licensed under MIT.

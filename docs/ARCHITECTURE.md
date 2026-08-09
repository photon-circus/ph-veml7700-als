# `ph-veml7700-als` architecture

This document is normative. **MUST** rules are review-blocking.

## 1. Architectural problem

The driver must maintain a trustworthy relationship between gain/integration
configuration, autonomous measurement timing, retained shutdown data, nominal
illuminance scaling, power-saving cadence, and polled threshold state—without
silently presenting an old count as fresh or a nominal scale as calibrated lux.

Governing maxim:

> Model the VEML7700 as an autonomous integrating optical sensor whose register
> data has timing and optical provenance, not as a collection of I²C words.

## 2. Family context, not framework inheritance

Shared Photon Circus conventions include repository shape, linting, local CI,
documentation, strict mock transports, behavioral models, package gates, and
external `ph-hil` schemas.

The crate deliberately omits a universal `DeviceCore`, internal transport trait,
configuration cache, initialization state machine, and shared error enum. It has
one fixed-address I²C transport, so direct ownership is clearer.

## 3. Layers

```text
Application / optical policy
        ↓
Board support / bus sharing / fixture
        ↓
Veml7700<I2C> operation sequencing
        ↓
Pure config, power, scaling, threshold, timing codecs
        ↓
Private register map
        ↓
embedded_hal_async::i2c::I2c
```

The application owns auto-ranging, calibration, cover/window compensation,
light-source interpretation, retry/backoff, logging, and long-term scheduling.
The BSP owns bus construction/recovery and physical power/fixture wiring.

## 4. Facade state

```rust
pub struct Veml7700<I2C> {
    i2c: I2C,
}
```

**MUST NOT:** cache configuration, power-saving state, thresholds, flags,
identity, samples, timing deadlines, or calibration.

**MUST:** `new()` is `const` and inert; `release()` returns the exact resource.

## 5. Register I/O

All helpers are private. Every 16-bit read/write uses low-byte-first wire order.
Every public bus failure preserves `I2C::Error` plus semantic operation and
register context.

## 6. Snapshot versus fresh measurement

`snapshot()` reads observed configuration and power-saving state followed by ALS
and white. It explicitly reports `SequentialRegisters`, does not wake the device,
and may return retained shutdown data.

`measure_once()` is a complete operation. Its public timing value can extend,
but cannot shorten, the conservative vendor-derived wait and carries the
integration-time selection it was derived for:

1. observe original configuration and power-saving state;
2. reject an enabled threshold monitor;
3. disable power-saving cadence;
4. install the requested gain/integration fields while explicitly shut down;
5. leave shutdown to create a known wake edge;
6. wait conservative wake-up plus integration interval;
7. enter shutdown again to freeze the completed result;
8. read ALS and white;
9. restore power-saving and configuration state.

If capture succeeds but restoration fails, the error carries the sample and marks
hardware state uncertain. If an earlier step and cleanup both fail, both failures
are represented.

## 7. Threshold-monitor domain

An enabled monitor owns:

```text
gain + integration time + low/high counts + persistence + PSM cadence + active state
```

Ordinary setters reject changes that would retarget this domain. The complete
arm operation disables monitoring, writes thresholds, applies cadence, then
enables final configuration last. No GPIO type appears anywhere in the crate.

## 8. Lux policy

The base crate offers exact integer nominal scales from the vendor table.
`MicroLux` is a unit-bearing integer value. The crate does not expose floating
point or an “accurate lux” name.

Empirical correction and optical calibration remain separate application policy
until a future reviewed contract defines input domain, fixed-point precision,
source/window assumptions, saturation behavior, and HIL evidence.

## 9. Source layout

```text
src/
├── lib.rs          crate docs, lints, re-exports only
├── driver.rs       I²C ownership and operation sequencing
├── error.rs        contextual and staged errors
├── register.rs     private pointers
├── id.rs           ID decoding
├── config.rs       gain/integration/persistence/power codec
├── power.rs        power-saving codec and documented cadence
├── measurement.rs  counts, snapshots, fresh samples
├── illuminance.rs  integer nominal scaling
├── threshold.rs    monitor domain and status
├── timing.rs       named wake/integration policy
└── testing/        strict scripted I²C and behavioral model
```

Module paths are not semver API. Public imports come from crate root.

## 10. Non-goals for v0.1

- automatic range selection;
- calibrated or corrected lux;
- VEML6030 family generalization;
- sync/blocking adapter;
- public raw register access;
- driver-owned retries;
- GPIO interrupt integration;
- dynamic allocation or executor coupling.
- publication to a package registry; the crate is packageable for inspection
  but Cargo publication is hard-disabled.

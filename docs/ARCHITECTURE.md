# Architecture

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

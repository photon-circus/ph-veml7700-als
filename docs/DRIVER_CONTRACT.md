# Driver contract

> **Authority: normative.** This records the public driver's cross-cutting
> ownership and guarantees. Generated Rustdoc owns signatures and each
> operation's exact behavior.

## Product boundary

`ph-veml7700-als` is a fixed-address async I²C driver. The caller owns bus
construction/recovery, delays, scheduling, power, board wiring, optical design,
calibration, retries, and application policy. The crate owns VEML7700 register
encoding, complete operations, nominal scaling, and truthful result/error
reporting. The fixed address is the device fact recorded by `S-05`; power,
pull-ups, and optical setup remain caller-owned policy rather than device claims.

## Evidence boundary

[`HARDWARE_CONTRACT.md`](HARDWARE_CONTRACT.md) is the shared evidence registry,
not this driver's policy. This contract and the driver code cite its stable
`S-nn` propositions, then choose driver consequences independently of the model.
Agreement about evidence does not require identical behavior.

## Driver state

`Veml7700<I2C>` stores only the I²C resource. Construction is inert and release
returns the exact resource. Configuration, power-saving state, threshold domain,
status, samples, and timing deadlines remain device-authoritative.

Every register word is encoded and decoded low byte first (`S-08`). The driver
reads `0x03` before acting, so it does not rely on the assumed power-on mode or
enable value. Its strict decoder does reject non-zero reserved bits, making that
part of the power-on assumption `S-11` a declared driver dependency. The known
mode field is `S-48`.

`PowerSavingMode::nominal_refresh_time_ms` implements only the exact `S-21`
domain and returns `None` for the undefined `S-22` or undocumented `S-44`
combinations. Configuration APIs still program those encodings.

## Snapshot and one-shot measurement

A snapshot reports observed configuration and sequential ALS/white register
values without claiming freshness. A complete one-shot operation installs a
known domain in shutdown, creates a shutdown-to-active wake edge, applies a
conditional policy wait, freezes results in shutdown, reads both channels, and
restores prior state. The wait reacts to `S-23`, `S-24`, and `S-55`, plus
caller-selectable margin; it is not a characterized silicon bound. The driver
freezes the pair before reading as its reaction to `S-25`. Errors retain capture
and restoration context.

## Threshold-monitor ownership

The monitored domain includes gain, integration time, thresholds, persistence,
power-saving cadence, and active state. Arming is disable-first and enable-last.
Ordinary methods reject changes that would silently retarget an enabled monitor.
No GPIO abstraction exists (`S-41`); status is polled over I²C. The driver
programs every `S-16` persistence encoding but makes no assertion-timing promise
for any of them; that boundary is `S-39`, `S-49`, and `S-50`.

## Optical boundary

Integer `MicroLux` applies `S-26`. It is not calibrated lux at a product
aperture. Window transmission, geometry, spectrum,
cosine response, part tolerance, high-lux correction, and auto-ranging belong
to a separately reviewed integration layer or application. The driver applies
only nominal scaling.

## Cross-cutting guarantees

Reconfiguration follows the shutdown-first source flow in `S-56`; threshold
arming is disable-first and enable-last and ends active as driver policy (`S-17`).
Operations avoid writes when the requested state already matches.

Async writes are not cancellation-safe. A failed or cancelled I²C write has
unknown commit state; cleanup and restoration occur only while a future is
polled to completion. Public errors retain the underlying bus error plus the
operation/stage information the driver actually knows. They never claim a write
was rolled back.

The driver exposes no raw-register or interrupt-GPIO ownership. Its software
default is the widest-range starting policy (`S-28`, `S-34`), distinct from the
decoded device reset domain (`S-12`).

# Bootstrap acceptance checklist

## Repository

- [ ] repository and package names accepted;
- [ ] Rust 1.92.0 MSRV accepted or deliberately changed everywhere;
- [ ] `python tools/validate-pack.py` passes;
- [ ] `tools/check.sh` or `tools/check.ps1` passes;
- [ ] all relative links resolve.

## Contracts

- [ ] official datasheet and application note pinned and hashed;
- [ ] hardware contract reviewed line by line;
- [ ] vendor register-count inconsistency reviewed;
- [ ] API contract frozen before public-surface expansion;
- [ ] no hidden GPIO, calibration, freshness, or flag-clear assumption remains.

## Implementation

- [ ] low-byte-first transactions verified;
- [ ] all legal encodings and reserved values tested;
- [ ] complete fresh operation restoration paths tested;
- [ ] threshold monitor owns cadence and measurement domain;
- [ ] runtime code has no `std`, `alloc`, unsafe, MCU HAL, or `ph-hil` dependency.

## HIL

- [ ] real managed ESP32 harness implemented and reviewed;
- [ ] fixture circuit checklist complete;
- [ ] mock plan runs and remains void for physical claims;
- [ ] core physical run sealed and reviewed;
- [ ] optical plan remains disabled until calibrated reference and fixture facts exist.

## Release

- [ ] formatting, tests, Clippy, docs, doctests, cross-target checks pass;
- [ ] cargo-deny, package listing, and publish dry-run pass;
- [ ] capability matrix reflects evidence rather than aspiration;
- [ ] release checklist and changelog complete.

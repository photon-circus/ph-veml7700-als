# Bootstrap acceptance checklist

## Repository

- [x] repository and package names accepted;
- [x] Rust 1.92.0 MSRV accepted or deliberately changed everywhere;
- [x] `python3 tools/validate-pack.py` passes;
- [x] `tools/check.sh` or `tools/check.ps1` passes;
- [x] all relative links resolve.
- [x] Cargo publication is disabled and automation contains no publish command
      or registry credential.

## Contracts

- [ ] official datasheet and application note pinned and hashed;
- [ ] hardware contract reviewed line by line;
- [ ] vendor register-count inconsistency reviewed;
- [x] API contract frozen before public-surface expansion;
- [x] no hidden GPIO, calibration, freshness, or flag-clear assumption remains.

## Implementation

- [x] low-byte-first transactions verified;
- [x] all legal encodings and reserved values tested;
- [x] complete fresh operation restoration paths tested;
- [x] threshold monitor owns cadence and measurement domain;
- [x] runtime code has no `std`, `alloc`, unsafe, MCU HAL, or `ph-hil` dependency.

## HIL

- [ ] real managed ESP32 harness implemented and reviewed;
- [ ] fixture circuit checklist complete;
- [x] deterministic mock harness builds and remains void for physical claims;
- [ ] core physical run sealed and reviewed;
- [ ] optical plan remains disabled until calibrated reference and fixture facts exist.

## Release

- [x] formatting, tests, Clippy, docs, doctests, cross-target checks pass;
- [x] cargo-deny and package construction/listing pass;
- [x] capability matrix reflects evidence rather than aspiration;
- [ ] release checklist and changelog complete.

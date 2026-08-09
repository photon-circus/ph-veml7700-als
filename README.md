# ph-veml7700-als

Contract-first development pack for an async `no_std` Rust driver for the
Vishay VEML7700 ambient-light sensor.

The design treats the device as an autonomous integrating optical sensor, not
as a register bag. It separates:

- a potentially stale register snapshot;
- a fresh measurement started from an explicit shutdown-to-active wake edge and conservatively waited for;
- nominal count-to-lux scaling from application-specific optical calibration;
- threshold-status polling from a fictitious interrupt GPIO (the VEML7700 has
  no dedicated interrupt pin); and
- normal measurement configuration from a threshold monitor whose physical
  meaning depends on gain, integration time, persistence, and power-saving
  cadence.

## Crate policy

- async-first `embedded-hal-async` I²C;
- `#![no_std]`, no allocator, no unsafe code;
- inert `const new()` and exact bus release;
- fixed 7-bit address `0x10`;
- concrete I²C errors preserved with operation context;
- no `init()`, cached register state, universal device framework, or public raw
  register accessor;
- integer nominal illuminance conversion; no floating-point requirement.

## Layout

- `crates/veml7700` — packageable, publication-disabled `ph-veml7700-als` crate;
- `docs/` — normative hardware, architecture, API, invariant, test, HIL, and
  implementation contracts;
- `apps/hil-runner` and `hil/` — external `ph-hil` integration boundary;
- `tools/validate-pack.py` — deterministic pack-consistency checks.

## First commands

```console
python3 tools/validate-pack.py
cargo fmt --all -- --check
cargo test -p ph-veml7700-als --no-default-features
cargo clippy -p ph-veml7700-als --all-targets --all-features -- -D warnings
```

This bootstrap is not a release or a physical support claim. Verify the pinned
Vishay documentation, compile every target, implement the managed harness, and
review sealed optical evidence before promoting capability status.

Cargo registry publication is hard-disabled. Repository automation may build
the package for inspection but contains no publish step or registry credential.

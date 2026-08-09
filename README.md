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

## Local CI

GitHub Actions is intentionally disabled while the crate is under development.
Run the complete CI matrix on a trusted local machine:

```console
./tools/check.sh
```

On Windows PowerShell, run `tools/check.ps1`. The local runner covers pack and
policy validation, formatting, tests, all features, Clippy, docs, embedded
targets, dependency policy, package inspection, and the mock HIL build.

This bootstrap is not a release or a physical support claim. Verify the pinned
Vishay documentation, compile every target, implement the managed harness, and
review sealed optical evidence before promoting capability status.

Cargo registry publication is hard-disabled. Repository automation may build
the package for inspection but contains no publish step or registry credential.

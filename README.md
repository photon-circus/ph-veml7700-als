# ph-veml7700-als

Async, `no_std`, allocation-free Rust driver for the Vishay VEML7700 ambient-
light sensor.

> [!WARNING]
> **Lifecycle:** Incubating.
> **Distribution:** Unpublished; the candidate version is
> `0.1.0-incubating.1` and the manifest retains `publish = false`.
> **Model conformance:** An independent I²C-level model covers `probe`, fresh
> measurement, power-saving cadence, threshold monitoring, and sequential
> ALS/white observation. Transport faults, arbitrary active reconfiguration,
> threshold-flag clearing, source-undeclared reset values, and unexercised
> public operations remain outside the current model claim.
> **Physical evidence:** None. No reviewed physical or calibrated-optical
> evidence exists. Evidence applies only to the named operations, and eventual
> publication would not imply hardware qualification.

## Responsibility

The crate owns complete single-device operations over caller-provided async I²C:

- explicit snapshot versus fresh-measurement semantics;
- fixed-address identity checks and little-endian register framing;
- conservative wake/integration timing with provenance;
- restoration-aware fresh capture;
- typed power-saving and threshold-monitor domains;
- raw ALS and white counts; and
- integer, nominal ALS count-to-lux scaling.

## Scope boundaries

The crate does not own an MCU, HAL, executor, bus recovery, optical fixture,
window/diffuser compensation, source-spectrum correction, empirical high-lux
correction, calibrated metrology, automatic ranging, or a fictitious interrupt
pin. The VEML7700 threshold output is polled through its status register.

Nominal lux is a datasheet-table conversion, not a claim about illuminance at a
finished product's aperture.

## Repository layout

```text
crates/veml7700/         driver and host-side tests
crates/veml7700-model/   independent device behavioral model (bounded slice)
docs/                    device, API, architecture, invariant, and test contracts
scripts/ci.sh            canonical bounded local verification
tools/check.ps1          PowerShell launcher for the same gate under Git Bash
```

Contracts: [hardware](docs/HARDWARE_CONTRACT.md),
[invariants](docs/INVARIANTS.md),
[architecture](docs/ARCHITECTURE.md),
[API](docs/API_CONTRACT.md),
[test plan](docs/TEST_PLAN.md),
[decisions](docs/DECISIONS.md),
[documentation standards](docs/DOCUMENTATION_STANDARDS.md),
[vendor record](docs/vendor/README.md).
The model's maintained claim is
[`crates/veml7700-model/README.md`](crates/veml7700-model/README.md).

## Local verification

Run `./scripts/ci.sh` under Git Bash or another POSIX-compatible shell, or
`./tools/check.ps1` from PowerShell, which locates Git Bash and runs the same
script.

Hosted CI is dispatch-only while the repository is private; its automatic
triggers and default-branch protection are both part of the visibility change.
It runs the same script with `CI_PROFILE=bounded`, which skips the dependency
policy, four of the five bare-metal targets, and packaging, printing each skip.
It is contributor feedback, never the release gate.

## Publication status

The package retains `publish = false`. Preparation, visibility, and crates.io
publication are separate maintainer-controlled steps in
[RELEASING.md](RELEASING.md).

## License

Licensed under the [MIT License](LICENSE).

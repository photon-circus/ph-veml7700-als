# ph-veml7700-als

Async, `no_std`, allocation-free Rust driver for the Vishay VEML7700 ambient-
light sensor.

> [!WARNING]
> **Lifecycle:** Incubating.
> **Distribution:** Unpublished; the candidate version is
> `0.1.0-incubating.1` and the manifest retains `publish = false`.
> **Model conformance:** An independent I²C-level model covers `probe` and one
> successful `measure_once` path only. The coupled fake remains test-only and
> is not that oracle; all other public operations are outside the current model
> claim.
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
tools/check.*            platform launchers for the same local gate
```

Start with [the documentation index](docs/README.md).

## Local verification

Run `./tools/check.sh` under Git Bash or another POSIX-compatible shell, or
`./tools/check.ps1` from PowerShell. A green gate establishes agreement with
the implemented host contracts only; it does not establish physical-device or
calibrated-optical behavior.

## Publication status

The package retains `publish = false`. Release preparation, repository
visibility, and crates.io publication are separate maintainer-controlled steps
described in [RELEASING.md](RELEASING.md). Model completeness, physical
evidence, hardware qualification, and `ph-hil` adoption are not publication
prerequisites; they limit only the claims they support.

## License

Licensed under the [MIT License](LICENSE).

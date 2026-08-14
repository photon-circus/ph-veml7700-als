# ph-veml7700-als

Async, `no_std`, allocation-free Rust driver for the Vishay VEML7700 ambient-
light sensor.

> [!WARNING]
> **Lifecycle:** Incubating.
> **Distribution:** Unpublished; the candidate version is
> `0.1.0-incubating.1` and the manifest retains `publish = false`.
> **Model conformance:** An independent I²C-level model covers twelve public
> operations at gain ×1/8 and 100 ms only, from shut-down and active starts,
> with high-threshold qualification only. `read_device_id`, `inspect`,
> `snapshot`, `set_measurement_config`, and custom-timing
> `measure_once_with_timing` have no conformance trace. See the coverage matrix
> for the exact domain.
> **Physical evidence:** None. No reviewed physical or calibrated-optical
> evidence exists. Evidence applies only to the named operations, and eventual
> publication would not imply hardware qualification.

**Consumers start at [`crates/veml7700/README.md`](crates/veml7700/README.md).**
That is the packaged documentation — install syntax, MSRV, supported targets,
features, the usage example, and the exact model-conformance coverage. It is
also the crate documentation, included verbatim by `lib.rs`, so there is one
description of this crate rather than two that can disagree.

This page is for people working *on* the repository.

## Layout

| Path | Contents |
| --- | --- |
| `crates/veml7700` | The driver. Published; packaged README is the consumer source of truth. |
| `crates/veml7700-model` | Independent behavioral model. **Repository-only and unpublished** — a test oracle, not a dependency. |
| `docs/` | Device contract, API contract, invariants, verification plan, decision log, vendor provenance. |
| `scripts/ci.sh` | The canonical gate. One implementation; `tools/check.ps1` launches it on Windows. |

Document authority varies and is stated per file in
[`CONTRIBUTING.md`](CONTRIBUTING.md) — not everything under `docs/` is normative.

## Verifying a change

```sh
CI_PROFILE=full sh scripts/ci.sh
```

`full` is authoritative. `bounded` is the subset hosted CI runs and is never
authoritative — a skipped check is not a passed check. `release` adds artifact
identity and refuses a dirty worktree. See [`CONTRIBUTING.md`](CONTRIBUTING.md)
for setup and the pinned toolchain.

## Publication status

The package retains `publish = false`. Preparation, repository visibility, and
crates.io publication are separate maintainer decisions recorded in
[`RELEASING.md`](RELEASING.md); none follows automatically from another.

## License

Licensed under the [MIT License](LICENSE).

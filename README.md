# ph-veml7700-als

Async, `no_std`, allocation-free Rust driver for the Vishay VEML7700 ambient-
light sensor.

[![Lifecycle: Incubating](https://img.shields.io/badge/lifecycle-incubating-orange.svg)](https://github.com/photon-circus/.github/blob/main/REPOSITORY_STANDARDS.md#31-lifecycle-values)
[![crates.io](https://img.shields.io/crates/v/ph-veml7700-als.svg)](https://crates.io/crates/ph-veml7700-als)
[![docs.rs](https://img.shields.io/docsrs/ph-veml7700-als)](https://docs.rs/ph-veml7700-als)
[![MSRV](https://img.shields.io/badge/MSRV-1.92.0-blue.svg)](Cargo.toml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> [!WARNING]
> This repository is incubating. The
> [driver crate README](crates/veml7700/README.md) owns consumer status and
> limitations; [`RELEASING.md`](RELEASING.md) owns the release procedure and the
> maintainer-reserved crates.io decision, which each version requires again.

The packaged driver README is included verbatim by `lib.rs`, so Git hosting and
docs.rs do not maintain competing consumer narratives.

This page is for people working *on* the repository.

## Layout

| Path | Contents |
| --- | --- |
| `crates/veml7700` | The driver crate, published as `ph-veml7700-als`; its packaged README is the consumer source of truth. |
| `crates/veml7700-model` | Independent behavioral model. **Repository-only and never published** — a test oracle, not a dependency. |
| `docs/` | Shared evidence, driver semantics, verification, concise rationale, and vendor provenance. |
| `xtask/` | The canonical local gate (`cargo xtask ci`). Policy lives in `xtask/data/*.ron`. |

Document authority varies and is routed by [`CONTRIBUTING.md`](CONTRIBUTING.md)
— not everything under `docs/` is normative.

## Verifying a change

```sh
cargo xtask ci
```

`full` is authoritative. `bounded` is the subset hosted CI runs and is never
authoritative — a skipped check is not a passed check. `release` adds artifact
identity and refuses a dirty worktree. See [`CONTRIBUTING.md`](CONTRIBUTING.md)
for setup and the pinned toolchain.

## License

Licensed under the [MIT License](LICENSE).

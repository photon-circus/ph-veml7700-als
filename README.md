# ph-veml7700-als

Async, `no_std`, allocation-free Rust driver for the Vishay VEML7700 ambient-
light sensor.

> [!WARNING]
> This repository is incubating. The
> [driver crate README](crates/veml7700/README.md) owns consumer status and
> limitations; [`RELEASING.md`](RELEASING.md) owns the separately reserved
> repository-visibility and crates.io decisions.

The packaged driver README is included verbatim by `lib.rs`, so Git hosting and
docs.rs do not maintain competing consumer narratives.

This page is for people working *on* the repository.

## Layout

| Path | Contents |
| --- | --- |
| `crates/veml7700` | The driver crate; its packaged README is the consumer source of truth. |
| `crates/veml7700-model` | Independent behavioral model. **Repository-only and unpublished** — a test oracle, not a dependency. |
| `docs/` | Shared evidence, driver semantics, verification, concise rationale, and vendor provenance. |
| `scripts/ci.sh` | The canonical gate. One implementation; `tools/check.ps1` launches it on Windows. |

Document authority varies and is routed by [`CONTRIBUTING.md`](CONTRIBUTING.md)
— not everything under `docs/` is normative.

## Verifying a change

```sh
CI_PROFILE=full sh scripts/ci.sh
```

`full` is authoritative. `bounded` is the subset hosted CI runs and is never
authoritative — a skipped check is not a passed check. `release` adds artifact
identity and refuses a dirty worktree. See [`CONTRIBUTING.md`](CONTRIBUTING.md)
for setup and the pinned toolchain.

## License

Licensed under the [MIT License](LICENSE).

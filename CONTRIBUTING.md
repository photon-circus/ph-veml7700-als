# Contributing

Everything you need to prepare and validate a change is on this page. You do not
need to read `AGENTS.md`; that file holds operational notes for automated
agents, not contributor policy.

## What this repository is

An async, allocation-free `no_std` VEML7700 ambient-light-sensor driver, plus an
independent behavioral model used only as a test oracle. The driver talks to a
caller-supplied `embedded-hal-async` I²C bus and owns nothing else — no board
support, no MCU examples, no fixtures, no hardware runners.

The project's defining constraint is **evidence honesty**: it is always clear
what established a claim, and model or simulated results never quietly become
hardware claims. Most review feedback traces back to that.

## Setup

Prerequisites:

| Tool | Version | How it is pinned |
| --- | --- | --- |
| Rust | 1.92.0, Edition 2024 | `rust-toolchain.toml` — rustup installs it automatically |
| Clippy, rustfmt | bundled | `rust-toolchain.toml` components |
| Bare-metal targets | five triples | `rust-toolchain.toml` targets |
| `cargo-deny` | 0.20.2 | asserted by the gate; `cargo install cargo-deny --version 0.20.2 --locked` |
| POSIX shell | any | Git Bash is fine on Windows |

```sh
git clone https://github.com/photon-circus/ph-veml7700-als
cd ph-veml7700-als
cargo fetch --locked
```

The toolchain file does the work: `cargo` commands inside the repository use
1.92.0 and the five reference targets without further setup.

`cargo-deny` is the exception: `deny.toml` configures the dependency and licence
policy but pins no binary version, so the gate asserts the version itself and
fails on a mismatch. That is deliberate — `cargo-deny` changes its own lint set
between releases, and an unpinned advisory tool makes the authoritative gate
non-reproducible.

## Verification profiles

| Profile | Command | Purpose |
| --- | --- | --- |
| `full` | `CI_PROFILE=full sh scripts/ci.sh` | Authoritative. Run before opening a PR. |
| `bounded` | `CI_PROFILE=bounded sh scripts/ci.sh` | The subset hosted CI runs. Never authoritative. |
| `release` | `CI_PROFILE=release sh scripts/ci.sh` | `full` plus artifact identity. Maintainer use at release. |

`release` refuses a dirty worktree, packages without `--allow-dirty`, and writes
`target/release-evidence/evidence.md` recording the commit, archive name and
SHA-256, file inventory, normalized manifest, and VCS metadata. It performs no
registry action — no publish, no tag, no release, no credentials.

## Changing the candidate version

Both product crates inherit `version` from `[workspace.package]` in the root
`Cargo.toml`, so a bump edits **one line**. The gate reads the resolved value
back through Cargo, so the check survives that inheritance.

What the gate cannot see is the copies of the literal in tracked prose. Grep
before and after any bump:

```sh
grep -rn '0\.1\.0-incubating\.1' --exclude-dir=.git --exclude-dir=target .
```

At the time of writing it appears in the root `Cargo.toml`, `Cargo.lock`, the
root and packaged READMEs, `AGENTS.md`, `CHANGELOG.md`, `RELEASING.md`,
`docs/DECISIONS.md`, and the bug-report form. A green gate does not mean you
found them all — and neither does this list, which is a copy of a grep result
and can be wrong. Run the grep.

`crates/veml7700/src/lib.rs` is deliberately absent: it includes the packaged
README with `#![doc = include_str!]` and holds no copy of its own.

## Verifying a change

There is exactly one implementation of the verification policy, `scripts/ci.sh`.
Add checks to that script, never to the workflow.

**Full — authoritative.** Run this before opening a pull request:

```sh
CI_PROFILE=full sh scripts/ci.sh
```

`./tools/check.ps1` is a thin PowerShell launcher that finds Git Bash and runs
the same script with the same arguments.

It ends with a line like `[ci] PASS (full): 14 steps, 0 skipped.` Paste that
line into your pull request — it is the evidence a reviewer has.

**Bounded — fast feedback only.** `CI_PROFILE=bounded` is the subset hosted CI
runs. It skips dependency policy, four of the five bare-metal targets, and
packaging, and prints each skip. **A skipped check is not a passed check**, so a
green bounded run never substitutes for a green full run.

While iterating, `cargo test` and `cargo clippy --all-targets` are quicker still,
but neither is evidence.

## Which layer owns your test

Putting a test in the wrong layer is the most common structural review problem.
Each layer may establish only what it is capable of establishing:

| Layer | Where | Owns | Must not be used to claim |
| --- | --- | --- | --- |
| Pure/unit | `crates/veml7700/src/*.rs` | codecs, finite domains, validation, units, timing construction | autonomous device behavior |
| Scripted I²C | `crates/veml7700/src/testing/` | exact address, pointer, byte order, payload, transaction count, injected transport failures | an independent device state machine |
| Model-only | `crates/veml7700-model/` | independent source interpretation, explicit time and stimuli, unsupported boundaries | agreement with the public driver |
| Driver-versus-model | `tests/conformance/` | public driver traces compared through the I²C boundary | operations or initial states it does not exercise |
| Doctests | `crates/veml7700/src/lib.rs`, READMEs | consumer syntax and compilation | runtime sequencing or hardware behavior |
| Target builds | gate | that the `no_std` surface compiles on five triples | wiring, timing, or silicon |

Two rules follow from the table and are enforced in review:

- **Autonomous device behavior belongs in the independent model, never in the
  driver's scripted transport tests.** A scripted test asserts transactions and
  injected failures. The moment it starts modeling what the device would do next
  on its own, it has become a second, coupled device model — which defeats the
  purpose of having an independent one.
- **The model is derived from `docs/HARDWARE_CONTRACT.md`, not from driver
  code.** It must not import driver codecs, constants, or state machines. If the
  driver and the model share a mistake, neither can catch it.

## Evidence language

Say exactly what produced a result. These distinctions are not stylistic:

- **Physical hardware** — observed on silicon. Nothing in this repository
  currently establishes this, and no document may imply otherwise.
- **Behavioral model** — predicted by `ph-veml7700-als-model`. Model agreement
  is evidence about interpretation, not about silicon.
- **Scripted transport / mock** — asserts what went over the bus, nothing about
  what a device would do with it.
- **Code reading** — an unexecuted claim.

Related wording rules:

- Never call a snapshot fresh. Snapshot methods state that data may be retained
  and that ALS and white are sequential reads.
- Nominal illuminance is never calibrated system lux, and every method that
  returns it says so.
- Register `0x06` is not a physical interrupt pin. There is no interrupt pin.
- Avoid unsupported superlatives — "accurate lux", "atomic pair",
  "interrupt-driven".
- Every complete multi-step operation documents its restoration and uncertainty
  behavior.

Use `VEML7700`, `I²C`, `ALS`, `white channel`, `gain ×1/8`, `integration time`,
`power saving`, `threshold monitor`, and `micro-lux` consistently.

## Document authority

Files under `docs/` do not share one status. Treat them as follows:

| Document | Authority |
| --- | --- |
| `docs/HARDWARE_CONTRACT.md` | **Normative** — interpreted device behavior |
| `docs/vendor/README.md` | **Evidence record** — source provenance and digests |
| `docs/DRIVER_CONTRACT.md` | **Normative** — driver semantics, ownership, dependency direction |
| `docs/INVARIANTS.md` | **Normative** — review-blocking truths |
| `docs/VERIFICATION.md` | Contributor procedure — which test layer establishes what |
| `docs/DECISIONS.md` | Non-normative — durable rationale, including superseded entries |
| `crates/veml7700-model/README.md` | **Normative** — the model's maintained claim |

A change that contradicts a normative document must change that document first,
in the same pull request, with rationale in `docs/DECISIONS.md`.

Note that every device fact in `docs/HARDWARE_CONTRACT.md` is currently marked
provisional pending owner verification of the vendor sources. Do not treat an
unchecked row as settled.

## What a reviewable contribution looks like

1. Behavioral claims trace to `docs/HARDWARE_CONTRACT.md`.
2. The public surface matches `docs/DRIVER_CONTRACT.md`, or that contract and
   `docs/DECISIONS.md` change first.
3. Every touched invariant has a protecting test.
4. Exact I²C address, pointer, byte order, payload, and transaction count are
   asserted where transport behavior matters.
5. Autonomous behavior is tested in the independent model. Conformance tests use
   the model crate and public driver APIs, not driver codecs.
6. `CHANGELOG.md` is updated beneath `Unreleased` for any user-visible change.
7. The full local gate passes.

## Pull request workflow

1. Open an issue first for anything beyond an obvious fix. Use the bug form for
   contract deviations and the feature proposal form for new capability. A
   proposal is a decision record, not authorization to implement.
2. Branch from `main`. One reviewable concern per pull request.
3. Fill in the pull request template. The evidence table and the evidence-source
   selection are the parts reviewers read first.
4. Run the full gate and paste its final line.
5. Expect review to focus on evidence provenance, layer ownership, and contract
   coupling at least as much as on the code.

Out of scope, and rejected on sight: raw-register APIs, cached device state,
automatic optical correction, calibration, MCU examples, board support, and
speculative physical-evidence infrastructure.

Do not change repository visibility or registry publication state, add
credentials, or create tags or releases. Those are maintainer-controlled steps
with recorded decisions; see `RELEASING.md`.

Do not commit the vendor PDFs. They are not redistributable, and the gate fails
if they become tracked; `docs/vendor/README.md` records how to retrieve them and
the digests to verify.

## Licensing

The project is MIT licensed. By contributing, you agree that your contributions
are licensed under the [MIT License](LICENSE). There is no separate contributor
licence agreement. Contribute only work you have the right to license this way —
in particular, do not paste vendor document text or code from an incompatible
licence.

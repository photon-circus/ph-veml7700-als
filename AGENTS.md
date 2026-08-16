# Agent guidance

Read `CONTRIBUTING.md` for contributor policy and document authority. This file
adds only operational limits for automated work.

## Scope and fan-out

The repository owns a `no_std` async VEML7700 driver plus pure,
scripted-transport, independent-model, and conformance tests. Do not add MCU or
board examples, fixtures, hardware runners, orchestration, or speculative
physical-evidence plans.

Do not create an issue, follow-up, decision entry, audit, or CI rule unless the
maintainer explicitly requests that artifact. Keep one evidence correction as
one bounded change across directly affected layers. Unrelated observations go
in the handoff; they are not blockers or repository work by default.

For a new peripheral, start with a disposable capability survey. Select
consumers before creating maintained evidence rows. Add a stable proposition
only when a concrete driver, model, conformance, approved-hardware, or bug path
needs it; never inventory every source fact in advance.

## Evidence boundary

`docs/HARDWARE_CONTRACT.md` is the one shared evidence registry. An `S-nn`
permanently identifies one proposition and its evidence. Driver and model cite
the same identifier but derive behavior independently; do not share codecs,
constants, timing helpers, state machines, or inference rules.

Outside the registry, state only the component consequence and cite the
applicable `S-nn`. Never copy the proposition, vendor coordinates, or a hardware
artifact. A retained `not currently relevant` legacy row creates no coverage,
validation, or implementation obligation.

Map a suspected device-behavior bug to its exact ID before deciding driver,
model, or hardware consequences. If no exact referent exists, add one atomic
proposition. Pure software defects may use no device proposition.

## Change discipline

- Keep `new()` inert and return the exact bus from `release()`.
- Keep snapshots observational and one-shot timing explicitly conditional.
- Preserve the complete threshold-monitor domain and reject silent retargeting.
- Keep white counts distinct from ALS nominal scaling.
- Preserve concrete bus errors, partial commits, and restoration uncertainty.
- Keep calibration and optical policy outside the driver.
- Keep autonomous device behavior in the independent model, never in scripted
  driver test state.
- Prefer `Unsupported`, injected input, or unknown model state over invented
  behavior.

Public behavior changes need protecting tests and a user-visible changelog entry.
Durable rationale changes only when the durable decision changes. Unsupported
model behavior, uncertainty, or a possible improvement is not a blocker unless
current behavior is known wrong or an explicit acceptance criterion cannot be
met.

## Validation and reserved actions

Run `./scripts/ci.sh`; the default `full` profile is the authoritative local
gate. On Windows, run the same script from within Git Bash. `bounded` is
non-authoritative feedback. Do not run `release` during ordinary development.

Do not publish to crates.io, add credentials, tag, or create a release. The
repository is public and the driver is published; neither fact authorizes a
further registry action. Each release remains a separate maintainer decision
governed by `RELEASING.md`.
